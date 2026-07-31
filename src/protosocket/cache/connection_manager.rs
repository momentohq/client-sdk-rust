use crate::{
    credential_provider::EndpointSecurity,
    protosocket::cache::{
        address_provider::{AddressProvider, Addresses},
        az_circuit::AzCircuit,
        cache_client_builder::Codec,
    },
    CredentialProvider,
};
use http::Uri;
use momento_protos::protosocket::cache::{
    cache_command::RpcKind, cache_response::Kind, unary::Command, AuthenticateCommand,
    AuthenticateResponse, CacheCommand, CacheResponse, Unary,
};
use protosocket_rpc::{
    client::{
        ClientConnector, TcpStreamConnector, UnverifiedTlsStreamConnector, WebpkiTlsStreamConnector,
    },
    ProtosocketControlCode,
};
use rustls_pki_types::ServerName;
use std::{
    convert::TryFrom,
    net::SocketAddr,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc,
    },
    time::Duration,
};

use crate::{ErrorSource, MomentoError, MomentoResult, ProtosocketCacheError};
use std::net::ToSocketAddrs;

#[derive(Debug)]
struct BackgroundAddressLoader {
    alive: Arc<AtomicBool>,
    _join_handle: tokio::task::JoinHandle<()>,
}

impl Drop for BackgroundAddressLoader {
    fn drop(&mut self) {
        if self.alive.swap(false, std::sync::atomic::Ordering::Relaxed) {
            log::info!("shutting down address refresher task");
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProtosocketConnectionManager {
    credential_provider: CredentialProvider,
    runtime: tokio::runtime::Handle,
    hostname: String,
    address_provider: Arc<AddressProvider>,
    _background_address_loader: Option<Arc<BackgroundAddressLoader>>,
    az_id: Option<String>,
    /// Shared across every pool slot, so one slot discovering the local zone is
    /// unreachable steers the others too.
    az_circuit: Arc<AzCircuit>,
    connection_sequence: Arc<AtomicUsize>,
    /// See [`Configuration::connect_timeout`](crate::protosocket::cache::Configuration::connect_timeout).
    connect_timeout: Duration,
}

/// Where a selected address came from. Log-only: this is deliberately not
/// surfaced to callers, because the service already reports which availability
/// zone traffic arrives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AzRouting {
    /// An address in the configured availability zone.
    Local,
    /// An address outside the configured zone, chosen because that zone is
    /// currently unreachable from this client.
    Fallback,
    /// No zone preference was configured, or none could be applied.
    Unpinned,
}

/// A failure to choose an address, before any socket work is attempted.
#[derive(Debug, Clone, Copy)]
enum ConnectFailure {
    NoAddresses,
}

impl From<ConnectFailure> for protosocket_rpc::Error {
    fn from(failure: ConnectFailure) -> Self {
        // protosocket_rpc::Error is a closed enum owned by that crate, so an
        // io::ErrorKind is the only way to carry a type out of here.
        let (kind, message) = match failure {
            ConnectFailure::NoAddresses => (
                std::io::ErrorKind::AddrNotAvailable,
                "no addresses available from address provider",
            ),
        };
        protosocket_rpc::Error::IoFailure(std::io::Error::new(kind, message).into())
    }
}

/// Whether a failed attempt is evidence about the reachability of the zone the
/// address belongs to.
#[derive(Debug)]
enum EstablishError {
    /// The endpoint did not answer, or answered unusably.
    Transport(protosocket_rpc::Error),
    /// The endpoint answered and rejected our credentials. That says nothing
    /// about the zone, so it must not open the circuit -- otherwise a bad API
    /// key would push every client off its local zone.
    Rejected(protosocket_rpc::Error),
}

impl EstablishError {
    fn into_inner(self) -> protosocket_rpc::Error {
        match self {
            EstablishError::Transport(error) | EstablishError::Rejected(error) => error,
        }
    }
}

impl ProtosocketConnectionManager {
    /// You should make one of these and clone it as needed for connection pools.
    /// It spawns a background task to refresh the address list every 30 seconds.
    /// This manager can be cloned and shared across connection pools.
    ///
    /// If you provide an `az_id`, connections will be preferentially made to
    /// addresses in that availability zone, if any are available.
    pub fn new(
        credential_provider: CredentialProvider,
        runtime: tokio::runtime::Handle,
        az_id: Option<String>,
        connect_timeout: Duration,
    ) -> MomentoResult<Self> {
        let hostname = Uri::from_str(&credential_provider.tls_cache_endpoint)
            .ok()
            .and_then(|uri| uri.host().map(|h| h.to_string()))
            .ok_or_else(|| {
                MomentoError::unknown_error(
                    "protosocket_connection_manager::new",
                    Some(format!(
                        "Could not parse TLS endpoint: {}",
                        credential_provider.tls_cache_endpoint
                    )),
                )
            })?;

        let address_provider = Arc::new(AddressProvider::new(credential_provider.clone()));

        let background_address_loader = match credential_provider.endpoint_security {
            EndpointSecurity::Tls => {
                log::debug!("spawning address refresh task for TLS endpoint");
                let alive = Arc::new(AtomicBool::new(true));
                let background_address_task = runtime.spawn(refresh_addresses_forever(
                    alive.clone(),
                    address_provider.clone(),
                    Duration::from_secs(30),
                ));
                Some(Arc::new(BackgroundAddressLoader {
                    alive,
                    _join_handle: background_address_task,
                }))
            }
            _ => {
                log::debug!("Skipping address refresh task because the endpoint is overridden");
                None
            }
        };

        Ok(Self {
            credential_provider,
            runtime,
            hostname,
            address_provider,
            _background_address_loader: background_address_loader,
            az_id,
            az_circuit: Default::default(),
            connection_sequence: Default::default(),
            connect_timeout,
        })
    }

    /// Choose the address for the next connection attempt.
    ///
    /// Preference for the configured availability zone is soft: when this client
    /// cannot reach that zone, it moves to the others rather than staying pinned
    /// to endpoints it has just failed to reach. Per-host health is deliberately
    /// not tracked here -- the `/endpoints` API publishes healthy hosts, so that
    /// is the control plane's job. What the control plane cannot see is a fault
    /// visible only from this client, which is what the circuit covers.
    async fn select_address(&self) -> Result<(SocketAddr, AzRouting), ConnectFailure> {
        let mut addresses = self.address_provider.get_addresses();
        if addresses.all().is_empty() {
            if let Err(e) = self.address_provider.try_refresh_addresses().await {
                log::warn!("error refreshing address list: {e:?}");
            }
            addresses = self.address_provider.get_addresses();
        }

        let (candidates, routing) = choose_candidates(
            &addresses,
            self.az_id.as_deref(),
            self.az_circuit.should_widen(),
        );

        let sequence = self
            .connection_sequence
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        round_robin(&candidates, sequence)
            .map(|address| (address, routing))
            .ok_or(ConnectFailure::NoAddresses)
    }

    /// Open and authenticate one connection, classifying any failure by whether
    /// it is evidence about the endpoint.
    async fn establish(
        &self,
        address: SocketAddr,
    ) -> Result<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>, EstablishError>
    {
        let unauthenticated_client = create_protosocket_connection(
            self.credential_provider.clone(),
            self.runtime.clone(),
            address,
            &self.hostname,
        )
        .await
        .map_err(EstablishError::Transport)?;

        authenticate_protosocket_client(
            unauthenticated_client,
            self.credential_provider.clone(),
            0xDEADBEEF,
        )
        .await
        .map_err(|e| classify_auth_failure(address, e))
    }
}

/// Narrow the published addresses down to the ones worth trying, and report
/// where they came from.
///
/// Split out from [`ProtosocketConnectionManager::select_address`] so it can be
/// exercised against a fixed address map.
fn choose_candidates(
    addresses: &Addresses,
    az_id: Option<&str>,
    should_widen: bool,
) -> (Vec<SocketAddr>, AzRouting) {
    let (candidates, routing) = match az_id {
        None => (addresses.all(), AzRouting::Unpinned),

        // The local zone is unreachable. Escape it entirely rather than widening
        // to every address, which would keep offering the very endpoints we are
        // trying to avoid.
        Some(az_id) if should_widen => (addresses.outside_az(az_id), AzRouting::Fallback),

        Some(az_id) => {
            let local = addresses.in_az(az_id);
            if local.is_empty() {
                // Quiet degradation is the failure mode most worth avoiding
                // here. This is usually an availability zone *name* passed where
                // an ID belongs, or a zone with no cache hosts.
                log::warn!(
                    "az_id {az_id} is not present in the address map; connecting without zone preference"
                );
                (addresses.all(), AzRouting::Unpinned)
            } else {
                (local, AzRouting::Local)
            }
        }
    };

    // Escaping the local zone can leave nothing behind, if it is the only zone
    // publishing addresses. A cross-zone connection beats no connection.
    if candidates.is_empty() {
        (addresses.all(), AzRouting::Unpinned)
    } else {
        (candidates, routing)
    }
}

/// Spread connections across the candidates. The candidate lists are sorted, so
/// advancing the sequence reliably lands on a different address.
fn round_robin(candidates: &[SocketAddr], sequence: usize) -> Option<SocketAddr> {
    if candidates.is_empty() {
        return None;
    }
    Some(candidates[sequence % candidates.len()])
}

/// Rejected credentials are a permanent failure that says nothing about the
/// endpoint. A transport error during the handshake means the endpoint itself is
/// suspect, and an unrecognized response means the connection is unusable.
fn classify_auth_failure(address: SocketAddr, error: MomentoError) -> EstablishError {
    match error.inner_error {
        Some(ErrorSource::Protosocket(ProtosocketCacheError::CommandError { cause })) => {
            EstablishError::Rejected(protosocket_rpc::Error::IoFailure(
                std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("authentication rejected by {address}: {}", cause.message),
                )
                .into(),
            ))
        }
        Some(ErrorSource::Protosocket(ProtosocketCacheError::Protosocket { cause })) => {
            EstablishError::Transport(cause)
        }
        other => EstablishError::Transport(protosocket_rpc::Error::IoFailure(
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unexpected authentication response from {address}: {other:?}"),
            )
            .into(),
        )),
    }
}

/// Resolve a `host:port` endpoint to a single socket address.
///
/// Resolution failures are reported as `AddrNotAvailable` rather than
/// `InvalidInput`: this call performs DNS, and a resolver blip is transient and
/// worth retrying, whereas a genuinely malformed endpoint is indistinguishable
/// from here. Erring toward retryable is the safer default.
fn resolve_endpoint(endpoint: &str) -> protosocket_rpc::Result<SocketAddr> {
    endpoint
        .to_socket_addrs()
        .map_err(|e| {
            protosocket_rpc::Error::IoFailure(
                std::io::Error::new(
                    std::io::ErrorKind::AddrNotAvailable,
                    format!("could not resolve endpoint {endpoint}: {e:?}"),
                )
                .into(),
            )
        })?
        .next()
        .ok_or_else(|| {
            protosocket_rpc::Error::IoFailure(
                std::io::Error::new(
                    std::io::ErrorKind::AddrNotAvailable,
                    format!("endpoint {endpoint} did not resolve to any address"),
                )
                .into(),
            )
        })
}

impl ClientConnector for ProtosocketConnectionManager {
    type Request = CacheCommand;
    type Response = CacheResponse;

    async fn connect(
        self,
    ) -> protosocket_rpc::Result<protosocket_rpc::client::RpcClient<Self::Request, Self::Response>>
    {
        let (address, routing) = match self.credential_provider.endpoint_security {
            EndpointSecurity::Tls if self.credential_provider.use_endpoints_http_api => {
                log::debug!("selecting address from address provider for TLS endpoint");
                self.select_address().await?
            }
            EndpointSecurity::Tls => {
                // Connect through the load balancer, which hides which zone the
                // backend is in -- so zone preference does not apply here.
                // Use the modified cache_endpoint with :9004 appended and https:// prefix removed
                let mut cache_endpoint = self
                    .credential_provider
                    .cache_endpoint
                    .strip_prefix("https://")
                    .unwrap_or(&self.credential_provider.cache_endpoint)
                    .to_string();
                cache_endpoint.push_str(":9004");

                (resolve_endpoint(&cache_endpoint)?, AzRouting::Unpinned)
            }
            _ => {
                log::debug!("using endpoint address directly for endpoint override");
                (
                    resolve_endpoint(&self.credential_provider.cache_endpoint)?,
                    AzRouting::Unpinned,
                )
            }
        };

        log::debug!("connecting over protosocket to {address} ({routing:?})");

        // A hang has to become a failure, or none of the bookkeeping below ever
        // runs. See `Configuration::set_connect_timeout`.
        let outcome = match tokio::time::timeout(self.connect_timeout, self.establish(address))
            .await
        {
            Ok(outcome) => outcome,
            Err(_elapsed) => {
                log::warn!(
                    "connect to {address} timed out after {:?}",
                    self.connect_timeout
                );
                Err(EstablishError::Transport(
                    protosocket_rpc::Error::IoFailure(
                        std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            format!("connect to {} exceeded {:?}", address, self.connect_timeout),
                        )
                        .into(),
                    ),
                ))
            }
        };

        match outcome {
            Ok(client) => {
                if routing == AzRouting::Local {
                    self.az_circuit.record_local_success();
                }
                log::debug!("successfully created and authenticated protosocket client");
                Ok(client)
            }
            Err(failure) => {
                // Only a transport failure against the local zone is evidence
                // that the zone is unreachable.
                if routing == AzRouting::Local && matches!(failure, EstablishError::Transport(_)) {
                    self.az_circuit.record_local_failure();
                }
                Err(failure.into_inner())
            }
        }
    }
}

async fn refresh_addresses_forever(
    alive: Arc<AtomicBool>,
    address_provider: Arc<AddressProvider>,
    interval: Duration,
) {
    let mut interval = tokio::time::interval(interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        if !alive.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        match address_provider.try_refresh_addresses().await {
            Ok(_) => {
                log::trace!("successfully refreshed address list");
            }
            Err(e) => {
                log::warn!("error refreshing address list: {e:?}");
            }
        }
    }
}

/// Returns `protosocket_rpc::Result` rather than `MomentoResult` so that the
/// `io::ErrorKind` from the transport survives. Converting to `MomentoError` and
/// back would force the error through a string, and callers need the kind to
/// tell a refused connection from a timeout from an unresolvable address.
async fn create_protosocket_connection(
    credential_provider: CredentialProvider,
    runtime: tokio::runtime::Handle,
    address: std::net::SocketAddr,
    hostname: &str,
) -> protosocket_rpc::Result<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>> {
    match credential_provider.endpoint_security {
        EndpointSecurity::Tls | EndpointSecurity::TlsOverride => {
            log::debug!("creating TLS connection to {address}");
            let server_name = server_name_for(hostname)?;
            let connector = WebpkiTlsStreamConnector::new(server_name);
            log::debug!("created TLS connector for server name: {}", hostname);
            create_connection_with_connector(address, connector, runtime).await
        }
        EndpointSecurity::Unverified => {
            log::debug!("creating unverified TLS connection to {address}");
            let server_name = server_name_for(hostname)?;
            let connector = UnverifiedTlsStreamConnector::new(server_name);
            create_connection_with_connector(address, connector, runtime).await
        }
        EndpointSecurity::Insecure => {
            log::debug!("creating tcp connection to {address}");
            // TODO: seems to hang when credential provider uses insecure endpoint with server expecting one of the other options,
            // probably dropping an error or need to set a timeout somewhere
            let connector = TcpStreamConnector;
            create_connection_with_connector(address, connector, runtime).await
        }
    }
}

/// Build the TLS server name for a hostname.
///
/// An unusable hostname is a configuration problem rather than a transport one,
/// so it is reported as `InvalidInput` and maps to a non-retryable error.
fn server_name_for(hostname: &str) -> protosocket_rpc::Result<ServerName<'static>> {
    ServerName::try_from(hostname.to_string()).map_err(|e| {
        protosocket_rpc::Error::IoFailure(
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("could not build a TLS server name from hostname {hostname}: {e:?}"),
            )
            .into(),
        )
    })
}

async fn create_connection_with_connector<C>(
    address: std::net::SocketAddr,
    connector: C,
    runtime: tokio::runtime::Handle,
) -> protosocket_rpc::Result<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>>
where
    C: protosocket_rpc::client::StreamConnector + Send + 'static,
{
    log::debug!("connector: {:?}", connector);
    let (client, connection) = protosocket_rpc::client::connect::<Codec, C>(
        address,
        &protosocket_rpc::client::Configuration::new(connector),
    )
    .await?;
    log::debug!("created protosocket client connection");

    // SDK expects to be run on a Tokio runtime, so we can go ahead and spawn a driver
    // task into the provided Tokio runtime to continually process protosocket requests.
    runtime.spawn(connection);

    log::info!("created connection and spawned driver task");

    Ok(client)
}

pub(crate) async fn authenticate_protosocket_client(
    client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
    credential_provider: CredentialProvider,
    message_id: u64,
) -> MomentoResult<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>> {
    let completion = client.send_unary(CacheCommand {
        message_id,
        control_code: ProtosocketControlCode::Normal as u32,
        rpc_kind: Some(RpcKind::Unary(Unary {
            command: Some(Command::Auth(AuthenticateCommand {
                token: credential_provider.clone().auth_token,
            })),
        })),
    })?;
    let response = completion.await?;
    match response.kind {
        Some(Kind::Auth(AuthenticateResponse {})) => {
            log::info!("authenticated protosocket client!");
            Ok(client)
        }
        Some(Kind::Error(error)) => Err(MomentoError::protosocket_command_error(error)),
        _ => Err(MomentoError::protosocket_unexpected_kind_error()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn address(last_octet: u8) -> SocketAddr {
        SocketAddr::from(([10, 0, 0, last_octet], 9004))
    }

    /// Built by deserializing the `/endpoints` wire format, so these exercise
    /// the real shape rather than a hand-assembled one.
    fn addresses(json: &str) -> Addresses {
        serde_json::from_str(json).expect("test fixture must parse")
    }

    /// usw2-az1 holds .1 and .2; usw2-az2 holds .3.
    fn two_zones() -> Addresses {
        addresses(
            r#"{
                "usw2-az1": [
                    {"socket_address": "10.0.0.1:9004"},
                    {"socket_address": "10.0.0.2:9004"}
                ],
                "usw2-az2": [{"socket_address": "10.0.0.3:9004"}]
            }"#,
        )
    }

    fn one_zone() -> Addresses {
        addresses(r#"{"usw2-az1": [{"socket_address": "10.0.0.1:9004"}]}"#)
    }

    #[test]
    fn a_reachable_local_zone_is_preferred() {
        let (candidates, routing) = choose_candidates(&two_zones(), Some("usw2-az1"), false);

        assert_eq!(candidates, vec![address(1), address(2)]);
        assert_eq!(routing, AzRouting::Local);
    }

    #[test]
    fn an_unreachable_local_zone_is_escaped_entirely() {
        let (candidates, routing) = choose_candidates(&two_zones(), Some("usw2-az1"), true);

        // Not merely widened: the local addresses must not reappear, or we would
        // keep offering the endpoints we are trying to avoid.
        assert_eq!(candidates, vec![address(3)]);
        assert_eq!(routing, AzRouting::Fallback);
    }

    #[test]
    fn escaping_the_only_zone_falls_back_rather_than_failing() {
        let (candidates, routing) = choose_candidates(&one_zone(), Some("usw2-az1"), true);

        assert_eq!(candidates, vec![address(1)]);
        assert_eq!(routing, AzRouting::Unpinned);
    }

    #[test]
    fn an_unknown_zone_connects_without_preference() {
        // An availability zone *name* where an ID belongs is the likely cause,
        // and it must not strand the client with nothing to connect to.
        let (candidates, routing) = choose_candidates(&two_zones(), Some("us-west-2a"), false);

        assert_eq!(candidates, vec![address(1), address(2), address(3)]);
        assert_eq!(routing, AzRouting::Unpinned);
    }

    #[test]
    fn no_configured_zone_uses_every_address() {
        let (candidates, routing) = choose_candidates(&two_zones(), None, false);

        assert_eq!(candidates, vec![address(1), address(2), address(3)]);
        assert_eq!(routing, AzRouting::Unpinned);
    }

    #[test]
    fn the_circuit_is_ignored_without_a_configured_zone() {
        let (candidates, routing) = choose_candidates(&two_zones(), None, true);

        assert_eq!(candidates, vec![address(1), address(2), address(3)]);
        assert_eq!(routing, AzRouting::Unpinned);
    }

    #[test]
    fn an_empty_address_map_yields_no_candidates() {
        let (candidates, _) = choose_candidates(&Addresses::default(), Some("usw2-az1"), false);

        assert!(candidates.is_empty());
        assert_eq!(round_robin(&candidates, 0), None);
    }

    #[test]
    fn round_robin_advances_with_the_sequence() {
        let candidates = vec![address(1), address(2), address(3)];

        assert_eq!(round_robin(&candidates, 0), Some(address(1)));
        assert_eq!(round_robin(&candidates, 1), Some(address(2)));
        assert_eq!(round_robin(&candidates, 2), Some(address(3)));
        assert_eq!(round_robin(&candidates, 3), Some(address(1)));
    }

    #[test]
    fn a_connect_failure_is_typed_as_unavailable_and_retryable() {
        let error: protosocket_rpc::Error = ConnectFailure::NoAddresses.into();

        match &error {
            protosocket_rpc::Error::IoFailure(io_error) => {
                assert_eq!(io_error.kind(), std::io::ErrorKind::AddrNotAvailable)
            }
            other => panic!("expected an IoFailure, got {:?}", other),
        }
        assert_eq!(
            MomentoError::protosocket_connect_error(error).error_code,
            crate::MomentoErrorCode::ServerUnavailable
        );
    }
}
