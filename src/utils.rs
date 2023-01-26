use tonic::transport::{Channel, ClientTlsConfig, Uri};

use crate::response::MomentoError;
use std::convert::TryFrom;
use std::time::{self, Duration};

pub fn is_ttl_valid(ttl: Duration) -> Result<(), MomentoError> {
    let max_ttl = Duration::from_millis(u64::MAX);
    if ttl > max_ttl {
        return Err(MomentoError::InvalidArgument(format!(
            "TTL provided, {}, needs to be less than the maximum TTL {}",
            ttl.as_secs(),
            max_ttl.as_secs()
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

pub fn connect_channel_lazily(uri_string: &str) -> Result<Channel, MomentoError> {
    let uri = Uri::try_from(uri_string)?;
    let endpoint = Channel::builder(uri)
        .keep_alive_while_idle(true)
        .http2_keep_alive_interval(time::Duration::from_secs(2 * 60))
        .tls_config(ClientTlsConfig::default())?;
    Ok(endpoint.connect_lazy())
}
