use tonic::transport::{Channel, Uri, ClientTlsConfig};

use crate::response::error::MomentoError;
use std::{num::NonZeroU64, convert::TryFrom};

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

pub async fn connect_channel(uri_string: &str, lazy: bool) -> Result<Channel, MomentoError> {
    let uri = Uri::try_from(uri_string)?;
    let endpoint = Channel::builder(uri)
        .tls_config(ClientTlsConfig::default())?;
    Ok(
        if lazy {
            endpoint.connect_lazy()
        } else {
            endpoint.connect().await?
        }
    )
}
