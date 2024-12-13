use momento_protos::cache_client::sorted_set_fetch_request::{by_index, ByIndex, Range};
use momento_protos::cache_client::SortedSetFetchRequest;
use momento_protos::common::Unbounded;

use crate::cache::messages::data::sorted_set::sorted_set_fetch_response::SortedSetFetchResponse;
use crate::cache::messages::MomentoRequest;
use crate::utils::prep_request_with_timeout;
use crate::{CacheClient, IntoBytes, MomentoResult};

/// The order with which to sort the elements by score in the sorted set.
/// The sort order determines the rank of the elements.
/// The elements with same score are ordered lexicographically.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SortedSetOrder {
    /// Scores are ordered from low to high. This is the default order.
    Ascending = 0,
    /// Scores are ordered from high to low.
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
/// * `order` - The order to sort the elements by. [SortedSetOrder::Ascending] or [SortedSetOrder::Descending].
///   Defaults to Ascending.
/// * `start_rank` - The rank of the first element to fetch. Defaults to 0. This rank is
///   inclusive, i.e. the element at this rank will be fetched.
/// * `end_rank` - The rank of the last element to fetch. This rank is exclusive, i.e. the
///   element at this rank will not be fetched. Defaults to -1, which fetches up until and
///   including the last element.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use std::convert::TryInto;
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{SortedSetOrder, SortedSetFetchResponse, SortedSetFetchByRankRequest};
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
///     .order(SortedSetOrder::Ascending)
///     .start_rank(1)
///     .end_rank(3);
///
/// let fetch_response = cache_client.send_request(fetch_request).await?;
///
/// let returned_elements: Vec<(String, f64)> = fetch_response.try_into()
///     .expect("elements 2 and 3 should be returned");
/// println!("{:?}", returned_elements);
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SortedSetFetchByRankRequest<S: IntoBytes> {
    cache_name: String,
    sorted_set_name: S,
    start_rank: Option<i32>,
    end_rank: Option<i32>,
    order: SortedSetOrder,
}

impl<S: IntoBytes> SortedSetFetchByRankRequest<S> {
    /// Constructs a new SortedSetFetchByRankRequest.
    pub fn new(cache_name: impl Into<String>, sorted_set_name: S) -> Self {
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            start_rank: None,
            end_rank: None,
            order: SortedSetOrder::Ascending,
        }
    }

    /// Set the start rank of the request.
    pub fn start_rank(mut self, start_rank: impl Into<Option<i32>>) -> Self {
        self.start_rank = start_rank.into();
        self
    }

    /// Set the end rank of the request.
    pub fn end_rank(mut self, end_rank: impl Into<Option<i32>>) -> Self {
        self.end_rank = end_rank.into();
        self
    }

    /// Set the order of the request.
    pub fn order(mut self, order: impl Into<Option<SortedSetOrder>>) -> Self {
        self.order = order.into().unwrap_or(SortedSetOrder::Ascending);
        self
    }
}

impl<S: IntoBytes> MomentoRequest for SortedSetFetchByRankRequest<S> {
    type Response = SortedSetFetchResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetFetchResponse> {
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
            cache_client.deadline_millis(),
            SortedSetFetchRequest {
                set_name,
                order: self.order as i32,
                with_scores: true,
                range: Some(Range::ByIndex(by_index)),
            },
        )?;

        let response = cache_client
            .next_data_client()
            .sorted_set_fetch(request)
            .await?
            .into_inner();

        SortedSetFetchResponse::from_fetch_response(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorted_set_fetch_by_rank_request_builder() {
        let cache_name = "my_cache";
        let sorted_set_name = "my_sorted_set";

        // Test with explicit values
        let request = SortedSetFetchByRankRequest::new(cache_name, sorted_set_name)
            .order(SortedSetOrder::Ascending)
            .start_rank(1)
            .end_rank(3);

        assert_eq!(request.cache_name, cache_name);
        assert_eq!(
            request.sorted_set_name.into_bytes(),
            sorted_set_name.as_bytes()
        );
        assert_eq!(request.start_rank, Some(1));
        assert_eq!(request.end_rank, Some(3));
        assert_eq!(request.order, SortedSetOrder::Ascending);

        // Test with Some values
        let request = SortedSetFetchByRankRequest::new(cache_name, sorted_set_name)
            .order(SortedSetOrder::Descending)
            .start_rank(Some(2))
            .end_rank(Some(4));

        assert_eq!(request.cache_name, cache_name);
        assert_eq!(
            request.sorted_set_name.into_bytes(),
            sorted_set_name.as_bytes()
        );
        assert_eq!(request.start_rank, Some(2));
        assert_eq!(request.end_rank, Some(4));
        assert_eq!(request.order, SortedSetOrder::Descending);

        // Test with None values
        let request = SortedSetFetchByRankRequest::new(cache_name, sorted_set_name)
            .order(None)
            .start_rank(None)
            .end_rank(None);

        assert_eq!(request.cache_name, cache_name);
        assert_eq!(
            request.sorted_set_name.into_bytes(),
            sorted_set_name.as_bytes()
        );
        assert_eq!(request.start_rank, None);
        assert_eq!(request.end_rank, None);
        assert_eq!(request.order, SortedSetOrder::Ascending);
    }
}
