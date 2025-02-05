use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::leaderboard::leaderboard_client_builder::{
    LeaderboardClientBuilder, NeedsConfiguration,
};
use crate::leaderboard::Configuration;
use crate::leaderboard::Leaderboard;

use momento_protos::leaderboard::leaderboard_client as leaderboard_proto;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

/// Client to work with Momento Leaderboards.
#[derive(Clone, Debug)]
pub struct LeaderboardClient {
    data_clients:
        Vec<leaderboard_proto::LeaderboardClient<InterceptedService<Channel, HeaderInterceptor>>>,
    configuration: Configuration,
}

impl LeaderboardClient {
    /// Returns a builder to construct a `LeaderboardClient`.
    pub fn builder() -> LeaderboardClientBuilder<NeedsConfiguration> {
        LeaderboardClientBuilder(NeedsConfiguration {})
    }

    pub(crate) fn new(
        data_clients: Vec<
            leaderboard_proto::LeaderboardClient<InterceptedService<Channel, HeaderInterceptor>>,
        >,
        configuration: Configuration,
    ) -> Self {
        Self {
            data_clients,
            configuration,
        }
    }

    /// Returns a `Leaderboard` client to work with a specific leaderboard.
    pub fn leaderboard(
        &self,
        cache_name: impl Into<String>,
        leaderboard_name: impl Into<String>,
    ) -> Leaderboard {
        Leaderboard::new(
            self.data_clients.clone(),
            self.configuration.deadline(),
            cache_name,
            leaderboard_name,
        )
    }
}
