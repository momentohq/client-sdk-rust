use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::CredentialProvider;

/// Bounds the TCP+TLS handshake for the `/endpoints` control-plane request.
const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

/// Bounds the entire `/endpoints` request, connect through response body.
const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub(crate) struct Addresses {
    #[serde(flatten)]
    azs: HashMap<AzId, Vec<Address>>,
}

impl Addresses {
    /// Addresses published for one availability zone ID. Empty if the zone
    /// isn't in the map -- that's a real signal, not a reason to widen.
    pub fn in_az(&self, az_id: &str) -> Vec<SocketAddr> {
        self.azs
            .get(&AzId(az_id.to_string()))
            .map(|addresses| addresses.iter().map(|a| a.socket_address).collect())
            .unwrap_or_default()
    }

    /// Every published address, across all availability zones.
    pub fn all(&self) -> Vec<SocketAddr> {
        self.sorted(self.azs.values().flatten())
    }

    /// Every published address outside the given zone. Unlike [`all`](Self::all),
    /// doesn't keep offering the addresses we're trying to escape.
    pub fn outside_az(&self, az_id: &str) -> Vec<SocketAddr> {
        let local = AzId(az_id.to_string());
        self.sorted(
            self.azs
                .iter()
                .filter(|(zone, _)| **zone != local)
                .flat_map(|(_, addresses)| addresses),
        )
    }

    /// Collect addresses in a stable order. `HashMap` iteration order varies
    /// call to call, and round-robin selection indexes into this list, so an
    /// undefined order would stop successive connects from rotating.
    fn sorted<'a>(&self, addresses: impl Iterator<Item = &'a Address>) -> Vec<SocketAddr> {
        let mut addresses: Vec<SocketAddr> = addresses.map(|a| a.socket_address).collect();
        addresses.sort_unstable();
        addresses
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
            .connect_timeout(HTTP_CONNECT_TIMEOUT)
            .timeout(HTTP_REQUEST_TIMEOUT)
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
                } else {
                    format!(
                        "{}/endpoints",
                        self.credential_provider
                            .cache_http_endpoint
                            .trim_end_matches('/')
                    )
                };
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

#[cfg(test)]
mod tests {
    use super::*;

    fn address(last_octet: u8) -> SocketAddr {
        SocketAddr::from(([10, 0, 0, last_octet], 9004))
    }

    /// Two zones: usw2-az1 holds .1 and .2, usw2-az2 holds .3.
    fn addresses() -> Addresses {
        Addresses {
            azs: HashMap::from([
                (
                    AzId("usw2-az1".to_string()),
                    vec![
                        Address {
                            socket_address: address(1),
                        },
                        Address {
                            socket_address: address(2),
                        },
                    ],
                ),
                (
                    AzId("usw2-az2".to_string()),
                    vec![Address {
                        socket_address: address(3),
                    }],
                ),
            ]),
        }
    }

    #[test]
    fn in_az_returns_only_that_zone() {
        assert_eq!(addresses().in_az("usw2-az1"), vec![address(1), address(2)]);
        assert_eq!(addresses().in_az("usw2-az2"), vec![address(3)]);
    }

    #[test]
    fn in_az_is_empty_for_an_unknown_zone() {
        // Covers an AZ *name* passed where an ID belongs, among other cases.
        assert!(addresses().in_az("us-west-2a").is_empty());
        assert!(addresses().in_az("usw2-az9").is_empty());
    }

    #[test]
    fn all_returns_every_zone() {
        assert_eq!(addresses().all(), vec![address(1), address(2), address(3)]);
    }

    #[test]
    fn outside_az_excludes_the_local_zone() {
        assert_eq!(addresses().outside_az("usw2-az1"), vec![address(3)]);
        assert_eq!(
            addresses().outside_az("usw2-az2"),
            vec![address(1), address(2)]
        );
    }

    #[test]
    fn outside_az_is_empty_when_there_is_nowhere_else_to_go() {
        let single_zone = Addresses {
            azs: HashMap::from([(
                AzId("usw2-az1".to_string()),
                vec![Address {
                    socket_address: address(1),
                }],
            )]),
        };
        assert!(single_zone.outside_az("usw2-az1").is_empty());
    }

    #[test]
    fn outside_az_returns_everything_for_an_unknown_zone() {
        assert_eq!(
            addresses().outside_az("usw2-az9"),
            vec![address(1), address(2), address(3)]
        );
    }

    #[test]
    fn ordering_is_stable_across_calls() {
        let addresses = addresses();
        let first = addresses.all();
        for _ in 0..20 {
            assert_eq!(addresses.all(), first);
        }
    }

    #[test]
    fn empty_map_yields_nothing() {
        let empty = Addresses::default();
        assert!(empty.all().is_empty());
        assert!(empty.in_az("usw2-az1").is_empty());
        assert!(empty.outside_az("usw2-az1").is_empty());
    }
}
