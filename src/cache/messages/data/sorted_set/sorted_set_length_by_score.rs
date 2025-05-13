use std::convert::TryFrom;

use momento_protos::{
    cache_client::{sorted_set_length_by_score_request, sorted_set_length_by_score_response},
    common::Unbounded,
};

use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoError,
    MomentoResult,
};

/// Boundary for a sorted set score range.
#[derive(Debug, PartialEq, Clone)]
pub enum ScoreBound {
    /// Include the score in the range.
    Inclusive(f64),
    /// Exclude the score from the range.
    Exclusive(f64),
}

/// Get the number of entries in a sorted set that fall between a minimum and maximum score.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `sorted_set_name` - name of the sorted set
///
/// # Optional Arguments
/// * `min_score` - the minimum score of the elements to fetch. Defaults to negative
///   infinity. Use [ScoreBound::Inclusive] or [ScoreBound::Exclusive] to specify whether
///   the minimum score is inclusive or exclusive.
/// * `max_score` - the maximum score of the elements to fetch. Defaults to positive
///   infinity. Use [ScoreBound::Inclusive] or [ScoreBound::Exclusive] to specify whether
///   the maximum score is inclusive or exclusive.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::cache::{SortedSetLengthByScoreResponse, SortedSetLengthByScoreRequest, ScoreBound};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let sorted_set_name = "sorted_set";
///
/// # cache_client.sorted_set_put_elements(&cache_name, sorted_set_name.to_string(), vec![("value1", 1.0), ("value2", 2.0)]).await;
///
/// let length_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
///     .min_score(ScoreBound::Inclusive(1.0))
///     .max_score(ScoreBound::Exclusive(5.0));
///
/// let length: u32 = cache_client.send_request(length_request).await?.try_into().expect("Expected a sorted set length!");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SortedSetLengthByScoreRequest<L: IntoBytes> {
    cache_name: String,
    sorted_set_name: L,
    min_score: Option<ScoreBound>,
    max_score: Option<ScoreBound>,
}

impl<L: IntoBytes> SortedSetLengthByScoreRequest<L> {
    /// Constructs a new SortedSetLengthByScoreRequest.
    pub fn new(cache_name: impl Into<String>, sorted_set_name: L) -> Self {
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            min_score: None,
            max_score: None,
        }
    }

    /// Set the minimum score of the request.
    pub fn min_score(mut self, min_score: impl Into<Option<ScoreBound>>) -> Self {
        self.min_score = min_score.into();
        self
    }

    /// Set the maximum score of the request.
    pub fn max_score(mut self, max_score: impl Into<Option<ScoreBound>>) -> Self {
        self.max_score = max_score.into();
        self
    }
}

impl<L: IntoBytes> MomentoRequest for SortedSetLengthByScoreRequest<L> {
    type Response = SortedSetLengthByScoreResponse;

    async fn send(
        self,
        cache_client: &CacheClient,
    ) -> MomentoResult<SortedSetLengthByScoreResponse> {
        let min_score = match self.min_score {
            Some(min_score) => match min_score {
                ScoreBound::Inclusive(score) => {
                    Some(sorted_set_length_by_score_request::Min::InclusiveMin(score))
                }
                ScoreBound::Exclusive(score) => {
                    Some(sorted_set_length_by_score_request::Min::ExclusiveMin(score))
                }
            },
            None => Some(sorted_set_length_by_score_request::Min::UnboundedMin(
                Unbounded {},
            )),
        };

        let max_score = match self.max_score {
            Some(max_score) => match max_score {
                ScoreBound::Inclusive(score) => {
                    Some(sorted_set_length_by_score_request::Max::InclusiveMax(score))
                }
                ScoreBound::Exclusive(score) => {
                    Some(sorted_set_length_by_score_request::Max::ExclusiveMax(score))
                }
            },
            None => Some(sorted_set_length_by_score_request::Max::UnboundedMax(
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
    use super::{ScoreBound, SortedSetLengthByScoreRequest};

    #[tokio::test]
    async fn test_sorted_set_length_by_score_request_with_inclusive_scores() {
        // Define the cache name and sorted set name
        let cache_name = "test_cache";
        let sorted_set_name = "test_sorted_set";

        // Create the fetch request with options
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
            .min_score(ScoreBound::Inclusive(2.0))
            .max_score(ScoreBound::Inclusive(3.0));

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(ScoreBound::Inclusive(2.0)));
        assert_eq!(fetch_request.max_score, Some(ScoreBound::Inclusive(3.0)));

        // Now pass in explicit Options to min score and max score
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
            .min_score(ScoreBound::Inclusive(1.0))
            .max_score(ScoreBound::Inclusive(5.0));

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(ScoreBound::Inclusive(1.0)));
        assert_eq!(fetch_request.max_score, Some(ScoreBound::Inclusive(5.0)));

        // Now pass in explicit None to min score and max score
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
            .min_score(None)
            .max_score(None);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, None);
        assert_eq!(fetch_request.max_score, None);

        // Now specify no extra options
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name);

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, None);
        assert_eq!(fetch_request.max_score, None);
    }

    #[tokio::test]
    async fn test_sorted_set_length_by_score_request_with_exclusive_scores() {
        // Define the cache name and sorted set name
        let cache_name = "test_cache";
        let sorted_set_name = "test_sorted_set";

        // Create the fetch request with options
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
            .min_score(ScoreBound::Exclusive(2.0))
            .max_score(ScoreBound::Exclusive(3.0));

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(ScoreBound::Exclusive(2.0)));
        assert_eq!(fetch_request.max_score, Some(ScoreBound::Exclusive(3.0)));

        // Now pass in explicit Options to min score and max score
        let fetch_request = SortedSetLengthByScoreRequest::new(cache_name, sorted_set_name)
            .min_score(ScoreBound::Exclusive(1.0))
            .max_score(ScoreBound::Exclusive(5.0));

        // Verify the built request
        assert_eq!(fetch_request.cache_name, cache_name);
        assert_eq!(fetch_request.sorted_set_name, sorted_set_name);
        assert_eq!(fetch_request.min_score, Some(ScoreBound::Exclusive(1.0)));
        assert_eq!(fetch_request.max_score, Some(ScoreBound::Exclusive(5.0)));
    }
}
