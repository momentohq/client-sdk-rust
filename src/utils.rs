use thiserror::Error;
use tonic::{
    codegen::http::uri::InvalidUri,
    transport::{Channel, ClientTlsConfig, Uri},
};

use crate::config::grpc_configuration::GrpcConfiguration;
use crate::response::MomentoError;
use crate::MomentoResult;
use std::convert::TryFrom;
use std::time::{self, Duration};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn is_ttl_valid(ttl: Duration) -> MomentoResult<()> {
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

pub(crate) fn is_cache_name_valid(cache_name: &str) -> Result<(), MomentoError> {
    if cache_name.trim().is_empty() {
        return Err(MomentoError::InvalidArgument {
            description: "Cache name cannot be empty".into(),
            source: None,
        });
    }
    Ok(())
}

pub(crate) fn is_key_id_valid(key_id: &str) -> Result<(), MomentoError> {
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
        .http2_keep_alive_interval(time::Duration::from_secs(30))
        .tls_config(ClientTlsConfig::default())?;
    Ok(endpoint.connect_lazy())
}

pub(crate) fn connect_channel_lazily_configurable(
    uri_string: &str,
    grpc_config: GrpcConfiguration,
) -> Result<Channel, ChannelConnectError> {
    let uri = Uri::try_from(uri_string)?;
    let endpoint = if grpc_config.keep_alive_while_idle {
        Channel::builder(uri)
            .keep_alive_while_idle(true)
            .http2_keep_alive_interval(grpc_config.keep_alive_interval)
            .keep_alive_timeout(grpc_config.keep_alive_timeout)
            .tls_config(ClientTlsConfig::default())?
    } else {
        Channel::builder(uri)
            .keep_alive_while_idle(false)
            .tls_config(ClientTlsConfig::default())?
    };
    Ok(endpoint.connect_lazy())
}

pub(crate) fn user_agent(user_agent_name: &str) -> String {
    format!("rust-{user_agent_name}:{VERSION}")
}

pub(crate) fn parse_string(raw: Vec<u8>) -> MomentoResult<String> {
    String::from_utf8(raw).map_err(|e| MomentoError::TypeError {
        description: std::borrow::Cow::Borrowed("item is not a utf-8 string"),
        source: Box::new(e),
    })
}
