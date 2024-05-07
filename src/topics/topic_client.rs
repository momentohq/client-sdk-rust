use momento_protos::cache_client::pubsub::{
    self, pubsub_client::PubsubClient, PublishRequest, SubscriptionRequest,
};
use tonic::{codegen::InterceptedService, transport::Channel};

use crate::topics::{IntoTopicValue, Subscription, SubscriptionState};
use crate::{
    grpc::header_interceptor::HeaderInterceptor,
    utils::{connect_channel_lazily, user_agent},
    MomentoResult,
};
use crate::{CredentialProvider, MomentoError};

type ChannelType = InterceptedService<Channel, HeaderInterceptor>;

pub struct TopicClient {
    client: pubsub::pubsub_client::PubsubClient<ChannelType>,
}

/// Work with topics, publishing and subscribing.
/// ```rust
/// use momento::topics::TopicClient;
/// use momento::{CredentialProvider};
/// use futures::StreamExt;
///
/// async {
///     let credential_provider = CredentialProvider::from_string("token".to_string())
///        .expect("could not get credentials");
///     // Get a topic client
///     let client = TopicClient::connect(
///         credential_provider,
///         Some("github-demo")
///     ).expect("could not connect");
///
///     // Make a subscription
///     let mut subscription = client
///         .subscribe("some_cache".to_string(), "some topic".to_string(), None)
///         .await
///         .expect("subscribe rpc failed");
///
///     // Consume the subscription
///     while let Some(item) = subscription.next().await {
///         println!("{item:?}")
///     }
/// };
/// ```
impl TopicClient {
    pub fn connect(
        credential_provider: CredentialProvider,
        user_application_name: Option<&str>,
    ) -> MomentoResult<Self> {
        let channel = connect_channel_lazily(&credential_provider.cache_endpoint)?;
        let authorized_channel = InterceptedService::new(
            channel,
            HeaderInterceptor::new(
                &credential_provider.auth_token,
                &user_agent(user_application_name.unwrap_or("sdk")),
            ),
        );
        Ok(Self {
            client: pubsub::pubsub_client::PubsubClient::new(authorized_channel),
        })
    }

    /// Publish a value to a topic.
    /// The cache is used as a namespace for your topics, and it needs to exist.
    /// You don't create topics, you just start using them.
    pub async fn publish(
        &self,
        cache_name: impl Into<String>,
        topic: impl Into<String>,
        value: impl IntoTopicValue,
    ) -> Result<(), MomentoError> {
        TopicClient::actually_publish(&mut self.client.clone(), cache_name, topic, value).await
    }

    /// Publish a value to a topic.
    /// The cache is used as a namespace for your topics, and it needs to exist.
    /// You don't create topics, you just start using them.
    ///
    /// Use this if you have &mut, as it will save you a small amount of overhead for reusing the client.
    pub async fn publish_mut(
        &mut self,
        cache_name: impl Into<String>,
        topic: impl Into<String>,
        value: impl IntoTopicValue,
    ) -> Result<(), MomentoError> {
        TopicClient::actually_publish(&mut self.client, cache_name, topic, value).await
    }

    async fn actually_publish(
        client: &mut PubsubClient<ChannelType>,
        cache_name: impl Into<String>,
        topic: impl Into<String>,
        value: impl IntoTopicValue,
    ) -> Result<(), MomentoError> {
        client
            .publish(PublishRequest {
                cache_name: cache_name.into(),
                topic: topic.into(),
                value: Some(pubsub::TopicValue {
                    kind: Some(value.into_topic_value()),
                }),
            })
            .await?;
        Ok(())
    }

    /// Subscribe to a topic.
    /// The cache is used as a namespace for your topics, and it needs to exist.
    /// You don't create topics, you just start using them.
    pub async fn subscribe(
        &self,
        cache_name: impl Into<String> + Clone,
        topic: impl Into<String> + Clone,
        resume_at_topic_sequence_number: Option<u64>,
    ) -> Result<Subscription, MomentoError> {
        TopicClient::actually_subscribe(
            self.client.clone(),
            cache_name,
            topic,
            resume_at_topic_sequence_number,
        )
        .await
    }

    async fn actually_subscribe(
        mut client: PubsubClient<ChannelType>,
        cache_name: impl Into<String> + Clone,
        topic: impl Into<String> + Clone,
        resume_at_topic_sequence_number: Option<u64>,
    ) -> Result<Subscription, MomentoError> {
        let tonic_stream = client
            .subscribe(SubscriptionRequest {
                cache_name: cache_name.clone().into(),
                topic: topic.clone().into(),
                resume_at_topic_sequence_number: resume_at_topic_sequence_number
                    .unwrap_or_default(),
            })
            .await?
            .into_inner();
        Ok(Subscription::new(
            client,
            cache_name.into(),
            topic.into(),
            resume_at_topic_sequence_number.unwrap_or_default(),
            SubscriptionState::Subscribed(tonic_stream),
        ))
    }
}
