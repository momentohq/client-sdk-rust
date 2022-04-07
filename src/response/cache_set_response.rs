/// Encapsulates the status of a cache set operation
#[derive(Debug)]
pub enum MomentoSetStatus {
    OK,
    ERROR,
}

/// Response for a cache set operation.
#[derive(Debug)]
pub struct MomentoSetResponse {
    /// The result of a cache set operation.
    pub result: MomentoSetStatus,
}
