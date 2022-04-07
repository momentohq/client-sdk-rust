/// Encapsulates the status of a cache get operation.
#[derive(Debug)]
pub enum MomentoGetStatus {
    /// Status if an item was found in cache.
    HIT,
    /// Status if an item was not found in cache.
    MISS,
    ERROR,
}

/// Response for a cache get operation.
#[derive(Debug)]
pub struct MomentoGetResponse {
    /// The result of a cache get operation.
    pub result: MomentoGetStatus,
    /// Value stored in the cache as u8 vector.
    pub value: Vec<u8>,
}

impl MomentoGetResponse {
    /// Returns a value stored in the cache as a UTF-8.
    pub fn as_string(&self) -> &str {
        return std::str::from_utf8(self.value.as_slice()).unwrap_or_default();
    }
}
