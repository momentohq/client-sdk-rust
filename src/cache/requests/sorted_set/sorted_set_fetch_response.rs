use std::convert::{TryFrom, TryInto};

use momento_protos::cache_client::sorted_set_fetch_response::found::Elements;
use momento_protos::cache_client::sorted_set_fetch_response::SortedSet;
use momento_protos::cache_client::SortedSetFetchResponse;

use crate::{
    MomentoResult, {ErrorSource, MomentoError, MomentoErrorCode},
};

/// Response object for a [SortedSetFetchByScoreRequest] or a [SortedSetFetchByRankRequest].
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// fn main() -> anyhow::Result<()> {
/// # use momento::cache::{SortedSetFetch, SortedSetElements};
/// # use momento::MomentoResult;
/// # let fetch_response = SortedSetFetch::Hit { value: SortedSetElements::default() };
/// use std::convert::TryInto;
/// let item: Vec<(String, f64)> = match fetch_response {
///   SortedSetFetch::Hit { value } => value.try_into().expect("I stored strings!"),
///   SortedSetFetch::Miss => panic!("I expected a hit!"),
/// };
/// # Ok(())
/// }
/// ```
///
/// Or, if you're storing raw bytes you can get at them simply:
/// ```
/// # use momento::cache::{SortedSetFetch, SortedSetElements};
/// # use momento::MomentoResult;
/// # let fetch_response = SortedSetFetch::Hit { value: SortedSetElements::default() };
/// use std::convert::TryInto;
/// let item: Vec<(Vec<u8>, f64)> = match fetch_response {
///  SortedSetFetch::Hit { value } => value.into(),
///  SortedSetFetch::Miss => panic!("I expected a hit!"),
/// };
/// ```
///
/// You can cast your result directly into a Result<Vec<(String, f64)>, MomentoError> suitable for
/// ?-propagation if you know you are expecting a Vec<(String, f64)> item.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::cache::{SortedSetFetch, SortedSetElements};
/// # use momento::MomentoResult;
/// # let fetch_response = SortedSetFetch::Hit { value: SortedSetElements::default() };
/// use std::convert::TryInto;
/// let item: MomentoResult<Vec<(String, f64)>> = fetch_response.try_into();
/// ```
///
/// You can also go straight into a `Vec<(Vec<u8>, f64)>` if you prefer:
/// ```
/// # use momento::cache::{SortedSetFetch, SortedSetElements};
/// # use momento::MomentoResult;
/// # let fetch_response = SortedSetFetch::Hit { value: SortedSetElements::default() };
/// use std::convert::TryInto;
/// let item: MomentoResult<Vec<(Vec<u8>, f64)>> = fetch_response.try_into();
/// ```
#[derive(Debug, PartialEq)]
pub enum SortedSetFetch {
    Hit { value: SortedSetElements },
    Miss,
}

impl SortedSetFetch {
    pub(crate) fn from_fetch_response(response: SortedSetFetchResponse) -> MomentoResult<Self> {
        match response.sorted_set {
            None => Ok(SortedSetFetch::Miss),
            Some(SortedSet::Missing(_)) => Ok(SortedSetFetch::Miss),
            Some(SortedSet::Found(elements)) => match elements.elements {
                None => Ok(SortedSetFetch::Hit {
                    value: SortedSetElements::new(Vec::new()),
                }),
                Some(elements) => match elements {
                    Elements::ValuesWithScores(values_with_scores) => {
                        let elements = values_with_scores
                            .elements
                            .into_iter()
                            .map(|element| (element.value, element.score))
                            .collect();
                        Ok(SortedSetFetch::Hit {
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
        }
    }
}

impl TryFrom<SortedSetFetch> for Vec<(Vec<u8>, f64)> {
    type Error = MomentoError;

    fn try_from(value: SortedSetFetch) -> Result<Self, Self::Error> {
        match value {
            SortedSetFetch::Hit { value: elements } => Ok(elements.elements),
            SortedSetFetch::Miss => Err(MomentoError {
                message: "sorted set was not found".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<SortedSetFetch> for Vec<(String, f64)> {
    type Error = MomentoError;

    fn try_from(value: SortedSetFetch) -> Result<Self, Self::Error> {
        match value {
            SortedSetFetch::Hit { value: elements } => elements.into_strings(),
            SortedSetFetch::Miss => Err(MomentoError {
                message: "sorted set was not found".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl From<Vec<(String, f64)>> for SortedSetFetch {
    fn from(elements: Vec<(String, f64)>) -> Self {
        SortedSetFetch::Hit {
            value: SortedSetElements::new(
                elements
                    .into_iter()
                    .map(|(element, score)| (element.into_bytes(), score))
                    .collect(),
            ),
        }
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct SortedSetElements {
    pub elements: Vec<(Vec<u8>, f64)>,
}

impl SortedSetElements {
    pub fn new(elements: Vec<(Vec<u8>, f64)>) -> Self {
        SortedSetElements { elements }
    }

    pub fn into_strings(self) -> MomentoResult<Vec<(String, f64)>> {
        self.try_into()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

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
