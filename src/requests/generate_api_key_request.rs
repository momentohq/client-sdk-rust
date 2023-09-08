pub enum ApiKeyExpiry {
    Never,
    Expires { valid_for_seconds: u32 },
}
