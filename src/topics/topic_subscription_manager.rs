use tonic::transport::Channel;

use crate::grpc::header_interceptor::HeaderInterceptor;
use momento_protos::cache_client::pubsub::pubsub_client::PubsubClient;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tonic::codegen::InterceptedService;

pub const MAX_CONCURRENT_STREAMS_PER_CHANNEL: usize = 100;

#[derive(Clone, Debug)]
pub struct TopicSubscriptionManager {
    client: PubsubClient<InterceptedService<Channel, HeaderInterceptor>>,
    available_slots: Arc<Semaphore>,
}

impl TopicSubscriptionManager {
    pub fn new(client: PubsubClient<InterceptedService<Channel, HeaderInterceptor>>) -> Self {
        Self {
            client,
            available_slots: Arc::new(Semaphore::new(MAX_CONCURRENT_STREAMS_PER_CHANNEL)),
        }
    }

    pub fn get_num_active_subscriptions(&self) -> usize {
        MAX_CONCURRENT_STREAMS_PER_CHANNEL - self.available_slots.available_permits()
    }

    pub(crate) fn try_reserve(&self) -> Option<OwnedSemaphorePermit> {
        Arc::clone(&self.available_slots).try_acquire_owned().ok()
    }

    pub fn client(&self) -> &PubsubClient<InterceptedService<Channel, HeaderInterceptor>> {
        &self.client
    }
}
