/* These are the public namespaces shown in import paths e.g.
    use momento::config::configurations::laptop;
    use momento::cache::{Get, GetRequest, Set, SetRequest};
*/
pub mod cache;
/// Configuration structs for the Momento clients.
pub mod config;
pub mod errors;
pub mod topics;
/*************************************************************/

mod cache_client;
mod cache_client_builder;
mod credential_provider;
mod grpc;
mod utils;

pub use self::errors::*;
pub use crate::cache_client::CacheClient;
pub use crate::credential_provider::CredentialProvider;

pub type MomentoResult<T> = Result<T, MomentoError>;

pub use crate::utils::{IntoBytes, IntoBytesIterable};
