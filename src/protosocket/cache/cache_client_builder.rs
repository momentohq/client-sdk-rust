use crate::protosocket::cache::cache_client::ProtosocketCacheError;
use crate::protosocket::cache::Configuration;
use crate::{CredentialProvider, MomentoError, MomentoResult, ProtosocketCacheClient};
use momento_protos::protosocket::cache::cache_command::RpcKind;
use momento_protos::protosocket::cache::cache_response::Kind;
use momento_protos::protosocket::cache::unary::Command;
use momento_protos::protosocket::cache::Unary;
use momento_protos::protosocket::cache::{
    AuthenticateCommand, AuthenticateResponse, CacheCommand, CacheResponse,
};
use protosocket_prost::ProstSerializer;
use protosocket_rpc::ProtosocketControlCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

pub type Serializer = ProstSerializer<CacheResponse, CacheCommand>;

#[derive(Clone, Debug)]
pub struct UnauthenticatedClient {
    client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
}

impl UnauthenticatedClient {
    pub fn new(client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>) -> Self {
        Self { client }
    }

    pub async fn authenticate(
        self,
        credential_provider: CredentialProvider,
    ) -> Result<ProtosocketCacheClient, ProtosocketCacheError> {
        // TODO: is it possible to send an "agent" header at this step to indicate
        // that the protosocket cache client is being used?

        let message_id = AtomicU64::new(0);
        let completion = self
            .client
            .send_unary(CacheCommand {
                message_id: message_id.fetch_add(1, Ordering::Relaxed),
                control_code: ProtosocketControlCode::Normal as u32,
                rpc_kind: Some(RpcKind::Unary(Unary {
                    command: Some(Command::Auth(AuthenticateCommand {
                        token: credential_provider.auth_token,
                    })),
                })),
            })
            .await?;
        let response = completion.await?;
        match response.kind {
            Some(Kind::Auth(AuthenticateResponse {})) => {
                Ok(ProtosocketCacheClient::new(message_id, self.client))
            }
            Some(Kind::Error(error)) => Err(ProtosocketCacheError::CommandError { cause: error }),
            _ => Err(ProtosocketCacheError::UnexpectedKind),
        }
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
    credential_provider: CredentialProvider,
    runtime: tokio::runtime::Handle,
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
            runtime,
            credential_provider: self.0.credential_provider,
        })
    }
}

impl ProtosocketCacheClientBuilder<ReadyToBuild> {
    /// Constructs a new CacheClientBuilder in the ReadyToBuild state.
    pub async fn build(self) -> MomentoResult<ProtosocketCacheClientBuilder<ReadyToAuthenticate>> {
        // Note: expects socket address, not DNS name
        let endpoint = &self.0.credential_provider.cache_endpoint;
        let address = endpoint
            .to_string()
            .parse()
            .map_err(|e: std::net::AddrParseError| {
                MomentoError::unknown_error("build", Some(e.to_string()))
            })?;

        let (client, connection) = protosocket_rpc::client::connect::<Serializer, Serializer>(
            address,
            &protosocket_rpc::client::Configuration::default(),
        )
        .await
        .map_err(|e: protosocket_rpc::Error| {
            MomentoError::unknown_error("build", Some(e.to_string()))
        })?;

        // SDK expects to be run on a Tokio runtime, so we can go ahead and spawn a driver
        // task into the provided Tokio runtime to continually process protosocket requests.
        self.0.runtime.spawn(connection);

        Ok(ProtosocketCacheClientBuilder(ReadyToAuthenticate {
            unauthenticated_client: UnauthenticatedClient::new(client),
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
