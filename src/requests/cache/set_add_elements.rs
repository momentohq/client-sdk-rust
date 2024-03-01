use crate::cache_client::CacheClient;
use crate::requests::cache::MomentoRequest;
use crate::simple_cache_client::prep_request;
use crate::{CollectionTtl, IntoBytes, MomentoResult};
use momento_protos::cache_client::SetUnionRequest;

/// ```
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use std::time::Duration;
/// use momento::{CredentialProviderBuilder};
/// use momento::requests::cache::set_add_elements::SetAddElements;
///
/// let credential_provider = CredentialProviderBuilder::from_environment_variable("MOMENTO_API_KEY".to_string())
///     .build()?;
/// let cache_name = "cache";
///
/// let cache_client = momento::CacheClient::new(credential_provider, Duration::from_secs(5))?;
///
/// let set_add_elements_response = cache_client.set_add_elements(cache_name.to_string(), "set", vec!["element1", "element2"]).await?;
/// assert_eq!(set_add_elements_response, SetAddElements {});
/// # Ok(())
/// # })
/// #
/// }
/// ```
pub struct SetAddElementsRequest<S: IntoBytes, E: IntoBytes> {
    cache_name: String,
    set_name: S,
    elements: Vec<E>,
    collection_ttl: Option<CollectionTtl>,
}

impl<S: IntoBytes, E: IntoBytes> SetAddElementsRequest<S, E> {
    pub fn new(cache_name: String, set_name: S, elements: Vec<E>) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name,
            set_name,
            elements,
            collection_ttl: Some(collection_ttl),
        }
    }
}

impl<S: IntoBytes, E: IntoBytes> MomentoRequest for SetAddElementsRequest<S, E> {
    type Response = SetAddElements;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SetAddElements> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let elements = self.elements.into_iter().map(|e| e.into_bytes()).collect();
        let set_name = self.set_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request(
            cache_name,
            SetUnionRequest {
                set_name,
                elements,
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
            },
        )?;

        let _ = cache_client.data_client.clone().set_union(request).await?;
        Ok(SetAddElements {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SetAddElements {}
