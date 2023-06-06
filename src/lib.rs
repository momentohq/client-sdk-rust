pub mod auth;
pub mod preview;
pub mod requests;
pub mod response;

mod credential_provider;
mod grpc;
mod simple_cache_client;
mod utils;

pub use crate::credential_provider::{CredentialProvider, CredentialProviderBuilder};
pub use crate::response::ErrorSource;
pub use crate::response::MomentoError;
pub use crate::simple_cache_client::{
    CollectionTtl, Fields, IntoBytes, SimpleCacheClient, SimpleCacheClientBuilder,
};

pub type MomentoResult<T> = Result<T, MomentoError>;

pub mod sorted_set {
    pub use momento_protos::cache_client::sorted_set_fetch_request::{Order, Range};
    pub use momento_protos::cache_client::SortedSetElement;
}
