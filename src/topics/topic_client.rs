use momento_protos::cache_client::pubsub;
use tonic::{codegen::InterceptedService, transport::Channel};

use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::topics::messages::MomentoRequest;
use crate::topics::topic_client_builder::{NeedsConfiguration, TopicClientBuilder};
use crate::topics::{Configuration, IntoTopicValue, PublishRequest, Subscription};
use crate::{MomentoError, MomentoResult};

use crate::topics::messages::publish::TopicPublish;
use crate::topics::messages::subscribe::SubscriptionRequest;

type ChannelType = InterceptedService<Channel, HeaderInterceptor>;

pub struct TopicClient {
    pub(crate) client: pubsub::pubsub_client::PubsubClient<ChannelType>,
    pub(crate) configuration: Configuration,
}

/// Client to work with Momento Topics, the pub/sub service.
///
/// # Example
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use momento::{CredentialProvider, TopicClient};
/// use futures::StreamExt;
///
/// let topic_client = match TopicClient::builder()
///     .configuration(momento::topics::configurations::laptop::latest())
///     .credential_provider(
///         CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
///             .expect("auth token should be valid"),
///     )
///     .build()
/// {
///     Ok(client) => client,
///     Err(err) => panic!("{err}"),
/// };
///
/// // Publish to a topic
/// topic_client.publish("cache", "topic", "value").await?;
///
/// // Subscribe to a topic and print received messages
/// let mut subscription = topic_client.subscribe("cache", "topic").await?;
/// while let Some(message) = subscription.next().await {
///    println!("Received message: {:?}", message);
/// }
///
/// # Ok(())
/// # })
/// # }
/// ```
impl TopicClient {
    /* constructor */
    pub fn builder() -> TopicClientBuilder<NeedsConfiguration> {
        TopicClientBuilder(NeedsConfiguration(()))
    }

    /// Publish a value to a topic.
    /// The cache is used as a namespace for your topics, and it needs to exist.
    /// You don't create topics, you just start using them.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache to use as a namespace for the topic.
    /// * `topic` - The name of the topic to publish to.
    /// * `value` - The value to publish to the topic.
    ///
    /// # Example
    ///
    /// See [TopicClient] for an example.
    pub async fn publish(
        &self,
        cache_name: impl Into<String>,
        topic: impl Into<String>,
        value: impl IntoTopicValue + std::marker::Send,
    ) -> MomentoResult<TopicPublish> {
        let request = PublishRequest::new(cache_name, topic, value);
        request.send(self).await
    }

    /// Subscribe to a topic.
    /// The cache is used as a namespace for your topics, and it needs to exist.
    /// You don't create topics, you just start using them.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache to use as a namespace for the topic.
    /// * `topic` - The name of the topic to publish to.
    ///
    /// # Optional Arguments
    ///
    /// * `resume_at_topic_sequence_number` - The sequence number to resume from. If not provided, the subscription will start from the latest message or from zero if starting a new subscription.
    ///
    /// # Example
    ///
    /// See [TopicClient] for an example.
    pub async fn subscribe(
        &self,
        cache_name: impl Into<String> + Clone,
        topic: impl Into<String> + Clone,
    ) -> Result<Subscription, MomentoError> {
        let request = SubscriptionRequest::new(cache_name, topic, None);
        request.send(self).await
    }

    /// Lower-level API to send any type of MomentoRequest to the server. This is used for cases when
    /// you want to set optional fields on a request that are not supported by the short-hand API for
    /// that request type.
    ///
    /// See [SubscriptionRequest] for an example of creating a request with optional fields.
    pub async fn send_request<R: MomentoRequest>(&self, request: R) -> MomentoResult<R::Response> {
        request.send(self).await
    }
}
