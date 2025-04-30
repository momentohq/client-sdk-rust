use derive_more::Display;

use momento_protos::cache_client::sorted_set_union_store_request;

use crate::cache::{CollectionTtl, MomentoRequest};
use crate::utils::prep_request_with_timeout;
use crate::{CacheClient, IntoBytes, MomentoResult};

/// Aggregate function to determine the final score for an element that exists in multiple source sets.
#[derive(Debug, Display, PartialEq, Eq, Clone)]
pub enum SortedSetAggregateFunction {
    /// Sum the weighted scores of an element across all the source sets. This is the default.
    Sum = 0,
    /// Use the minimum of the weight scores of an element across all the source sets.
    Min = 1,
    /// Use the maximum of the weight scores of an element across all the source sets.
    Max = 2,
}

/// A source for a sorted set union store request.
#[derive(Debug, PartialEq, Clone)]
pub struct SortedSetUnionStoreSource<S: IntoBytes> {
    sorted_set_name: S,
    weight: f32,
}

impl<S: IntoBytes> SortedSetUnionStoreSource<S> {
    /// Constructs a new SortedSetUnionStoreSource.
    pub fn new(sorted_set_name: impl Into<S>, weight: f32) -> Self {
        Self {
            sorted_set_name: sorted_set_name.into(),
            weight,
        }
    }

    /// Get the weight of the source. The weight is a multiplier applied to the score of each
    /// element in the set before aggregation. Negative and zero weights are allowed.
    pub fn weight(self) -> f32 {
        self.weight
    }

    /// Get the sorted set name of the source.
    pub fn sorted_set_name(self) -> S {
        self.sorted_set_name
    }
}

/// Compute the union of multiple sorted sets and store the result in a destination sorted set.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `sorted_set_name` - name of the destination sorted set. This set is not implicitly included as a source.
/// * `sources` - the sorted sets to compute the union for.
///
/// # Optional Arguments
/// * `aggregate` - the aggregate function to use to determine the final score for an element that exists in multiple source sets. Defaults to [SortedSetAggregateFunction::Sum].
/// * `collection_ttl` - the time-to-live for the collection. If not provided, the client's default time-to-live is used.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{
///     SortedSetUnionStoreResponse, SortedSetUnionStoreRequest, CollectionTtl,
///     SortedSetAggregateFunction, SortedSetUnionStoreSource
/// };
/// # let (cache_client, cache_name) = create_doctest_cache_client();
///
/// let destination_sorted_set_name = "sorted_set";
/// let sources = vec![
///     SortedSetUnionStoreSource::new("one_sorted_set", 1.0),
///     SortedSetUnionStoreSource::new("two_sorted_set", 2.0),
/// ];
/// let union_request = SortedSetUnionStoreRequest::new(cache_name, destination_sorted_set_name, sources)
///     .aggregate(SortedSetAggregateFunction::Sum)
///     .ttl(CollectionTtl::default());
///
/// let destination_length: u32 = cache_client.send_request(union_request).await?.into();
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SortedSetUnionStoreRequest<S: IntoBytes> {
    cache_name: String,
    sorted_set_name: S,
    sources: Vec<SortedSetUnionStoreSource<S>>,
    aggregate: SortedSetAggregateFunction,
    collection_ttl: Option<CollectionTtl>,
}

impl<S: IntoBytes> SortedSetUnionStoreRequest<S> {
    /// Constructs a new SortedSetUnionStoreRequest.
    pub fn new(
        cache_name: impl Into<String>,
        sorted_set_name: S,
        sources: impl Into<Vec<SortedSetUnionStoreSource<S>>>,
    ) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            sources: sources.into(),
            aggregate: SortedSetAggregateFunction::Sum,
            collection_ttl: Some(collection_ttl),
        }
    }

    /// Set the aggregate function of the request.
    pub fn aggregate(mut self, aggregate: impl Into<Option<SortedSetAggregateFunction>>) -> Self {
        self.aggregate = aggregate.into().unwrap_or(SortedSetAggregateFunction::Sum);
        self
    }

    /// Set the time-to-live for the collection.
    pub fn ttl(mut self, collection_ttl: impl Into<Option<CollectionTtl>>) -> Self {
        self.collection_ttl = collection_ttl.into();
        self
    }
}

impl<S: IntoBytes> MomentoRequest for SortedSetUnionStoreRequest<S> {
    type Response = SortedSetUnionStoreResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetUnionStoreResponse> {
        let sources: Vec<sorted_set_union_store_request::Source> = self
            .sources
            .into_iter()
            .map(|source| sorted_set_union_store_request::Source {
                set_name: source.sorted_set_name.into_bytes(),
                weight: source.weight,
            })
            .collect();

        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;

        let request = prep_request_with_timeout(
            cache_name,
            cache_client.deadline_millis(),
            momento_protos::cache_client::SortedSetUnionStoreRequest {
                set_name,
                sources,
                aggregate: self.aggregate as i32,
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
            },
        )?;

        let response = cache_client
            .next_data_client()
            .sorted_set_union_store(request)
            .await?;
        Ok(SortedSetUnionStoreResponse {
            length: response.into_inner().length,
        })
    }
}

/// Response for a successful sorted set union store request.
///
/// You can cast the result into a u32 value or access the length field directly.
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SortedSetUnionStoreResponse;
/// # let response = SortedSetUnionStoreResponse { length: 5 };
/// let destination_length: u32 = response.into();
/// # let response = SortedSetUnionStoreResponse { length: 5 };
/// let also_destination_length = response.length;
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct SortedSetUnionStoreResponse {
    /// The number of elements in the destination set after the union.
    /// The length is 0 if the result of the union was an empty set.
    pub length: u32,
}

impl From<SortedSetUnionStoreResponse> for u32 {
    fn from(value: SortedSetUnionStoreResponse) -> Self {
        value.length
    }
}

#[cfg(test)]
mod test {
    use crate::cache::{
        CollectionTtl, SortedSetAggregateFunction, SortedSetUnionStoreRequest,
        SortedSetUnionStoreSource,
    };

    // test the sorted set request with options
    #[tokio::test]
    async fn test_sorted_set_union_store_request_with_options() {
        // Define the cache name and sorted set name
        let cache_name = "test_cache";
        let sorted_set_name = "test_sorted_set";
        let sources = vec![
            SortedSetUnionStoreSource::new("one_sorted_set", 1.0),
            SortedSetUnionStoreSource::new("two_sorted_set", 2.0),
        ];

        // Create the fetch request with options
        let fetch_request =
            SortedSetUnionStoreRequest::new(cache_name, sorted_set_name, sources.clone())
                .aggregate(SortedSetAggregateFunction::Min)
                .ttl(CollectionTtl::default());

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.sources, sources);
        assert_eq!(fetch_request.aggregate, SortedSetAggregateFunction::Min);
        assert_eq!(fetch_request.collection_ttl, Some(CollectionTtl::default()));

        // Now pass in explicit Options to aggregate and ttl
        let fetch_request =
            SortedSetUnionStoreRequest::new(cache_name, sorted_set_name, sources.clone())
                .aggregate(Some(SortedSetAggregateFunction::Max))
                .ttl(Some(CollectionTtl::default()));

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.aggregate, SortedSetAggregateFunction::Max);
        assert_eq!(fetch_request.collection_ttl, Some(CollectionTtl::default()));

        // Now pass in explicit None to aggregate and ttl
        let fetch_request =
            SortedSetUnionStoreRequest::new(cache_name, sorted_set_name, sources.clone())
                .aggregate(None)
                .ttl(None);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.sources, sources);
        assert_eq!(fetch_request.aggregate, SortedSetAggregateFunction::Sum);
        assert_eq!(fetch_request.collection_ttl, None);

        // Pass in no options
        let fetch_request =
            SortedSetUnionStoreRequest::new(cache_name, sorted_set_name, sources.clone());

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.sources, sources);
        assert_eq!(fetch_request.aggregate, SortedSetAggregateFunction::Sum);
        assert_eq!(fetch_request.collection_ttl, Some(CollectionTtl::default()));
    }
}
