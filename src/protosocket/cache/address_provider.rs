use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use crate::CredentialProvider;

// Todo: should make the connections try to balance better than random
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
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Eq, PartialEq, Hash)]
pub(crate) struct AzId(String);

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Address {
    socket_address: SocketAddr,
}

#[derive(Debug)]
pub(crate) struct AddressProvider {
    addresses: Mutex<Arc<Addresses>>,
    client: reqwest::Client,
    credential_provider: CredentialProvider,
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

impl AddressProvider {
    /// Looks for an address list from the provided endpoint.
    #[allow(clippy::expect_used)]
    pub fn new(credential_provider: CredentialProvider) -> Self {
        let client = reqwest::Client::builder()
            .tls_built_in_native_certs(true)
            .tls_built_in_root_certs(true)
            .build()
            .expect("must be able to build client");
        Self {
            addresses: Default::default(),
            client,
            credential_provider,
        }
    }

    #[allow(clippy::expect_used)]
    pub fn get_addresses(&self) -> impl std::ops::Deref<Target = Addresses> {
        self.addresses
            .lock()
            .expect("local mutex must not be poisoned")
            .clone()
    }

    #[allow(clippy::expect_used)]
    pub async fn try_refresh_addresses(&self) -> Result<(), RefreshError> {
        match self.credential_provider.endpoint_security {
            crate::credential_provider::EndpointSecurity::Tls => {
                log::debug!(
                    "refreshing address list with private endpoints? {}",
                    self.credential_provider.use_private_endpoints
                );
                let url = if self.credential_provider.use_private_endpoints {
                    format!(
                        "{}/endpoints?private=true",
                        self.credential_provider
                            .cache_http_endpoint
                            .trim_end_matches('/')
                    )
                } else if self.credential_provider.use_endpoints {
                    format!(
                        "{}/endpoints",
                        self.credential_provider
                            .cache_http_endpoint
                            .trim_end_matches('/')
                    )
                } else {
                    let mut url = self
                        .credential_provider
                        .cache_endpoint
                        .trim_end_matches('/')
                        .to_string();
                    url.push_str(":9004");
                    url
                };
                println!("Using API URL: {}", url);
                let request = self
                    .client
                    .get(url)
                    .header("authorization", &self.credential_provider.auth_token)
                    .build()?;
                let response = self.client.execute(request).await?;

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().await?;
                    return Err(RefreshError::BadStatus((status, text)));
                }

                let response = response.text().await?;
                let addresses = match serde_json::from_str(&response) {
                    Ok(addresses) => addresses,
                    Err(e) => {
                        log::warn!("error parsing address list JSON: {response}");
                        return Err(RefreshError::Json(e));
                    }
                };
                log::debug!("refreshed address list: {addresses:?}");
                let addresses = Arc::new(addresses);
                *self
                    .addresses
                    .lock()
                    .expect("local mutex must not be poisoned") = addresses;
            }
            _ => {
                log::debug!("skipping address refresh for non-TLS endpoint");
            }
        }
        Ok(())
    }
}
