use momento_protos::cache_client::sorted_set_fetch_request::by_score::Score;
use momento_protos::cache_client::sorted_set_fetch_request::{by_score, ByScore, Range};
use momento_protos::cache_client::{SortedSetFetchRequest, Unbounded};

use crate::requests::cache::sorted_set::sorted_set_fetch_by_rank::SortOrder;
use crate::requests::cache::sorted_set::sorted_set_fetch_by_rank::SortOrder::Ascending;
use crate::requests::cache::MomentoRequest;
use crate::response::cache::sorted_set_fetch::SortedSetFetch;
use crate::simple_cache_client::prep_request_with_timeout;
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
/// * `order` - The order to sort the elements by. [SortOrder::Ascending] or [SortOrder::Descending].
/// Defaults to Ascending.
/// * `min_score` - The minimum score (inclusive) of the elements to fetch. Defaults to negative
/// infinity.
/// * `max_score` - The maximum score (inclusive) of the elements to fetch. Defaults to positive
/// infinity.
/// * `offset` - The number of elements to skip before returning the first element. Defaults to
/// 0. Note: this is not the rank of the first element to return, but the number of elements of
/// the result set to skip before returning the first element.
/// * `count` - The maximum number of elements to return. Defaults to all elements.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use std::convert::TryInto;
/// # use momento_test_util::create_doctest_client;
/// # tokio_test::block_on(async {
/// use momento::requests::cache::sorted_set::sorted_set_fetch_by_rank::SortOrder;
/// use momento::requests::cache::sorted_set::sorted_set_fetch_by_score::SortedSetFetchByScoreRequest;
/// use momento::response::cache::sorted_set_fetch::SortedSetFetch;
/// # let (cache_client, cache_name) = create_doctest_client();
/// let sorted_set_name = "sorted_set";
///
/// let put_element_response = cache_client.sorted_set_put_elements(
///     cache_name.to_string(),
///     sorted_set_name.to_string(),
///     vec![("value1", 1.0), ("value2", 2.0), ("value3", 3.0), ("value4", 4.0)]
/// ).await?;
///
/// let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, sorted_set_name)
///     .with_order(SortOrder::Ascending)
///     .with_min_score(2.0)
///     .with_max_score(3.0);
///
/// let fetch_response = cache_client.send_request(fetch_request).await?;
///
/// let returned_elements: Vec<(String, f64)> = fetch_response.try_into()
///     .expect("elements 2 and 3 should be returned");
/// println!("{:?}", returned_elements);
/// # Ok(())
/// # })
/// # }
pub struct SortedSetFetchByScoreRequest<S: IntoBytes> {
    cache_name: String,
    sorted_set_name: S,
    min_score: Option<f64>,
    max_score: Option<f64>,
    order: SortOrder,
    offset: Option<u32>,
    count: Option<i32>,
}

impl<S: IntoBytes> SortedSetFetchByScoreRequest<S> {
    pub fn new(cache_name: String, sorted_set_name: S) -> Self {
        Self {
            cache_name,
            sorted_set_name,
            min_score: None,
            max_score: None,
            order: Ascending,
            offset: None,
            count: None,
        }
    }

    /// Set the minimum score of the request.
    pub fn with_min_score(mut self, min_score: f64) -> Self {
        self.min_score = Some(min_score);
        self
    }

    /// Set the maximum score of the request.
    pub fn with_max_score(mut self, max_score: f64) -> Self {
        self.max_score = Some(max_score);
        self
    }

    /// Set the order of the request.
    pub fn with_order(mut self, order: SortOrder) -> Self {
        self.order = order;
        self
    }

    /// Set the offset of the request.
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set the count of the request.
    pub fn with_count(mut self, count: i32) -> Self {
        self.count = Some(count);
        self
    }
}

impl<S: IntoBytes> MomentoRequest for SortedSetFetchByScoreRequest<S> {
    type Response = SortedSetFetch;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetFetch> {
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;

        let by_score = ByScore {
            min: Some(
                self.min_score
                    .map(|score| {
                        by_score::Min::MinScore(Score {
                            score,
                            exclusive: false,
                        })
                    })
                    .unwrap_or(by_score::Min::UnboundedMin(Unbounded {})),
            ),
            max: Some(
                self.max_score
                    .map(|score| {
                        by_score::Max::MaxScore(Score {
                            score,
                            exclusive: false,
                        })
                    })
                    .unwrap_or(by_score::Max::UnboundedMax(Unbounded {})),
            ),
            offset: self.offset.unwrap_or(0),
            count: self.count.unwrap_or(-1),
        };

        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            SortedSetFetchRequest {
                set_name,
                order: self.order as i32,
                with_scores: true,
                range: Some(Range::ByScore(by_score)),
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
