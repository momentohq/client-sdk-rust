use std::sync::{atomic::AtomicUsize, Arc};

use crate::{
    grpc::header_interceptor::HeaderInterceptor,
    topics::Configuration,
    utils::{self, connect_channel_lazily, ChannelConnectError},
    CredentialProvider, MomentoResult, TopicClient,
};
use momento_protos::cache_client::pubsub::pubsub_client::PubsubClient;
use tonic::{service::interceptor::InterceptedService, transport::Channel};

use super::topic_subscription_manager::{
    TopicSubscriptionManager, MAX_CONCURRENT_STREAMS_PER_CHANNEL,
};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct TopicClientBuilder<State>(pub State);

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct NeedsConfiguration(pub ());

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct NeedsCredentialProvider {
    configuration: Configuration,
}

#[derive(PartialEq, Eq, Clone, Debug)]
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
        // Create a pool of grpc channels for unary operations. Default to 4 channels.
        // TODO: Make this configurable.
        let mut unary_clients = Vec::new();
        for _ in 0..4 {
            unary_clients.push(create_pubsub_client(
                &self.0.credential_provider.cache_endpoint,
                &self.0.credential_provider.auth_token,
            )?);
        }

        // Create a pool of grpc channels for streaming operations. Default to 4 channels.
        // TODO: Make this configurable.
        let mut streaming_clients = Vec::new();
        let num_stream_clients = 1;
        for _ in 0..num_stream_clients {
            let stream_manager = TopicSubscriptionManager::new(create_pubsub_client(
                &self.0.credential_provider.cache_endpoint,
                &self.0.credential_provider.auth_token,
            )?);
            streaming_clients.push(stream_manager);
        }

        Ok(TopicClient {
            unary_client_index: Arc::new(AtomicUsize::new(0)),
            streaming_client_index: Arc::new(AtomicUsize::new(0)),
            unary_clients,
            streaming_clients,
            configuration: self.0.configuration,
            max_concurrent_streams: num_stream_clients * MAX_CONCURRENT_STREAMS_PER_CHANNEL,
        })
    }
}

fn create_pubsub_client(
    endpoint: &str,
    auth_token: &str,
) -> Result<PubsubClient<InterceptedService<Channel, HeaderInterceptor>>, ChannelConnectError> {
    let agent_value = &utils::user_agent("topic");
    let channel = connect_channel_lazily(endpoint)?;
    let authorized_channel =
        InterceptedService::new(channel, HeaderInterceptor::new(auth_token, agent_value));
    Ok(PubsubClient::new(authorized_channel))
}
