use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MomentoCreateSigningKeyResponse {
    pub key_id: String,
    pub key: String,
    pub expires_at: u64,
    pub endpoint: String,
}
