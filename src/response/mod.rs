mod cache_dictionary_fetch_response;
mod cache_dictionary_get_response;
mod cache_dictionary_increment_response;
mod cache_get_response;
mod cache_set_fetch_response;
mod cache_sorted_set_fetch_response;
mod create_signing_key_response;
mod dictionary_pairs;
mod error;
mod flush_cache_response;
mod generate_api_token_response;
mod list_cache_response;
mod list_signing_keys_response;

use crate::MomentoResult;

pub use self::cache_dictionary_fetch_response::*;
pub use self::cache_dictionary_get_response::*;
pub use self::cache_dictionary_increment_response::*;
pub use self::cache_get_response::*;
pub use self::cache_set_fetch_response::*;
pub use self::cache_sorted_set_fetch_response::*;
pub use self::create_signing_key_response::*;
pub use self::dictionary_pairs::*;
pub use self::error::*;
pub use self::flush_cache_response::*;
pub use self::generate_api_token_response::*;
pub use self::list_cache_response::*;
pub use self::list_signing_keys_response::*;

#[derive(Debug, Clone)]
pub struct ListCacheEntry {
    value: Vec<Vec<u8>>,
}

impl ListCacheEntry {
    pub(crate) fn new(value: Vec<Vec<u8>>) -> Self {
        Self { value }
    }

    pub fn into_value(self) -> Vec<Vec<u8>> {
        self.value
    }

    pub fn value(&self) -> &[Vec<u8>] {
        &self.value
    }
}

pub type MomentoListFetchResponse = Option<ListCacheEntry>;

pub enum MomentoSetDifferenceResponse {
    Found,
    Missing,
}

/// Response type for a response with no data.
#[derive(Debug, Clone)]
pub struct MomentoSetResponse(());

impl MomentoSetResponse {
    pub(crate) fn new() -> Self {
        Self(())
    }
}

/// Response type for a response with no data.
#[derive(Debug, Clone)]
pub struct MomentoDictionarySetResponse(());

impl MomentoDictionarySetResponse {
    pub(crate) fn new() -> Self {
        Self(())
    }
}

/// Response type for a response with no data.
#[derive(Debug, Clone)]
pub struct MomentoDictionaryDeleteResponse(());

impl MomentoDictionaryDeleteResponse {
    pub(crate) fn new() -> Self {
        Self(())
    }
}

/// Response type for a response with no data.
#[derive(Debug, Clone)]
pub struct MomentoDeleteResponse(());

impl MomentoDeleteResponse {
    pub(crate) fn new() -> Self {
        Self(())
    }
}

pub(crate) fn parse_string(raw: Vec<u8>) -> MomentoResult<String> {
    String::from_utf8(raw).map_err(|e| MomentoError::TypeError {
        description: std::borrow::Cow::Borrowed("item is not a utf-8 string"),
        source: Box::new(e),
    })
}
