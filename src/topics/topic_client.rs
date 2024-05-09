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
    /* constructor */
    pub fn builder() -> TopicClientBuilder<NeedsConfiguration> {
        TopicClientBuilder(NeedsConfiguration(()))
    }

    /// Publish a value to a topic.
    /// The cache is used as a namespace for your topics, and it needs to exist.
    /// You don't create topics, you just start using them.
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
