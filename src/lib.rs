pub mod auth;
pub mod response;

mod endpoint_resolver;
mod grpc;
mod jwt;
mod simple_cache_client;
mod utils;

pub use crate::response::MomentoError;
pub use crate::simple_cache_client::{
    Fields, IntoBytes, SimpleCacheClient, SimpleCacheClientBuilder, CollectionTtl,
};

pub type MomentoResult<T> = Result<T, MomentoError>;
