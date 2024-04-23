use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    iter::FromIterator,
};

use crate::{MomentoError, MomentoResult};
use crate::utils::parse_string;

#[derive(Debug, PartialEq, Eq)]
pub struct DictionaryPairs {
    pub(crate) raw_value: Vec<(Vec<u8>, Vec<u8>)>,
}

impl DictionaryPairs {
    pub fn new(raw_value: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        Self { raw_value }
    }

    /// Convert a value into a typed collection of your choosing.
    /// ```
    /// # use std::collections::{BTreeMap, HashMap};
    /// # use std::iter::FromIterator;
    /// # use momento::response::DictionaryPairs;
    /// let value = DictionaryPairs::new(vec![]);
    /// let different_map: BTreeMap<Vec<u8>, Vec<u8>> = value.collect_into();
    /// ```
    ///
    ///
    /// Or you can get clever and use any collection type you want.
    /// ```
    /// # use std::collections::{BTreeMap, HashMap};
    /// # use std::iter::FromIterator;
    /// # use momento::response::DictionaryPairs;
    /// let value = DictionaryPairs::new(vec![]);
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
impl IntoIterator for DictionaryPairs {
    type Item = (Vec<u8>, Vec<u8>);

    type IntoIter = std::vec::IntoIter<(Vec<u8>, Vec<u8>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw_value.into_iter()
    }
}

/// The native type conversion for momento dictionaries
impl From<DictionaryPairs> for Vec<(Vec<u8>, Vec<u8>)> {
    fn from(value: DictionaryPairs) -> Self {
        value.raw_value
    }
}

impl TryFrom<DictionaryPairs> for HashMap<String, Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: DictionaryPairs) -> Result<Self, Self::Error> {
        value
            .into_iter()
            .map(|(k, v)| parse_string(k).map(|s| (s, v)))
            .collect()
    }
}

impl TryFrom<DictionaryPairs> for HashMap<String, String> {
    type Error = MomentoError;

    fn try_from(value: DictionaryPairs) -> Result<Self, Self::Error> {
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
