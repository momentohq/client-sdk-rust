use std::collections::HashMap;

/// Encapsulates the status of a cache disctionary get operation.
#[derive(Debug)]
pub enum MomentoDictionaryGetStatus {
    /// Status if the dictionary was found
    FOUND,
    /// Status if the dictionary was missing
    MISSING,
    ERROR,
}

/// Response for a cache get operation.
#[derive(Debug)]
pub struct MomentoDictionaryGetResponse {
    /// The result of a cache dictionary get operation.
    pub result: MomentoDictionaryGetStatus,
    /// The dictionary contents if it was found.
    pub dictionary: Option<HashMap<Vec<u8>, Vec<u8>>>,
}
