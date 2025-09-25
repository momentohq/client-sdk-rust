use std::{
    convert::TryFrom,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    credential_provider::EndpointSecurity, protosocket::cache::cache_client_builder::Serializer,
    CredentialProvider,
};
use bb8::ManageConnection;
use momento_protos::protosocket::cache::{
    cache_command::RpcKind, cache_response::Kind, unary::Command, AuthenticateCommand,
    AuthenticateResponse, CacheCommand, CacheResponse, Unary,
};
use protosocket_rpc::{
    client::{TcpStreamConnector, UnverifiedTlsStreamConnector, WebpkiTlsStreamConnector},
    ProtosocketControlCode,
};
use rustls_pki_types::ServerName;

use crate::{MomentoError, MomentoResult};

#[derive(Clone, Debug)]
pub(crate) struct ProtosocketConnection {
    client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
    message_id: std::sync::Arc<AtomicU64>,
}

impl ProtosocketConnection {
    pub fn new(
        client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
        message_id: AtomicU64,
    ) -> Self {
        Self {
            client,
            message_id: std::sync::Arc::new(message_id),
        }
    }

    pub fn client(&self) -> &protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse> {
        &self.client
    }

    pub fn message_id(&self) -> u64 {
        self.message_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn is_alive(&self) -> bool {
        self.client.is_alive()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProtosocketConnectionManager {
    credential_provider: CredentialProvider,
    runtime: tokio::runtime::Handle,
    endpoint_address: std::net::SocketAddr,
    hostname: String,
}

impl ProtosocketConnectionManager {
    pub fn new(
        credential_provider: CredentialProvider,
        runtime: tokio::runtime::Handle,
    ) -> MomentoResult<Self> {
        let endpoint = &credential_provider.cache_endpoint;
        let endpoint_address =
            endpoint
                .to_string()
                .parse()
                .map_err(|e: std::net::AddrParseError| {
                    MomentoError::unknown_error("build", Some(e.to_string()))
                })?;

        let hostname = endpoint
            .split(":")
            .next()
            .unwrap_or("localhost")
            .to_string();
        Ok(Self {
            credential_provider,
            runtime,
            endpoint_address,
            hostname,
        })
    }
}

impl ManageConnection for ProtosocketConnectionManager {
    type Connection = ProtosocketConnection;
    type Error = MomentoError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let unauthenticated_client = create_protosocket_connection(
            self.credential_provider.clone(),
            self.runtime.clone(),
            self.endpoint_address,
            &self.hostname,
        )
        .await?;
        let (client, message_id) = authenticate_protosocket_client(
            unauthenticated_client,
            self.credential_provider.clone(),
        )
        .await?;
        Ok(ProtosocketConnection::new(client, message_id))
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        match conn.is_alive() {
            true => Ok(()),
            false => Err(MomentoError::unknown_error(
                "is_valid",
                Some("protosocket connection is not healthy".to_string()),
            )),
        }
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        !conn.is_alive()
    }
}

async fn create_protosocket_connection(
    credential_provider: CredentialProvider,
    runtime: tokio::runtime::Handle,
    address: std::net::SocketAddr,
    hostname: &str,
) -> MomentoResult<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>> {
    match credential_provider.endpoint_security {
        EndpointSecurity::Tls => {
            let server_name = ServerName::try_from(hostname.to_string()).map_err(|_| {
                MomentoError::unknown_error(
                    "build",
                    Some(format!(
                        "Error creating server name from hostname: {}",
                        hostname
                    )),
                )
            })?;
            let connector = WebpkiTlsStreamConnector::new(server_name);
            create_connection_with_connector(address, connector, runtime).await
        }
        EndpointSecurity::Unverified => {
            let server_name = ServerName::try_from(hostname.to_string()).map_err(|_| {
                MomentoError::unknown_error(
                    "build",
                    Some(format!(
                        "Error creating server name from hostname: {}",
                        hostname
                    )),
                )
            })?;
            let connector = UnverifiedTlsStreamConnector::new(server_name);
            create_connection_with_connector(address, connector, runtime).await
        }
        EndpointSecurity::Insecure => {
            // TODO: seems to hang when credential provider uses insecure endpoint with server expecting one of the other options,
            // probably dropping an error or need to set a timeout somewhere
            let connector = TcpStreamConnector {};
            create_connection_with_connector(address, connector, runtime).await
        }
    }
}

async fn create_connection_with_connector<C>(
    address: std::net::SocketAddr,
    connector: C,
    runtime: tokio::runtime::Handle,
) -> MomentoResult<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>>
where
    C: protosocket_rpc::client::StreamConnector + Send + 'static,
{
    let (client, connection) = protosocket_rpc::client::connect::<Serializer, Serializer, C>(
        address,
        &protosocket_rpc::client::Configuration::new(connector),
    )
    .await?;

    // SDK expects to be run on a Tokio runtime, so we can go ahead and spawn a driver
    // task into the provided Tokio runtime to continually process protosocket requests.
    runtime.spawn(connection);

    log::info!("created connection and spawned driver task");

    Ok(client)
}

pub(crate) async fn authenticate_protosocket_client(
    client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
    credential_provider: CredentialProvider,
) -> MomentoResult<(
    protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
    AtomicU64,
)> {
    let message_id = AtomicU64::new(0);
    let completion = client
        .send_unary(CacheCommand {
            message_id: message_id.fetch_add(1, Ordering::Relaxed),
            control_code: ProtosocketControlCode::Normal as u32,
            rpc_kind: Some(RpcKind::Unary(Unary {
                command: Some(Command::Auth(AuthenticateCommand {
                    token: credential_provider.clone().auth_token,
                })),
            })),
        })
        .await?;
    let response = completion.await?;
    match response.kind {
        Some(Kind::Auth(AuthenticateResponse {})) => {
            log::info!("authenticated protosocket client!");
            Ok((client, message_id))
        }
        Some(Kind::Error(error)) => Err(MomentoError::protosocket_command_error(error)),
        _ => Err(MomentoError::protosocket_unexpected_kind_error()),
    }
}
