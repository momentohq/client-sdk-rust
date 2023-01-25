use thiserror::Error;
use tonic::{
    codegen::http::uri::InvalidUri,
    transport::{Channel, ClientTlsConfig, Uri},
};

use crate::response::MomentoError;
use std::{convert::TryFrom, num::NonZeroU64, time};

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

#[derive(Debug, Error)]
pub(crate) enum ChannelConnectError {
    #[error("URI was invalid: {}", .0)]
    BadUri(#[from] InvalidUri),

    #[error("unable to connect to server: {}", .0)]
    Connection(#[from] tonic::transport::Error),
}

impl From<ChannelConnectError> for MomentoError {
    fn from(value: ChannelConnectError) -> Self {
        match value {
            ChannelConnectError::BadUri(err) => err.into(),
            ChannelConnectError::Connection(err) => err.into(),
        }
    }
}

pub(crate) fn connect_channel_lazily(uri_string: &str) -> Result<Channel, ChannelConnectError> {
    let uri = Uri::try_from(uri_string)?;
    let endpoint = Channel::builder(uri)
        .keep_alive_while_idle(true)
        .http2_keep_alive_interval(time::Duration::from_secs(2 * 60))
        .tls_config(ClientTlsConfig::default())?;
    Ok(endpoint.connect_lazy())
}
