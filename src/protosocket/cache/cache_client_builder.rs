use crate::credential_provider::EndpointSecurity;
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
use protosocket_rpc::client::{
    TcpStreamConnector, UnverifiedTlsStreamConnector, WebpkiTlsStreamConnector,
};
use protosocket_rpc::ProtosocketControlCode;
use rustls_pki_types::ServerName;
use std::convert::TryFrom;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio_rustls::rustls::crypto::aws_lc_rs::default_provider;

pub type Serializer = ProstSerializer<CacheResponse, CacheCommand>;

#[derive(Clone, Debug)]
pub struct UnauthenticatedClient {
    client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
    default_ttl: Duration,
    configuration: Configuration,
}

impl UnauthenticatedClient {
    pub fn new(
        client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
        default_ttl: Duration,
        configuration: Configuration,
    ) -> Self {
        Self {
            client,
            default_ttl,
            configuration,
        }
    }

    pub async fn authenticate(
        self,
        credential_provider: CredentialProvider,
    ) -> MomentoResult<ProtosocketCacheClient> {
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
            Some(Kind::Auth(AuthenticateResponse {})) => Ok(ProtosocketCacheClient::new(
                message_id,
                self.client,
                self.default_ttl,
                self.configuration,
            )),
            Some(Kind::Error(error)) => Err(MomentoError::protosocket_command_error(error)),
            _ => Err(MomentoError::protosocket_unexpected_kind_error()),
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
        // Note: expects socket address, not DNS name
        let endpoint = self.0.credential_provider.clone().cache_endpoint;
        let address = endpoint
            .to_string()
            .parse()
            .map_err(|e: std::net::AddrParseError| {
                MomentoError::unknown_error("build", Some(e.to_string()))
            })?;

        tokio_rustls::rustls::crypto::CryptoProvider::install_default(default_provider())
            .map_err(|_| MomentoError::unknown_error("build", None))?;

        match self.0.credential_provider.endpoint_security {
            EndpointSecurity::Tls => {
                // TODO: use a default value or panic here?
                let hostname = endpoint
                    .split(":")
                    .next()
                    .unwrap_or("localhost")
                    .to_string();
                let server_name = ServerName::try_from(hostname)
                    .map_err(|_| MomentoError::unknown_error("build", None))?;
                let with_tls = WebpkiTlsStreamConnector::new(server_name);
                let (client, connection) = protosocket_rpc::client::connect::<
                    Serializer,
                    Serializer,
                    WebpkiTlsStreamConnector,
                >(
                    address,
                    &protosocket_rpc::client::Configuration::new(with_tls),
                )
                .await?;
                // SDK expects to be run on a Tokio runtime, so we can go ahead and spawn a driver
                // task into the provided Tokio runtime to continually process protosocket requests.
                self.0.runtime.spawn(connection);

                Ok(ProtosocketCacheClientBuilder(ReadyToAuthenticate {
                    unauthenticated_client: UnauthenticatedClient::new(
                        client,
                        self.0.default_ttl,
                        self.0.configuration,
                    ),
                    credential_provider: self.0.credential_provider,
                }))
            }
            EndpointSecurity::Unverified => {
                let hostname = endpoint
                    .split(":")
                    .next()
                    .unwrap_or("localhost")
                    .to_string();
                let server_name = ServerName::try_from(hostname)
                    .map_err(|_| MomentoError::unknown_error("build", None))?;
                let with_tls = UnverifiedTlsStreamConnector::new(server_name);
                let (client, connection) = protosocket_rpc::client::connect::<
                    Serializer,
                    Serializer,
                    UnverifiedTlsStreamConnector,
                >(
                    address,
                    &protosocket_rpc::client::Configuration::new(with_tls),
                )
                .await?;
                // SDK expects to be run on a Tokio runtime, so we can go ahead and spawn a driver
                // task into the provided Tokio runtime to continually process protosocket requests.
                self.0.runtime.spawn(connection);

                Ok(ProtosocketCacheClientBuilder(ReadyToAuthenticate {
                    unauthenticated_client: UnauthenticatedClient::new(
                        client,
                        self.0.default_ttl,
                        self.0.configuration,
                    ),
                    credential_provider: self.0.credential_provider,
                }))
            }
            EndpointSecurity::Insecure => {
                // TODO: seems to hang when credential provider uses insecure endpoint with server expecting one of the other options,
                // probably dropping an error or need to set a timeout somewhere
                let without_tls = TcpStreamConnector {};
                let (client, connection) =
                    protosocket_rpc::client::connect::<Serializer, Serializer, TcpStreamConnector>(
                        address,
                        &protosocket_rpc::client::Configuration::new(without_tls),
                    )
                    .await?;
                // SDK expects to be run on a Tokio runtime, so we can go ahead and spawn a driver
                // task into the provided Tokio runtime to continually process protosocket requests.
                self.0.runtime.spawn(connection);

                Ok(ProtosocketCacheClientBuilder(ReadyToAuthenticate {
                    unauthenticated_client: UnauthenticatedClient::new(
                        client,
                        self.0.default_ttl,
                        self.0.configuration,
                    ),
                    credential_provider: self.0.credential_provider,
                }))
            }
        }
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
