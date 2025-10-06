use std::{
    convert::TryFrom,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc,
    },
    time::Duration,
};

use crate::{
    credential_provider::EndpointSecurity,
    protosocket::cache::{address_provider::AddressProvider, cache_client_builder::Serializer},
    CredentialProvider,
};
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

use crate::{MomentoError, MomentoResult};

#[derive(Clone, Debug)]
pub(crate) struct ProtosocketConnectionManager {
    credential_provider: CredentialProvider,
    runtime: tokio::runtime::Handle,
    hostname: String,
    address_provider: Arc<AddressProvider>,
    alive: Arc<AtomicBool>,
    az_id: Option<String>,
    connection_sequence: Arc<AtomicUsize>,
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
    ) -> MomentoResult<Self> {
        let endpoint = match http::Uri::from_str(&credential_provider.cache_endpoint) {
            Ok(endpoint) => endpoint,
            Err(e) => {
                return Err(MomentoError::unknown_error(
                    "protosocket_connection_manager::new",
                    Some(format!(
                        "Error parsing endpoint URI from credential provider: {}: {:?}",
                        &credential_provider.cache_endpoint, e
                    )),
                ));
            }
        };

        let hostname = match endpoint.host() {
            Some(hostname) => hostname.to_string(),
            None => {
                return Err(MomentoError::unknown_error(
                    "protosocket_connection_manager::new",
                    Some(format!(
                        "Could not extract hostname from endpoint URI: {}",
                        &credential_provider.cache_endpoint
                    )),
                ));
            }
        };

        let address_provider = Arc::new(AddressProvider::new(credential_provider.clone()));
        let alive = Arc::new(AtomicBool::new(true));

        runtime.spawn(refresh_addresses_forever(
            alive.clone(),
            address_provider.clone(),
            Duration::from_secs(30),
        ));

        Ok(Self {
            credential_provider,
            runtime,
            hostname,
            address_provider,
            alive,
            az_id,
            connection_sequence: Default::default(),
        })
    }
}

impl Drop for ProtosocketConnectionManager {
    fn drop(&mut self) {
        if self.alive.swap(false, std::sync::atomic::Ordering::Relaxed) {
            log::info!("shutting down address refresher task");
        }
    }
}

impl ClientConnector for ProtosocketConnectionManager {
    type Request = CacheCommand;
    type Response = CacheResponse;

    async fn connect(
        self,
    ) -> protosocket_rpc::Result<protosocket_rpc::client::RpcClient<Self::Request, Self::Response>>
    {
        if self
            .address_provider
            .get_addresses()
            .for_az(self.az_id.as_deref())
            .is_empty()
        {
            if let Err(e) = self.address_provider.try_refresh_addresses().await {
                log::warn!("error refreshing address list: {e:?}");
            }
        }
        let addresses = self
            .address_provider
            .get_addresses()
            .for_az(self.az_id.as_deref());
        if addresses.is_empty() {
            return Err(protosocket_rpc::Error::IoFailure(
                std::io::Error::new(
                    std::io::ErrorKind::AddrNotAvailable,
                    "No addresses available from address provider",
                )
                .into(),
            ));
        }
        let address = addresses[self
            .connection_sequence
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % addresses.len()];
        log::debug!("connecting over protosocket to {address}");

        let unauthenticated_client = create_protosocket_connection(
            self.credential_provider.clone(),
            self.runtime.clone(),
            address,
            &self.hostname,
        )
        .await
        .map_err(|e| {
            protosocket_rpc::Error::IoFailure(
                std::io::Error::other(format!("could not connect {e:?}")).into(),
            )
        })?;
        let client = authenticate_protosocket_client(
            unauthenticated_client,
            self.credential_provider.clone(),
            0xDEADBEEF,
        )
        .await
        .map_err(|e| {
            protosocket_rpc::Error::IoFailure(
                std::io::Error::other(format!("could not authenticate {e:?}")).into(),
            )
        })?;
        Ok(client)
    }
}

async fn refresh_addresses_forever(
    alive: Arc<AtomicBool>,
    address_provider: Arc<AddressProvider>,
    interval: std::time::Duration,
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

async fn create_protosocket_connection(
    credential_provider: CredentialProvider,
    runtime: tokio::runtime::Handle,
    address: std::net::SocketAddr,
    hostname: &str,
) -> MomentoResult<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>> {
    match credential_provider.endpoint_security {
        EndpointSecurity::Tls => {
            log::debug!("creating TLS connection to {address}");
            let server_name = ServerName::try_from(hostname.to_string()).map_err(|_| {
                MomentoError::unknown_error(
                    "build",
                    Some(format!(
                        "Error creating server name from hostname: {}",
                        hostname
                    )),
                )
            })?;
            let connector = WebpkiTlsStreamConnector::new(server_name);
            create_connection_with_connector(address, connector, runtime).await
        }
        EndpointSecurity::Unverified => {
            log::debug!("creating unverified TLS connection to {address}");
            let server_name = ServerName::try_from(hostname.to_string()).map_err(|_| {
                MomentoError::unknown_error(
                    "build",
                    Some(format!(
                        "Error creating server name from hostname: {}",
                        hostname
                    )),
                )
            })?;
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

async fn create_connection_with_connector<C>(
    address: std::net::SocketAddr,
    connector: C,
    runtime: tokio::runtime::Handle,
) -> MomentoResult<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>>
where
    C: protosocket_rpc::client::StreamConnector + Send + 'static,
{
    let (client, connection) = protosocket_rpc::client::connect::<Serializer, Serializer, C>(
        address,
        &protosocket_rpc::client::Configuration::new(connector),
    )
    .await?;

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
    let completion = client
        .send_unary(CacheCommand {
            message_id,
            control_code: ProtosocketControlCode::Normal as u32,
            rpc_kind: Some(RpcKind::Unary(Unary {
                command: Some(Command::Auth(AuthenticateCommand {
                    token: credential_provider.clone().auth_token,
                })),
            })),
        })
        .await?;
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
