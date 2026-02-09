use momento_protos::token::token_client::TokenClient;
use tonic::{codegen::InterceptedService, transport::Channel};

use crate::auth::auth_client_builder::{AuthClientBuilder, NeedsCredentialProvider};
use crate::grpc::header_interceptor::HeaderInterceptor;

use crate::auth::messages::MomentoRequest;
use crate::{utils, CredentialProvider, MomentoResult};

use crate::auth::expiration::ExpiresIn;

use crate::auth::messages::generate_disposable_token::{
    GenerateDisposableTokenRequest, GenerateDisposableTokenResponse,
};

use super::permissions::disposable_token_scope::DisposableTokenScope;

type ChannelType = InterceptedService<Channel, HeaderInterceptor>;

/// Client to work with Momento auth APIs.
///
/// # Example
/// To instantiate an [AuthClient], you need to provide a [CredentialProvider](crate::CredentialProvider).
///
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use momento::{CredentialProvider, AuthClient};
///
/// let auth_client = match AuthClient::builder()
///     .credential_provider(
///         CredentialProvider::from_env_var("V1_API_KEY".to_string())
///             .expect("API key should be valid"),
///     )
///     .build()
/// {
///     Ok(client) => client,
///     Err(err) => panic!("{err}"),
/// };
/// # Ok(())
/// # })
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct AuthClient {
    pub(crate) token_client: TokenClient<ChannelType>,
    pub(crate) credential_provider: CredentialProvider,
}

impl AuthClient {
    /// Creates a new instance of the AuthClient.
    ///
    /// Note: AuthClient does not take a configuration but this is subject to change.
    ///
    /// # Arguments
    /// - `credential_provider` - A [CredentialProvider](crate::CredentialProvider) to use for authenticating with Momento.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # tokio_test::block_on(async {
    /// use momento::{CredentialProvider, AuthClient};
    ///
    /// let auth_client = match AuthClient::builder()
    ///     .credential_provider(
    ///         CredentialProvider::from_env_var("V1_API_KEY".to_string())
    ///             .expect("API key should be valid"),
    ///     )
    ///     .build()
    /// {
    ///     Ok(client) => client,
    ///     Err(err) => panic!("{err}"),
    /// };
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub fn builder() -> AuthClientBuilder<NeedsCredentialProvider> {
        AuthClientBuilder(NeedsCredentialProvider(()))
    }

    /// Generates a new disposable, fine-grained access token.
    ///
    /// # Arguments
    ///
    /// * `scope` - The permission scope that the token will have.
    /// * `expires_in` - The duration for which the token will be valid. Note: disposable tokens must expire within 25 hours.
    ///
    /// # Optional Arguments
    /// If you use [send_request](AuthClient::send_request) to generate a token using a
    /// [GenerateDisposableTokenRequest], you can also provide the following optional arguments:
    ///
    /// * `props` - A collection of optional arguments for the request. Currently contains only `token_id`, which can be used to identify which token was used for messages published on Momento Topics.
    ///
    /// # Example
    /// Assumes that an AuthClient named `auth_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_auth_client;
    /// # tokio_test::block_on(async {
    /// # let auth_client = create_doctest_auth_client();
    /// use momento::auth::{ExpiresIn, DisposableTokenScopes};
    ///
    /// let expiry = ExpiresIn::minutes(5);
    /// let permission_scope = DisposableTokenScopes::cache_key_read_write("cache", "key");
    /// let response = auth_client.generate_disposable_token(permission_scope, expiry).await?;
    /// # assert!(!response.clone().auth_token().is_empty());
    /// println!("Generated disposable token with read-write access to key 'key' in cache 'cache': {}", response);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](AuthClient::send_request) method to get an item using a [GenerateDisposableTokenRequest].
    pub async fn generate_disposable_token(
        &self,
        scope: DisposableTokenScope,
        expires_in: ExpiresIn,
    ) -> MomentoResult<GenerateDisposableTokenResponse> {
        utils::is_disposable_token_expiry_valid(expires_in.clone())?;
        let request = GenerateDisposableTokenRequest::new(scope, expires_in);
        request.send(self).await
    }

    /// Lower-level API to send any type of MomentoRequest to the server. This is used for cases when
    /// you want to set optional fields on a request that are not supported by the short-hand API for
    /// that request type.
    ///
    /// See [GenerateDisposableTokenRequest] for an example of creating a request with optional fields.
    pub async fn send_request<R: MomentoRequest>(&self, request: R) -> MomentoResult<R::Response> {
        request.send(self).await
    }

    pub(crate) fn token_client(&self) -> TokenClient<ChannelType> {
        self.token_client.clone()
    }

    pub(crate) fn credentials(&self) -> CredentialProvider {
        self.credential_provider.clone()
    }
}
