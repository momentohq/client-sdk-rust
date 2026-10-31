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

use super::topic_subscription_manager::{
    StreamReservation, TopicSubscriptionManager, MAX_CONCURRENT_STREAMS_PER_CHANNEL,
};

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
///         CredentialProvider::from_default_env_var_v2()
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
}

impl TopicClient {
    /// Constructs a TopicClient to use Momento Topics
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
    ///         CredentialProvider::from_default_env_var_v2()
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
    ) -> MomentoResult<(
        PubsubClient<InterceptedService<Channel, HeaderInterceptor>>,
        StreamReservation,
    )> {
        let max_concurrent_streams =
            self.streaming_clients.len() * MAX_CONCURRENT_STREAMS_PER_CHANNEL;
        let start = self
            .streaming_client_index
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Choose the starting channel once so concurrent callers cannot make this scan
        // revisit a full channel while skipping another channel with free capacity.
        for offset in 0..self.streaming_clients.len() {
            let index =
                (start % self.streaming_clients.len() + offset) % self.streaming_clients.len();
            let topic_manager = &self.streaming_clients[index];
            if let Some(reservation) = topic_manager.try_reserve() {
                log::debug!(
                    "Starting new subscription on grpc channel {} which now has {} streams",
                    index,
                    topic_manager.get_num_active_subscriptions()
                );
                self.warn_if_near_capacity();
                return Ok((topic_manager.client().clone(), reservation));
            }
        }

        // If no more streams available, return an error
        Err(MomentoError::max_concurrent_streams_reached(
            self.count_number_of_active_subscriptions(),
            self.streaming_clients.len(),
            max_concurrent_streams,
        ))
    }

    fn count_number_of_active_subscriptions(&self) -> usize {
        self.streaming_clients
            .iter()
            .map(|client| client.get_num_active_subscriptions())
            .sum()
    }

    fn warn_if_near_capacity(&self) {
        let max_concurrent_streams =
            self.streaming_clients.len() * MAX_CONCURRENT_STREAMS_PER_CHANNEL;
        let num_active_subscriptions = self.count_number_of_active_subscriptions();
        // If we are approaching the maximum number of concurrent streams, log a warning.
        let remaining_streams = max_concurrent_streams - num_active_subscriptions;
        if remaining_streams < 10 {
            log::warn!(
                "Only {remaining_streams} streams remaining.  You may hit the limit of {max_concurrent_streams} concurrent streams soon."
            );
        }
    }
}

#[cfg(test)]
mod subscription_capacity_tests {
    use super::*;
    use crate::topics::configurations;
    use std::sync::atomic::Ordering;
    use tokio::net::TcpListener;
    use tokio::time::{timeout, Duration};

    // --- Local subscription-capacity helpers ---
    fn local_client(channels: usize, endpoint: &str) -> TopicClient {
        let streaming_clients = (0..channels)
            .map(|_| {
                let channel = Channel::from_shared(endpoint.to_owned())
                    .expect("local endpoint must be valid")
                    .connect_lazy();
                TopicSubscriptionManager::new(PubsubClient::new(InterceptedService::new(
                    channel,
                    HeaderInterceptor::new("local-test-key", "local-test-agent"),
                )))
            })
            .collect();
        TopicClient {
            unary_client_index: Arc::new(AtomicUsize::new(0)),
            streaming_client_index: Arc::new(AtomicUsize::new(0)),
            unary_clients: Vec::new(),
            streaming_clients,
            configuration: configurations::Laptop::latest().into(),
        }
    }

    async fn local_listener() -> (TcpListener, String) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("local listener must bind");
        let endpoint = format!(
            "http://{}",
            listener
                .local_addr()
                .expect("listener must have an address")
        );
        (listener, endpoint)
    }
    // --- End local subscription-capacity helpers ---

    #[tokio::test]
    async fn full_channel_is_skipped_when_another_channel_has_capacity() {
        let client = local_client(2, "http://127.0.0.1:1");
        let mut held = Vec::new();
        for _ in 0..MAX_CONCURRENT_STREAMS_PER_CHANNEL {
            client.streaming_client_index.store(0, Ordering::Relaxed);
            held.push(client.get_next_streaming_client().expect("capacity exists"));
        }
        client.streaming_client_index.store(0, Ordering::Relaxed);
        let _next = client
            .get_next_streaming_client()
            .expect("second channel has capacity");

        assert_eq!(
            client.streaming_clients[0].get_num_active_subscriptions(),
            100
        );
        assert_eq!(
            client.streaming_clients[1].get_num_active_subscriptions(),
            1
        );
    }

    #[tokio::test]
    async fn wrapped_round_robin_index_still_scans_each_channel() {
        let client = local_client(3, "http://127.0.0.1:1");
        let mut held = Vec::new();
        for index in 0..2 {
            for _ in 0..MAX_CONCURRENT_STREAMS_PER_CHANNEL {
                client
                    .streaming_client_index
                    .store(index, Ordering::Relaxed);
                held.push(client.get_next_streaming_client().expect("capacity exists"));
            }
        }
        client
            .streaming_client_index
            .store(usize::MAX, Ordering::Relaxed);
        let _next = client
            .get_next_streaming_client()
            .expect("another channel has capacity");

        assert_eq!(
            client.streaming_clients[0].get_num_active_subscriptions(),
            100
        );
        assert_eq!(
            client.streaming_clients[1].get_num_active_subscriptions(),
            100
        );
        assert_eq!(
            client.streaming_clients[2].get_num_active_subscriptions(),
            1
        );
        assert_eq!(client.streaming_client_index.load(Ordering::Relaxed), 0);

        let _after_wrap = client
            .get_next_streaming_client()
            .expect("final channel still has capacity after index wrap");
        assert_eq!(
            client.streaming_clients[2].get_num_active_subscriptions(),
            2
        );
        assert_eq!(client.streaming_client_index.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn concurrent_callers_share_bounded_capacity_and_release_it() {
        let client = local_client(2, "http://127.0.0.1:1");
        let barrier = Arc::new(std::sync::Barrier::new(9));
        let (active, accepted) = std::thread::scope(|scope| {
            let workers = (0..8)
                .map(|_| {
                    let client = client.clone();
                    let barrier = barrier.clone();
                    scope.spawn(move || {
                        let held = (0..32)
                            .filter_map(|_| client.get_next_streaming_client().ok())
                            .collect::<Vec<_>>();
                        barrier.wait();
                        barrier.wait();
                        held.len()
                    })
                })
                .collect::<Vec<_>>();
            barrier.wait();
            let active = client
                .streaming_clients
                .iter()
                .map(TopicSubscriptionManager::get_num_active_subscriptions)
                .collect::<Vec<_>>();
            barrier.wait();
            let accepted: usize = workers
                .into_iter()
                .map(|worker| worker.join().expect("capacity worker must finish"))
                .sum();
            (active, accepted)
        });

        assert_eq!(active, vec![100, 100]);
        assert_eq!(accepted, 200);
        assert_eq!(client.count_number_of_active_subscriptions(), 0);
    }

    #[tokio::test]
    async fn dropping_reservations_restores_capacity_for_client_clones() {
        let client = local_client(2, "http://127.0.0.1:1");
        let clone = client.clone();
        let mut held = Vec::new();
        for _ in 0..2 * MAX_CONCURRENT_STREAMS_PER_CHANNEL {
            held.push(client.get_next_streaming_client().expect("capacity exists"));
        }
        assert!(clone.get_next_streaming_client().is_err());
        drop(held);

        assert_eq!(clone.count_number_of_active_subscriptions(), 0);
        assert!(clone.get_next_streaming_client().is_ok());
    }

    #[tokio::test]
    async fn failed_subscription_handshake_restores_capacity() {
        let (listener, endpoint) = local_listener().await;
        let server = tokio::spawn(async move {
            let (connection, _) = listener.accept().await.expect("client must connect");
            drop(connection);
        });
        let client = local_client(1, &endpoint);
        let result = timeout(Duration::from_secs(2), client.subscribe("cache", "topic"))
            .await
            .expect("local handshake must terminate");
        assert!(result.is_err());
        server.await.expect("server task must finish");

        assert_eq!(client.count_number_of_active_subscriptions(), 0);
    }

    #[tokio::test]
    async fn cancelling_subscription_handshake_restores_capacity() {
        let (_listener, endpoint) = local_listener().await;
        let client = local_client(1, &endpoint);
        let mut subscribing = Box::pin(client.subscribe("cache", "topic"));
        assert!(futures::poll!(subscribing.as_mut()).is_pending());
        assert_eq!(client.count_number_of_active_subscriptions(), 1);
        drop(subscribing);

        assert_eq!(client.count_number_of_active_subscriptions(), 0);
    }

    #[tokio::test]
    async fn dropping_established_subscription_restores_capacity() {
        let (listener, endpoint) = local_listener().await;
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.expect("client must connect");
            let mut connection = h2::server::handshake(socket)
                .await
                .expect("local HTTP/2 handshake must succeed");
            let mut responses = Vec::new();
            while let Some(request) = connection.accept().await {
                let (_, mut respond) = request.expect("local request must be valid");
                let response = http::Response::builder()
                    .status(200)
                    .header("content-type", "application/grpc")
                    .body(())
                    .expect("local response must be valid");
                responses.push(
                    respond
                        .send_response(response, false)
                        .expect("local response must send"),
                );
            }
        });
        let client = local_client(1, &endpoint);
        let subscription = timeout(Duration::from_secs(2), client.subscribe("cache", "topic"))
            .await
            .expect("local subscription must start")
            .expect("local subscription must succeed");
        assert_eq!(client.count_number_of_active_subscriptions(), 1);
        drop(subscription);
        server.abort();

        assert_eq!(client.count_number_of_active_subscriptions(), 0);
    }
}
