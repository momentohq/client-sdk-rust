use crate::{utils::parse_string, MomentoError};
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryGetField {
    Hit { value: DictionaryGetFieldValue },
    Miss,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DictionaryGetFieldValue {
    pub(crate) raw_item: Vec<u8>,
}

impl DictionaryGetFieldValue {
    pub fn new(raw_item: Vec<u8>) -> Self {
        Self { raw_item }
    }
}

impl TryFrom<DictionaryGetFieldValue> for String {
    type Error = MomentoError;

    fn try_from(value: DictionaryGetFieldValue) -> Result<Self, Self::Error> {
        parse_string(value.raw_item)
    }
}

impl From<DictionaryGetFieldValue> for Vec<u8> {
    fn from(value: DictionaryGetFieldValue) -> Self {
        value.raw_item
    }
}
