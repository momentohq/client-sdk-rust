/// Encapsulates the status of a cache dictionary set operation
#[derive(Debug)]
pub enum MomentoDictionarySetStatus {
    OK,
    ERROR,
}

/// Response for a cache dictionary set operation.
#[derive(Debug)]
pub struct MomentoDictionarySetResponse {
    /// The result of a cache set operation.
    pub result: MomentoDictionarySetStatus,
}
