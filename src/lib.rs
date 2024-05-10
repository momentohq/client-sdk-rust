//!
//! # Momento
//! Welcome to the Momento Rust SDK!
//!
//! This crate contains a `CacheClient` for interacting with a serverless Momento Cache, and a
//! `TopicClient` for interacting with serverless Momento Topics (pub/sub messaging). For more detailed
//! information see the [Momento documentation](https://docs.momentohq.com).
//! 
//! Construct instances of these clients using `CacheClient::builder()` and `TopicClient::builder()`.
//!
//! A few conventions you will find in the SDK that are worth knowing about:
//!
//! ## Asynchronous APIs
//!
//! All APIs in the SDK are asynchronous and return `Future`s. This means that you will need to use an
//! async runtime; we recommend [`tokio`](https://tokio.rs). Examples that include this dependency
//! and illustrate the use of `#[tokio::main]` can be found in
//! [the `example` directory of the github repo](https://github.com/momentohq/client-sdk-rust/tree/main/example).
//!
//!
//! ## Configuration
//! 
//! Pre-built configurations are provided, with settings such as timeouts and keep-alives tuned
//! to appropriate values for different environments. For example:
//! 
//! - `momento::cache::configurations::Laptop::latest()` - suitable for a development environment with lenient timeouts
//! - `momento::cache::configurations::InRegion::latest()` - suitable for a production configuration with more strict timeouts.
//! 
//! These configurations can be passed to the `CacheClient` and `TopicClient` builders.
//! For advanced use cases you can build your own configurations rather than using the pre-builts.
//!
//! ## Credential Providers
//! 
//! The `CredentialProvider` struct is used to provide the API key for the Momento service. The two
//! most common factory functions for creating a `CredentialProvider` are:
//!
//! - `CredentialProvider::from_env_var` - reads the API key from an environment variable
//! - `CredentialProvider::from_string` - takes the API key as a string; can be used when retrieving the key from a secret manager, etc.
//!
//! ## Error Handling
//! 
//! Most APIs return a `MomentoResult`, which is just a type alias for `Result<T, MomentoError>`. You
//! can use a `match` statement to handle the `Result` or use the `?` operator to propagate errors.
//!
//! ## Enum Response Types, Type Coercion via `into` and `try_into`
//! 
//! Many APIs may have more than one type of response that they can return. For example, `CacheClient::get`
//! may return a cache hit or a cache miss. These response are represented as enums, which you can
//! interact with via a `match` statement, or you can use `try_into` to try to directly coerce the response
//! into your desired type.
//! 
//! All Momento cache values are stored as `vec<u8>`, but if you are using UTF-8 strings, you can use `try_into`
//! for these coercions as well.
//! 
//! Here are a few examples of how you can interact with a `CacheClient::get` response:
//! 
//! Using a `match`:
//! 
//! ```
//! let item: String = match(cache_client.get(&cache_name, "key").await?) {
//!     Get::Hit { value } => value.try_into()?,
//!     Get::Miss => return Err(anyhow::Error::msg("cache miss"))
//! };
//! ```
//! 
//! Or directly via `try_into`:
//! 
//! ```
//! let item: String = cache_client.get(&cache_name, "key").await?.try_into()?;
//! ```
//! 
//! If you are using Momento collection data types, such as lists and dictionaries, we support
//! `into` for the main Rust types that you would expect to be able to use to represent these. For
//! example, for Momento dictionaries:
//! 
//! ```
//! let dictionary: HashMap<String, String> =
//!     cache_client.dictionary_fetch(&cache_name, "dictionary_key")
//!     .await?
//!     .try_into()?;
//! ```
//!

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
