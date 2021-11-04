use std::fmt;

use tonic::{codegen::http, Status};

#[derive(Debug)]
pub enum MomentoError {
    InvalidJwt,
    InvalidArgument(String),
    Unknown,
    InternalServerError,
    PermissionDenied,
    Unauthenticated,
    Unavailable,
    NotFound(String),
    AlreadyExists,
}

impl std::fmt::Display for MomentoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MomentoError::InvalidJwt => write!(f, "Invalid jwt passed"),
            MomentoError::InvalidArgument(e) => write!(f, "Invalid argument passed: {}", e),
            MomentoError::Unknown => write!(f, "Unknown error occured"),
            MomentoError::InternalServerError => write!(f, "Internal server error"),
            MomentoError::PermissionDenied => write!(f, "Permission denied"),
            MomentoError::Unauthenticated => write!(f, "User not authenticated"),
            MomentoError::Unavailable => write!(f, "Service Unavailable"),
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
        Self::InvalidArgument(s)
    }
}

impl From<tonic::transport::Error> for MomentoError {
    fn from(e: tonic::transport::Error) -> Self {
        Self::Unknown
    }
}

impl From<tonic::Status> for MomentoError {
    fn from(s: tonic::Status) -> Self {
        status_to_error(s)
    }
}

fn status_to_error(status: tonic::Status) -> MomentoError {
    match status.code() {
        tonic::Code::Ok => todo!(),
        tonic::Code::Cancelled => MomentoError::Unknown,
        tonic::Code::Unknown => MomentoError::Unknown,
        tonic::Code::InvalidArgument => MomentoError::InvalidArgument(status.message().to_string()),
        tonic::Code::DeadlineExceeded => MomentoError::Unknown,
        tonic::Code::NotFound => MomentoError::NotFound(status.message().to_string()),
        tonic::Code::AlreadyExists => MomentoError::AlreadyExists,
        tonic::Code::PermissionDenied => MomentoError::PermissionDenied,
        tonic::Code::ResourceExhausted => MomentoError::Unknown,
        tonic::Code::FailedPrecondition => MomentoError::Unknown,
        tonic::Code::Aborted => MomentoError::Unknown,
        tonic::Code::OutOfRange => MomentoError::Unknown,
        tonic::Code::Unimplemented => MomentoError::Unknown,
        tonic::Code::Internal => MomentoError::InternalServerError,
        tonic::Code::Unavailable => MomentoError::Unavailable,
        tonic::Code::DataLoss => MomentoError::Unknown,
        tonic::Code::Unauthenticated => MomentoError::Unauthenticated,
    }
}
