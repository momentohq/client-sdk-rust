#[derive(Debug)]
pub struct MomentoCreateSigningKeyResponse {
    pub user_id: String,
    pub endpoint: String,
    pub key: String,
    pub expires_at: u64,
}
