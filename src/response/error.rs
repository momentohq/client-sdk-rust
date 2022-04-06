use tonic::codegen::http;

/// Exception type for resulting from invalid interactions with Momento Services.
#[derive(Debug)]
pub enum MomentoError {
    /// Momento Service encountered an unexpected exception while trying to fulfill the request.
    InternalServerError(String),
    /// Invalid parameters sent to Momento Services.
    BadRequest(String),
    /// Insufficient permissions to execute an operation.
    PermissionDenied(String),
    /// Authentication token is not provided or is invalid.
    Unauthenticated(String),
    /// Requested resource or the resource on which an operation was requested doesn't exist.
    NotFound(String),
    /// A resource already exists.
    AlreadyExists(String),
    /// Operation was cancelled.
    Cancelled(String),
    /// Requested operation did not complete in allotted time.
    Timeout(String),
    /// Requested operation couldn't be completed because system limits were hit.
    LimitExceeded(String),
    /// Represents all client side exceptions thrown by the SDK.
    /// his exception typically implies that the request wasn't sent to the service successfully or if the service responded, the sdk couldn't interpret the response.
    /// An example would be SDK client was unable to convert the user provided data into a valid request that was expected by the service.
    ClientSdkError(String),
    /// SDK client side validation fails.
    InvalidArgument(String),
}

impl std::fmt::Display for MomentoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MomentoError::InternalServerError(e)
            | MomentoError::BadRequest(e)
            | MomentoError::PermissionDenied(e)
            | MomentoError::Unauthenticated(e)
            | MomentoError::NotFound(e)
            | MomentoError::AlreadyExists(e)
            | MomentoError::Cancelled(e)
            | MomentoError::Timeout(e)
            | MomentoError::LimitExceeded(e)
            | MomentoError::ClientSdkError(e)
            | MomentoError::InvalidArgument(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for MomentoError {}

impl From<http::uri::InvalidUri> for MomentoError {
    fn from(e: http::uri::InvalidUri) -> Self {
        // the uri gets derived from the jwt
        Self::ClientSdkError(e.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for MomentoError {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        let err_msg = "Failed to parse Auth Token".to_string();
        Self::ClientSdkError(err_msg)
    }
}

impl From<String> for MomentoError {
    fn from(s: String) -> Self {
        Self::BadRequest(s)
    }
}

impl From<tonic::transport::Error> for MomentoError {
    fn from(e: tonic::transport::Error) -> Self {
        Self::InternalServerError(e.to_string())
    }
}

impl From<tonic::Status> for MomentoError {
    fn from(s: tonic::Status) -> Self {
        status_to_error(s)
    }
}

fn status_to_error(status: tonic::Status) -> MomentoError {
    match status.code() {
        tonic::Code::InvalidArgument
        | tonic::Code::Unimplemented
        | tonic::Code::OutOfRange
        | tonic::Code::FailedPrecondition => MomentoError::BadRequest(status.message().to_string()),
        tonic::Code::Cancelled => MomentoError::Cancelled(status.message().to_string()),
        tonic::Code::DeadlineExceeded => MomentoError::Timeout(status.message().to_string()),
        tonic::Code::PermissionDenied => {
            MomentoError::PermissionDenied(status.message().to_string())
        }
        tonic::Code::Unauthenticated => MomentoError::Unauthenticated(status.message().to_string()),
        tonic::Code::ResourceExhausted => MomentoError::LimitExceeded(status.message().to_string()),
        tonic::Code::NotFound => MomentoError::NotFound(status.message().to_string()),
        tonic::Code::AlreadyExists => MomentoError::AlreadyExists(status.message().to_string()),
        tonic::Code::Unknown
        | tonic::Code::Aborted
        | tonic::Code::Internal
        | tonic::Code::Unavailable
        | tonic::Code::DataLoss
        | _ => MomentoError::InternalServerError(status.message().to_string()),
    }
}
