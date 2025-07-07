use crate::{
    functions::FunctionClient,
    grpc::header_interceptor::HeaderInterceptor,
    utils::{self, connect_channel_lazily, ChannelConnectError},
    CredentialProvider, MomentoResult,
};
use momento_protos::function::function_registry_client::FunctionRegistryClient;
use tonic::{service::interceptor::InterceptedService, transport::Channel};

/// A builder for creating a [FunctionClient].
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FunctionClientBuilder<State>(pub(in crate::functions) State);

/// Internal state marker for the builder.
#[derive(PartialEq, Eq, Clone, Debug)]
#[doc(hidden)]
pub struct NeedsCredentialProvider;

/// Internal state marker for the builder.
#[derive(PartialEq, Eq, Clone, Debug)]
#[doc(hidden)]
pub struct ReadyToBuild {
    credential_provider: CredentialProvider,
}

impl FunctionClientBuilder<NeedsCredentialProvider> {
    /// Set the credential provider for the client.
    pub fn credential_provider(
        self,
        credential_provider: CredentialProvider,
    ) -> FunctionClientBuilder<ReadyToBuild> {
        FunctionClientBuilder(ReadyToBuild {
            credential_provider,
        })
    }
}

impl FunctionClientBuilder<ReadyToBuild> {
    /// Build the configured client.
    pub fn build(self) -> MomentoResult<FunctionClient> {
        let ReadyToBuild {
            credential_provider,
        } = self.0;
        let client = create_function_client(
            &credential_provider.cache_endpoint,
            &credential_provider.auth_token,
        )?;

        Ok(FunctionClient::new(client))
    }
}

fn create_function_client(
    endpoint: &str,
    auth_token: &str,
) -> Result<
    FunctionRegistryClient<InterceptedService<Channel, HeaderInterceptor>>,
    ChannelConnectError,
> {
    let agent_value = &utils::user_agent("function");
    let channel = connect_channel_lazily(endpoint)?;
    let authorized_channel =
        InterceptedService::new(channel, HeaderInterceptor::new(auth_token, agent_value));
    Ok(FunctionRegistryClient::new(authorized_channel))
}
