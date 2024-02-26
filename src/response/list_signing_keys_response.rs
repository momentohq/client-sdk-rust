use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Response signing key for list of signing keys.
#[derive(Debug, Serialize, Deserialize)]
pub struct MomentoSigningKey {
    pub key_id: String,
    pub expires_at: SystemTime,
    pub endpoint: String,
}

/// The result of a signing key list operation.
#[derive(Debug)]
pub struct MomentoListSigningKeyResult {
    /// Vector of signing key information defined in MomentoSigningKey.
    pub signing_keys: Vec<MomentoSigningKey>,
}
