use crate::jwt::{decode_jwt, Claims};
use crate::response::error::MomentoError;
use std::num::NonZeroU64;

pub fn is_ttl_valid(ttl: &NonZeroU64) -> Result<(), MomentoError> {
    let max_ttl = u64::MAX / 1000_u64;
    if ttl.get() > max_ttl {
        return Err(MomentoError::InvalidArgument(format!(
            "TTL provided, {}, needs to be less than the maximum TTL {}",
            ttl, max_ttl
        )));
    }
    Ok(())
}

pub fn is_cache_name_valid(cache_name: &str) -> Result<(), MomentoError> {
    if cache_name.trim().is_empty() {
        return Err(MomentoError::InvalidArgument(
            "Cache name cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub fn is_key_id_valid(key_id: &str) -> Result<(), MomentoError> {
    if key_id.trim().is_empty() {
        return Err(MomentoError::InvalidArgument(
            "Key ID cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub fn get_claims(auth_token: &str) -> Claims {
    decode_jwt(auth_token).unwrap()
}
