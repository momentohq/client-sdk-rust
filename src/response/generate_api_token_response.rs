use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyWithEndpoint {
    pub api_key: String,
    pub endpoint: String,
}

/// The result of a generate api token operation.
#[derive(Debug)]
pub struct MomentoGenerateApiTokenResult {
    pub api_token: String,
    pub refresh_token: String,
    pub valid_until: SystemTime,
}
