use momento_protos::cache_client::pubsub::{
    self, pubsub_client::PubsubClient, PublishRequest, SubscriptionRequest,
};
use tonic::{codegen::InterceptedService, transport::Channel};

use crate::MomentoError;
use crate::{
    endpoint_resolver::MomentoEndpointsResolver,
    grpc::header_interceptor::HeaderInterceptor,
    utils::{connect_channel_lazily, user_agent},
    MomentoResult,
};

type ChannelType = InterceptedService<Channel, HeaderInterceptor>;

pub struct TopicClient {
    client: pubsub::pubsub_client::PubsubClient<ChannelType>,
}

/// Work with topics, publishing and subscribing.
/// ```rust
/// use momento::preview::topics::TopicClient;
///
/// async {
///     // Get a topic client
///     let client = TopicClient::connect(
///         "token".to_string(),
///         Some("some.momento.endpoint".to_string()),
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
///     while let Some(item) = subscription.item().await.expect("subscription stream interrupted") {
///         println!("{item:?}")
///     }
/// };
/// ```
impl TopicClient {
    pub fn connect(
        auth_token: String,
        momento_endpoint: Option<String>,
        user_application_name: Option<&str>,
    ) -> MomentoResult<Self> {
        let momento_endpoints = MomentoEndpointsResolver::resolve(&auth_token, momento_endpoint)?;
        let channel = connect_channel_lazily(&momento_endpoints.data_endpoint.url)?;
        let authorized_channel = InterceptedService::new(
            channel,
            HeaderInterceptor::new(
                &auth_token,
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
        cache_name: String,
        topic: String,
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
        cache_name: String,
        topic: String,
        value: impl IntoTopicValue,
    ) -> Result<(), MomentoError> {
        TopicClient::actually_publish(&mut self.client, cache_name, topic, value).await
    }

    async fn actually_publish(
        client: &mut PubsubClient<ChannelType>,
        cache_name: String,
        topic: String,
        value: impl IntoTopicValue,
    ) -> Result<(), MomentoError> {
        client
            .publish(PublishRequest {
                cache_name,
                topic,
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
        cache_name: String,
        topic: String,
        resume_at_topic_sequence_number: Option<u64>,
    ) -> Result<Subscription, MomentoError> {
        TopicClient::actually_subscribe(
            &mut self.client.clone(),
            cache_name,
            topic,
            resume_at_topic_sequence_number,
        )
        .await
    }

    /// Subscribe to a topic.
    /// The cache is used as a namespace for your topics, and it needs to exist.
    /// You don't create topics, you just start using them.
    ///
    /// Use this if you have &mut, as it will save you a small amount of overhead for reusing the client.
    pub async fn subscribe_mut(
        &mut self,
        cache_name: String,
        topic: String,
        resume_at_topic_sequence_number: Option<u64>,
    ) -> Result<Subscription, MomentoError> {
        TopicClient::actually_subscribe(
            &mut self.client,
            cache_name,
            topic,
            resume_at_topic_sequence_number,
        )
        .await
    }

    async fn actually_subscribe(
        client: &mut PubsubClient<ChannelType>,
        cache_name: String,
        topic: String,
        resume_at_topic_sequence_number: Option<u64>,
    ) -> Result<Subscription, MomentoError> {
        let tonic_stream = client
            .subscribe(SubscriptionRequest {
                cache_name,
                topic,
                resume_at_topic_sequence_number: resume_at_topic_sequence_number
                    .unwrap_or_default(),
            })
            .await?
            .into_inner();
        Ok(Subscription {
            inner: tonic_stream,
        })
    }
}

/// A stream of items from a topic.
/// This will run more or less forever and yield items as long as you're
/// subscribed and someone is publishing.
pub struct Subscription {
    inner: tonic::Streaming<pubsub::SubscriptionItem>,
}

impl Subscription {
    /// Wait for the next item in the stream.
    ///
    /// Result::Ok(Some(item))    -> the server sent you a subscription item!
    /// Result::Ok(None)          -> the server is done - there will be no more items!
    /// Result::Err(MomentoError) -> something went wrong - log it and maybe reach out if you need help!
    pub async fn item(&mut self) -> Result<Option<SubscriptionItem>, MomentoError> {
        self.inner
            .message()
            .await
            .map_err(|e| e.into())
            .map(Subscription::map_into)
    }

    /// Yeah this is a pain, but doing it here lets us yield a simpler-typed subscription stream.
    /// Also, we don't want to expose protocol buffers types outside of the sdk, so some type map
    /// had to happen. It's all one-off at the moment though so might as well leave it as one
    /// triangle expression =)
    fn map_into(possible_item: Option<pubsub::SubscriptionItem>) -> Option<SubscriptionItem> {
        match possible_item {
            Some(item) => match item.kind {
                Some(kind) => match kind {
                    pubsub::subscription_item::Kind::Item(item) => match item.value {
                        Some(value) => {
                            let sequence_number = item.topic_sequence_number;
                            match value.kind {
                                Some(topic_value_kind) => {
                                    Some(SubscriptionItem::Value(SubscriptionValue {
                                        topic_sequence_number: sequence_number,
                                        kind: match topic_value_kind {
                                            pubsub::topic_value::Kind::Text(text) => {
                                                ValueKind::Text(text)
                                            }
                                            pubsub::topic_value::Kind::Binary(binary) => {
                                                ValueKind::Binary(binary)
                                            }
                                        },
                                    }))
                                }
                                // Broken protocol
                                None => Some(SubscriptionItem::Discontinuity(Discontinuity {
                                    last_sequence_number: None,
                                    new_sequence_number: sequence_number,
                                })),
                            }
                        }
                        None => None, // Broken protocol
                    },
                    pubsub::subscription_item::Kind::Discontinuity(_) => todo!(),
                },
                None => None, // Broken protocol,
            },
            None => None, // Normal end-of-stream from server
        }
    }
}

/// An item from a topic.
#[derive(Debug)]
pub enum SubscriptionItem {
    Value(SubscriptionValue),
    /// Sometimes something will break in a subscription. It is an unfortunate reality
    /// of network programming that errors occur. We do our best to tell you what we
    /// know about those errors when they occur.
    /// You might not care about these, and that's okay! It's probably a good idea to
    /// log them though, so you can reach out for help if you notice something naughty
    /// that hurts your users.
    Discontinuity(Discontinuity),
}

/// An actual published value from a topic.
#[derive(Debug)]
pub struct SubscriptionValue {
    /// The published value.
    pub kind: ValueKind,
    /// Best-effort sequence number for the topic. This is not transactional, it's just
    /// to help you know when things are probably working well or probably not working well.
    pub topic_sequence_number: u64,
}

#[derive(Debug)]
pub enum ValueKind {
    /// A value that was published to the topic as a string.
    Text(String),
    /// A value that was published to the topic as raw bytes.
    Binary(Vec<u8>),
}

/// Sometimes something will break in a subscription. It is an unfortunate reality
/// of network programming that errors occur. We do our best to tell you what we
/// know about those errors when they occur.
/// You might not care about these, and that's okay! It's probably a good idea to
/// log them though, so you can reach out for help if you notice something naughty
/// that hurts your users.
#[derive(Debug)]
pub struct Discontinuity {
    /// The last sequence number we know we processed for this stream on your
    /// behalf - it is not necessarily the last sequence number you received!
    pub last_sequence_number: Option<u64>,

    /// This discontinuity's sequence number. The next item on the stream should
    /// be a value with the next sequence after this.
    pub new_sequence_number: u64,
}

/// How a value should be presented on a subscription stream
pub trait IntoTopicValue {
    /// Consume self into a kind of topic value.
    fn into_topic_value(self) -> pubsub::topic_value::Kind;
}

/// A convenience for you to pass into publish directly if you
/// want to manually construct topic values.
impl IntoTopicValue for pubsub::topic_value::Kind {
    fn into_topic_value(self) -> pubsub::topic_value::Kind {
        self
    }
}

/// A convenience, this conversion copies the string. If you care
/// you should use String instead, or directly use a Kind.
///
/// A Text topic value.
impl IntoTopicValue for &str {
    fn into_topic_value(self) -> pubsub::topic_value::Kind {
        pubsub::topic_value::Kind::Text(self.to_string())
    }
}

/// A Text topic value.
impl IntoTopicValue for String {
    fn into_topic_value(self) -> pubsub::topic_value::Kind {
        pubsub::topic_value::Kind::Text(self)
    }
}

/// A Binary topic value.
impl IntoTopicValue for Vec<u8> {
    fn into_topic_value(self) -> pubsub::topic_value::Kind {
        pubsub::topic_value::Kind::Binary(self)
    }
}
