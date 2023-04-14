use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
};

use crate::MomentoError;

use super::DictionaryPairs;

/// Response for a dictionary fetch operation.
#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryFetch {
    /// The dictionary item existed in the cache
    Hit { value: DictionaryPairs },
    /// The dictionary item did not exist
    Miss,
}

impl TryFrom<DictionaryFetch> for HashMap<Vec<u8>, Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: DictionaryFetch) -> Result<Self, Self::Error> {
        match value {
            DictionaryFetch::Hit { value } => Ok(value.into()),
            DictionaryFetch::Miss => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("dictionary was not found"),
            }),
        }
    }
}

impl TryFrom<DictionaryFetch> for HashMap<String, Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: DictionaryFetch) -> Result<Self, Self::Error> {
        match value {
            DictionaryFetch::Hit { value } => value.try_into(),
            DictionaryFetch::Miss => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("dictionary was not found"),
            }),
        }
    }
}

impl TryFrom<DictionaryFetch> for HashMap<String, String> {
    type Error = MomentoError;

    fn try_from(value: DictionaryFetch) -> Result<Self, Self::Error> {
        match value {
            DictionaryFetch::Hit { value } => value.try_into(),
            DictionaryFetch::Miss => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("dictionary was not found"),
            }),
        }
    }
}
