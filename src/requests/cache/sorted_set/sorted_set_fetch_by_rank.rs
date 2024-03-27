use momento_protos::cache_client::sorted_set_fetch_request::{by_index, ByIndex, Range};
use momento_protos::cache_client::{SortedSetFetchRequest, Unbounded};

use crate::requests::cache::sorted_set::sorted_set_fetch_response::SortedSetFetch;
use crate::requests::cache::MomentoRequest;
use crate::simple_cache_client::prep_request_with_timeout;
use crate::{CacheClient, IntoBytes, MomentoResult};

#[repr(i32)]
pub enum SortOrder {
    Ascending = 0,
    Descending = 1,
}

/// Request to fetch the elements in a sorted set by their rank.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the sorted set.
/// * `sorted_set_name` - The name of the sorted set to add an element to.
///
/// # Optional Arguments
///
/// * `order` - The order to sort the elements by. [SortOrder::Ascending] or [SortOrder::Descending].
/// Defaults to Ascending.
/// * `start_rank` - The rank of the first element to fetch. Defaults to 0. This rank is
/// inclusive, i.e. the element at this rank will be fetched.
/// * `end_rank` - The rank of the last element to fetch. This rank is exclusive, i.e. the
/// element at this rank will not be fetched. Defaults to -1, which fetches up until and
/// including the last element.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use std::convert::TryInto;
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::requests::cache::sorted_set::sorted_set_fetch_by_rank::SortOrder;
/// use momento::requests::cache::sorted_set::sorted_set_fetch_by_rank::SortedSetFetchByRankRequest;
/// use momento::requests::cache::sorted_set::sorted_set_fetch_response::SortedSetFetch;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let sorted_set_name = "sorted_set";
///
/// let put_element_response = cache_client.sorted_set_put_elements(
///     cache_name.to_string(),
///     sorted_set_name.to_string(),
///     vec![("value1", 1.0), ("value2", 2.0), ("value3", 3.0), ("value4", 4.0)]
/// ).await?;
///
/// let fetch_request = SortedSetFetchByRankRequest::new(cache_name, sorted_set_name)
///     .with_order(SortOrder::Ascending)
///     .with_start_rank(1)
///     .with_end_rank(3);
///
/// let fetch_response = cache_client.send_request(fetch_request).await?;
///
/// let returned_elements: Vec<(String, f64)> = fetch_response.try_into()
///     .expect("elements 2 and 3 should be returned");
/// println!("{:?}", returned_elements);
/// # Ok(())
/// # })
/// # }
pub struct SortedSetFetchByRankRequest<S: IntoBytes> {
    cache_name: String,
    sorted_set_name: S,
    start_rank: Option<i32>,
    end_rank: Option<i32>,
    order: SortOrder,
}

impl<S: IntoBytes> SortedSetFetchByRankRequest<S> {
    pub fn new(cache_name: impl Into<String>, sorted_set_name: S) -> Self {
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            start_rank: None,
            end_rank: None,
            order: SortOrder::Ascending,
        }
    }

    /// Set the start rank of the request.
    pub fn with_start_rank(mut self, start_rank: i32) -> Self {
        self.start_rank = Some(start_rank);
        self
    }

    /// Set the end rank of the request.
    pub fn with_end_rank(mut self, end_rank: i32) -> Self {
        self.end_rank = Some(end_rank);
        self
    }

    /// Set the order of the request.
    pub fn with_order(mut self, order: SortOrder) -> Self {
        self.order = order;
        self
    }
}

impl<S: IntoBytes> MomentoRequest for SortedSetFetchByRankRequest<S> {
    type Response = SortedSetFetch;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetFetch> {
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;

        let by_index = ByIndex {
            start: Some(
                self.start_rank
                    .map(by_index::Start::InclusiveStartIndex)
                    .unwrap_or_else(|| by_index::Start::UnboundedStart(Unbounded {})),
            ),
            end: Some(
                self.end_rank
                    .map(by_index::End::ExclusiveEndIndex)
                    .unwrap_or_else(|| by_index::End::UnboundedEnd(Unbounded {})),
            ),
        };

        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            SortedSetFetchRequest {
                set_name,
                order: self.order as i32,
                with_scores: true,
                range: Some(Range::ByIndex(by_index)),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .sorted_set_fetch(request)
            .await?
            .into_inner();

        SortedSetFetch::from_fetch_response(response)
    }
}
