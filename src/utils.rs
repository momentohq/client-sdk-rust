use thiserror::Error;
use tonic::{
    codegen::http::uri::InvalidUri,
    transport::{Channel, ClientTlsConfig, Uri},
    Request,
};

use crate::MomentoResult;
use crate::{
    config::grpc_configuration::GrpcConfiguration,
    {ErrorSource, MomentoError, MomentoErrorCode},
};
use std::convert::TryFrom;
use std::time::{self, Duration};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn request_meta_data<T>(
    request: &mut tonic::Request<T>,
    cache_name: &str,
) -> MomentoResult<()> {
    tonic::metadata::AsciiMetadataValue::try_from(cache_name)
        .map(|value| {
            request.metadata_mut().append("cache", value);
        })
        .map_err(|e| MomentoError {
            message: format!("Could not treat cache name as a header value: {e}"),
            error_code: MomentoErrorCode::InvalidArgumentError,
            inner_error: Some(crate::ErrorSource::Unknown(Box::new(e))),
            details: None,
        })
}

pub(crate) fn prep_request_with_timeout<R>(
    cache_name: &str,
    timeout: Duration,
    request: R,
) -> MomentoResult<Request<R>> {
    is_cache_name_valid(cache_name)?;

    let mut request = Request::new(request);
    request_meta_data(&mut request, cache_name)?;
    request.set_timeout(timeout);
    Ok(request)
}

pub(crate) fn is_ttl_valid(ttl: Duration) -> MomentoResult<()> {
    let max_ttl = Duration::from_millis(u64::MAX);
    if ttl > max_ttl {
        return Err(MomentoError {
            message: format!(
                "TTL provided, {}, needs to be less than the maximum TTL {}",
                ttl.as_secs(),
                max_ttl.as_secs()
            ),
            error_code: MomentoErrorCode::InvalidArgumentError,
            inner_error: None,
            details: None,
        });
    }
    Ok(())
}

pub(crate) fn is_cache_name_valid(cache_name: &str) -> Result<(), MomentoError> {
    if cache_name.trim().is_empty() {
        return Err(MomentoError {
            message: "Cache name cannot be empty".into(),
            error_code: MomentoErrorCode::InvalidArgumentError,
            inner_error: None,
            details: None,
        });
    }
    Ok(())
}

pub(crate) fn is_key_id_valid(key_id: &str) -> Result<(), MomentoError> {
    if key_id.trim().is_empty() {
        return Err(MomentoError {
            message: "Key ID cannot be empty".into(),
            error_code: MomentoErrorCode::InvalidArgumentError,
            inner_error: None,
            details: None,
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
            ChannelConnectError::BadUri(err) => MomentoError {
                message: "bad uri".into(),
                error_code: MomentoErrorCode::InvalidArgumentError,
                inner_error: Some(ErrorSource::InvalidUri(err)),
                details: None,
            },
            ChannelConnectError::Connection(err) => MomentoError {
                message: "connection failed".into(),
                error_code: MomentoErrorCode::InternalServerError,
                inner_error: Some(ErrorSource::Unknown(err.into())),
                details: None,
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
    let mut channel_builder = Channel::builder(uri).tls_config(ClientTlsConfig::default())?;
    if let Some(keep_alive_while_idle) = grpc_config.keep_alive_while_idle {
        channel_builder = channel_builder.keep_alive_while_idle(keep_alive_while_idle);
    }
    if let Some(keep_alive_interval) = grpc_config.keep_alive_interval {
        channel_builder = channel_builder.http2_keep_alive_interval(keep_alive_interval);
    }
    if let Some(keep_alive_timeout) = grpc_config.keep_alive_timeout {
        channel_builder = channel_builder.keep_alive_timeout(keep_alive_timeout);
    }
    Ok(channel_builder.connect_lazy())
}

pub(crate) fn user_agent(user_agent_name: &str) -> String {
    format!("rust-{user_agent_name}:{VERSION}")
}

pub(crate) fn parse_string(raw: Vec<u8>) -> MomentoResult<String> {
    String::from_utf8(raw).map_err(|e| MomentoError {
        message: "item is not a utf-8 string".to_string(),
        error_code: MomentoErrorCode::TypeError,
        inner_error: Some(ErrorSource::Unknown(Box::new(e))),
        details: None,
    })
}

/// Convenience trait for converting strings into bytes and allowing
/// methods to accept either string or byte values.
pub trait IntoBytes: Send {
    fn into_bytes(self) -> Vec<u8>;
}

impl<T: Send> IntoBytes for T
where
    T: Into<Vec<u8>>,
{
    fn into_bytes(self) -> Vec<u8> {
        self.into()
    }
}

/// Convenience trait for converting a list of IntoBytes items into
/// a list of byte values.
pub trait IntoBytesIterable: Send {
    fn into_bytes(self) -> Vec<Vec<u8>>;
}

impl<T: Send> IntoBytesIterable for Vec<T>
where
    T: IntoBytes,
{
    fn into_bytes(self) -> Vec<Vec<u8>> {
        self.into_iter().map(|item| item.into_bytes()).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_request_meta_data() {
        let mut request = tonic::Request::new(());
        let cache_name = "my_cache";
        let result = request_meta_data(&mut request, cache_name);
        assert!(result.is_ok());
        assert_eq!(
            request.metadata().get("cache"),
            Some(&tonic::metadata::AsciiMetadataValue::from_str(cache_name).unwrap())
        );
    }

    #[test]
    fn test_is_ttl_valid() {
        let ttl = Duration::from_secs(10);
        let result = is_ttl_valid(ttl);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_ttl_valid_max_ttl() {
        let ttl = Duration::from_secs(u64::MAX);
        let result = is_ttl_valid(ttl);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.error_code, MomentoErrorCode::InvalidArgumentError);
        assert_eq!(
            error.message,
            format!(
                "TTL provided, {}, needs to be less than the maximum TTL {}",
                ttl.as_secs(),
                Duration::from_millis(u64::MAX).as_secs()
            )
        );
    }

    #[test]
    fn test_is_cache_name_valid() {
        let cache_name = "my_cache";
        let result = is_cache_name_valid(cache_name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_cache_name_valid_empty() {
        let cache_name = "";
        let result = is_cache_name_valid(cache_name);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.error_code, MomentoErrorCode::InvalidArgumentError);
        assert_eq!(error.message, "Cache name cannot be empty");
    }

    #[test]
    fn test_is_key_id_valid() {
        let key_id = "my_key";
        let result = is_key_id_valid(key_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_key_id_valid_empty() {
        let key_id = "";
        let result = is_key_id_valid(key_id);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.error_code, MomentoErrorCode::InvalidArgumentError);
        assert_eq!(error.message, "Key ID cannot be empty");
    }

    #[tokio::test]
    async fn test_connect_channel_lazily() {
        let uri_string = "http://localhost:50051";
        let result = connect_channel_lazily(uri_string);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connect_channel_lazily_configurable() {
        let uri_string = "http://localhost:50051";
        let grpc_config = GrpcConfiguration {
            keep_alive_while_idle: Some(true),
            keep_alive_interval: Some(Duration::from_secs(30)),
            keep_alive_timeout: Some(Duration::from_secs(60)),
            deadline: Duration::from_secs(30),
        };
        let result = connect_channel_lazily_configurable(uri_string, grpc_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_user_agent() {
        let user_agent_name = "my_app";
        let expected_user_agent = format!("rust-{user_agent_name}:{VERSION}");
        let result = user_agent(user_agent_name);
        assert_eq!(result, expected_user_agent);
    }

    #[test]
    fn test_parse_string() {
        let raw = vec![104, 101, 108, 108, 111];
        let result = parse_string(raw);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_into_bytes() {
        let value = "hello";
        let result: Vec<u8> = value.into_bytes();
        assert_eq!(result, vec![104, 101, 108, 108, 111]);
    }

    #[test]
    fn test_into_bytes_iterable() {
        let values = vec!["hello", "world"];
        let result: Vec<Vec<u8>> = values.into_bytes();
        assert_eq!(
            result,
            vec![vec![104, 101, 108, 108, 111], vec![119, 111, 114, 108, 100]]
        );
    }
}
