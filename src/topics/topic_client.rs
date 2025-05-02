use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use momento_protos::cache_client::pubsub::pubsub_client::PubsubClient;
use tonic::{codegen::InterceptedService, transport::Channel};

use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::topics::messages::MomentoRequest;
use crate::topics::topic_client_builder::{NeedsConfiguration, TopicClientBuilder};
use crate::topics::{Configuration, IntoTopicValue, PublishRequest, Subscription};
use crate::{MomentoError, MomentoResult};

use crate::topics::messages::publish::TopicPublishResponse;
use crate::topics::messages::subscribe::SubscribeRequest;

use super::topic_subscription_manager::TopicSubscriptionManager;

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
    pub(crate) unary_client_index: Arc<AtomicUsize>,
    pub(crate) streaming_client_index: Arc<AtomicUsize>,
    pub(crate) unary_clients: Vec<PubsubClient<InterceptedService<Channel, HeaderInterceptor>>>,
    pub(crate) streaming_clients: Vec<TopicSubscriptionManager>,
    pub(crate) configuration: Configuration,
    pub(crate) max_concurrent_streams: usize,
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
        let request = SubscribeRequest::new(cache_name, topic, None, None);
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

    pub(crate) fn get_next_unary_client(
        &self,
    ) -> PubsubClient<InterceptedService<Channel, HeaderInterceptor>> {
        let index = self
            .unary_client_index
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let num_clients = self.unary_clients.len();
        self.unary_clients[index % num_clients].clone()
    }

    pub(crate) fn get_next_streaming_client(
        &self,
    ) -> MomentoResult<PubsubClient<InterceptedService<Channel, HeaderInterceptor>>> {
        // First check if there is enough capacity to make a new subscription.
        self.check_number_of_concurrent_streams()?;

        // Max number of attempts is set to the max number of concurrent streams in order to preserve
        // the round-robin system (incrementing nextManagerIndex) but to not cut short the number
        //  of attempts in case there are many subscriptions starting up at the same time.
        for _ in 0..self.max_concurrent_streams {
            let next_manager_index = self
                .streaming_client_index
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let topic_manager =
                &self.streaming_clients[next_manager_index % self.streaming_clients.len()];
            let new_count = topic_manager.increment_num_active_subscriptions();
            if new_count <= self.max_concurrent_streams {
                log::debug!(
                    "Starting new subscription on grpc channel {} which now has {} streams",
                    next_manager_index % self.streaming_clients.len(),
                    new_count
                );
                return Ok(topic_manager.client().clone());
            }
            topic_manager.decrement_num_active_subscriptions();
        }

        // If no more streams available, return an error
        Err(MomentoError::max_concurrent_streams_reached(
            self.count_number_of_active_subscriptions(),
            self.streaming_clients.len(),
            self.max_concurrent_streams,
        ))
    }

    fn count_number_of_active_subscriptions(&self) -> usize {
        self.streaming_clients
            .iter()
            .map(|client| client.get_num_active_subscriptions())
            .sum()
    }

    fn check_number_of_concurrent_streams(&self) -> MomentoResult<()> {
        let num_active_subscriptions = self.count_number_of_active_subscriptions();
        if num_active_subscriptions >= self.max_concurrent_streams {
            return Err(MomentoError::max_concurrent_streams_reached(
                num_active_subscriptions,
                self.streaming_clients.len(),
                self.max_concurrent_streams,
            ));
        }

        // If we are approaching the maximum number of concurrent streams, log a warning.
        let remaining_streams = self.max_concurrent_streams - num_active_subscriptions;
        if remaining_streams < 10 {
            log::warn!(
                "Only {} streams remaining.  You may hit the limit of {} concurrent streams soon.",
                remaining_streams,
                self.max_concurrent_streams
            );
        }

        Ok(())
    }
}
