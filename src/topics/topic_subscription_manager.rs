use tonic::transport::Channel;

use crate::grpc::header_interceptor::HeaderInterceptor;
use momento_protos::cache_client::pubsub::pubsub_client::PubsubClient;
use std::sync::{atomic::AtomicUsize, Arc};
use tonic::codegen::InterceptedService;

pub const MAX_CONCURRENT_STREAMS_PER_CHANNEL: usize = 100;

#[derive(Clone, Debug)]
pub struct TopicSubscriptionManager {
    client: PubsubClient<InterceptedService<Channel, HeaderInterceptor>>,
    num_active_subscriptions: Arc<AtomicUsize>,
}

impl TopicSubscriptionManager {
    pub fn new(client: PubsubClient<InterceptedService<Channel, HeaderInterceptor>>) -> Self {
        Self {
            client,
            num_active_subscriptions: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get_num_active_subscriptions(&self) -> usize {
        self.num_active_subscriptions
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn increment_num_active_subscriptions(&self) -> usize {
        self.num_active_subscriptions
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn decrement_num_active_subscriptions(&self) -> usize {
        self.num_active_subscriptions
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn client(&self) -> &PubsubClient<InterceptedService<Channel, HeaderInterceptor>> {
        &self.client
    }
}
