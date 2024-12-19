use momento_protos::token::token_client::TokenClient;
use tonic::{codegen::InterceptedService, transport::Channel};

use crate::auth::auth_client_builder::{AuthClientBuilder, NeedsCredentialProvider};
use crate::grpc::header_interceptor::HeaderInterceptor;

use crate::auth::messages::MomentoRequest;
use crate::{utils, CredentialProvider, IntoBytes, MomentoResult};

use crate::auth::expiration::ExpiresIn;

use crate::auth::messages::generate_disposable_token::{
    GenerateDisposableTokenRequest, GenerateDisposableTokenResponse,
};

use super::permissions::disposable_token_scope::DisposableTokenScope;

type ChannelType = InterceptedService<Channel, HeaderInterceptor>;

/// TODO
#[derive(Clone, Debug)]
pub struct AuthClient {
    pub(crate) token_client: TokenClient<ChannelType>,
    pub(crate) credential_provider: CredentialProvider,
}

impl AuthClient {
    /// Creates a new instance of the AuthClient.
    ///
    /// Note: AuthClient does not take a configuration but this is subject to change.
    pub fn builder() -> AuthClientBuilder<NeedsCredentialProvider> {
        AuthClientBuilder(NeedsCredentialProvider(()))
    }

    pub async fn generate_disposable_token(
        &self,
        scope: DisposableTokenScope<impl IntoBytes>,
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
