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
    CollectionTtl, Fields, IntoBytes, SimpleCacheClient, SimpleCacheClientBuilder,
};

pub type MomentoResult<T> = Result<T, MomentoError>;

pub mod sorted_set {
    pub use momento_protos::cache_client::sorted_set_fetch_request::{Order, Range};
    pub use momento_protos::cache_client::SortedSetElement;
}

pub mod tokens {
    pub use momento_protos::control_client::generate_api_token_request::Expires;
    pub use momento_protos::control_client::generate_api_token_request::Expiry;
    pub use momento_protos::control_client::generate_api_token_request::Never;
}
