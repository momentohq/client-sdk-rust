use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    iter::FromIterator,
};

use crate::{MomentoError, MomentoResult};

use super::parse_string;

/// Response for a cache get operation.
#[derive(Debug)]
pub enum DictionaryGet {
    /// The dictionary item existed in the cache
    Found { value: DictionaryGetValue },
    /// The dictionary item did not exist
    Missing,
}

#[derive(Debug)]
pub struct DictionaryGetValue {
    pub(crate) raw_value: Vec<(Vec<u8>, Vec<u8>)>,
}

impl DictionaryGetValue {
    pub fn new(raw_value: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        Self { raw_value }
    }

    /// Convert a value into a typed collection of your choosing.
    /// ```
    /// # use std::collections::{BTreeMap, HashMap};
    /// # use std::iter::FromIterator;
    /// # use momento::response::DictionaryGetValue;
    /// let value = DictionaryGetValue::new(vec![]);
    /// let different_map: BTreeMap<Vec<u8>, Vec<u8>> = value.collect_into();
    /// ```
    ///
    ///
    /// Or you can get clever and use any collection type you want.
    /// ```
    /// # use std::collections::{BTreeMap, HashMap};
    /// # use std::iter::FromIterator;
    /// # use momento::response::DictionaryGetValue;
    /// let value = DictionaryGetValue::new(vec![]);
    /// let a_list: Vec<(Vec<u8>, Vec<u8>)> = value.collect_into();
    /// ```
    pub fn collect_into<Collection: FromIterator<(Vec<u8>, Vec<u8>)>>(self) -> Collection
    where
        Self: Sized,
    {
        self.raw_value.into_iter().collect()
    }

    pub fn into_string_keys(self) -> MomentoResult<HashMap<String, Vec<u8>>> {
        self.try_into()
    }

    pub fn into_strings(self) -> MomentoResult<HashMap<String, String>> {
        self.try_into()
    }
}

/// You can turn this into anything you want; the iterator is backed by
/// the original vector we read from the network layer, so this is a
/// no-cost operation.
impl IntoIterator for DictionaryGetValue {
    type Item = (Vec<u8>, Vec<u8>);

    type IntoIter = std::vec::IntoIter<(Vec<u8>, Vec<u8>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw_value.into_iter()
    }
}

/// The native type conversion for momento dictionaries
impl From<DictionaryGetValue> for Vec<(Vec<u8>, Vec<u8>)> {
    fn from(value: DictionaryGetValue) -> Self {
        value.raw_value
    }
}

impl From<DictionaryGetValue> for HashMap<Vec<u8>, Vec<u8>> {
    fn from(value: DictionaryGetValue) -> Self {
        value.into_iter().collect()
    }
}

impl TryFrom<DictionaryGetValue> for HashMap<String, Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: DictionaryGetValue) -> Result<Self, Self::Error> {
        value
            .into_iter()
            .map(|(k, v)| parse_string(k).map(|s| (s, v)))
            .collect()
    }
}

impl TryFrom<DictionaryGetValue> for HashMap<String, String> {
    type Error = MomentoError;

    fn try_from(value: DictionaryGetValue) -> Result<Self, Self::Error> {
        value
            .into_iter()
            .map(|(k, v)| {
                parse_string(k).and_then(|key_string| {
                    parse_string(v).map(|value_string| (key_string, value_string))
                })
            })
            .collect()
    }
}

impl TryFrom<DictionaryGet> for HashMap<Vec<u8>, Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: DictionaryGet) -> Result<Self, Self::Error> {
        match value {
            DictionaryGet::Found { value } => Ok(value.into()),
            DictionaryGet::Missing => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("dictionary was not found"),
            }),
        }
    }
}

impl TryFrom<DictionaryGet> for HashMap<String, Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: DictionaryGet) -> Result<Self, Self::Error> {
        match value {
            DictionaryGet::Found { value } => value.try_into(),
            DictionaryGet::Missing => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("dictionary was not found"),
            }),
        }
    }
}

impl TryFrom<DictionaryGet> for HashMap<String, String> {
    type Error = MomentoError;

    fn try_from(value: DictionaryGet) -> Result<Self, Self::Error> {
        match value {
            DictionaryGet::Found { value } => value.try_into(),
            DictionaryGet::Missing => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("dictionary was not found"),
            }),
        }
    }
}
