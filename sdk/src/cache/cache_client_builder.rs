use crate::cache::Configuration;
use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::{utils, CacheClient, CredentialProvider, MomentoResult};
use std::time::Duration;
use tonic::codegen::InterceptedService;

use crate::utils::ChannelConnectError;
use momento_protos::cache_client::scs_client::ScsClient;
use momento_protos::control_client::scs_control_client::ScsControlClient;
use tonic::transport::Channel;
use crate::config::grpc_configuration::GrpcConfiguration;
use crate::config::transport_strategy::TransportStrategy;

pub struct CacheClientBuilder<State>(pub State);

pub struct NeedsDefaultTtl(pub ());

pub struct NeedsConfiguration {
    default_ttl: Duration,
}

pub struct NeedsCredentialProvider {
    default_ttl: Duration,
    configuration: Configuration,
}

pub struct ReadyToBuild {
    default_ttl: Duration,
    configuration: Configuration,
    credential_provider: CredentialProvider,
}

impl CacheClientBuilder<NeedsDefaultTtl> {
    pub fn default_ttl(self, default_ttl: Duration) -> CacheClientBuilder<NeedsConfiguration> {
        CacheClientBuilder(NeedsConfiguration { default_ttl })
    }
}

impl CacheClientBuilder<NeedsConfiguration> {
    pub fn configuration(
        self,
        configuration: impl Into<Configuration>,
    ) -> CacheClientBuilder<NeedsCredentialProvider> {
        CacheClientBuilder(NeedsCredentialProvider {
            default_ttl: self.0.default_ttl,
            configuration: configuration.into(),
        })
    }
}

impl CacheClientBuilder<NeedsCredentialProvider> {
    pub fn credential_provider(
        self,
        credential_provider: CredentialProvider,
    ) -> CacheClientBuilder<ReadyToBuild> {
        CacheClientBuilder(ReadyToBuild {
            default_ttl: self.0.default_ttl,
            configuration: self.0.configuration,
            credential_provider,
        })
    }
}

impl CacheClientBuilder<ReadyToBuild> {
    pub fn with_num_connections(self, num_connections: u32) -> CacheClientBuilder<ReadyToBuild> {
        let grpc_configuration = self.0.configuration.transport_strategy.grpc_configuration;
        let transport_strategy = TransportStrategy{
            grpc_configuration: GrpcConfiguration{
              num_channels: num_connections,
                ..grpc_configuration
            },
        };
        
        CacheClientBuilder(ReadyToBuild {
            configuration: Configuration{
                transport_strategy,
            },
            ..self.0
        })
    }

    pub fn build(self) -> MomentoResult<CacheClient> {
        let agent_value = &utils::user_agent("cache");

        let data_channels_result: Result<Vec<Channel>, ChannelConnectError> =
            (0..self.0.configuration.transport_strategy.grpc_configuration.num_channels)
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

        let data_clients: Vec<ScsClient<InterceptedService<Channel, HeaderInterceptor>>> =
            data_channels
                .into_iter()
                .map(|c| {
                    let data_interceptor = InterceptedService::new(
                        c,
                        HeaderInterceptor::new(&self.0.credential_provider.auth_token, agent_value),
                    );
                    ScsClient::new(data_interceptor)
                })
                .collect();
        let control_client = ScsControlClient::new(control_interceptor);

        Ok(CacheClient {
            data_clients,
            control_client,
            configuration: self.0.configuration,
            item_default_ttl: self.0.default_ttl,
        })
    }
}
