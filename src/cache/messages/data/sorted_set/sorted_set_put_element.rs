use momento_protos::cache_client::{SortedSetElement, SortedSetPutRequest};

use crate::cache::messages::MomentoRequest;
use crate::cache::CollectionTtl;
use crate::utils::prep_request_with_timeout;
use crate::{CacheClient, IntoBytes, MomentoResult};

/// Request to add an element to a sorted set. If the element already exists, its score is updated.
/// Creates the sorted set if it does not exist.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the sorted set.
/// * `sorted_set_name` - The name of the sorted set to add an element to.
/// * `value` - The value of the element to add. Must be able to be converted to a `Vec<u8>`.
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
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{CollectionTtl, SortedSetPutElementResponse, SortedSetPutElementRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let sorted_set_name = "sorted_set";
///
/// let put_element_request = SortedSetPutElementRequest::new(
///     cache_name,
///     sorted_set_name,
///     "value",
///     1.0
/// ).ttl(CollectionTtl::default());
///
/// match cache_client.send_request(put_element_request).await {
///     Ok(_) => println!("Element added to sorted set"),
///     Err(e) => eprintln!("Error adding elements to sorted set: {}", e),
/// }
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
    /// Constructs a new SortedSetPutElementRequest.
    pub fn new(cache_name: impl Into<String>, sorted_set_name: S, value: V, score: f64) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            value,
            score,
            collection_ttl: Some(collection_ttl),
        }
    }

    /// Set the time-to-live for the collection.
    pub fn ttl(mut self, collection_ttl: impl Into<Option<CollectionTtl>>) -> Self {
        self.collection_ttl = collection_ttl.into();
        self
    }
}

impl<S: IntoBytes, V: IntoBytes> MomentoRequest for SortedSetPutElementRequest<S, V> {
    type Response = SortedSetPutElementResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetPutElementResponse> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let element = SortedSetElement {
            value: self.value.into_bytes(),
            score: self.score,
        };
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.deadline_millis(),
            SortedSetPutRequest {
                set_name,
                elements: vec![element],
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
            },
        )?;

        let _ = cache_client
            .next_data_client()
            .sorted_set_put(request)
            .await?;
        Ok(SortedSetPutElementResponse {})
    }
}

/// The response type for a successful sorted set put element request.
#[derive(Debug, PartialEq, Eq)]
pub struct SortedSetPutElementResponse {}
