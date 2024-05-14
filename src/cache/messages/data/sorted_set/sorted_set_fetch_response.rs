use std::convert::{TryFrom, TryInto};

use momento_protos::cache_client::sorted_set_fetch_response::found::Elements;
use momento_protos::cache_client::sorted_set_fetch_response::SortedSet;
use momento_protos::cache_client::SortedSetFetchResponse as ProtoSortedSetFetchResponse;

use crate::{ErrorSource, MomentoError, MomentoErrorCode, MomentoResult};

/// Response object for a [SortedSetFetchByScoreRequest](crate::cache::SortedSetFetchByScoreRequest) or a [SortedSetFetchByRankRequest](crate::cache::SortedSetFetchByRankRequest).
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// fn main() -> anyhow::Result<()> {
/// # use momento::cache::{SortedSetFetchResponse, SortedSetElements};
/// # use momento::MomentoResult;
/// # let fetch_response = SortedSetFetchResponse::Hit { value: SortedSetElements::default() };
/// use std::convert::TryInto;
/// let item: Vec<(String, f64)> = match fetch_response {
///   SortedSetFetchResponse::Hit { value } => value.try_into().expect("I stored strings!"),
///   SortedSetFetchResponse::Miss => panic!("I expected a hit!"),
/// };
/// # Ok(())
/// }
/// ```
///
/// Or, if you're storing raw bytes you can get at them simply:
/// ```
/// # use momento::cache::{SortedSetFetchResponse, SortedSetElements};
/// # use momento::MomentoResult;
/// # let fetch_response = SortedSetFetchResponse::Hit { value: SortedSetElements::default() };
/// use std::convert::TryInto;
/// let item: Vec<(Vec<u8>, f64)> = match fetch_response {
///  SortedSetFetchResponse::Hit { value } => value.into(),
///  SortedSetFetchResponse::Miss => panic!("I expected a hit!"),
/// };
/// ```
///
/// You can cast your result directly into a Result<Vec<(String, f64)>, MomentoError> suitable for
/// ?-propagation if you know you are expecting a Vec<(String, f64)> item.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::cache::{SortedSetFetchResponse, SortedSetElements};
/// # use momento::MomentoResult;
/// # let fetch_response = SortedSetFetchResponse::Hit { value: SortedSetElements::default() };
/// use std::convert::TryInto;
/// let item: MomentoResult<Vec<(String, f64)>> = fetch_response.try_into();
/// ```
///
/// You can also go straight into a `Vec<(Vec<u8>, f64)>` if you prefer:
/// ```
/// # use momento::cache::{SortedSetFetchResponse, SortedSetElements};
/// # use momento::MomentoResult;
/// # let fetch_response = SortedSetFetchResponse::Hit { value: SortedSetElements::default() };
/// use std::convert::TryInto;
/// let item: MomentoResult<Vec<(Vec<u8>, f64)>> = fetch_response.try_into();
/// ```
#[derive(Debug, PartialEq)]
pub enum SortedSetFetchResponse {
    /// The sorted set was found.
    Hit {
        /// The elements in the sorted set.
        value: SortedSetElements,
    },
    /// The sorted set was not found.
    Miss,
}

impl SortedSetFetchResponse {
    pub(crate) fn from_fetch_response(
        response: ProtoSortedSetFetchResponse,
    ) -> MomentoResult<Self> {
        match response.sorted_set {
            Some(SortedSet::Missing(_)) => Ok(SortedSetFetchResponse::Miss),
            Some(SortedSet::Found(elements)) => match elements.elements {
                None => Ok(SortedSetFetchResponse::Hit {
                    value: SortedSetElements::new(Vec::new()),
                }),
                Some(elements) => match elements {
                    Elements::ValuesWithScores(values_with_scores) => {
                        let elements = values_with_scores
                            .elements
                            .into_iter()
                            .map(|element| (element.value, element.score))
                            .collect();
                        Ok(SortedSetFetchResponse::Hit {
                            value: SortedSetElements::new(elements),
                        })
                    }
                    Elements::Values(_) => Err(MomentoError {
                        message:
                            "sorted_set_fetch_by_index response included elements without values"
                                .into(),
                        error_code: MomentoErrorCode::UnknownError,
                        inner_error: Some(ErrorSource::Unknown(
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "unexpected response",
                            )
                            .into(),
                        )),
                        details: None,
                    }),
                },
            },
            _ => Err(MomentoError::unknown_error(
                "SortedSetFetch",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

impl TryFrom<SortedSetFetchResponse> for Vec<(Vec<u8>, f64)> {
    type Error = MomentoError;

    fn try_from(value: SortedSetFetchResponse) -> Result<Self, Self::Error> {
        match value {
            SortedSetFetchResponse::Hit { value: elements } => Ok(elements.elements),
            SortedSetFetchResponse::Miss => Err(MomentoError::miss("SortedSetFetch")),
        }
    }
}

impl TryFrom<SortedSetFetchResponse> for Vec<(String, f64)> {
    type Error = MomentoError;

    fn try_from(value: SortedSetFetchResponse) -> Result<Self, Self::Error> {
        match value {
            SortedSetFetchResponse::Hit { value: elements } => elements.into_strings(),
            SortedSetFetchResponse::Miss => Err(MomentoError::miss("SortedSetFetch")),
        }
    }
}

impl From<Vec<(String, f64)>> for SortedSetFetchResponse {
    fn from(elements: Vec<(String, f64)>) -> Self {
        SortedSetFetchResponse::Hit {
            value: SortedSetElements::new(
                elements
                    .into_iter()
                    .map(|(element, score)| (element.into_bytes(), score))
                    .collect(),
            ),
        }
    }
}

/// A collection of elements from a sorted set.
#[derive(Debug, PartialEq, Default)]
pub struct SortedSetElements {
    /// The elements in the sorted set.
    pub elements: Vec<(Vec<u8>, f64)>,
}

impl SortedSetElements {
    /// Constructs a new SortedSetElements from the given (value, score) pairs.
    pub fn new(elements: Vec<(Vec<u8>, f64)>) -> Self {
        SortedSetElements { elements }
    }

    /// Converts the elements into a Vec<(String, f64)>.
    pub fn into_strings(self) -> MomentoResult<Vec<(String, f64)>> {
        self.try_into()
    }

    /// Returns the number of elements in the sorted set.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns true if the sorted set is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl From<SortedSetElements> for Vec<(Vec<u8>, f64)> {
    fn from(value: SortedSetElements) -> Self {
        value.elements
    }
}

impl TryFrom<SortedSetElements> for Vec<(String, f64)> {
    type Error = MomentoError;

    fn try_from(value: SortedSetElements) -> Result<Self, Self::Error> {
        let mut result = Vec::with_capacity(value.elements.len());
        for element in value.elements {
            match String::from_utf8(element.0) {
                Ok(value) => {
                    result.push((value, element.1));
                }
                Err(e) => {
                    return Err::<Self, Self::Error>(MomentoError {
                        message: "element value was not a valid utf-8 string".to_string(),
                        error_code: MomentoErrorCode::TypeError,
                        inner_error: Some(ErrorSource::Unknown(Box::new(e))),
                        details: None,
                    });
                }
            }
        }
        Ok(result)
    }
}
