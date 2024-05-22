use momento_protos::cache_client::pubsub;
use tonic::{codegen::InterceptedService, transport::Channel};

use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::topics::messages::MomentoRequest;
use crate::topics::topic_client_builder::{NeedsConfiguration, TopicClientBuilder};
use crate::topics::{Configuration, IntoTopicValue, PublishRequest, Subscription};
use crate::{MomentoError, MomentoResult};

use crate::topics::messages::publish::TopicPublishResponse;
use crate::topics::messages::subscribe::SubscribeRequest;

type ChannelType = InterceptedService<Channel, HeaderInterceptor>;

/// Client to work with Momento Topics, the pub/sub service.
///
/// # Example
/// To instantiate a `TopicClient`, you need to provide a configuration and a [CredentialProvider](crate::CredentialProvider).
/// Prebuilt configurations tuned for different environments are available in the [topics::configurations](crate::topics::configurations) module.
///
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use momento::{topics::configurations, CredentialProvider, TopicClient};
///
/// let topic_client = match TopicClient::builder()
///     .configuration(configurations::Laptop::latest())
///     .credential_provider(
///         CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
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
pub struct TopicClient {
    pub(crate) client: pubsub::pubsub_client::PubsubClient<ChannelType>,
    pub(crate) configuration: Configuration,
}

impl TopicClient {
    /// Constructs a TopicClient to use Momento Topics
    ///
    /// # Arguments
    /// - `configuration` - Prebuilt configurations tuned for different environments are available in the [topics::configurations](crate::topics::configurations) module.
    /// - `credential_provider` - A [CredentialProvider](crate::CredentialProvider) to use for authenticating with Momento.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # tokio_test::block_on(async {
    /// use momento::{topics::configurations, CredentialProvider, TopicClient};
    ///
    /// let topic_client = match TopicClient::builder()
    ///     .configuration(configurations::Laptop::latest())
    ///     .credential_provider(
    ///         CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
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
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # tokio_test::block_on(async {
    /// use momento::{CredentialProvider, TopicClient};
    /// use momento::topics::TopicPublishResponse;
    /// # let (topic_client, cache_name) = momento_test_util::create_doctest_topic_client();
    ///
    /// // Publish to a topic
    /// match topic_client.publish(cache_name, "topic", "value").await? {
    ///     TopicPublishResponse {} => println!("Published message!"),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn publish(
        &self,
        cache_name: impl Into<String>,
        topic: impl Into<String>,
        value: impl IntoTopicValue + std::marker::Send,
    ) -> MomentoResult<TopicPublishResponse> {
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
    /// # Example
    ///
    /// ```no_run
    /// # fn main() -> anyhow::Result<()> {
    /// # tokio_test::block_on(async {
    /// use momento::{topics::configurations, CredentialProvider, TopicClient};
    /// use futures::StreamExt;
    /// # let (topic_client, cache_name) = momento_test_util::create_doctest_topic_client();
    ///
    /// // Subscribe to a topic. Note: your subscription must be declared as `mut`!
    /// let mut subscription = topic_client.subscribe(cache_name, "topic").await?;
    ///
    /// // Consume messages from the subscription using `next()`
    /// while let Some(message) = subscription.next().await {
    ///    match message.kind {
    ///             momento::topics::ValueKind::Text(t) => println!("Received message as string: {:?}", t),
    ///             momento::topics::ValueKind::Binary(b) => println!("Received message as bytes: {:?}", b),
    ///         }
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    ///
    /// Learn more about how to use a Momento Topics [Subscription].
    pub async fn subscribe(
        &self,
        cache_name: impl Into<String> + Clone,
        topic: impl Into<String> + Clone,
    ) -> Result<Subscription, MomentoError> {
        let request = SubscribeRequest::new(cache_name, topic, None);
        request.send(self).await
    }

    /// Lower-level API to send any type of MomentoRequest to the server. This is used for cases when
    /// you want to set optional fields on a request that are not supported by the short-hand API for
    /// that request type.
    ///
    /// See [SubscribeRequest] for an example of creating a request with optional fields.
    pub async fn send_request<R: MomentoRequest>(&self, request: R) -> MomentoResult<R::Response> {
        request.send(self).await
    }
}
