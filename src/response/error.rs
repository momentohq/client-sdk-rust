use tonic::codegen::http;

#[derive(Debug)]
pub enum MomentoError {
    InvalidJwt,
    ValidationError(String),
    InternalServerError,
    PermissionDenied,
    Unauthenticated,
    NotFound(String),
    AlreadyExists,
}

impl std::fmt::Display for MomentoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MomentoError::InvalidJwt => write!(f, "Invalid jwt passed"),
            MomentoError::ValidationError(e) => write!(f, "Invalid argument passed: {}", e),
            MomentoError::InternalServerError => write!(f, "Internal server error"),
            MomentoError::PermissionDenied => write!(f, "Permission denied"),
            MomentoError::Unauthenticated => write!(f, "User not authenticated"),
            MomentoError::NotFound(e) => write!(f, "{}", e),
            MomentoError::AlreadyExists => write!(f, "Cache already exists"),
        }
    }
}

impl std::error::Error for MomentoError {}

impl From<http::uri::InvalidUri> for MomentoError {
    fn from(e: http::uri::InvalidUri) -> Self {
        // the uri gets derived from the jwt
        Self::InvalidJwt
    }
}

impl From<jsonwebtoken::errors::Error> for MomentoError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Self::InvalidJwt
    }
}

impl From<String> for MomentoError {
    fn from(s: String) -> Self {
        Self::ValidationError(s)
    }
}

impl From<tonic::transport::Error> for MomentoError {
    fn from(e: tonic::transport::Error) -> Self {
        Self::InternalServerError
    }
}

impl From<tonic::Status> for MomentoError {
    fn from(s: tonic::Status) -> Self {
        status_to_error(s)
    }
}

fn status_to_error(status: tonic::Status) -> MomentoError {
    match status.code() {
        tonic::Code::InvalidArgument => MomentoError::ValidationError(status.message().to_string()),
        tonic::Code::NotFound => MomentoError::NotFound(status.message().to_string()),
        tonic::Code::AlreadyExists => MomentoError::AlreadyExists,
        tonic::Code::PermissionDenied => MomentoError::PermissionDenied,
        tonic::Code::FailedPrecondition => {
            MomentoError::ValidationError(status.message().to_string())
        }
        tonic::Code::Unauthenticated => MomentoError::Unauthenticated,
        _ => MomentoError::InternalServerError,
    }
}
