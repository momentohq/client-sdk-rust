use std::{borrow::Cow, error::Error, fmt::Debug};

use tonic::codegen::http;

use crate::auth::AuthError;

/// Exception type for resulting from invalid interactions with Momento Services.
#[derive(Debug, thiserror::Error)]
pub enum MomentoError {
    /// Momento Service encountered an unexpected exception while trying to fulfill the request.
    #[error("internal server error: {description}")]
    InternalServerError {
        description: Cow<'static, str>,
        #[source]
        source: ErrorSource,
    },

    /// Invalid parameters sent to Momento Services.
    #[error("bad request: {description}")]
    BadRequest {
        description: Cow<'static, str>,
        #[source]
        source: Option<ErrorSource>,
    },

    /// Insufficient permissions to execute an operation.
    #[error("permission denied: {description}")]
    PermissionDenied {
        description: Cow<'static, str>,
        #[source]
        source: ErrorSource,
    },

    /// Authentication token is not provided or is invalid.
    #[error("the user could not be authenticated: {description}")]
    Unauthenticated {
        description: Cow<'static, str>,
        #[source]
        source: ErrorSource,
    },

    /// Requested resource or the resource on which an operation was requested doesn't exist.
    #[error("not found: {description}")]
    NotFound {
        description: Cow<'static, str>,
        #[source]
        source: ErrorSource,
    },

    /// A resource already exists.
    #[error("resource already exists: {description}")]
    AlreadyExists {
        description: Cow<'static, str>,
        #[source]
        source: ErrorSource,
    },

    /// Operation was cancelled.
    #[error("operation cancelled: {description}")]
    Cancelled {
        description: Cow<'static, str>,
        #[source]
        source: ErrorSource,
    },

    /// Requested operation did not complete in allotted time.
    #[error("operation timed out: {description}")]
    Timeout {
        description: Cow<'static, str>,
        #[source]
        source: ErrorSource,
    },

    /// Requested operation couldn't be completed because system limits were hit.
    #[error("a limit was exceeded: {description}")]
    LimitExceeded {
        description: Cow<'static, str>,
        #[source]
        source: ErrorSource,
    },

    /// Represents all client side exceptions thrown by the SDK.
    /// his exception typically implies that the request wasn't sent to the service successfully or if the service responded, the sdk couldn't interpret the response.
    /// An example would be SDK client was unable to convert the user provided data into a valid request that was expected by the service.
    #[error("client error: {description}")]
    ClientSdkError {
        description: Cow<'static, str>,
        #[source]
        source: ErrorSource,
    },

    /// SDK client side validation fails.
    #[error("invalid argument: {description}")]
    InvalidArgument {
        description: Cow<'static, str>,
        #[source]
        source: Option<ErrorSource>,
    },

    /// Requested operation was interrupted.
    /// This may happen to a Topic subscription, or it may show up due to an HTTP2 GOAWAY graceful reconnection request from the server.
    /// Whatever the case, you can probably retry the request.
    #[error("operation interrupted: {description}")]
    Interrupted {
        description: Cow<'static, str>,
        #[source]
        source: ErrorSource,
    },

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

impl From<String> for MomentoError {
    fn from(description: String) -> Self {
        MomentoError::BadRequest {
            description: description.into(),
            source: None,
        }
    }
}

impl From<tonic::Status> for MomentoError {
    fn from(s: tonic::Status) -> Self {
        status_to_error(s)
    }
}

fn status_to_error(status: tonic::Status) -> MomentoError {
    log::debug!("translating raw status to error: {status:?}");
    match status.code() {
        tonic::Code::InvalidArgument => MomentoError::BadRequest {
            description: "invalid argument".into(),
            source: Some(status.into()),
        },
        tonic::Code::Unimplemented => MomentoError::BadRequest {
            description: "unimplemented".into(),
            source: Some(status.into()),
        },
        tonic::Code::OutOfRange => MomentoError::BadRequest {
            description: "out of range".into(),
            source: Some(status.into()),
        },
        tonic::Code::FailedPrecondition => MomentoError::BadRequest {
            description: "failed precondition".into(),
            source: Some(status.into()),
        },
        tonic::Code::Cancelled => MomentoError::Cancelled {
            description: "cancelled".into(),
            source: status.into(),
        },
        tonic::Code::DeadlineExceeded => MomentoError::Timeout {
            description: "timed out".into(),
            source: status.into(),
        },
        tonic::Code::PermissionDenied => MomentoError::PermissionDenied {
            description: "permission denied".into(),
            source: status.into(),
        },
        tonic::Code::Unauthenticated => MomentoError::Unauthenticated {
            description: "unauthenticated".into(),
            source: status.into(),
        },
        tonic::Code::ResourceExhausted => MomentoError::LimitExceeded {
            description: "resource exhausted".into(),
            source: status.into(),
        },
        tonic::Code::NotFound => MomentoError::NotFound {
            description: "not found".into(),
            source: status.into(),
        },
        tonic::Code::AlreadyExists => MomentoError::AlreadyExists {
            description: "already exists".into(),
            source: status.into(),
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
                            MomentoError::Interrupted {
                                description:
                                    "the request was interrupted by the server without an error"
                                        .into(),
                                source: status.into(),
                            }
                        } else {
                            MomentoError::ClientSdkError {
                                description: "the request was terminated locally without an error"
                                    .into(),
                                source: status.into(),
                            }
                        }
                    } else {
                        MomentoError::InternalServerError {
                            description: "an internal http2 error terminated the request".into(),
                            source: status.into(),
                        }
                    }
                }
                None => MomentoError::InternalServerError {
                    description: "an unknown error terminated the request".into(),
                    source: status.into(),
                },
            }
        }
        tonic::Code::Aborted => MomentoError::InternalServerError {
            description: "aborted".into(),
            source: status.into(),
        },
        tonic::Code::Internal => MomentoError::InternalServerError {
            description: "internal error".into(),
            source: status.into(),
        },
        tonic::Code::Unavailable => MomentoError::InternalServerError {
            description: "service unavailable".into(),
            source: status.into(),
        },
        tonic::Code::DataLoss => MomentoError::InternalServerError {
            description: "data loss".into(),
            source: status.into(),
        },
        _ => MomentoError::InternalServerError {
            description: "unknown error".into(),
            source: status.into(),
        },
    }
}
