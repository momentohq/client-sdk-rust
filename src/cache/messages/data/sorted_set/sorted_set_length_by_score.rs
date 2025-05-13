use std::convert::TryFrom;

use momento_protos::{
    cache_client::{sorted_set_length_by_score_request, sorted_set_length_by_score_response},
    common::Unbounded,
};

use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoError,
    MomentoResult,
};

/// Get the number of entries in a sorted set that fall between a minimum and maximum score.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `sorted_set_name` - name of the sorted set
///
/// # Optional Arguments
/// * `min_score` - the minimum score (inclusive) of the elements to fetch. Defaults to negative
///   infinity.
/// * `max_score` - the maximum score (inclusive) of the elements to fetch. Defaults to positive
///   infinity.
/// * `exclusive_min_score` - the minimum score (exclusive) of the elements to fetch. Defaults to negative infinity.
/// * `exclusive_max_score` - the maximum score (exclusive) of the elements to fetch. Defaults to positive
///   infinity.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::cache::{SortedSetLengthByScoreResponse, SortedSetLengthByScoreRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let sorted_set_name = "sorted_set";
///
/// # cache_client.sorted_set_put_elements(&cache_name, sorted_set_name.to_string(), vec![("value1", 1.0), ("value2", 2.0)]).await;
///
/// let length_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
///     .min_score(1.0)
///     .exclusive_max_score(5.0);
///
/// let length: u32 = cache_client.send_request(length_request).await?.try_into().expect("Expected a sorted set length!");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SortedSetLengthByScoreRequest<L: IntoBytes> {
    cache_name: String,
    sorted_set_name: L,
    min_score: Option<f64>,
    min_score_exclusive: Option<bool>,
    max_score: Option<f64>,
    max_score_exclusive: Option<bool>,
}

impl<L: IntoBytes> SortedSetLengthByScoreRequest<L> {
    /// Constructs a new SortedSetLengthByScoreRequest.
    pub fn new(cache_name: impl Into<String>, sorted_set_name: L) -> Self {
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            min_score: None,
            min_score_exclusive: None,
            max_score: None,
            max_score_exclusive: None,
        }
    }

    /// Set the inclusive minimum score of the request.
    pub fn min_score(mut self, min_score: impl Into<Option<f64>>) -> Self {
        if let Some(min) = min_score.into() {
            self.min_score = Some(min);
            self.min_score_exclusive = None;
        }
        self
    }

    /// Set the exclusive minimum score of the request.
    pub fn exclusive_min_score(mut self, min_score: impl Into<Option<f64>>) -> Self {
        if let Some(min) = min_score.into() {
            self.min_score = Some(min);
            self.min_score_exclusive = Some(true);
        }
        self
    }

    /// Set the inclusive maximum score of the request.
    pub fn max_score(mut self, max_score: impl Into<Option<f64>>) -> Self {
        if let Some(max) = max_score.into() {
            self.max_score = Some(max);
            self.max_score_exclusive = None;
        }
        self
    }

    /// Set the exclusive maximum score of the request.
    pub fn exclusive_max_score(mut self, max_score: impl Into<Option<f64>>) -> Self {
        if let Some(max) = max_score.into() {
            self.max_score = Some(max);
            self.max_score_exclusive = Some(true);
        }
        self
    }
}

impl<L: IntoBytes> MomentoRequest for SortedSetLengthByScoreRequest<L> {
    type Response = SortedSetLengthByScoreResponse;

    async fn send(
        self,
        cache_client: &CacheClient,
    ) -> MomentoResult<SortedSetLengthByScoreResponse> {
        let min_score = match (self.min_score, self.min_score_exclusive) {
            (Some(min_score), Some(true)) => Some(
                sorted_set_length_by_score_request::Min::ExclusiveMin(min_score),
            ),
            (Some(min_score), _) => Some(sorted_set_length_by_score_request::Min::InclusiveMin(
                min_score,
            )),
            (None, _) => Some(sorted_set_length_by_score_request::Min::UnboundedMin(
                Unbounded {},
            )),
        };

        let max_score = match (self.max_score, self.max_score_exclusive) {
            (Some(max_score), Some(true)) => Some(
                sorted_set_length_by_score_request::Max::ExclusiveMax(max_score),
            ),
            (Some(max_score), _) => Some(sorted_set_length_by_score_request::Max::InclusiveMax(
                max_score,
            )),
            (None, _) => Some(sorted_set_length_by_score_request::Max::UnboundedMax(
                Unbounded {},
            )),
        };

        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.deadline_millis(),
            momento_protos::cache_client::SortedSetLengthByScoreRequest {
                set_name: self.sorted_set_name.into_bytes(),
                min: min_score,
                max: max_score,
            },
        )?;

        let response = cache_client
            .next_data_client()
            .sorted_set_length_by_score(request)
            .await?
            .into_inner();

        match response.sorted_set {
            Some(sorted_set_length_by_score_response::SortedSet::Missing(_)) => {
                Ok(SortedSetLengthByScoreResponse::Miss)
            }
            Some(sorted_set_length_by_score_response::SortedSet::Found(found)) => {
                Ok(SortedSetLengthByScoreResponse::Hit {
                    length: found.length,
                })
            }
            _ => Err(MomentoError::unknown_error(
                "SortedSetLengthByScore",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response for a sorted set length by score operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SortedSetLengthByScoreResponse;
/// use std::convert::TryInto;
/// # let response = SortedSetLengthByScoreResponse::Hit { length: 5 };
/// let length: u32 = match response {
///     SortedSetLengthByScoreResponse::Hit { length } => length.try_into().expect("Expected a list length!"),
///     SortedSetLengthByScoreResponse::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<u32, MomentoError> suitable for
/// ?-propagation if you know you are expecting a SortedSetLengthByScoreResponse::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SortedSetLengthByScoreResponse;
/// use std::convert::TryInto;
/// # let response = SortedSetLengthByScoreResponse::Hit { length: 5 };
/// let length: MomentoResult<u32> = response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum SortedSetLengthByScoreResponse {
    /// The sorted set was found.
    Hit {
        /// The number of elements in the sorted set within the specified score range.
        length: u32,
    },
    /// The sorted set was not found.
    Miss,
}

impl TryFrom<SortedSetLengthByScoreResponse> for u32 {
    type Error = MomentoError;

    fn try_from(value: SortedSetLengthByScoreResponse) -> Result<Self, Self::Error> {
        match value {
            SortedSetLengthByScoreResponse::Hit { length } => Ok(length),
            SortedSetLengthByScoreResponse::Miss => {
                Err(MomentoError::miss("SortedSetLengthByScore"))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::SortedSetLengthByScoreRequest;

    #[tokio::test]
    async fn test_sorted_set_length_by_score_request_with_inclusive_scores() {
        // Define the cache name and sorted set name
        let cache_name = "test_cache";
        let sorted_set_name = "test_sorted_set";

        // Create the fetch request with options
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
            .min_score(2.0)
            .max_score(3.0);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(2.0));
        assert_eq!(fetch_request.min_score_exclusive, None);
        assert_eq!(fetch_request.max_score, Some(3.0));
        assert_eq!(fetch_request.max_score_exclusive, None);

        // Now pass in explicit Options to min score and max score
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
            .min_score(Some(1.0))
            .max_score(Some(5.0));

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(1.0));
        assert_eq!(fetch_request.min_score_exclusive, None);
        assert_eq!(fetch_request.max_score, Some(5.0));
        assert_eq!(fetch_request.max_score_exclusive, None);

        // Now pass in explicit None to min score and max score
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
            .min_score(None)
            .max_score(None);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, None);
        assert_eq!(fetch_request.min_score_exclusive, None);
        assert_eq!(fetch_request.max_score, None);
        assert_eq!(fetch_request.max_score_exclusive, None);

        // Now specify no extra options
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, None);
        assert_eq!(fetch_request.min_score_exclusive, None);
        assert_eq!(fetch_request.max_score, None);
        assert_eq!(fetch_request.max_score_exclusive, None);
    }

    #[tokio::test]
    async fn test_sorted_set_length_by_score_request_with_exclusive_scores() {
        // Define the cache name and sorted set name
        let cache_name = "test_cache";
        let sorted_set_name = "test_sorted_set";

        // Create the fetch request with options
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
            .exclusive_min_score(2.0)
            .exclusive_max_score(3.0);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(2.0));
        assert_eq!(fetch_request.min_score_exclusive, Some(true));
        assert_eq!(fetch_request.max_score, Some(3.0));
        assert_eq!(fetch_request.max_score_exclusive, Some(true));

        // Now pass in explicit Options to min score and max score
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
            .exclusive_min_score(Some(1.0))
            .exclusive_max_score(Some(5.0));

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(1.0));
        assert_eq!(fetch_request.min_score_exclusive, Some(true));
        assert_eq!(fetch_request.max_score, Some(5.0));
        assert_eq!(fetch_request.max_score_exclusive, Some(true));

        // Now pass in explicit None to min score and max score
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
            .exclusive_min_score(None)
            .exclusive_max_score(None);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, None);
        assert_eq!(fetch_request.min_score_exclusive, None);
        assert_eq!(fetch_request.max_score, None);
        assert_eq!(fetch_request.max_score_exclusive, None);

        // Now specify no extra options
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, None);
        assert_eq!(fetch_request.min_score_exclusive, None);
        assert_eq!(fetch_request.max_score, None);
        assert_eq!(fetch_request.max_score_exclusive, None);
    }

    #[tokio::test]
    async fn test_sorted_set_length_by_score_request_with_conflicting_scores() {
        // Define the cache name and sorted set name
        let cache_name = "test_cache";
        let sorted_set_name = "test_sorted_set";

        // Create the fetch request with all score options, but only the last revisions should be used.
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
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
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
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
