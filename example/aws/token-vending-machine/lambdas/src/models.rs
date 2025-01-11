use serde::{Deserialize, Serialize};

/// Secret Body Definition used for deserializing from the
/// AWS Secerts Manager.  This matches the definition set in the
/// CDK Code in the infra project
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MomentoSecretString {
    pub momento_secret: String,
}

/// VendedToken represents the return payload sent back to the
/// client which holds the disposable token and the epoch when the
/// token expires. The expires_at allows the client to manage when
/// to request a new token
#[derive(Serialize, Debug)]
#[serde(rename = "camelCase")]
pub struct VendedToken {
    pub auth_token: String,
    pub expires_at: u64,
}

/// TokenRequest represents the incoming payload that a client
/// will request a new token with.  It includes the Cache and the Topic
/// which the client is requesting a disposable token for
#[derive(Deserialize, Debug)]
pub struct TokenRequest {
    #[serde(rename = "cacheName")]
    pub cache_name: String,
    #[serde(rename = "topicName")]
    pub topic_name: String,
}
