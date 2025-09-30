use std::{convert::TryFrom, sync::Arc};

use crate::{
    credential_provider::EndpointSecurity, protosocket::cache::cache_client_builder::Serializer,
    CredentialProvider,
};
use momento_protos::protosocket::cache::{
    cache_command::RpcKind, cache_response::Kind, unary::Command, AuthenticateCommand,
    AuthenticateResponse, CacheCommand, CacheResponse, Unary,
};
use protosocket_rpc::{
    client::{
        ClientConnector, TcpStreamConnector, UnverifiedTlsStreamConnector, WebpkiTlsStreamConnector,
    },
    ProtosocketControlCode,
};
use rustls_pki_types::ServerName;

use crate::{MomentoError, MomentoResult};

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

impl ClientConnector for ProtosocketConnectionManager {
    type Request = CacheCommand;
    type Response = CacheResponse;

    async fn connect(
        self,
    ) -> protosocket_rpc::Result<protosocket_rpc::client::RpcClient<Self::Request, Self::Response>>
    {
        let unauthenticated_client = create_protosocket_connection(
            self.credential_provider.clone(),
            self.runtime.clone(),
            self.endpoint_address,
            &self.hostname,
        )
        .await
        .map_err(|e| {
            protosocket_rpc::Error::IoFailure(Arc::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))
        })?;
        let client = authenticate_protosocket_client(
            unauthenticated_client,
            self.credential_provider.clone(),
            rand::random::<u64>(), // TODO: use something other than random u64?
        )
        .await
        .map_err(|e| {
            protosocket_rpc::Error::IoFailure(Arc::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))
        })?;
        Ok(client)
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
    message_id: u64,
) -> MomentoResult<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>> {
    let completion = client
        .send_unary(CacheCommand {
            message_id,
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
            Ok(client)
        }
        Some(Kind::Error(error)) => Err(MomentoError::protosocket_command_error(error)),
        _ => Err(MomentoError::protosocket_unexpected_kind_error()),
    }
}
