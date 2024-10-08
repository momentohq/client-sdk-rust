use std::convert::TryFrom;

use momento_protos::cache_client::{
    sorted_set_get_score_response::{self, SortedSetGetScoreResponsePart},
    ECacheResult,
};

use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoError,
    MomentoResult,
};

/// Get the score of a specific element in a sorted set.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `sorted_set_name` - name of the sorted set
/// * `value` - the sorted set value to get the score of
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::cache::{SortedSetGetScoreResponse, SortedSetGetScoreRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let sorted_set_name = "sorted_set";
///
/// let put_element_response = cache_client.sorted_set_put_elements(
///     cache_name.to_string(),
///     sorted_set_name.to_string(),
///     vec![("value1", 1.0), ("value2", 2.0), ("value3", 3.0), ("value4", 4.0)]
/// ).await?;
///
/// let get_score_request = SortedSetGetScoreRequest::new(cache_name, sorted_set_name, "value1");
/// let score: f64 = cache_client.send_request(get_score_request).await?.try_into().expect("Expected a score!");
/// # assert_eq!(score, 1.0);
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SortedSetGetScoreRequest<L: IntoBytes, V: IntoBytes> {
    cache_name: String,
    sorted_set_name: L,
    value: V,
}

impl<L: IntoBytes, V: IntoBytes> SortedSetGetScoreRequest<L, V> {
    /// Constructs a new SortedSetGetScoreRequest.
    pub fn new(cache_name: impl Into<String>, sorted_set_name: L, value: V) -> Self {
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            value,
        }
    }
}

impl<L: IntoBytes, V: IntoBytes> MomentoRequest for SortedSetGetScoreRequest<L, V> {
    type Response = SortedSetGetScoreResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetGetScoreResponse> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::SortedSetGetScoreRequest {
                set_name: self.sorted_set_name.into_bytes(),
                values: vec![self.value.into_bytes()],
            },
        )?;

        let response = cache_client
            .next_data_client()
            .sorted_set_get_score(request)
            .await?
            .into_inner();

        match response.sorted_set {
            Some(sorted_set_get_score_response::SortedSet::Found(found)) => {
                let mut responses: Vec<SortedSetGetScoreResponsePart> =
                    found.elements.into_iter().collect();

                match responses.pop() {
                    Some(element) => {
                        if element.result == ECacheResult::Hit as i32 {
                            Ok(SortedSetGetScoreResponse::Hit {
                                score: element.score,
                            })
                        } else {
                            Ok(SortedSetGetScoreResponse::Miss)
                        }
                    }
                    None => Err(MomentoError::unknown_error(
                        "SortedSetGetScore",
                        Some("Expected to receive one element".to_string()),
                    )),
                }
            }
            Some(sorted_set_get_score_response::SortedSet::Missing(_)) => {
                Ok(SortedSetGetScoreResponse::Miss)
            }
            _ => Err(MomentoError::unknown_error(
                "SortedSetGetScore",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response for a sorted set get score operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SortedSetGetScoreResponse;
/// use std::convert::TryInto;
/// # let response = SortedSetGetScoreResponse::Hit { score: 5.0 };
/// let score: f64 = match response {
///     SortedSetGetScoreResponse::Hit { score } => score.try_into().expect("Expected a score!"),
///     SortedSetGetScoreResponse::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<f64, MomentoError> suitable for
/// ?-propagation if you know you are expecting a SortedSetGetScoreResponse::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SortedSetGetScoreResponse;
/// use std::convert::TryInto;
/// # let response = SortedSetGetScoreResponse::Hit { score: 5.0 };
/// let score: MomentoResult<f64> = response.try_into();
/// ```
#[derive(Debug, PartialEq, PartialOrd)]
pub enum SortedSetGetScoreResponse {
    /// The sorted set was found.
    Hit {
        /// The score of the element in the sorted set.
        score: f64,
    },
    /// The sorted set was not found.
    Miss,
}

impl TryFrom<SortedSetGetScoreResponse> for f64 {
    type Error = MomentoError;

    fn try_from(value: SortedSetGetScoreResponse) -> Result<Self, Self::Error> {
        match value {
            SortedSetGetScoreResponse::Hit { score } => Ok(score),
            SortedSetGetScoreResponse::Miss => Err(MomentoError::miss("SortedSetGetScore")),
        }
    }
}
