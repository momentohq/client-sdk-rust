use crate::config::grpc_configuration::GrpcConfiguration;
use crate::config::transport_strategy::TransportStrategy;
use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::leaderboard::{Configuration, LeaderboardClient};
use crate::utils::ChannelConnectError;
use crate::{utils, CredentialProvider, MomentoResult};

use momento_protos::control_client::scs_control_client::ScsControlClient;
use momento_protos::leaderboard::leaderboard_client::LeaderboardClient as SLbClient;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

pub struct LeaderboardClientBuilder<State>(pub State);

pub struct NeedsConfiguration {}

pub struct NeedsCredentialProvider {
    configuration: Configuration,
}

pub struct ReadyToBuild {
    configuration: Configuration,
    credential_provider: CredentialProvider,
}

impl LeaderboardClientBuilder<NeedsConfiguration> {
    pub fn configuration(
        self,
        configuration: impl Into<Configuration>,
    ) -> LeaderboardClientBuilder<NeedsCredentialProvider> {
        LeaderboardClientBuilder(NeedsCredentialProvider {
            configuration: configuration.into(),
        })
    }
}

impl LeaderboardClientBuilder<NeedsCredentialProvider> {
    pub fn credential_provider(
        self,
        credential_provider: CredentialProvider,
    ) -> LeaderboardClientBuilder<ReadyToBuild> {
        LeaderboardClientBuilder(ReadyToBuild {
            configuration: self.0.configuration,
            credential_provider,
        })
    }
}

impl LeaderboardClientBuilder<ReadyToBuild> {
    pub fn with_num_connections(
        self,
        num_connections: usize,
    ) -> LeaderboardClientBuilder<ReadyToBuild> {
        let grpc_configuration = self.0.configuration.transport_strategy.grpc_configuration;
        let transport_strategy = TransportStrategy {
            grpc_configuration: GrpcConfiguration {
                num_channels: num_connections,
                ..grpc_configuration
            },
        };

        LeaderboardClientBuilder(ReadyToBuild {
            configuration: Configuration { transport_strategy },
            ..self.0
        })
    }

    pub fn build(self) -> MomentoResult<LeaderboardClient> {
        let agent_value = &utils::user_agent("cache");

        let data_channels_result: Result<Vec<Channel>, ChannelConnectError> = (0..self
            .0
            .configuration
            .transport_strategy
            .grpc_configuration
            .num_channels)
            .map(|_| {
                utils::connect_channel_lazily_configurable(
                    &self.0.credential_provider.cache_endpoint,
                    self.0
                        .configuration
                        .transport_strategy
                        .grpc_configuration
                        .clone(),
                )
            })
            .collect();

        let data_channels = data_channels_result?;

        let control_channel = utils::connect_channel_lazily_configurable(
            &self.0.credential_provider.control_endpoint,
            self.0
                .configuration
                .transport_strategy
                .grpc_configuration
                .clone(),
        )?;

        let control_interceptor = InterceptedService::new(
            control_channel,
            HeaderInterceptor::new(&self.0.credential_provider.auth_token, agent_value),
        );

        let data_clients: Vec<SLbClient<InterceptedService<Channel, HeaderInterceptor>>> =
            data_channels
                .into_iter()
                .map(|c| {
                    let data_interceptor = InterceptedService::new(
                        c,
                        HeaderInterceptor::new(&self.0.credential_provider.auth_token, agent_value),
                    );
                    SLbClient::new(data_interceptor)
                })
                .collect();
        let control_client = ScsControlClient::new(control_interceptor);

        Ok(LeaderboardClient::new(
            data_clients,
            control_client,
            self.0.configuration,
        ))
    }
}
