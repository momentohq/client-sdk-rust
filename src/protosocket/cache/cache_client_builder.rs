use crate::protosocket::cache::utils::{
    authenticate_protosocket_client, create_protosocket_connection, HealthyProtosocket,
};
use crate::protosocket::cache::Configuration;
use crate::{CredentialProvider, MomentoError, MomentoResult, ProtosocketCacheClient};
use momento_protos::protosocket::cache::CacheCommand;
use momento_protos::protosocket::cache::CacheResponse;
use protosocket_prost::ProstSerializer;
use std::time::Duration;

pub type Serializer = ProstSerializer<CacheResponse, CacheCommand>;

#[derive(Clone, Debug)]
pub struct UnauthenticatedClient {
    client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
    default_ttl: Duration,
    configuration: Configuration,
    runtime: tokio::runtime::Handle,
}

impl UnauthenticatedClient {
    pub fn new(
        client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
        default_ttl: Duration,
        configuration: Configuration,
        runtime: tokio::runtime::Handle,
    ) -> Self {
        Self {
            client,
            default_ttl,
            configuration,
            runtime,
        }
    }

    pub async fn authenticate(
        self,
        credential_provider: CredentialProvider,
    ) -> MomentoResult<ProtosocketCacheClient> {
        let (client, message_id) =
            authenticate_protosocket_client(self.client, credential_provider.clone()).await?;
        Ok(ProtosocketCacheClient::new(
            HealthyProtosocket::new(client, message_id, credential_provider, self.runtime),
            self.default_ttl,
            self.configuration,
        ))
    }
}

/// The initial state of the ProtosocketCacheClientBuilder.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ProtosocketCacheClientBuilder<State>(pub State);

/// The state of the ProtosocketCacheClientBuilder when it is waiting for a default TTL.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct NeedsDefaultTtl(pub ());

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

/// The state of the ProtosocketCacheClientBuilder when it is ready to authenticate with the server.
#[derive(Clone, Debug)]
pub struct ReadyToAuthenticate {
    credential_provider: CredentialProvider,
    unauthenticated_client: UnauthenticatedClient,
}

impl ProtosocketCacheClientBuilder<NeedsDefaultTtl> {
    /// Constructs a new CacheClientBuilder in the NeedsDefaultTtl state.
    pub fn default_ttl(
        self,
        default_ttl: Duration,
    ) -> ProtosocketCacheClientBuilder<NeedsConfiguration> {
        ProtosocketCacheClientBuilder(NeedsConfiguration { default_ttl })
    }
}

impl ProtosocketCacheClientBuilder<NeedsConfiguration> {
    /// Constructs a new CacheClientBuilder in the NeedsConfiguration state.
    pub fn configuration(
        self,
        configuration: impl Into<Configuration>,
    ) -> ProtosocketCacheClientBuilder<NeedsCredentialProvider> {
        ProtosocketCacheClientBuilder(NeedsCredentialProvider {
            default_ttl: self.0.default_ttl,
            configuration: configuration.into(),
        })
    }
}

impl ProtosocketCacheClientBuilder<NeedsCredentialProvider> {
    /// Constructs a new CacheClientBuilder in the NeedsCredentialProvider state.
    pub fn credential_provider(
        self,
        credential_provider: CredentialProvider,
    ) -> ProtosocketCacheClientBuilder<NeedsRuntime> {
        ProtosocketCacheClientBuilder(NeedsRuntime {
            default_ttl: self.0.default_ttl,
            configuration: self.0.configuration,
            credential_provider,
        })
    }
}

impl ProtosocketCacheClientBuilder<NeedsRuntime> {
    /// Constructs a new CacheClientBuilder in the NeedsRuntime state.
    pub fn runtime(
        self,
        runtime: tokio::runtime::Handle,
    ) -> ProtosocketCacheClientBuilder<ReadyToBuild> {
        ProtosocketCacheClientBuilder(ReadyToBuild {
            default_ttl: self.0.default_ttl,
            runtime,
            credential_provider: self.0.credential_provider,
            configuration: self.0.configuration,
        })
    }
}

impl ProtosocketCacheClientBuilder<ReadyToBuild> {
    /// Constructs a new CacheClientBuilder in the ReadyToBuild state.
    pub async fn build(self) -> MomentoResult<ProtosocketCacheClientBuilder<ReadyToAuthenticate>> {
        let client = create_protosocket_connection(
            self.0.credential_provider.clone(),
            self.0.runtime.clone(),
        )
        .await?;

        Ok(ProtosocketCacheClientBuilder(ReadyToAuthenticate {
            unauthenticated_client: UnauthenticatedClient::new(
                client,
                self.0.default_ttl,
                self.0.configuration,
                self.0.runtime,
            ),
            credential_provider: self.0.credential_provider,
        }))
    }
}

impl ProtosocketCacheClientBuilder<ReadyToAuthenticate> {
    /// Authenticates the protosocket client with the server.
    pub async fn authenticate(self) -> MomentoResult<ProtosocketCacheClient> {
        self.0
            .unauthenticated_client
            .authenticate(self.0.credential_provider)
            .await
            .map_err(|e| MomentoError::unknown_error("authenticate", Some(e.to_string())))
    }
}
