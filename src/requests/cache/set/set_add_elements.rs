use momento_protos::cache_client::SetUnionRequest;

use crate::cache_client::CacheClient;
use crate::requests::cache::MomentoRequest;
use crate::simple_cache_client::prep_request_with_timeout;
use crate::{CollectionTtl, IntoBytes, MomentoResult};

/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use momento::requests::cache::set::set_add_elements::SetAddElements;
///
/// let set_add_elements_response = cache_client.set_add_elements(cache_name.to_string(), "set", vec!["element1", "element2"]).await?;
/// assert_eq!(set_add_elements_response, SetAddElements {});
/// # Ok(())
/// # })
/// # }
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

    pub fn with_ttl(mut self, collection_ttl: CollectionTtl) -> Self {
        self.collection_ttl = Some(collection_ttl);
        self
    }
}

impl<S: IntoBytes, E: IntoBytes> MomentoRequest for SetAddElementsRequest<S, E> {
    type Response = SetAddElements;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SetAddElements> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let elements = self.elements.into_iter().map(|e| e.into_bytes()).collect();
        let set_name = self.set_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
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
