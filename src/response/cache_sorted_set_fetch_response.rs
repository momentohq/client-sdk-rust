use crate::sorted_set::SortedSetElement;
use crate::MomentoError;
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
            SortedSetFetch::Miss => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("sorted set was not found"),
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
                            return Err::<Self, Self::Error>(MomentoError::TypeError {
                                description: std::borrow::Cow::Borrowed(
                                    "element value was not a valid utf-8 string",
                                ),
                                source: Box::new(e),
                            })
                        }
                    }
                }
                Ok(result)
            }
            SortedSetFetch::Miss => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("sorted set was not found"),
            }),
        }
    }
}
