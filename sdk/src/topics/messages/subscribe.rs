use crate::{
    topics::{MomentoRequest, Subscription, SubscriptionState},
    utils::prep_request_with_timeout,
    MomentoResult, TopicClient,
};

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
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use momento::{CredentialProvider, TopicClient};
/// use futures::StreamExt;
/// use momento::topics::SubscribeRequest;
/// # let (topic_client, cache_name) = momento_test_util::create_doctest_topic_client();
///
/// // Subscribe to a topic and resume from sequence number 10 and page 2
/// let request = SubscribeRequest::new(cache_name, "topic", Some(10), Some(2));
///
/// // Note: your subscription must be declared as `mut`!
/// let mut subscription = topic_client.send_request(request).await?;
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
pub struct SubscribeRequest {
    cache_name: String,
    topic: String,
    resume_at_topic_sequence_number: Option<u64>,
    resume_at_sequence_page: Option<u64>,
}

impl SubscribeRequest {
    /// Create a new SubscribeRequest.
    pub fn new(
        cache_name: impl Into<String>,
        topic: impl Into<String>,
        resume_at_topic_sequence_number: Option<u64>,
        resume_at_sequence_page: Option<u64>,
    ) -> Self {
        Self {
            cache_name: cache_name.into(),
            topic: topic.into(),
            resume_at_topic_sequence_number,
            resume_at_sequence_page,
        }
    }
}

impl MomentoRequest for SubscribeRequest {
    type Response = Subscription;

    async fn send(self, topic_client: &TopicClient) -> MomentoResult<Subscription> {
        let request = prep_request_with_timeout(
            &self.cache_name.to_string(),
            topic_client.configuration.deadline_millis(),
            momento_protos::cache_client::pubsub::SubscriptionRequest {
                cache_name: self.cache_name.to_string(),
                topic: self.topic.to_string(),
                resume_at_topic_sequence_number: self
                    .resume_at_topic_sequence_number
                    .unwrap_or_default(),
                sequence_page: self.resume_at_topic_sequence_number.unwrap_or_default()
            },
        )?;

        let stream = topic_client
            .client
            .clone()
            .subscribe(request)
            .await?
            .into_inner();
        Ok(Subscription::new(
            topic_client.client.clone(),
            self.cache_name,
            self.topic,
            self.resume_at_topic_sequence_number.unwrap_or_default(),
            self.resume_at_sequence_page.unwrap_or_default(),
            SubscriptionState::Subscribed(stream),
        ))
    }
}
