use thiserror::Error;
use tonic::{
    codegen::http::uri::InvalidUri,
    transport::{Channel, ClientTlsConfig, Uri},
};

use crate::response::MomentoError;
use crate::MomentoResult;
use std::convert::TryFrom;
use std::time::{self, Duration};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn is_ttl_valid(ttl: Duration) -> MomentoResult<()> {
    let max_ttl = Duration::from_millis(u64::MAX);
    if ttl > max_ttl {
        return Err(MomentoError::InvalidArgument {
            description: format!(
                "TTL provided, {}, needs to be less than the maximum TTL {}",
                ttl.as_secs(),
                max_ttl.as_secs()
            )
            .into(),
            source: None,
        });
    }
    Ok(())
}

pub fn is_cache_name_valid(cache_name: &str) -> Result<(), MomentoError> {
    if cache_name.trim().is_empty() {
        return Err(MomentoError::InvalidArgument {
            description: "Cache name cannot be empty".into(),
            source: None,
        });
    }
    Ok(())
}

pub fn is_key_id_valid(key_id: &str) -> Result<(), MomentoError> {
    if key_id.trim().is_empty() {
        return Err(MomentoError::InvalidArgument {
            description: "Key ID cannot be empty".into(),
            source: None,
        });
    }
    Ok(())
}

#[derive(Debug, Error)]
pub(crate) enum ChannelConnectError {
    #[error("URI was invalid")]
    BadUri(#[from] InvalidUri),

    #[error("unable to connect to server")]
    Connection(#[from] tonic::transport::Error),
}

impl From<ChannelConnectError> for MomentoError {
    fn from(value: ChannelConnectError) -> Self {
        match value {
            ChannelConnectError::BadUri(err) => MomentoError::InvalidArgument {
                description: "bad uri".into(),
                source: Some(err.into()),
            },
            ChannelConnectError::Connection(err) => MomentoError::InternalServerError {
                description: "connection failed".into(),
                source: err.into(),
            },
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

pub fn user_agent(user_agent_name: &str) -> String {
    format!("rust-{user_agent_name}:{VERSION}")
}
