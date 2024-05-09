use momento_protos::cache_client::pubsub::pubsub_client::PubsubClient;
use tonic::service::interceptor::InterceptedService;

use crate::{
    grpc::header_interceptor::HeaderInterceptor,
    topics::Configuration,
    utils::{self, connect_channel_lazily},
    CredentialProvider, MomentoResult, TopicClient,
};

pub struct TopicClientBuilder<State>(pub State);

pub struct NeedsConfiguration(pub ());

pub struct NeedsCredentialProvider {
    configuration: Configuration,
}

pub struct ReadyToBuild {
    configuration: Configuration,
    credential_provider: CredentialProvider,
}

impl TopicClientBuilder<NeedsConfiguration> {
    pub fn configuration(
        self,
        configuration: impl Into<Configuration>,
    ) -> TopicClientBuilder<NeedsCredentialProvider> {
        TopicClientBuilder(NeedsCredentialProvider {
            configuration: configuration.into(),
        })
    }
}

impl TopicClientBuilder<NeedsCredentialProvider> {
    pub fn credential_provider(
        self,
        credential_provider: CredentialProvider,
    ) -> TopicClientBuilder<ReadyToBuild> {
        TopicClientBuilder(ReadyToBuild {
            configuration: self.0.configuration,
            credential_provider,
        })
    }
}

impl TopicClientBuilder<ReadyToBuild> {
    pub fn build(self) -> MomentoResult<TopicClient> {
        let agent_value = &utils::user_agent("sdk");
        let channel = connect_channel_lazily(&self.0.credential_provider.cache_endpoint)?;
        let authorized_channel = InterceptedService::new(
            channel,
            HeaderInterceptor::new(&self.0.credential_provider.auth_token, agent_value),
        );
        Ok(TopicClient {
            client: PubsubClient::new(authorized_channel),
            configuration: self.0.configuration,
        })
    }
}
