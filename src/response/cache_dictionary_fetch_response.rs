use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
};

use crate::{MomentoError, MomentoErrorCode};

use super::DictionaryPairs;

/// Response for a dictionary fetch operation.
#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryFetch {
    /// The dictionary item existed in the cache
    Hit { value: DictionaryPairs },
    /// The dictionary item did not exist
    Miss,
}

impl TryFrom<DictionaryFetch> for HashMap<String, Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: DictionaryFetch) -> Result<Self, Self::Error> {
        match value {
            DictionaryFetch::Hit { value } => value.try_into(),
            DictionaryFetch::Miss => Err(MomentoError {
                message: "dictionary was not found".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<DictionaryFetch> for HashMap<String, String> {
    type Error = MomentoError;

    fn try_from(value: DictionaryFetch) -> Result<Self, Self::Error> {
        match value {
            DictionaryFetch::Hit { value } => value.try_into(),
            DictionaryFetch::Miss => Err(MomentoError {
                message: "dictionary was not found".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}
