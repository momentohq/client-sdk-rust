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
/// See [TopicClient] for an example.
pub struct SubscriptionRequest {
    cache_name: String,
    topic: String,
    resume_at_topic_sequence_number: Option<u64>,
}

impl SubscriptionRequest {
    pub fn new(
        cache_name: impl Into<String>,
        topic: impl Into<String>,
        resume_at_topic_sequence_number: Option<u64>,
    ) -> Self {
        Self {
            cache_name: cache_name.into(),
            topic: topic.into(),
            resume_at_topic_sequence_number,
        }
    }
}

impl MomentoRequest for SubscriptionRequest {
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
            SubscriptionState::Subscribed(stream),
        ))
    }
}
