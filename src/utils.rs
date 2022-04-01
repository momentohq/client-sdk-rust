use crate::jwt::decode_jwt;
use crate::response::error::MomentoError;

pub fn is_ttl_valid(ttl: &u32) -> Result<(), MomentoError> {
    // max_ttl will be 4294967 since 2^32 / 1000 = 4294967.296
    let max_ttl = u32::MAX / 1000 as u32;
    if *ttl > max_ttl {
        return Err(MomentoError::InvalidArgument(format!(
            "TTL provided, {}, needs to be less than the maximum TTL {}",
            ttl, max_ttl
        )));
    }
    return Ok(());
}

pub fn is_cache_name_valid(cache_name: &str) -> Result<(), MomentoError> {
    if cache_name.trim().is_empty() {
        return Err(MomentoError::InvalidArgument(
            "Cache name cannot be empty".to_string(),
        ));
    }
    return Ok(());
}

pub fn is_key_id_valid(key_id: &str) -> Result<(), MomentoError> {
    if key_id.trim().is_empty() {
        return Err(MomentoError::InvalidArgument(
            "Key ID cannot be empty".to_string(),
        ));
    }
    return Ok(());
}

pub fn get_sub(auth_token: &str) -> String {
    return decode_jwt(&auth_token).unwrap().sub;
}
