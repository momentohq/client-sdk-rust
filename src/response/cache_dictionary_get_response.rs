use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
};

use crate::MomentoError;

use super::DictionaryPairs;

/// Response for a cache get operation.
#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryGet {
    /// The dictionary item existed in the cache
    Hit { value: DictionaryPairs },
    /// The dictionary item did not exist
    Miss,
}

impl TryFrom<DictionaryGet> for HashMap<String, Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: DictionaryGet) -> Result<Self, Self::Error> {
        match value {
            DictionaryGet::Hit { value } => value.try_into(),
            DictionaryGet::Miss => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("dictionary was not found"),
            }),
        }
    }
}

impl TryFrom<DictionaryGet> for HashMap<String, String> {
    type Error = MomentoError;

    fn try_from(value: DictionaryGet) -> Result<Self, Self::Error> {
        match value {
            DictionaryGet::Hit { value } => value.try_into(),
            DictionaryGet::Miss => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("dictionary was not found"),
            }),
        }
    }
}
