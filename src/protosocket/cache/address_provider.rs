use std::{
    collections::{HashMap, HashSet},
    net::{SocketAddr, ToSocketAddrs},
    sync::{atomic::AtomicBool, Arc, RwLock},
    time::Duration,
};

use crate::{
    credential_provider::EndpointSecurity, CredentialProvider, MomentoError, MomentoResult,
};

const ADDRESS_REFRESH_INTERVAL_SECS: u64 = 30;

#[derive(Debug)]
pub(crate) struct BackgroundAddressLoader {
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

#[derive(Debug, Default)]
pub(crate) struct AddressState {
    addresses: Arc<Addresses>,
    generation: u64,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub(crate) struct Addresses {
    #[serde(flatten)]
    azs: HashMap<AzId, Vec<Address>>,
}

impl Addresses {
    /// Get a list of socket addresses, optionally filtered by availability zone ID.
    /// If an az_id is provided, only addresses in that availability zone will be returned,
    pub fn for_az(&self, az_id: Option<&str>) -> Vec<SocketAddr> {
        if let Some(az_id) = az_id {
            if let Some(addresses) = self.azs.get(&AzId(az_id.to_string())) {
                if !addresses.is_empty() {
                    return addresses.iter().map(|a| a.socket_address).collect();
                }
            }
        }
        self.azs
            .values()
            .flat_map(|addresses| addresses.iter().map(|a| a.socket_address))
            .collect()
    }

    fn all_socket_addresses(&self) -> HashSet<SocketAddr> {
        self.azs
            .values()
            .flat_map(|addrs| addrs.iter().map(|a| a.socket_address))
            .collect()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Eq, PartialEq, Hash)]
pub(crate) struct AzId(String);

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Address {
    socket_address: SocketAddr,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RefreshError {
    Reqwest(#[from] reqwest::Error),
    Json(#[from] serde_json::Error),
    Uri(#[from] http::uri::InvalidUri),
    BadStatus((reqwest::StatusCode, String)),
}
impl std::fmt::Display for RefreshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefreshError::Reqwest(e) => write!(f, "Reqwest error: {e}"),
            RefreshError::Json(e) => write!(f, "JSON error: {e}"),
            RefreshError::Uri(e) => write!(f, "URI error: {e}"),
            RefreshError::BadStatus((status, text)) => write!(f, "Bad status: {status}, {text}"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum AddressProvider {
    Refreshing {
        addresses: Arc<RwLock<AddressState>>,
        _background_loader: BackgroundAddressLoader,
    },
    Static {
        address: SocketAddr,
    },
}

impl AddressProvider {
    pub async fn new(
        credential_provider: CredentialProvider,
        runtime: tokio::runtime::Handle,
    ) -> MomentoResult<Self> {
        if credential_provider.use_endpoints_http_api {
            Self::new_with_refresh(credential_provider, runtime).await
        } else {
            Self::new_with_static_address(credential_provider)
        }
    }

    fn new_with_static_address(credential_provider: CredentialProvider) -> MomentoResult<Self> {
        let address_string =
            if matches!(credential_provider.endpoint_security, EndpointSecurity::Tls) {
                // TLS: make sure https:// is stripped and force port 9004
                let endpoint = credential_provider
                    .cache_endpoint
                    .strip_prefix("https://")
                    .unwrap_or(&credential_provider.cache_endpoint);
                let hostname = endpoint.split(':').next().unwrap_or(endpoint);
                format!("{}:9004", hostname)
            } else {
                // Non-TLS: use as-is, expect proper socket address format
                credential_provider.cache_endpoint.clone()
            };

        let address: SocketAddr = address_string
            .to_socket_addrs()
            .map_err(|e| {
                protosocket_rpc::Error::IoFailure(
                    std::io::Error::other(format!(
                        "could not parse socket address from endpoint: {}: {:?}",
                        &credential_provider.cache_endpoint, e
                    ))
                    .into(),
                )
            })?
            .next()
            .ok_or_else(|| {
                protosocket_rpc::Error::IoFailure(
                    std::io::Error::other(format!(
                        "no addresses resolved for endpoint: {}",
                        &credential_provider.cache_endpoint
                    ))
                    .into(),
                )
            })?;

        Ok(Self::Static { address })
    }

    /// You should make one of these and clone it as needed for connection pools.
    /// It spawns a background task to refresh the address list every 30 seconds.
    /// This manager can be cloned and shared across connection pools.
    async fn new_with_refresh(
        credential_provider: CredentialProvider,
        runtime: tokio::runtime::Handle,
    ) -> MomentoResult<Self> {
        let client = reqwest::Client::builder()
            .tls_built_in_native_certs(true)
            .tls_built_in_root_certs(true)
            .build()
            .map_err(|err| {
                MomentoError::unknown_error(
                    "address_provider::new",
                    Some(format!(
                        "Could not create a client to retrieve addresses: {}",
                        err
                    )),
                )
            })?;

        let addresses = Arc::new(RwLock::new(AddressState::default()));

        try_refresh_addresses(&client, &credential_provider, &addresses)
            .await
            .map_err(|err| {
                MomentoError::unknown_error(
                    "address_provider::new",
                    Some(format!("Could not load addresses: {}", err)),
                )
            })?;

        log::debug!("spawning address refresh task for TLS endpoint");
        let alive = Arc::new(AtomicBool::new(true));

        let task_client = client.clone();
        let task_credential_provider = credential_provider.clone();
        let task_addresses = Arc::clone(&addresses); // Share the same lock

        let join_handle = runtime.spawn(refresh_addresses_forever(
            alive.clone(),
            task_client,
            task_credential_provider,
            task_addresses,
            Duration::from_secs(ADDRESS_REFRESH_INTERVAL_SECS),
        ));

        let background_loader = BackgroundAddressLoader {
            alive,
            _join_handle: join_handle,
        };

        Ok(Self::Refreshing {
            addresses,
            _background_loader: background_loader,
        })
    }

    #[allow(clippy::expect_used)]
    pub fn get_addresses(&self, az_id: Option<&str>) -> Vec<SocketAddr> {
        match &self {
            Self::Refreshing { addresses, .. } => addresses
                .read()
                .expect("address state lock must not be poisoned")
                .addresses
                .clone()
                .for_az(az_id),
            Self::Static { address } => vec![*address],
        }
    }

    pub fn get_generation(&self) -> u64 {
        match &self {
            Self::Refreshing { addresses, .. } => {
                addresses
                    .read()
                    .expect("address state lock must not be poisoned")
                    .generation
            }
            Self::Static { address: _address } => 0,
        }
    }
}

async fn refresh_addresses_forever(
    alive: Arc<AtomicBool>,
    client: reqwest::Client,
    credential_provider: CredentialProvider,
    addresses: Arc<RwLock<AddressState>>,
    interval: Duration,
) {
    let mut interval = tokio::time::interval(interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;
        if !alive.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        match try_refresh_addresses(&client, &credential_provider, &addresses).await {
            Ok(_) => log::trace!("successfully refreshed address list"),
            Err(e) => log::warn!("error refreshing address list: {e:?}"),
        }
    }
}

#[allow(clippy::expect_used)]
async fn try_refresh_addresses(
    client: &reqwest::Client,
    credential_provider: &CredentialProvider,
    addresses: &Arc<RwLock<AddressState>>,
) -> Result<(), RefreshError> {
    log::debug!(
        "refreshing address list with private endpoints? {}",
        credential_provider.use_private_endpoints
    );
    let url = if credential_provider.use_private_endpoints {
        format!(
            "{}/endpoints?private=true",
            credential_provider
                .cache_http_endpoint
                .trim_end_matches('/')
        )
    } else {
        format!(
            "{}/endpoints",
            credential_provider
                .cache_http_endpoint
                .trim_end_matches('/')
        )
    };
    let request = client
        .get(url)
        .header("authorization", &credential_provider.auth_token)
        .build()?;
    let response = client.execute(request).await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await?;
        return Err(RefreshError::BadStatus((status, text)));
    }

    let response = response.text().await?;
    let new_addresses: Addresses = match serde_json::from_str(&response) {
        Ok(addresses) => addresses,
        Err(e) => {
            log::warn!("error parsing address list JSON: {response}");
            return Err(RefreshError::Json(e));
        }
    };
    log::debug!("refreshed address list: {new_addresses:?}");

    let needs_update = {
        let current_state = addresses
            .read()
            .expect("address state lock must not be poisoned");
        current_state.addresses.all_socket_addresses() != new_addresses.all_socket_addresses()
    };

    if needs_update {
        let mut current_state = addresses
            .write()
            .expect("address state lock must not be poisoned");
        let new_generation = current_state.generation + 1;
        *current_state = AddressState {
            addresses: Arc::new(new_addresses),
            generation: new_generation,
        };
        log::debug!("address list changed, new generation: {new_generation}");
    }

    Ok(())
}
