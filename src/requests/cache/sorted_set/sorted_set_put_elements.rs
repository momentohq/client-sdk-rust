use momento_protos::cache_client::{SortedSetElement, SortedSetPutRequest};

use crate::requests::cache::MomentoRequest;
use crate::simple_cache_client::prep_request_with_timeout;
use crate::{CacheClient, CollectionTtl, IntoBytes, MomentoResult};

/// Request to add elements to a sorted set. If an element already exists, its score is updated.
/// Creates the sorted set if it does not exist.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the sorted set.
/// * `sorted_set_name` - The name of the sorted set ot add an element to.
/// * `elements` - The values and scores to add. The values must be able to be converted to a `Vec<u8>`.
///
/// # Optional Arguments
///
/// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::CollectionTtl;
/// use momento::requests::cache::sorted_set::sorted_set_put_elements::SortedSetPutElements;
/// use momento::requests::cache::sorted_set::sorted_set_put_elements::SortedSetPutElementsRequest;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let sorted_set_name = "sorted_set";
///
/// let put_elements_request = SortedSetPutElementsRequest::new(
///     cache_name.to_string(),
///     sorted_set_name.to_string(),
///     vec![("value1", 1.0), ("value2", 2.0)]
/// ).with_ttl(CollectionTtl::default());
///
/// let put_elements_response = cache_client.send_request(put_elements_request).await?;
///
/// assert_eq!(put_elements_response, SortedSetPutElements {});
/// # Ok(())
/// # })
/// # }
pub struct SortedSetPutElementsRequest<S: IntoBytes, E: IntoBytes> {
    cache_name: String,
    sorted_set_name: S,
    elements: Vec<(E, f64)>,
    collection_ttl: Option<CollectionTtl>,
}

impl<S: IntoBytes, E: IntoBytes> SortedSetPutElementsRequest<S, E> {
    pub fn new(cache_name: impl Into<String>, sorted_set_name: S, elements: Vec<(E, f64)>) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            elements,
            collection_ttl: Some(collection_ttl),
        }
    }

    /// Set the time-to-live for the collection.
    pub fn with_ttl(mut self, collection_ttl: CollectionTtl) -> Self {
        self.collection_ttl = Some(collection_ttl);
        self
    }
}

impl<S: IntoBytes, E: IntoBytes> MomentoRequest for SortedSetPutElementsRequest<S, E> {
    type Response = SortedSetPutElements;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetPutElements> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let elements = self
            .elements
            .into_iter()
            .map(|e| SortedSetElement {
                value: e.0.into_bytes(),
                score: e.1,
            })
            .collect();
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            SortedSetPutRequest {
                set_name,
                elements,
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
            },
        )?;

        let _ = cache_client
            .data_client
            .clone()
            .sorted_set_put(request)
            .await?;
        Ok(SortedSetPutElements {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SortedSetPutElements {}
