use crate::protosocket::cache::address_provider::AddressProvider;
use crate::protosocket::cache::connection_manager::ProtosocketConnectionManager;
use crate::protosocket::cache::connection_pool::ConnectionPool;
use crate::protosocket::cache::Configuration;
use crate::{CredentialProvider, MomentoResult, ProtosocketCacheClient};
use momento_protos::protosocket::cache::CacheCommand;
use momento_protos::protosocket::cache::CacheResponse;
use protosocket_prost::ProstSerializer;
use std::sync::Arc;
use std::time::Duration;

pub type Serializer = ProstSerializer<CacheResponse, CacheCommand>;

/// The initial state of the ProtosocketCacheClientBuilder.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ProtosocketCacheClientBuilder<State> {
    state: State,
}

pub(crate) fn initial() -> ProtosocketCacheClientBuilder<NeedsDefaultTtl> {
    ProtosocketCacheClientBuilder {
        state: NeedsDefaultTtl,
    }
}

/// The state of the ProtosocketCacheClientBuilder when it is waiting for a default TTL.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct NeedsDefaultTtl;

/// The state of the ProtosocketCacheClientBuilder when it is waiting for a configuration.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct NeedsConfiguration {
    default_ttl: Duration,
}

/// The state of the ProtosocketCacheClientBuilder when it is waiting for a credential provider.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct NeedsCredentialProvider {
    default_ttl: Duration,
    configuration: Configuration,
}

/// The state of the ProtosocketCacheClientBuilder when it is waiting for a credential provider.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct NeedsRuntime {
    default_ttl: Duration,
    configuration: Configuration,
    credential_provider: CredentialProvider,
}

/// The state of the ProtosocketCacheClientBuilder when it is ready to build a ProtosocketCacheClient.
#[derive(Clone, Debug)]
pub struct ReadyToBuild {
    default_ttl: Duration,
    credential_provider: CredentialProvider,
    runtime: tokio::runtime::Handle,
    configuration: Configuration,
}

impl ProtosocketCacheClientBuilder<NeedsDefaultTtl> {
    /// Constructs a new CacheClientBuilder in the NeedsDefaultTtl state.
    pub fn default_ttl(
        self,
        default_ttl: Duration,
    ) -> ProtosocketCacheClientBuilder<NeedsConfiguration> {
        ProtosocketCacheClientBuilder {
            state: NeedsConfiguration { default_ttl },
        }
    }
}

impl ProtosocketCacheClientBuilder<NeedsConfiguration> {
    /// Constructs a new CacheClientBuilder in the NeedsConfiguration state.
    pub fn configuration(
        self,
        configuration: impl Into<Configuration>,
    ) -> ProtosocketCacheClientBuilder<NeedsCredentialProvider> {
        ProtosocketCacheClientBuilder {
            state: NeedsCredentialProvider {
                default_ttl: self.state.default_ttl,
                configuration: configuration.into(),
            },
        }
    }
}

impl ProtosocketCacheClientBuilder<NeedsCredentialProvider> {
    /// Constructs a new CacheClientBuilder in the NeedsCredentialProvider state.
    pub fn credential_provider(
        self,
        credential_provider: CredentialProvider,
    ) -> ProtosocketCacheClientBuilder<NeedsRuntime> {
        ProtosocketCacheClientBuilder {
            state: NeedsRuntime {
                default_ttl: self.state.default_ttl,
                configuration: self.state.configuration,
                credential_provider,
            },
        }
    }
}

impl ProtosocketCacheClientBuilder<NeedsRuntime> {
    /// Constructs a new CacheClientBuilder in the NeedsRuntime state.
    pub fn runtime(
        self,
        runtime: tokio::runtime::Handle,
    ) -> ProtosocketCacheClientBuilder<ReadyToBuild> {
        ProtosocketCacheClientBuilder {
            state: ReadyToBuild {
                default_ttl: self.state.default_ttl,
                runtime,
                credential_provider: self.state.credential_provider,
                configuration: self.state.configuration,
            },
        }
    }
}

impl ProtosocketCacheClientBuilder<ReadyToBuild> {
    /// Constructs a new CacheClientBuilder in the ReadyToBuild state.
    pub async fn build(self) -> MomentoResult<ProtosocketCacheClient> {
        let ReadyToBuild {
            default_ttl,
            credential_provider,
            runtime,
            configuration,
        } = self.state;
        let client_connector =
            ProtosocketConnectionManager::new(credential_provider.clone(), runtime.clone())?;

        let address_provider = AddressProvider::new(credential_provider, runtime).await?;

        let client_pool = ConnectionPool::new(
            client_connector,
            configuration.connection_count,
            Arc::new(address_provider),
            configuration.az_id.clone(),
        )
        .await?;

        Ok(ProtosocketCacheClient::new(
            client_pool,
            default_ttl,
            configuration,
        ))
    }
}
