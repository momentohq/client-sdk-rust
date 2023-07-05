use crate::sorted_set::Elements;
use crate::MomentoError;
use core::convert::TryFrom;
use momento_protos::cache_client::sorted_set_fetch_response::SortedSet;

#[derive(Debug)]
#[non_exhaustive]
pub struct MomentoSortedSetFetchResponse {
    pub value: Option<SortedSet>,
}

pub enum SortedSetFetch {
    Found { elements: Elements },
    Missing,
}

impl TryFrom<SortedSetFetch> for Vec<(Vec<u8>, f64)> {
    type Error = MomentoError;

    fn try_from(value: SortedSetFetch) -> Result<Self, Self::Error> {
        match value {
            SortedSetFetch::Found { elements } => match elements {
                Elements::ValuesWithScores(mut values) => {
                    let result: Vec<(Vec<u8>, f64)> = values
                        .elements
                        .drain(..)
                        .map(|e| (e.value, e.score))
                        .collect();
                    Ok(result)
                }
                Elements::Values(_) => Err(MomentoError::TypeError {
                    description: std::borrow::Cow::Borrowed(
                        "response did not contain element scores",
                    ),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "element did not contain a score",
                    )),
                }),
            },
            SortedSetFetch::Missing => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("sorted set was not found"),
            }),
        }
    }
}

impl TryFrom<SortedSetFetch> for Vec<Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: SortedSetFetch) -> Result<Self, Self::Error> {
        match value {
            SortedSetFetch::Found { elements } => match elements {
                Elements::ValuesWithScores(mut values) => {
                    let result: Vec<Vec<u8>> = values.elements.drain(..).map(|e| e.value).collect();
                    Ok(result)
                }
                Elements::Values(values) => Ok(values.values),
            },
            SortedSetFetch::Missing => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("sorted set was not found"),
            }),
        }
    }
}

impl TryFrom<SortedSetFetch> for Vec<(String, f64)> {
    type Error = MomentoError;

    fn try_from(value: SortedSetFetch) -> Result<Self, Self::Error> {
        match value {
            SortedSetFetch::Found { elements } => match elements {
                Elements::ValuesWithScores(mut values) => {
                    let mut result = Vec::with_capacity(values.elements.len());
                    for element in values.elements.drain(..) {
                        let value = String::from_utf8(element.value).map_err(|e| {
                            return Err::<Self, Self::Error>(MomentoError::TypeError {
                                description: std::borrow::Cow::Borrowed(
                                    "element value was not a valid utf-8 string",
                                ),
                                source: Box::new(e),
                            });
                        });
                        result.push((value.unwrap(), element.score));
                    }
                    Ok(result)
                }
                Elements::Values(_) => Err(MomentoError::TypeError {
                    description: std::borrow::Cow::Borrowed(
                        "response did not contain element scores",
                    ),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "element did not contain a score",
                    )),
                }),
            },
            SortedSetFetch::Missing => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("sorted set was not found"),
            }),
        }
    }
}

impl TryFrom<SortedSetFetch> for Vec<String> {
    type Error = MomentoError;

    fn try_from(value: SortedSetFetch) -> Result<Self, Self::Error> {
        match value {
            SortedSetFetch::Found { elements } => match elements {
                Elements::ValuesWithScores(mut values) => {
                    let mut result = Vec::with_capacity(values.elements.len());
                    for element in values.elements.drain(..) {
                        let value = String::from_utf8(element.value).map_err(|e| {
                            return Err::<Self, Self::Error>(MomentoError::TypeError {
                                description: std::borrow::Cow::Borrowed(
                                    "element value was not a valid utf-8 string",
                                ),
                                source: Box::new(e),
                            });
                        });
                        result.push(value.unwrap());
                    }
                    Ok(result)
                }
                Elements::Values(mut values) => {
                    let mut result = Vec::with_capacity(values.values.len());
                    for value in values.values.drain(..) {
                        let value = String::from_utf8(value).map_err(|e| {
                            return Err::<Self, Self::Error>(MomentoError::TypeError {
                                description: std::borrow::Cow::Borrowed(
                                    "element value was not a valid utf-8 string",
                                ),
                                source: Box::new(e),
                            });
                        });
                        result.push(value.unwrap());
                    }
                    Ok(result)
                }
            },
            SortedSetFetch::Missing => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("sorted set was not found"),
            }),
        }
    }
}
