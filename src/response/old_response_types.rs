/// Response type for a response with no data.
#[derive(Debug, Clone)]
pub struct MomentoDeleteResponse(());

impl MomentoDeleteResponse {
    pub(crate) fn new() -> Self {
        Self(())
    }
}

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
