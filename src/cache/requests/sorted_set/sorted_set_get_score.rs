use std::convert::TryFrom;

use momento_protos::cache_client::{sorted_set_get_score_response, ECacheResult};

use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoError,
    MomentoErrorCode, MomentoResult,
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
/// use momento::cache::{SortedSetGetScore, SortedSetGetScoreRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let sorted_set_name = "sorted_set";
///
/// # cache_client.sorted_set_put_elements(&cache_name, sorted_set_name.to_string(), vec![("value1", 1.0), ("value2", 2.0)]).await;
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
    pub fn new(cache_name: impl Into<String>, sorted_set_name: L, value: V) -> Self {
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            value,
        }
    }
}

impl<L: IntoBytes, V: IntoBytes> MomentoRequest for SortedSetGetScoreRequest<L, V> {
    type Response = SortedSetGetScore;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetGetScore> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::SortedSetGetScoreRequest {
                set_name: self.sorted_set_name.into_bytes(),
                values: vec![self.value.into_bytes()],
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .sorted_set_get_score(request)
            .await?
            .into_inner();

        match response.sorted_set {
            Some(sorted_set_get_score_response::SortedSet::Found(found)) => {
                let found_element = found
                    .elements
                    .first()
                    .expect("Expected to receive one element");
                if found_element.result == ECacheResult::Hit as i32 {
                    Ok(SortedSetGetScore::Hit {
                        score: found_element.score,
                    })
                } else {
                    Ok(SortedSetGetScore::Miss)
                }
            }
            Some(sorted_set_get_score_response::SortedSet::Missing(_)) => {
                Ok(SortedSetGetScore::Miss)
            }
            _ => Err(MomentoError {
                message:
                    "Unknown error has occurred, unable to parse sorted_set_get_score_response"
                        .into(),
                error_code: MomentoErrorCode::UnknownError,
                inner_error: None,
                details: None,
            }),
        }
    }
}

/// Response for a sorted set get score operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SortedSetGetScore;
/// use std::convert::TryInto;
/// # let response = SortedSetGetScore::Hit { score: 5.0 };
/// let score: f64 = match response {
///     SortedSetGetScore::Hit { score } => score.try_into().expect("Expected a score!"),
///     SortedSetGetScore::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<f64, MomentoError> suitable for
/// ?-propagation if you know you are expecting a SortedSetGetScore::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SortedSetGetScore;
/// use std::convert::TryInto;
/// # let response = SortedSetGetScore::Hit { score: 5.0 };
/// let score: MomentoResult<f64> = response.try_into();
/// ```
#[derive(Debug, PartialEq)]
pub enum SortedSetGetScore {
    Hit { score: f64 },
    Miss,
}

impl TryFrom<SortedSetGetScore> for f64 {
    type Error = MomentoError;

    fn try_from(value: SortedSetGetScore) -> Result<Self, Self::Error> {
        match value {
            SortedSetGetScore::Hit { score } => Ok(score),
            SortedSetGetScore::Miss => Err(MomentoError {
                message: "list length response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}
