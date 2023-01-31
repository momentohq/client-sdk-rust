pub mod auth;
pub mod preview;
pub mod response;

mod endpoint_resolver;
mod grpc;
mod jwt;
mod simple_cache_client;
mod utils;

pub use crate::response::ErrorSource;
pub use crate::response::MomentoError;
pub use crate::simple_cache_client::{
    CollectionTtl, IntoBytes, SimpleCacheClient, SimpleCacheClientBuilder,
};

pub type MomentoResult<T> = Result<T, MomentoError>;
