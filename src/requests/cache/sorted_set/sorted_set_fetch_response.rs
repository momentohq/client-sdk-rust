use std::convert::{TryFrom, TryInto};

use momento_protos::cache_client::sorted_set_fetch_response::found::Elements;
use momento_protos::cache_client::sorted_set_fetch_response::SortedSet;
use momento_protos::cache_client::SortedSetFetchResponse;

use crate::{
    requests::{ErrorSource, MomentoError, MomentoErrorCode},
    MomentoResult,
};

// TODO this needs to be moved to the requests directory

#[derive(Debug, PartialEq)]
pub enum SortedSetFetch {
    Hit { elements: SortedSetElements },
    Miss,
}

impl SortedSetFetch {
    pub(crate) fn from_fetch_response(response: SortedSetFetchResponse) -> MomentoResult<Self> {
        match response.sorted_set {
            None => Ok(SortedSetFetch::Miss),
            Some(SortedSet::Missing(_)) => Ok(SortedSetFetch::Miss),
            Some(SortedSet::Found(elements)) => match elements.elements {
                None => Ok(SortedSetFetch::Hit {
                    elements: SortedSetElements::new(Vec::new()),
                }),
                Some(elements) => match elements {
                    Elements::ValuesWithScores(values_with_scores) => {
                        let elements = values_with_scores
                            .elements
                            .into_iter()
                            .map(|element| (element.value, element.score))
                            .collect();
                        Ok(SortedSetFetch::Hit {
                            elements: SortedSetElements::new(elements),
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
            SortedSetFetch::Hit { elements } => Ok(elements.elements),
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
            SortedSetFetch::Hit { elements } => elements.into_strings(),
            SortedSetFetch::Miss => Err(MomentoError {
                message: "sorted set was not found".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}

#[derive(Debug, PartialEq)]
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

impl TryFrom<SortedSetElements> for Vec<(Vec<u8>, f64)> {
    type Error = MomentoError;

    fn try_from(value: SortedSetElements) -> Result<Self, Self::Error> {
        Ok(value.elements)
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
