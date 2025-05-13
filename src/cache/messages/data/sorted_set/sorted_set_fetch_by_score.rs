use momento_protos::cache_client::sorted_set_fetch_request::by_score::Score;
use momento_protos::cache_client::sorted_set_fetch_request::{by_score, ByScore, Range};
use momento_protos::cache_client::SortedSetFetchRequest;
use momento_protos::common::Unbounded;

use crate::cache::messages::data::sorted_set::sorted_set_fetch_by_rank::SortedSetOrder;
use crate::cache::messages::data::sorted_set::sorted_set_fetch_by_rank::SortedSetOrder::Ascending;
use crate::cache::messages::data::sorted_set::sorted_set_fetch_response::SortedSetFetchResponse;
use crate::cache::messages::MomentoRequest;
use crate::utils::prep_request_with_timeout;
use crate::{CacheClient, IntoBytes, MomentoResult};

/// Fetch the elements in the given sorted set by their score.
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
/// * `min_score` - The minimum score (inclusive) of the elements to fetch. Defaults to negative
///   infinity.
/// * `max_score` - The maximum score (inclusive) of the elements to fetch. Defaults to positive
///   infinity.
/// * `exclusive_min_score` - the minimum score (exclusive) of the elements to fetch. Defaults to negative infinity.
/// * `exclusive_max_score` - the maximum score (exclusive) of the elements to fetch. Defaults to positive
///   infinity.
/// * `offset` - The number of elements to skip before returning the first element. Defaults to
/// 0. Note: this is not the rank of the first element to return, but the number of elements of
///    the result set to skip before returning the first element.
/// * `count` - The maximum number of elements to return. Defaults to all elements.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use std::convert::TryInto;
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{SortedSetOrder, SortedSetFetchResponse, SortedSetFetchByScoreRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let sorted_set_name = "sorted_set";
///
/// let put_element_response = cache_client.sorted_set_put_elements(
///     cache_name.to_string(),
///     sorted_set_name.to_string(),
///     vec![("value1", 1.0), ("value2", 2.0), ("value3", 3.0), ("value4", 4.0)]
/// ).await?;
///
/// let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, sorted_set_name)
///     .order(SortedSetOrder::Ascending)
///     .min_score(2.0)
///     .max_score(3.0);
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
pub struct SortedSetFetchByScoreRequest<S: IntoBytes> {
    cache_name: String,
    sorted_set_name: S,
    min_score: Option<f64>,
    min_score_exclusive: Option<bool>,
    max_score: Option<f64>,
    max_score_exclusive: Option<bool>,
    order: SortedSetOrder,
    offset: Option<u32>,
    count: Option<i32>,
}

impl<S: IntoBytes> SortedSetFetchByScoreRequest<S> {
    /// Constructs a new SortedSetFetchByScoreRequest.
    pub fn new(cache_name: impl Into<String>, sorted_set_name: S) -> Self {
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            min_score: None,
            min_score_exclusive: None,
            max_score: None,
            max_score_exclusive: None,
            order: Ascending,
            offset: None,
            count: None,
        }
    }

    /// Set the minimum score of the request.
    pub fn min_score(mut self, min_score: impl Into<Option<f64>>) -> Self {
        if let Some(min) = min_score.into() {
            self.min_score = Some(min);
            self.min_score_exclusive = None;
        }
        self
    }

    /// Set the minimum score of the request.
    pub fn exclusive_min_score(mut self, exclusive_min_score: impl Into<Option<f64>>) -> Self {
        if let Some(min) = exclusive_min_score.into() {
            self.min_score = Some(min);
            self.min_score_exclusive = Some(true);
        }
        self
    }

    /// Set the maximum score of the request.
    pub fn max_score(mut self, max_score: impl Into<Option<f64>>) -> Self {
        if let Some(max) = max_score.into() {
            self.max_score = Some(max);
            self.max_score_exclusive = None;
        }
        self
    }

    /// Set the maximum score of the request.
    pub fn exclusive_max_score(mut self, exclusive_max_score: impl Into<Option<f64>>) -> Self {
        if let Some(max) = exclusive_max_score.into() {
            self.max_score = Some(max);
            self.max_score_exclusive = Some(true);
        }
        self
    }

    /// Set the order of the request.
    pub fn order(mut self, order: impl Into<Option<SortedSetOrder>>) -> Self {
        self.order = order.into().unwrap_or(SortedSetOrder::Ascending);
        self
    }

    /// Set the offset of the request.
    pub fn offset(mut self, offset: impl Into<Option<u32>>) -> Self {
        self.offset = offset.into();
        self
    }

    /// Set the count of the request.
    pub fn count(mut self, count: impl Into<Option<i32>>) -> Self {
        self.count = count.into();
        self
    }
}

impl<S: IntoBytes> MomentoRequest for SortedSetFetchByScoreRequest<S> {
    type Response = SortedSetFetchResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetFetchResponse> {
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;

        let min_score = match (self.min_score, self.min_score_exclusive) {
            (Some(score), Some(true)) => Some(by_score::Min::MinScore(Score {
                score,
                exclusive: true,
            })),
            (Some(score), _) => Some(by_score::Min::MinScore(Score {
                score,
                exclusive: false,
            })),
            (None, _) => Some(by_score::Min::UnboundedMin(Unbounded {})),
        };

        let max_score = match (self.max_score, self.max_score_exclusive) {
            (Some(score), Some(true)) => Some(by_score::Max::MaxScore(Score {
                score,
                exclusive: true,
            })),
            (Some(score), _) => Some(by_score::Max::MaxScore(Score {
                score,
                exclusive: false,
            })),
            (None, _) => Some(by_score::Max::UnboundedMax(Unbounded {})),
        };

        let by_score = ByScore {
            min: min_score,
            max: max_score,
            offset: self.offset.unwrap_or(0),
            count: self.count.unwrap_or(-1),
        };

        let request = prep_request_with_timeout(
            cache_name,
            cache_client.deadline_millis(),
            SortedSetFetchRequest {
                set_name,
                order: self.order as i32,
                with_scores: true,
                range: Some(Range::ByScore(by_score)),
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
mod test {
    use super::SortedSetFetchByScoreRequest;
    use crate::cache::SortedSetOrder;

    #[tokio::test]
    async fn test_sorted_set_fetch_by_score_request_with_inclusive_options() {
        // Define the cache name and sorted set name
        let cache_name = "test_cache";
        let sorted_set_name = "test_sorted_set";

        // Create the fetch request with options
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, sorted_set_name)
            .order(SortedSetOrder::Ascending)
            .min_score(2.0)
            .max_score(3.0)
            .offset(1)
            .count(2);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(2.0));
        assert_eq!(fetch_request.min_score_exclusive, None);
        assert_eq!(fetch_request.max_score, Some(3.0));
        assert_eq!(fetch_request.max_score_exclusive, None);
        assert_eq!(fetch_request.order, SortedSetOrder::Ascending);
        assert_eq!(fetch_request.offset, Some(1));
        assert_eq!(fetch_request.count, Some(2));

        // Now pass in explicit Options to min score, max score, offset, and count
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, sorted_set_name)
            .order(SortedSetOrder::Ascending)
            .min_score(Some(2.0))
            .max_score(Some(3.0))
            .offset(Some(1))
            .count(Some(2));

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(2.0));
        assert_eq!(fetch_request.min_score_exclusive, None);
        assert_eq!(fetch_request.max_score, Some(3.0));
        assert_eq!(fetch_request.max_score_exclusive, None);
        assert_eq!(fetch_request.order, SortedSetOrder::Ascending);
        assert_eq!(fetch_request.offset, Some(1));
        assert_eq!(fetch_request.count, Some(2));

        // Now pass in explicit None to min score, max score, offset, and count
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, sorted_set_name)
            .order(SortedSetOrder::Ascending)
            .min_score(None)
            .max_score(None)
            .offset(None)
            .count(None);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, None);
        assert_eq!(fetch_request.min_score_exclusive, None);
        assert_eq!(fetch_request.max_score, None);
        assert_eq!(fetch_request.max_score_exclusive, None);
        assert_eq!(fetch_request.order, SortedSetOrder::Ascending);
        assert_eq!(fetch_request.offset, None);
        assert_eq!(fetch_request.count, None);
    }

    #[tokio::test]
    async fn test_sorted_set_fetch_by_score_request_with_exclusive_options() {
        // Define the cache name and sorted set name
        let cache_name = "test_cache";
        let sorted_set_name = "test_sorted_set";

        // Create the fetch request with options
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, sorted_set_name)
            .order(SortedSetOrder::Ascending)
            .exclusive_min_score(2.0)
            .exclusive_max_score(3.0)
            .offset(1)
            .count(2);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(2.0));
        assert_eq!(fetch_request.min_score_exclusive, Some(true));
        assert_eq!(fetch_request.max_score, Some(3.0));
        assert_eq!(fetch_request.max_score_exclusive, Some(true));
        assert_eq!(fetch_request.order, SortedSetOrder::Ascending);
        assert_eq!(fetch_request.offset, Some(1));
        assert_eq!(fetch_request.count, Some(2));

        // Now pass in explicit Options to min score, max score, offset, and count
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, sorted_set_name)
            .order(SortedSetOrder::Ascending)
            .exclusive_min_score(Some(2.0))
            .exclusive_max_score(Some(3.0))
            .offset(Some(1))
            .count(Some(2));

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(2.0));
        assert_eq!(fetch_request.min_score_exclusive, Some(true));
        assert_eq!(fetch_request.max_score, Some(3.0));
        assert_eq!(fetch_request.max_score_exclusive, Some(true));
        assert_eq!(fetch_request.order, SortedSetOrder::Ascending);
        assert_eq!(fetch_request.offset, Some(1));
        assert_eq!(fetch_request.count, Some(2));

        // Now pass in explicit None to min score, max score, offset, and count
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, sorted_set_name)
            .order(SortedSetOrder::Ascending)
            .exclusive_min_score(None)
            .exclusive_max_score(None)
            .offset(None)
            .count(None);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, None);
        assert_eq!(fetch_request.min_score_exclusive, None);
        assert_eq!(fetch_request.max_score, None);
        assert_eq!(fetch_request.max_score_exclusive, None);
        assert_eq!(fetch_request.order, SortedSetOrder::Ascending);
        assert_eq!(fetch_request.offset, None);
        assert_eq!(fetch_request.count, None);
    }

    #[tokio::test]
    async fn test_sorted_set_fetch_by_score_request_with_conflicting_scores() {
        // Define the cache name and sorted set name
        let cache_name = "test_cache";
        let sorted_set_name = "test_sorted_set";

        // Create the fetch request with all score options, but only the last revisions should be used.
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, sorted_set_name)
            .exclusive_min_score(4.0)
            .exclusive_max_score(5.0)
            .min_score(1.0)
            .max_score(2.0);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(1.0));
        assert_eq!(fetch_request.min_score_exclusive, None);
        assert_eq!(fetch_request.max_score, Some(2.0));
        assert_eq!(fetch_request.max_score_exclusive, None);

        // Verify exclusive is used when order is switched.
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, sorted_set_name)
            .min_score(1.0)
            .max_score(2.0)
            .exclusive_min_score(4.0)
            .exclusive_max_score(5.0);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(4.0));
        assert_eq!(fetch_request.min_score_exclusive, Some(true));
        assert_eq!(fetch_request.max_score, Some(5.0));
        assert_eq!(fetch_request.max_score_exclusive, Some(true));
    }
}
