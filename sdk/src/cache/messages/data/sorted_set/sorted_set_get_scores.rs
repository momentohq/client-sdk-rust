use std::convert::TryFrom;

use momento_protos::cache_client::{
    sorted_set_get_score_response::{self, SortedSetGetScoreResponsePart},
    ECacheResult,
};

use crate::{cache::SortedSetGetScoreResponse, MomentoErrorCode};
use crate::{
    cache::{MomentoRequest, SortedSetElement},
    utils::{parse_string, prep_request_with_timeout},
    CacheClient, IntoBytes, IntoBytesIterable, MomentoError, MomentoResult,
};

/// Get the scores of a specific elements in a sorted set.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `sorted_set_name` - name of the sorted set
/// * `values` - the values in the sorted set to the scores of
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::cache::{SortedSetGetScoresRequest};
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
/// let get_score_request = SortedSetGetScoresRequest::new(cache_name, sorted_set_name, vec!["value1", "value2"]);
/// let elements: Vec<SortedSetElement> = cache_client.send_request(get_score_request).await?.try_into().expect("Expected a score!");
/// # assert_eq!(elements.len(), 2);
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SortedSetGetScoresRequest<L: IntoBytes, V: IntoBytesIterable> {
    cache_name: String,
    sorted_set_name: L,
    values: V,
}

impl<L: IntoBytes, V: IntoBytesIterable> SortedSetGetScoresRequest<L, V> {
    /// Constructs a new SortedSetGetScoresRequest.
    pub fn new(cache_name: impl Into<String>, sorted_set_name: L, values: V) -> Self {
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            values,
        }
    }
}

impl<L: IntoBytes, V: IntoBytesIterable + Clone> MomentoRequest
    for SortedSetGetScoresRequest<L, V>
{
    type Response = SortedSetGetScoresResponse<V>;

    async fn send(
        self,
        cache_client: &CacheClient,
    ) -> MomentoResult<SortedSetGetScoresResponse<V>> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::SortedSetGetScoreRequest {
                set_name: self.sorted_set_name.into_bytes(),
                values: self.values.clone().into_bytes(),
            },
        )?;

        let get_scores_response = cache_client
            .data_client
            .clone()
            .sorted_set_get_score(request)
            .await?
            .into_inner();

        match get_scores_response.sorted_set {
            Some(sorted_set_get_score_response::SortedSet::Found(found)) => {
                let parts: Vec<SortedSetGetScoreResponsePart> =
                    found.elements.into_iter().collect();

                let responses = parts
                    .iter()
                    .map(|element| {
                        if element.result() == ECacheResult::Hit {
                            SortedSetGetScoreResponse::Hit {
                                score: element.score,
                            }
                        } else {
                            SortedSetGetScoreResponse::Miss
                        }
                    })
                    .collect::<Vec<SortedSetGetScoreResponse>>();

                Ok(SortedSetGetScoresResponse::Hit {
                    responses,
                    values: self.values,
                })
            }
            Some(sorted_set_get_score_response::SortedSet::Missing(_)) => {
                Ok(SortedSetGetScoresResponse::Miss)
            }
            _ => Err(MomentoError::unknown_error(
                "SortedSetGetScores",
                Some(format!("{:#?}", get_scores_response)),
            )),
        }
    }
}

impl<F: IntoBytesIterable + Clone> TryFrom<SortedSetGetScoresResponse<F>>
    for Vec<SortedSetElement<Vec<u8>>>
{
    type Error = MomentoError;

    fn try_from(value: SortedSetGetScoresResponse<F>) -> Result<Self, Self::Error> {
        match value {
            SortedSetGetScoresResponse::Hit {
                values, responses, ..
            } => {
                let mut result = Vec::new();
                for (value, response) in values.into_bytes().into_iter().zip(responses.into_iter())
                {
                    match response {
                        SortedSetGetScoreResponse::Hit { score } => {
                            let ele = SortedSetElement { score, value };
                            result.push(ele);
                        }
                        SortedSetGetScoreResponse::Miss => (),
                    }
                }
                Ok(result)
            }
            // In other SDKs we do not convert a `Miss` into an empty array
            SortedSetGetScoresResponse::Miss => Err(MomentoError {
                message: "sorted set get scores response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl<F: IntoBytesIterable + Clone> TryFrom<SortedSetGetScoresResponse<F>>
    for Vec<SortedSetElement<String>>
{
    type Error = MomentoError;

    fn try_from(value: SortedSetGetScoresResponse<F>) -> Result<Self, Self::Error> {
        match value {
            SortedSetGetScoresResponse::Hit {
                values, responses, ..
            } => {
                let mut result = Vec::new();
                for (bytes_value, response) in
                    values.into_bytes().into_iter().zip(responses.into_iter())
                {
                    match response {
                        SortedSetGetScoreResponse::Hit { score } => {
                            let value: String = parse_string(bytes_value)?;
                            let ele = SortedSetElement { score, value };
                            result.push(ele);
                        }
                        SortedSetGetScoreResponse::Miss => (),
                    }
                }
                Ok(result)
            }
            // In other SDKs we do not convert a `Miss` into an empty array
            SortedSetGetScoresResponse::Miss => Err(MomentoError {
                message: "sorted set get scores response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum SortedSetGetScoresResponse<T: IntoBytesIterable + Clone> {
    /// The sorted set was found.
    Hit {
        /// The responses for each element.
        responses: Vec<SortedSetGetScoreResponse>,
        values: T,
    },
    /// The sorted set was not found.
    Miss,
}

impl<T: IntoBytes + Clone> Default for SortedSetGetScoresResponse<Vec<T>> {
    fn default() -> Self {
        SortedSetGetScoresResponse::Hit {
            responses: vec![],
            values: vec![],
        }
    }
}
