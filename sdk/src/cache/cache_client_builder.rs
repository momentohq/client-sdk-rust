use crate::cache::Configuration;
use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::{utils, CacheClient, CredentialProvider, MomentoResult};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tonic::codegen::InterceptedService;

use momento_protos::cache_client::scs_client::ScsClient;

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
    pub fn build(self) -> MomentoResult<CacheClient> {
        let agent_value = &utils::user_agent("cache");

        let data_channel = utils::connect_channel_lazily_configurable(
            &self.0.credential_provider.cache_endpoint,
            self.0
                .configuration
                .transport_strategy
                .grpc_configuration
                .clone(),
        )?;

        let data_interceptor = InterceptedService::new(
            data_channel,
            HeaderInterceptor::new(&self.0.credential_provider.auth_token, agent_value),
        );

        let data_client = ScsClient::new(data_interceptor);

        Ok(CacheClient {
            data_client,
            control_client: Arc::new(Mutex::new(None)),
            credential_provider: self.0.credential_provider,
            configuration: self.0.configuration,
            item_default_ttl: self.0.default_ttl,
        })
    }
}
