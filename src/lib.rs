pub mod cache;
pub use cache::CacheClient;

pub mod config;
mod credential_provider;
pub use credential_provider::CredentialProvider;

pub mod errors;
pub use errors::*;

mod grpc;

pub mod topics;
pub use topics::TopicClient;

mod utils;
pub use crate::utils::{IntoBytes, IntoBytesIterable};

pub type MomentoResult<T> = Result<T, MomentoError>;
