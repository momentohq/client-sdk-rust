use crate::{
    auth::{
        expiration::{ExpiresAt, ExpiresIn},
        permissions::disposable_token_scope::DisposableTokenScope,
    },
    utils::is_disposable_token_id_valid,
    AuthClient, IntoBytes, MomentoResult,
};

use super::{permissions_conversions::permissions_from_disposable_token_scope, MomentoRequest};
use base64::{engine::general_purpose, Engine as _};
use momento_protos::token::generate_disposable_token_request::Expires;

/// Optional arguments for generating a disposable token.
/// Currently, the only optional argument is the `token_id`,
/// which can be used to identify which token was used for
/// publishing messages on Momento Topics.
pub struct DisposableTokenProps {
    pub token_id: Option<String>,
}

pub struct GenerateDisposableTokenRequest<K: IntoBytes> {
    scope: DisposableTokenScope<K>,
    expires_in: ExpiresIn,
    props: Option<DisposableTokenProps>,
}

impl<K: IntoBytes> GenerateDisposableTokenRequest<K> {
    pub fn new(scope: DisposableTokenScope<K>, expires_in: ExpiresIn) -> Self {
        Self {
            scope,
            expires_in,
            props: None,
        }
    }

    /// Set the optional DisposableTokenProps for the request.
    pub fn props(mut self, props: DisposableTokenProps) -> Self {
        self.props = Some(props);
        self
    }

    /// Set the optional `token_id`` field of the optional DisposableTokenProps for the request.
    pub fn token_id(mut self, token_id: String) -> Self {
        self.props = Some(DisposableTokenProps {
            token_id: Some(token_id),
        });
        self
    }
}

impl<K: IntoBytes> MomentoRequest for GenerateDisposableTokenRequest<K> {
    type Response = GenerateDisposableTokenResponse;

    async fn send(self, client: &AuthClient) -> MomentoResult<Self::Response> {
        let request = momento_protos::token::GenerateDisposableTokenRequest {
            expires: Some(Expires {
                valid_for_seconds: self.expires_in.to_seconds() as u32,
            }),
            auth_token: client.credentials().auth_token,
            permissions: Some(permissions_from_disposable_token_scope(self.scope)),
            token_id: match self.props {
                Some(props) => match props.token_id {
                    Some(token_id) => {
                        is_disposable_token_id_valid(&token_id)?;
                        token_id
                    }
                    None => "".to_string(),
                },
                None => "".to_string(),
            },
        };
        let response = client
            .token_client()
            .generate_disposable_token(request)
            .await?
            .into_inner();

        // We must b64 encode {endpoint: endpoint, api_key: apiKey} to return a valid
        // auth token that can be accepted by a CredentialProvider.
        let auth_token = general_purpose::STANDARD.encode(format!(
            "{{\"endpoint\": \"{}\", \"api_key\": \"{}\"}}",
            response.endpoint, response.api_key
        ));

        Ok(GenerateDisposableTokenResponse {
            auth_token,
            endpoint: response.endpoint,
            expires_at: ExpiresAt::from_epoch(response.valid_until),
        })
    }
}

/// Response for a generate disposable token operation.
///
/// TODO: examples
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GenerateDisposableTokenResponse {
    auth_token: String,
    endpoint: String,
    expires_at: ExpiresAt,
}

impl std::fmt::Display for GenerateDisposableTokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GenerateDisposableTokenResponse {{ auth_token beginning with: {}, endpoint: {}, expires_at epoch: {} }}",
            &self.auth_token[..5], self.endpoint, self.expires_at.epoch()
        )
    }
}

impl GenerateDisposableTokenResponse {
    /// Returns the generated disposable token.
    pub fn auth_token(self) -> String {
        self.auth_token
    }

    /// Returns the endpoint to connect to.
    pub fn endpoint(self) -> String {
        self.endpoint
    }

    /// Returns when the token expires.
    pub fn expires_at(self) -> ExpiresAt {
        self.expires_at
    }
}
