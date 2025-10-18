use std::{error::Error, fmt::Debug, str::from_utf8};

use momento_protos::protosocket::common::CommandError;
use tonic::codegen::http;
use tonic::metadata::errors::ToStrError;
use tonic::metadata::MetadataMap;

/// Error codes to indicate the type of error that occurred
#[derive(Debug, Clone, PartialEq)]
pub enum MomentoErrorCode {
    /// Invalid argument passed to Momento client
    InvalidArgumentError,
    /// Service returned an unknown response
    UnknownServiceError,
    /// Resource with specified name already exists
    AlreadyExistsError,
    /// Cache with specified name doesn't exist
    CacheNotFoundError,
    /// Store with specified name doesn't exist
    StoreNotFoundError,
    /// Item with specified key doesn't exist
    ItemNotFoundError,
    /// An unexpected error occurred while trying to fulfill the request
    InternalServerError,
    /// Insufficient permissions to perform operation
    PermissionError,
    /// Invalid authentication credentials to connect to service
    AuthenticationError,
    /// Request was cancelled by the server
    CancelledError,
    /// Request rate, bandwidth, or object size exceeded the limits for the account
    LimitExceededError,
    /// Request was invalid
    BadRequestError,
    /// Client's configured timeout was exceeded
    TimeoutError,
    /// Server was unable to handle the request
    ServerUnavailable,
    /// A client resource (most likely memory) was exhausted
    ClientResourceExhausted,
    /// System is not in a state required for the operation's execution
    FailedPreconditionError,
    /// Unknown error has occurred
    UnknownError,
    /// Cache request responded with a Miss
    Miss,
    /// Type error
    TypeError,
}

/// Contains details about the error from GRPC if such details are available
#[derive(Debug, thiserror::Error)]
#[error("{details}")]
pub struct MomentoGrpcErrorDetails {
    /// Error code from our backing library, Tonic
    pub code: tonic::Code,

    /// Error details
    pub details: String,

    /// Error message
    pub message: String,

    /// Error metadata
    pub metadata: tonic::metadata::MetadataMap,
}

impl From<tonic::Status> for MomentoGrpcErrorDetails {
    fn from(status: tonic::Status) -> Self {
        MomentoGrpcErrorDetails {
            code: status.code(),
            details: from_utf8(status.details()).unwrap_or_default().into(),
            message: status.message().into(),
            metadata: status.metadata().clone(),
        }
    }
}

/// Base struct for all errors thrown by the SDK
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct MomentoError {
    /// Error message
    pub message: String,

    /// Error code indicating type of error that occurred
    pub error_code: MomentoErrorCode,

    /// The source of the error if an inner source is provided
    #[source]
    pub inner_error: Option<ErrorSource>,
}

impl MomentoError {
    pub(crate) fn unknown_error(method_name: &str, details: Option<String>) -> Self {
        Self {
            message: "Unknown error has occurred, unable to parse ".to_string()
                + method_name
                + " : "
                + details.as_deref().unwrap_or(""),
            error_code: MomentoErrorCode::UnknownError,
            inner_error: None,
        }
    }

    pub(crate) fn miss(method_name: &str) -> Self {
        Self {
            message: "Received a MISS for ".to_string() + method_name,
            error_code: MomentoErrorCode::UnknownError,
            inner_error: None,
        }
    }

    pub(crate) fn max_concurrent_streams_reached(
        num_grpc_channels: usize,
        num_active_subscriptions: usize,
        max_concurrent_streams: usize,
    ) -> Self {
        Self {
            message: format!("Number of active streams: {num_active_subscriptions}; number of grpc channels: {num_grpc_channels}; max concurrent streams: {max_concurrent_streams}; Already at maximum number of concurrent grpc streams, cannot make new subscribe requests"),
            error_code: MomentoErrorCode::ClientResourceExhausted,
            inner_error: None,
        }
    }

    pub(crate) fn protosocket_command_error(error: CommandError) -> Self {
        Self {
            message: error.message.clone(),
            error_code: MomentoErrorCode::UnknownError,
            inner_error: Some(ProtosocketCacheError::CommandError { cause: error }.into()),
        }
    }

    pub(crate) fn protosocket_unexpected_kind_error() -> Self {
        Self {
            message: "Unexpected return kind!".to_string(),
            error_code: MomentoErrorCode::UnknownError,
            inner_error: Some(ProtosocketCacheError::UnexpectedKind.into()),
        }
    }

    pub(crate) fn protosocket_timeout_error() -> Self {
        Self {
            message: "Protosocket request timed out".to_string(),
            error_code: MomentoErrorCode::TimeoutError,
            inner_error: None,
        }
    }

    /// Returns details about the internal grpc error if available
    pub fn details(&self) -> Option<MomentoGrpcErrorDetails> {
        if let Some(ErrorSource::TonicStatus(status)) = &self.inner_error {
            Some(status.clone().into())
        } else {
            None
        }
    }
}

/// Indicates an error source
#[derive(Debug, thiserror::Error)]
pub enum ErrorSource {
    /// A source you will need to downcast if you need to do something with it.
    #[error("unknown source")]
    Unknown(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// Caused by something in our backing library Tonic
    #[error("tonic transport error")]
    TonicTransport(#[from] tonic::transport::Error),

    /// Caused by something in our backing library Tonic
    #[error("tonic status error")]
    TonicStatus(#[from] tonic::Status),

    /// Caused by a malformed URI
    #[error("uri is invalid")]
    InvalidUri(#[from] http::uri::InvalidUri),

    /// Caused by unparseable response metadata
    #[error("unable to parse response metadata value")]
    MetadataValueError(#[from] ToStrError),

    /// Caused by a protosocket error
    #[error("protosocket error")]
    Protosocket(#[from] ProtosocketCacheError),
}

impl From<tonic::Status> for MomentoError {
    fn from(s: tonic::Status) -> Self {
        status_to_error(s)
    }
}

pub(crate) fn status_to_error(status: tonic::Status) -> MomentoError {
    log::debug!("translating raw status to error: {status:?}");
    match status.code() {
        tonic::Code::InvalidArgument => MomentoError {
            message: "Invalid argument passed to Momento client".into(),
            error_code: MomentoErrorCode::InvalidArgumentError,
            inner_error: Some(status.into()),
        },
        tonic::Code::Unimplemented => MomentoError {
            message: "The request was invalid; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::BadRequestError,
            inner_error: Some(status.clone().into()),
        },
        tonic::Code::OutOfRange => MomentoError {
            message: "The request was invalid; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::BadRequestError,
            inner_error: Some(status.clone().into()),
        },
        tonic::Code::FailedPrecondition => MomentoError {
            message: "System is not in a state required for the operation's execution".into(),
            error_code: MomentoErrorCode::FailedPreconditionError,
            inner_error: Some(status.clone().into()),
        },
        tonic::Code::Cancelled => MomentoError {
            message: "The request was cancelled by the server; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::CancelledError,
            inner_error: Some(status.clone().into()),
        },
        tonic::Code::DeadlineExceeded => MomentoError {
            message: "The client's configured timeout was exceeded; you may need to use a Configuration with more lenient timeouts".into(),
            error_code: MomentoErrorCode::TimeoutError,
            inner_error: Some(status.clone().into()),
        },
        tonic::Code::PermissionDenied => MomentoError {
            message: "Insufficient permissions to perform an operation on a cache".into(),
            error_code: MomentoErrorCode::PermissionError,
            inner_error: Some(status.clone().into()),
        },
        tonic::Code::Unauthenticated => MomentoError {
            message: "Invalid authentication credentials to connect to cache service".into(),
            error_code: MomentoErrorCode::AuthenticationError,
            inner_error: Some(status.clone().into()),
        },
        tonic::Code::ResourceExhausted => MomentoError {
            message: determine_limit_exceeded_message_wrapper(status.metadata(), status.message()),
            error_code: MomentoErrorCode::LimitExceededError,
            inner_error: Some(status.clone().into()),
        },
        tonic::Code::NotFound => {
            match status.metadata().get("err") {
                None => MomentoError {
                    message: "A cache with the specified name does not exist.  To resolve this error, make sure you have created the cache before attempting to use it".into(),
                    error_code: MomentoErrorCode::CacheNotFoundError,
                    inner_error: Some(status.clone().into()),
                },
                Some(err) => match err.to_str() {
                    Ok(err_str) => {
                        match err_str {
                            "store_not_found" => MomentoError {
                                message: "A store with the specified name does not exist.  To resolve this error, make sure you have created the store before attempting to use it".into(),
                                error_code: MomentoErrorCode::StoreNotFoundError,
                                inner_error: Some(status.clone().into()),
                            },
                            "item_not_found" => MomentoError {
                                message: "An item with the specified key does not exist.  To resolve this error, make sure you have created the item before attempting to use it".into(),
                                error_code: MomentoErrorCode::ItemNotFoundError,
                                inner_error: Some(status.clone().into()),
                            },
                            _ => MomentoError {
                                message: "A cache with the specified name does not exist.  To resolve this error, make sure you have created the cache before attempting to use it".into(),
                                error_code: MomentoErrorCode::CacheNotFoundError,
                                inner_error: Some(status.clone().into()),
                            }
                        }
                    }
                    Err(e) => MomentoError {
                        message: "Unknown error has occurred, unable to convert the error metadata into a string".into(),
                        error_code: MomentoErrorCode::UnknownError,
                        inner_error: Some(e.into()),
                    }
                }
            }
        },
        tonic::Code::AlreadyExists => MomentoError {
            message: "A cache with the specified name already exists.  To resolve this error, either delete the existing cache and make a new one, or use a different name".into(),
            error_code: MomentoErrorCode::AlreadyExistsError,
            inner_error: Some(status.clone().into()),
        },
        tonic::Code::Unknown => {
            match status
                .source()
                .and_then(|e| e.downcast_ref::<hyper::Error>())
                .and_then(|hyper_error| hyper_error.source())
                .and_then(|hyper_source| hyper_source.downcast_ref::<h2::Error>())
            {
                Some(h2_detailed_error) => {
                    if Some(h2::Reason::NO_ERROR) == h2_detailed_error.reason() {
                        if h2_detailed_error.is_remote() {
                            MomentoError {
                                message: "An unexpected error occurred while trying to fulfill the request, the request was interrupted by the server without an error; please contact us at support@momentohq.com".into(),
                                error_code: MomentoErrorCode::InternalServerError,
                                inner_error: Some(status.clone().into()),
                            }
                        } else {
                            MomentoError {
                                message: "Unknown error has occurred, the request was terminated locally without an error".into(),
                                error_code: MomentoErrorCode::UnknownError,
                                inner_error: Some(status.clone().into()),
                            }
                        }
                    } else {
                        MomentoError {
                            message: "An unexpected error occurred while trying to fulfill the request, an internal http2 error terminated the request; please contact us at support@momentohq.com".into(),
                            error_code: MomentoErrorCode::InternalServerError,
                            inner_error: Some(status.clone().into()),
                        }
                    }
                }
                None => MomentoError {
                    message: "An unexpected error occurred while trying to fulfill the request, an unknown error terminated the request; please contact us at support@momentohq.com".into(),
                    error_code: MomentoErrorCode::InternalServerError,
                    inner_error: Some(status.clone().into()),
                }
            }
        }
        tonic::Code::Aborted => MomentoError {
            message: "An unexpected error occurred while trying to fulfill the request, request was aborted; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::InternalServerError,
            inner_error: Some(status.clone().into()),
        },
        tonic::Code::Internal => {
            if status.message().to_lowercase().contains("h2 protocol error") {
                MomentoError {
                    message: "Message size limit was exceeded; consider increasing the max send and receive message sizes in your GrpcConfiguration".into(),
                    error_code: MomentoErrorCode::LimitExceededError,
                    inner_error: Some(status.clone().into()),
                }
            } else {
                MomentoError {
                message: "An unexpected internal error occurred while trying to fulfill the request; please contact us at support@momentohq.com".into(),
                error_code: MomentoErrorCode::InternalServerError,
                inner_error: Some(status.clone().into()),
            }
        }
        },
        tonic::Code::Unavailable => MomentoError {
            message: "The server was unavailable to handle the request; consider retrying.  If the error persists, please contact Momento.".into(),
            error_code: MomentoErrorCode::ServerUnavailable,
            inner_error: Some(status.clone().into()),
        },
        tonic::Code::DataLoss => MomentoError {
            message: "An unexpected data loss error occurred while trying to fulfill the request; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::InternalServerError,
            inner_error: Some(status.clone().into()),
        },
        _ => MomentoError {
            message: "The service returned an unknown response; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::UnknownServiceError,
            inner_error: Some(status.clone().into()),
        },
    }
}

enum LimitExceededMessageWrapper {
    TopicSubscriptions,
    OperationsRate,
    ThroughputRate,
    RequestSize,
    ItemSize,
    ElementSize,
    Unknown,
}

impl LimitExceededMessageWrapper {
    pub fn value(&self) -> &str {
        match self {
            LimitExceededMessageWrapper::TopicSubscriptions => {
                "Topic subscriptions limit exceeded for this account"
            }
            LimitExceededMessageWrapper::OperationsRate => {
                "Request rate limit exceeded for this account"
            }
            LimitExceededMessageWrapper::ThroughputRate => {
                "Bandwidth limit exceeded for this account"
            }
            LimitExceededMessageWrapper::RequestSize => {
                "Request size limit exceeded for this account"
            }
            LimitExceededMessageWrapper::ItemSize => "Item size limit exceeded for this account",
            LimitExceededMessageWrapper::ElementSize => {
                "Element size limit exceeded for this account"
            }
            LimitExceededMessageWrapper::Unknown => "Limit exceeded for this account",
        }
    }
}

fn determine_limit_exceeded_message_wrapper(metadata: &MetadataMap, message: &str) -> String {
    let wrapper;

    // If provided, we use the `err` metadata value to determine the most
    // appropriate error message to return.
    if let Some(err_cause) = metadata.get("err") {
        if let Ok(err_str) = err_cause.to_str() {
            wrapper = match err_str {
                "topic_subscriptions_limit_exceeded" => {
                    LimitExceededMessageWrapper::TopicSubscriptions
                }
                "operations_rate_limit_exceeded" => LimitExceededMessageWrapper::OperationsRate,
                "throughput_rate_limit_exceeded" => LimitExceededMessageWrapper::ThroughputRate,
                "request_size_limit_exceeded" => LimitExceededMessageWrapper::RequestSize,
                "item_size_limit_exceeded" => LimitExceededMessageWrapper::ItemSize,
                "element_size_limit_exceeded" => LimitExceededMessageWrapper::ElementSize,
                _ => LimitExceededMessageWrapper::Unknown,
            };
            return wrapper.value().to_string();
        }
    }

    // If `err` metadata is unavailable, try to use the error details field
    // to return an appropriate error message.
    let lower_cased_message = message.to_lowercase();
    wrapper = if lower_cased_message.contains("subscribers") {
        LimitExceededMessageWrapper::TopicSubscriptions
    } else if lower_cased_message.contains("operations") {
        LimitExceededMessageWrapper::OperationsRate
    } else if lower_cased_message.contains("throughput") {
        LimitExceededMessageWrapper::ThroughputRate
    } else if lower_cased_message.contains("request limit") {
        LimitExceededMessageWrapper::RequestSize
    } else if lower_cased_message.contains("item size") {
        LimitExceededMessageWrapper::ItemSize
    } else if lower_cased_message.contains("element size") {
        LimitExceededMessageWrapper::ElementSize
    } else {
        LimitExceededMessageWrapper::Unknown
    };
    wrapper.value().to_string()
}

/// Errors that can occur when using the protosocket cache client
#[derive(Debug, thiserror::Error)]
pub enum ProtosocketCacheError {
    /// Caused by a protosocket client connection error
    #[error("Failed to connect to protosocket: {cause:?}")]
    Protosocket {
        /// The error that occurred when connecting to the protosocket
        #[from]
        cause: protosocket_rpc::Error,
    },

    /// Protosocket command returned an error
    #[error("Command error: {cause:?}")]
    CommandError {
        /// The error that occurred when executing the command
        cause: CommandError,
    },

    /// Protosocket command returned an unexpected kind of response
    #[error("Unexpected return kind!")]
    UnexpectedKind,
}

impl From<protosocket_rpc::Error> for MomentoError {
    fn from(error: protosocket_rpc::Error) -> Self {
        MomentoError {
            message: error.to_string(),
            error_code: MomentoErrorCode::UnknownError,
            inner_error: Some(ErrorSource::Protosocket(
                ProtosocketCacheError::Protosocket { cause: error },
            )),
        }
    }
}
