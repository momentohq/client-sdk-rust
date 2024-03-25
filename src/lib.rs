pub mod auth;
/// Configuration structs for the Momento clients.
pub mod config;
pub mod preview;
pub mod requests;
pub mod response;

mod cache_client;
mod cache_client_builder;
mod compression_utils;
mod credential_provider;
mod grpc;
mod simple_cache_client;
mod utils;

pub use crate::credential_provider::CredentialProvider;
pub use crate::requests::{ErrorSource, MomentoError};
pub use crate::simple_cache_client::{
    CollectionTtl, Fields, IntoBytes, SimpleCacheClient, SimpleCacheClientBuilder,
};

pub use crate::cache_client::CacheClient;

pub type MomentoResult<T> = Result<T, MomentoError>;

pub mod sorted_set {
    pub use momento_protos::cache_client::sorted_set_fetch_request::{Order, Range};
    pub use momento_protos::cache_client::sorted_set_fetch_response::found::Elements;
    pub use momento_protos::cache_client::sorted_set_fetch_response::SortedSet;
    pub use momento_protos::cache_client::SortedSetElement;
}
