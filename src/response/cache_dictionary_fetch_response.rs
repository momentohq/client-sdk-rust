use std::collections::HashMap;

/// Encapsulates the status of a cache disctionary get operation.
#[derive(Debug)]
pub enum MomentoDictionaryFetchStatus {
    /// Status if the dictionary was found
    FOUND,
    /// Status if the dictionary was missing
    MISSING,
    ERROR,
}

/// Response for a cache get operation.
#[derive(Debug)]
pub struct MomentoDictionaryFetchResponse {
    /// The result of a cache dictionary get operation.
    pub result: MomentoDictionaryFetchStatus,
    /// The dictionary contents if it was found.
    pub dictionary: Option<HashMap<Vec<u8>, Vec<u8>>>,
}
