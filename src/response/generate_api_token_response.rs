use std::fmt::Debug;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

/// The Momento key and associated endpoint that the key works with. This struct gets base64 encoded
/// and returned as a single string to the customer. This string is what the customer will pass to our
/// sdks, hence why we are calling this struct an ApiToken.
#[derive(Serialize, Deserialize)]
pub struct ApiToken {
    pub api_key: String,
    pub endpoint: String,
}

impl Debug for ApiToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiToken")
            .field("api_key", &"<redacted>")
            .field("endpoint", &self.endpoint)
            .finish()
    }
}

/// The response of a generate api token operation.
#[derive(Serialize, Deserialize)]
pub struct MomentoGenerateApiTokenResponse {
    pub api_token: String,
    pub refresh_token: String,
    pub valid_until: SystemTime,
}

impl Debug for MomentoGenerateApiTokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MomentoGenerateApiTokenResponse")
            .field("api_token", &"<redacted>")
            .field("refresh_token", &"<redacted>")
            .field("valid_until", &self.valid_until)
            .finish()
    }
}
