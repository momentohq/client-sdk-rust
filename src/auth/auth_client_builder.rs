use momento_protos::token::token_client::TokenClient;
use tonic::service::interceptor::InterceptedService;

use crate::{
    grpc::header_interceptor::HeaderInterceptor,
    utils::{self, connect_channel_lazily},
    AuthClient, CredentialProvider, MomentoResult,
};

pub struct AuthClientBuilder<State>(pub State);

pub struct NeedsCredentialProvider(pub ());

pub struct ReadyToBuild {
    credential_provider: CredentialProvider,
}

impl AuthClientBuilder<NeedsCredentialProvider> {
    pub fn credential_provider(
        self,
        credential_provider: CredentialProvider,
    ) -> AuthClientBuilder<ReadyToBuild> {
        AuthClientBuilder(ReadyToBuild {
            credential_provider,
        })
    }
}

impl AuthClientBuilder<ReadyToBuild> {
    pub fn build(self) -> MomentoResult<AuthClient> {
        let agent_value = &utils::user_agent("auth");
        let channel = connect_channel_lazily(&self.0.credential_provider.token_endpoint)?;
        let authorized_channel = InterceptedService::new(
            channel,
            HeaderInterceptor::new(&self.0.credential_provider.auth_token, agent_value),
        );
        Ok(AuthClient {
            token_client: TokenClient::new(authorized_channel),
            credential_provider: self.0.credential_provider,
        })
    }
}
