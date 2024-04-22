/* These are the public namespaces shown in import paths e.g.
    use momento::config::configurations::laptop;
    use momento::cache::{Get, GetRequest, Set, SetRequest};
*/
pub mod auth;
pub mod cache;
/// Configuration structs for the Momento clients.
pub mod config;
pub mod errors;
pub mod response;
pub mod topics;
/*************************************************************/

mod cache_client;
mod cache_client_builder;
mod compression_utils;
mod credential_provider;
mod grpc;
mod simple_cache_client;
mod utils;

pub use self::errors::*;
pub use crate::credential_provider::CredentialProvider;
pub use crate::response::simple_cache_client_sorted_set;
pub use crate::simple_cache_client::{
    CollectionTtl, Fields, IntoBytes, SimpleCacheClient, SimpleCacheClientBuilder,
};

pub use crate::cache_client::CacheClient;

pub type MomentoResult<T> = Result<T, MomentoError>;
