use momento_protos::cache_client::{SortedSetIncrementRequest, SortedSetIncrementResponse};

use crate::cache::CollectionTtl;
use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoResult,
};

/// Increments the score of an element in a sorted set.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the sorted set.
/// * `sorted_set_name` - The name of the sorted set to add an element to.
/// * `value` - the sorted set value to get the rank of
/// * `amount` - the amount to increment the score by
///
/// # Optional Arguments
///
/// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento::cache::messages::data::sorted_set::sorted_set_increment_score::SortedSetIncrementScoreRequest;
/// use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{CollectionTtl};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let sorted_set_name = "sorted_set";
///
/// let increment_score_request = SortedSetIncrementScoreRequest::new(
///     cache_name,
///     sorted_set_name,
///     "value",
///     1.0
/// ).ttl(CollectionTtl::default());
///
/// match cache_client.send_request(increment_score_request).await {
///     Ok(res) => println!("Score incremented in sorted set {}", res.score),
///     Err(e) => eprintln!("Error incrementing score to sorted set: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
pub struct SortedSetIncrementScoreRequest<S: IntoBytes, V: IntoBytes> {
    cache_name: String,
    sorted_set_name: S,
    value: V,
    amount: f64,
    collection_ttl: Option<CollectionTtl>,
}

impl<S: IntoBytes, V: IntoBytes> SortedSetIncrementScoreRequest<S, V> {
    /// Constructs a new SortedSetIncrementScoreRequest.
    pub fn new(cache_name: impl Into<String>, sorted_set_name: S, value: V, amount: f64) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            value,
            amount,
            collection_ttl: Some(collection_ttl),
        }
    }

    /// Set the time-to-live for the collection.
    pub fn ttl(mut self, collection_ttl: impl Into<Option<CollectionTtl>>) -> Self {
        self.collection_ttl = collection_ttl.into();
        self
    }
}

impl<S: IntoBytes, V: IntoBytes> MomentoRequest for SortedSetIncrementScoreRequest<S, V> {
    type Response = SortedSetIncrementScoreResponse;

    async fn send(
        self,
        cache_client: &CacheClient,
    ) -> MomentoResult<SortedSetIncrementScoreResponse> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;
        let value = self.value.into_bytes();
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            SortedSetIncrementRequest {
                set_name,
                value,
                amount: self.amount,
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .sorted_set_increment(request)
            .await?;

        let SortedSetIncrementResponse { score } = response.into_inner();
        Ok(SortedSetIncrementScoreResponse { score })
    }
}

/// The incremented score of the item in the sorted set
#[derive(Debug, PartialEq, PartialOrd)]
pub struct SortedSetIncrementScoreResponse {
    /// The score value
    pub score: f64,
}
