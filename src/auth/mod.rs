/// Objects for representing expiry times.
pub mod expiration;
pub use expiration::{Expiration, ExpiresAt, ExpiresIn};

/// Permissions structure for creating API keys and disposable tokens
pub mod permissions;
pub use permissions::disposable_token_scope::*;
pub use permissions::disposable_token_scopes::DisposableTokenScopes;
pub use permissions::permission_scope::*;
pub use permissions::permission_scopes::PermissionScopes;

/// Auth API requests and responses
pub mod messages;
pub use messages::generate_disposable_token::{
    GenerateDisposableTokenRequest, GenerateDisposableTokenResponse,
};
pub use messages::MomentoRequest;

mod auth_client;
mod auth_client_builder;
pub use auth_client::AuthClient;
