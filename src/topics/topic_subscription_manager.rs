use tonic::transport::Channel;

use crate::grpc::header_interceptor::HeaderInterceptor;
use momento_protos::cache_client::pubsub::pubsub_client::PubsubClient;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tonic::codegen::InterceptedService;

pub const MAX_CONCURRENT_STREAMS_PER_CHANNEL: usize = 100;

#[derive(Clone, Debug)]
pub struct TopicSubscriptionManager {
    client: PubsubClient<InterceptedService<Channel, HeaderInterceptor>>,
    num_active_subscriptions: Arc<AtomicUsize>,
}

/// Releases channel capacity when a handshake or its resulting subscription ends.
pub(crate) struct StreamReservation {
    num_active_subscriptions: Arc<AtomicUsize>,
}

impl Drop for StreamReservation {
    fn drop(&mut self) {
        self.num_active_subscriptions
            .fetch_sub(1, Ordering::Relaxed);
    }
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

    pub(crate) fn try_reserve(&self) -> Option<StreamReservation> {
        self.num_active_subscriptions
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |active| {
                (active < MAX_CONCURRENT_STREAMS_PER_CHANNEL).then_some(active + 1)
            })
            .ok()
            .map(|_| StreamReservation {
                num_active_subscriptions: Arc::clone(&self.num_active_subscriptions),
            })
    }

    pub fn client(&self) -> &PubsubClient<InterceptedService<Channel, HeaderInterceptor>> {
        &self.client
    }
}
