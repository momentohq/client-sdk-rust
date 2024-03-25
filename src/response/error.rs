use std::{borrow::Cow, error::Error, fmt::Debug};

use tonic::codegen::http;

use crate::auth::AuthError;

#[derive(Debug)]
pub enum MomentoErrorCode {
    /// Invalid argument passed to Momento client
    InvalidArgumentError,
    /// Service returned an unknown response
    UnknownServiceError,
    /// Resource with specified name already exists
    AlreadyExistsError,
    /// Cache with specified name doesn't exist
    NotFoundError,
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
}

#[derive(Debug, thiserror::Error)]
#[error("{details}")]
pub struct MomentoGrpcErrorDetails {
    pub code: tonic::Code,
    pub details: Cow<'static, str>,
    pub metadata: Option<tonic::metadata::MetadataMap>,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct SdkError {
    pub message: Cow<'static, str>,
    pub error_code: MomentoErrorCode,
    #[source]
    pub inner_error: Option<ErrorSource>,
    pub details: Option<MomentoGrpcErrorDetails>,
}

/// Exception type for resulting from invalid interactions with Momento Services.
#[derive(Debug, thiserror::Error)]
pub enum MomentoError {
    /// System is not in a state required for the operation's execution
    #[error("FailedPreconditionError: {0}")]
    FailedPrecondition(SdkError),

    /// Server was unable to handle the request
    #[error("ServerUnavailable: {0}")]
    ServerUnavailable(SdkError),

    /// Service returned an unknown response
    #[error("UnknownServiceError: {0}")]
    UnknownServiceError(SdkError),

    /// Momento Service encountered an unexpected exception while trying to fulfill the request.
    #[error("InternalServerError: {0}")]
    InternalServerError(SdkError),

    /// Invalid parameters sent to Momento Services.
    #[error("BadRequest: {0}")]
    BadRequest(SdkError),

    /// Insufficient permissions to execute an operation.
    #[error("PermissionDenied: {0}")]
    PermissionDenied(SdkError),

    /// Authentication token is not provided or is invalid.
    #[error("Unauthenticated: {0}")]
    Unauthenticated(SdkError),

    /// Requested resource or the resource on which an operation was requested doesn't exist.
    #[error("NotFound: {0}")]
    NotFound(SdkError),

    /// A resource already exists.
    #[error("AlreadyExists: {0}")]
    AlreadyExists(SdkError),

    /// Operation was cancelled.
    #[error("Cancelled: {0}")]
    Cancelled(SdkError),

    /// Requested operation did not complete in allotted time.
    #[error("Timeout: {0}")]
    Timeout(SdkError),

    /// Requested operation couldn't be completed because system limits were hit.
    #[error("LimitExceeded: {0}")]
    LimitExceeded(SdkError),

    /// Represents all client side exceptions thrown by the SDK.
    /// This exception typically implies that the request wasn't sent to the service successfully or if the service responded, the sdk couldn't interpret the response.
    /// An example would be SDK client was unable to convert the user provided data into a valid request that was expected by the service.
    #[error("ClientSdkError: {0}")]
    ClientSdkError(SdkError),

    /// SDK client side validation fails.
    #[error("InvalidArgument: {0}")]
    InvalidArgument(SdkError),

    /// Requested operation was interrupted.
    /// This may happen to a Topic subscription, or it may show up due to an HTTP2 GOAWAY graceful reconnection request from the server.
    /// Whatever the case, you can probably retry the request.
    #[error("Interrupted: {0}")]
    Interrupted(SdkError),

    /// Tried to use a missing result as a present result.
    /// This may happen when you try to convert a GetResponse directly into a String, when you are opinionated that a miss should ?-propagate out.
    #[error("cannot treat as a hit: {description}")]
    Miss { description: Cow<'static, str> },

    /// Tried to parse a value as something it could not be parsed into.
    /// This may happen when you try to convert a GetResponse directly into a String, when what is in the value is not actually a utf-8 string.
    #[error("cannot treat as the requested type: {description}")]
    TypeError {
        description: Cow<'static, str>,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorSource {
    /// A source you will need to downcast if you need to do something with it.
    #[error("unknown source")]
    Unknown(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// A detailed error from the Auth module
    #[error("auth error")]
    AuthError(#[from] AuthError),

    /// Caused by something in our backing library Tonic
    #[error("tonic transport error")]
    TonicTransport(#[from] tonic::transport::Error),

    /// Caused by something in our backing library Tonic
    #[error("tonic status error")]
    TonicStatus(#[from] tonic::Status),

    /// Caused by a malformed URI
    #[error("uri is invalid")]
    InvalidUri(#[from] http::uri::InvalidUri),
}

impl From<tonic::Status> for MomentoError {
    fn from(s: tonic::Status) -> Self {
        status_to_error(s)
    }
}

fn status_to_error(status: tonic::Status) -> MomentoError {
    log::debug!("translating raw status to error: {status:?}");
    match status.code() {
        tonic::Code::InvalidArgument => MomentoError::InvalidArgument(SdkError {
            message: "Invalid argument passed to Momento client".into(),
            error_code: MomentoErrorCode::InvalidArgumentError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "invalid argument".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::Unimplemented => MomentoError::BadRequest(SdkError {
            message: "The request was invalid; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::BadRequestError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "unimplemented".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::OutOfRange => MomentoError::BadRequest(SdkError {
            message: "The request was invalid; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::BadRequestError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "out of range".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::FailedPrecondition => MomentoError::FailedPrecondition(SdkError {
            message: "System is not in a state required for the operation's execution".into(),
            error_code: MomentoErrorCode::FailedPreconditionError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "failed precondition".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::Cancelled => MomentoError::Cancelled(SdkError {
            message: "The request was cancelled by the server; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::CancelledError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "cancelled".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::DeadlineExceeded => MomentoError::Timeout(SdkError {
            message: "The client's configured timeout was exceeded; you may need to use a Configuration with more lenient timeouts".into(),
            error_code: MomentoErrorCode::TimeoutError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "timed out".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::PermissionDenied => MomentoError::PermissionDenied(SdkError {
            message: "Insufficient permissions to perform an operation on a cache".into(),
            error_code: MomentoErrorCode::PermissionError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "permission denied".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::Unauthenticated => MomentoError::Unauthenticated(SdkError {
            message: "Invalid authentication credentials to connect to cache service".into(),
            error_code: MomentoErrorCode::AuthenticationError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "unauthenticated".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::ResourceExhausted => MomentoError::LimitExceeded(SdkError {
            message: "Request rate, bandwidth, or object size exceeded the limits for this account.  To resolve this error, reduce your usage as appropriate or contact us at support@momentohq.com to request a limit increase".into(),
            error_code: MomentoErrorCode::LimitExceededError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "resource exhausted".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::NotFound => MomentoError::NotFound(SdkError {
            message: "A cache with the specified name does not exist.  To resolve this error, make sure you have created the cache before attempting to use it".into(),
            error_code: MomentoErrorCode::NotFoundError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "not found".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::AlreadyExists => MomentoError::AlreadyExists(SdkError {
            message: "A cache with the specified name already exists.  To resolve this error, either delete the existing cache and make a new one, or use a different name".into(),
            error_code: MomentoErrorCode::AlreadyExistsError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "already exists".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
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
                            MomentoError::Interrupted(SdkError {
                                message: "An unexpected error occurred while trying to fulfill the request; please contact us at support@momentohq.com".into(),
                                error_code: MomentoErrorCode::InternalServerError,
                                inner_error: Some(status.clone().into()),
                                details: MomentoGrpcErrorDetails {
                                    code: status.code(),
                                    details: "the request was interrupted by the server without an error".into(),
                                    metadata: Some(status.metadata().clone())
                                }.into()
                            })
                        } else {
                            MomentoError::ClientSdkError(SdkError {
                                message: "Unknown error has occurred".into(),
                                error_code: MomentoErrorCode::UnknownError,
                                inner_error: Some(status.clone().into()),
                                details: MomentoGrpcErrorDetails {
                                    code: status.code(),
                                    details: "the request was terminated locally without an error".into(),
                                    metadata: Some(status.metadata().clone())
                                }.into()
                            })
                        }
                    } else {
                        MomentoError::InternalServerError(SdkError {
                            message: "An unexpected error occurred while trying to fulfill the request; please contact us at support@momentohq.com".into(),
                            error_code: MomentoErrorCode::InternalServerError,
                            inner_error: Some(status.clone().into()),
                            details: MomentoGrpcErrorDetails {
                                code: status.code(),
                                details: "an internal http2 error terminated the request".into(),
                                metadata: Some(status.metadata().clone())
                            }.into()
                        })
                    }
                }
                None => MomentoError::InternalServerError(SdkError {
                    message: "An unexpected error occurred while trying to fulfill the request; please contact us at support@momentohq.com".into(),
                    error_code: MomentoErrorCode::InternalServerError,
                    inner_error: Some(status.clone().into()),
                    details: MomentoGrpcErrorDetails {
                        code: status.code(),
                        details: "an unknown error terminated the request".into(),
                        metadata: Some(status.metadata().clone())
                    }.into()
                })
            }
        }
        tonic::Code::Aborted => MomentoError::InternalServerError(SdkError {
            message: "An unexpected error occurred while trying to fulfill the request; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::InternalServerError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "aborted".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::Internal => MomentoError::InternalServerError(SdkError {
            message: "An unexpected error occurred while trying to fulfill the request; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::InternalServerError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "internal error".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::Unavailable => MomentoError::ServerUnavailable(SdkError {
            message: "The server was unable to handle the request; consider retrying.  If the error persists, please contact Momento.".into(),
            error_code: MomentoErrorCode::ServerUnavailable,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "service unavailable".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        tonic::Code::DataLoss => MomentoError::InternalServerError(SdkError {
            message: "An unexpected error occurred while trying to fulfill the request; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::InternalServerError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "data loss".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
        _ => MomentoError::UnknownServiceError(SdkError {
            message: "The service returned an unknown response; please contact us at support@momentohq.com".into(),
            error_code: MomentoErrorCode::UnknownServiceError,
            inner_error: Some(status.clone().into()),
            details: MomentoGrpcErrorDetails {
                code: status.code(),
                details: "unknown error".into(),
                metadata: Some(status.metadata().clone())
            }.into()
        }),
    }
}
