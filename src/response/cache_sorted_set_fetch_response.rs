use crate::response::simple_cache_client_sorted_set::SortedSetElement;
use crate::MomentoErrorCode;
use crate::{ErrorSource, MomentoError};
use core::convert::TryFrom;

#[derive(Debug)]
#[non_exhaustive]
pub struct MomentoSortedSetFetchResponse {
    pub value: Option<Vec<SortedSetElement>>,
}

pub enum SortedSetFetch {
    Hit { elements: Vec<SortedSetElement> },
    Miss,
}

impl TryFrom<SortedSetFetch> for Vec<(Vec<u8>, f64)> {
    type Error = MomentoError;

    fn try_from(value: SortedSetFetch) -> Result<Self, Self::Error> {
        match value {
            SortedSetFetch::Hit { mut elements } => {
                let result: Vec<(Vec<u8>, f64)> =
                    elements.drain(..).map(|e| (e.value, e.score)).collect();
                Ok(result)
            }
            SortedSetFetch::Miss => Err(MomentoError {
                message: "sorted set was not found".to_string(),
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
            SortedSetFetch::Hit { mut elements } => {
                let mut result = Vec::with_capacity(elements.len());
                for element in elements.drain(..) {
                    match String::from_utf8(element.value) {
                        Ok(value) => {
                            result.push((value, element.score));
                        }
                        Err(e) => {
                            return Err::<Self, Self::Error>(MomentoError {
                                message: "element value was not a valid utf-8 string".to_string(),
                                error_code: MomentoErrorCode::TypeError,
                                inner_error: Some(ErrorSource::Unknown(Box::new(e))),
                                details: None,
                            })
                        }
                    }
                }
                Ok(result)
            }
            SortedSetFetch::Miss => Err(MomentoError {
                message: "sorted set was not found".to_string(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}
