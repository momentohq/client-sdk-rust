use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// The results of a singing key operation.
#[derive(Debug, Serialize, Deserialize)]
pub struct MomentoCreateSigningKeyResponse {
    /// The ID of the key
    pub key_id: String,
    /// Key itself
    pub key: String,
    /// When the key expires
    pub expires_at: SystemTime,
    /// Endpoint for creating a pre-signed url
    pub endpoint: String,
}
