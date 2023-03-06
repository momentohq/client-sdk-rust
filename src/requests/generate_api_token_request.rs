pub enum TokenExpiry {
    Never,
    Expires { valid_for_seconds: u32 },
}
