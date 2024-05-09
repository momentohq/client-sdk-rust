use crate::{
    topics::{MomentoRequest, Subscription, SubscriptionState},
    utils::prep_request_with_timeout,
    MomentoResult, TopicClient,
};

/// TODO
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
