use momento_protos::cache_client::{SortedSetElement, SortedSetPutRequest};

use crate::requests::cache::MomentoRequest;
use crate::simple_cache_client::prep_request_with_timeout;
use crate::{CacheClient, CollectionTtl, IntoBytes, MomentoResult};

/// Request to add an element to a sorted set. If the element already exists, its score is updated.
/// Creates the sorted set if it does not exist.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the sorted set.
/// * `sorted_set_name` - The name of the sorted set ot add an element to.
/// * `value` - The value of the element to add. Must be able to be converted to a Vec<u8>.
/// * `score` - The score of the element to add.
///
/// # Optional Arguments
///
/// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_client;
/// # tokio_test::block_on(async {
/// use momento::CollectionTtl;
/// use momento::requests::cache::sorted_set::sorted_set_put_element::SortedSetPutElement;
/// use momento::requests::cache::sorted_set::sorted_set_put_element::SortedSetPutElementRequest;
/// # let (cache_client, cache_name) = create_doctest_client();
/// let sorted_set_name = "sorted_set";
///
/// let put_element_request = SortedSetPutElementRequest::new(
///     cache_name.to_string(),
///     sorted_set_name.to_string(),
///     "value",
///     1.0
/// ).with_ttl(CollectionTtl::default());
///
/// let create_cache_response = cache_client.send_request(put_element_request).await?;
///
/// assert_eq!(create_cache_response, SortedSetPutElement {});
/// # Ok(())
/// # })
/// # }
pub struct SortedSetPutElementRequest<S: IntoBytes, V: IntoBytes> {
    cache_name: String,
    sorted_set_name: S,
    value: V,
    score: f64,
    collection_ttl: Option<CollectionTtl>,
}

impl<S: IntoBytes, V: IntoBytes> SortedSetPutElementRequest<S, V> {
    pub fn new(cache_name: String, sorted_set_name: S, value: V, score: f64) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name,
            sorted_set_name,
            value,
            score,
            collection_ttl: Some(collection_ttl),
        }
    }

    /// Set the time-to-live for the collection.
    pub fn with_ttl(mut self, collection_ttl: CollectionTtl) -> Self {
        self.collection_ttl = Some(collection_ttl);
        self
    }
}

impl<S: IntoBytes, V: IntoBytes> MomentoRequest for SortedSetPutElementRequest<S, V> {
    type Response = SortedSetPutElement;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetPutElement> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let element = SortedSetElement {
            value: self.value.into_bytes(),
            score: self.score,
        };
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            SortedSetPutRequest {
                set_name,
                elements: vec![element],
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
            },
        )?;

        let _ = cache_client
            .data_client
            .clone()
            .sorted_set_put(request)
            .await?;
        Ok(SortedSetPutElement {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SortedSetPutElement {}
