/// Response signing key for list of signing keys.
#[derive(Debug)]
pub struct MomentoSigningKey {
    pub key_id: String,
    pub expires_at: u64,
    pub endpoint: String,
}

/// The result of a signing key list operation.
#[derive(Debug)]
pub struct MomentoListSigningKeyResult {
    /// Vector of signing key information defined in MomentoSigningKey.
    pub signing_keys: Vec<MomentoSigningKey>,
    /// Next Page Token returned by Simple Cache Service along with the list of signing keys.
    pub next_token: String,
}
