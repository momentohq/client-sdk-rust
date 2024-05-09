use futures::future::BoxFuture;
use futures::{Future, FutureExt};
use momento_protos::cache_client::pubsub::SubscriptionRequest;
use momento_protos::cache_client::pubsub::{self, pubsub_client::PubsubClient};
use tonic::{codegen::InterceptedService, transport::Channel};

use crate::grpc::header_interceptor::HeaderInterceptor;

type ChannelType = InterceptedService<Channel, HeaderInterceptor>;

/// A stream of items from a topic.
/// This will run more or less forever and yield items as long as you're
/// subscribed and someone is publishing.
///
/// A Subscription is a `futures::Stream<SubscriptionItem>`. It will try to
/// stay connected for as long as you try to consume it.
///
/// # Examples
///
/// Below are a couple examples of spawning a task with a subscriber.
///
/// You can call `abort` on the task handle after some time has passed.
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use momento::{CredentialProvider, TopicClient};
/// use futures::StreamExt;
/// 
/// let topic_client = TopicClient::builder()
///     .configuration(momento::topics::configurations::laptop::latest())
///     .credential_provider(
///         CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
///             .expect("auth token should be valid"),
///     )
///     .build()?;
/// 
/// let mut subscription = topic_client.subscribe("cache", "my-topic").await?;
/// let subscriber_handle = tokio::spawn(async move {
///     println!("Subscriber should keep receiving until thread is killed");
///     while let Some(message) = subscription.next().await {
///         println!("[1] Received message: {:?}", message);
///     }
/// });
///
/// tokio::time::sleep(std::time::Duration::from_secs(10)).await;
/// subscriber_handle.abort();
/// 
/// # Ok(())
/// # })
/// # }
/// ```
///
/// Or you can break out of the `next()` loop after consuming some number of messages.
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use momento::{CredentialProvider, TopicClient};
/// use futures::StreamExt;
/// 
/// let topic_client = TopicClient::builder()
///     .configuration(momento::topics::configurations::laptop::latest())
///     .credential_provider(
///         CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
///             .expect("auth token should be valid"),
///     )
///     .build()?;
/// 
/// let mut subscription = topic_client.subscribe("cache", "my-topic").await?;
/// tokio::spawn(async move {
///     println!("Subscriber should receive 10 messages then exist");
///     for _ in 0..10 {
///         let message = subscription.next().await;
///         println!("[2] Received message: {:?}", message);
///     }
/// });
/// # Ok(())
/// # })
/// # }
/// ```
pub struct Subscription {
    client: PubsubClient<ChannelType>,
    cache_name: String,
    topic: String,
    current_sequence_number: u64,
    current_subscription: SubscriptionState,
}

type SubscriptionFuture = BoxFuture<
    'static,
    Result<tonic::Response<tonic::Streaming<pubsub::SubscriptionItem>>, tonic::Status>,
>;

pub enum SubscriptionState {
    Subscribed(tonic::Streaming<pubsub::SubscriptionItem>),
    Resubscribing {
        subscription_future: SubscriptionFuture,
    },
}

enum MapKind {
    Heartbeat,
    RealItem(SubscriptionItem),
    BrokenProtocolMissingAttribute(&'static str),
}

impl Subscription {
    pub fn new(
        client: PubsubClient<ChannelType>,
        cache_name: String,
        topic: String,
        current_sequence_number: u64,
        current_subscription: SubscriptionState,
    ) -> Subscription {
        Subscription {
            client,
            cache_name,
            topic,
            current_sequence_number,
            current_subscription,
        }
    }

    /// Yeah this is a pain, but doing it here lets us yield a simpler-typed subscription stream.
    /// Also, we don't want to expose protocol buffers types outside of the sdk, so some type map
    /// had to happen. It's all one-off at the moment though so might as well leave it as one
    /// triangle expression =)
    fn map_into(item: pubsub::SubscriptionItem) -> MapKind {
        match item.kind {
            Some(kind) => match kind {
                pubsub::subscription_item::Kind::Item(item) => match item.value {
                    Some(value) => {
                        let sequence_number = item.topic_sequence_number;
                        match value.kind {
                            Some(topic_value_kind) => {
                                MapKind::RealItem(SubscriptionItem::Value(SubscriptionValue {
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
                            // This is kind of a broken protocol situation - but we do have a sequence number
                            // so communicating the discontinuity at least allows downstream consumers to
                            // take action on a partially-unsupported stream.
                            None => {
                                MapKind::RealItem(SubscriptionItem::Discontinuity(Discontinuity {
                                    last_sequence_number: None,
                                    new_sequence_number: sequence_number,
                                }))
                            }
                        }
                    }
                    None => MapKind::BrokenProtocolMissingAttribute("value kind"),
                },
                pubsub::subscription_item::Kind::Discontinuity(discontinuity) => {
                    MapKind::RealItem(SubscriptionItem::Discontinuity(Discontinuity {
                        last_sequence_number: Some(discontinuity.last_topic_sequence),
                        new_sequence_number: discontinuity.new_topic_sequence,
                    }))
                }
                pubsub::subscription_item::Kind::Heartbeat(_) => MapKind::Heartbeat,
            },
            None => MapKind::BrokenProtocolMissingAttribute("item kind"),
        }
    }

    fn resubscribe(&self) -> SubscriptionFuture {
        let mut client = self.client.clone();
        let cache_name = self.cache_name.clone();
        let topic = self.topic.clone();
        let resume_at_topic_sequence_number = self.current_sequence_number;
        async move {
            client
                .subscribe(SubscriptionRequest {
                    cache_name,
                    topic,
                    resume_at_topic_sequence_number,
                })
                .await
        }
        .boxed()
    }
}

impl futures::Stream for Subscription {
    type Item = SubscriptionItem;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            match &mut self.as_mut().current_subscription {
                SubscriptionState::Subscribed(subscription) => {
                    match std::pin::pin!(subscription).poll_next(context) {
                        std::task::Poll::Ready(possible_result) => match possible_result {
                            Some(result) => match result {
                                Ok(item) => match Self::map_into(item) {
                                    MapKind::RealItem(item) => {
                                        log::trace!("received an item: {item:?}");
                                        match &item {
                                            SubscriptionItem::Value(v) => {
                                                self.current_sequence_number =
                                                    v.topic_sequence_number
                                            }
                                            SubscriptionItem::Discontinuity(d) => {
                                                self.current_sequence_number = d.new_sequence_number
                                            }
                                        }
                                        break std::task::Poll::Ready(Some(item));
                                    }
                                    MapKind::Heartbeat => {
                                        log::trace!("received a heartbeat - skipping...");
                                    }
                                    MapKind::BrokenProtocolMissingAttribute(e) => {
                                        log::debug!("bad item! Missing {e} - skipping...");
                                    }
                                },
                                Err(e) => {
                                    log::debug!(
                                        "error talking to momento! {e:?} - Reconnecting..."
                                    );
                                    self.current_subscription = SubscriptionState::Resubscribing {
                                        subscription_future: self.resubscribe(),
                                    };
                                }
                            },
                            None => {
                                log::debug!("stream closed - reconnecting...");
                                self.current_subscription = SubscriptionState::Resubscribing {
                                    subscription_future: self.resubscribe(),
                                };
                            }
                        },
                        std::task::Poll::Pending => {
                            // Nobody has published anything just yet.
                            break std::task::Poll::Pending;
                        }
                    }
                }
                SubscriptionState::Resubscribing {
                    subscription_future,
                } => {
                    match std::pin::pin!(subscription_future).poll(context) {
                        std::task::Poll::Ready(subscription_result) => match subscription_result {
                            Ok(new_subscription) => {
                                log::trace!("state transitioned back to subscribed");
                                self.current_subscription =
                                    SubscriptionState::Subscribed(new_subscription.into_inner());
                            }
                            Err(e) => {
                                log::debug!(
                                    "error while trying to resubscribe. {e:?} - trying again..."
                                );
                                self.current_subscription = SubscriptionState::Resubscribing {
                                    subscription_future: self.resubscribe(),
                                };
                            }
                        },
                        std::task::Poll::Pending => {
                            // Not reconnected just yet.
                            break std::task::Poll::Pending;
                        }
                    }
                }
            }
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
