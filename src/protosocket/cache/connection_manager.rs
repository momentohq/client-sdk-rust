use crate::{
    credential_provider::EndpointSecurity, protosocket::cache::cache_client_builder::Serializer,
    CredentialProvider,
};
use crate::{MomentoError, MomentoResult};
use http::Uri;
use momento_protos::protosocket::cache::{
    cache_command::RpcKind, cache_response::Kind, unary::Command, AuthenticateCommand,
    AuthenticateResponse, CacheCommand, CacheResponse, Unary,
};
use protosocket_rpc::{
    client::{TcpStreamConnector, UnverifiedTlsStreamConnector, WebpkiTlsStreamConnector},
    ProtosocketControlCode,
};
use rustls_pki_types::ServerName;
use std::net::SocketAddr;
use std::{convert::TryFrom, str::FromStr};

#[derive(Clone, Debug)]
pub(crate) struct ProtosocketConnectionManager {
    credential_provider: CredentialProvider,
    runtime: tokio::runtime::Handle,
    hostname: String,
}

impl ProtosocketConnectionManager {
    pub fn new(
        credential_provider: CredentialProvider,
        runtime: tokio::runtime::Handle,
    ) -> MomentoResult<Self> {
        let hostname = Uri::from_str(&credential_provider.tls_cache_endpoint)
            .ok()
            .and_then(|uri| uri.host().map(|h| h.to_string()))
            .ok_or_else(|| {
                MomentoError::unknown_error(
                    "protosocket_connection_manager::new",
                    Some(format!(
                        "Could not parse TLS endpoint: {}",
                        &credential_provider.tls_cache_endpoint
                    )),
                )
            })?;

        Ok(Self {
            credential_provider,
            runtime,
            hostname,
        })
    }

    pub async fn connect(
        self,
        address: SocketAddr,
    ) -> protosocket_rpc::Result<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>>
    {
        log::debug!("connecting over protosocket to {address}");
        let unauthenticated_client = create_protosocket_connection(
            self.credential_provider.clone(),
            self.runtime.clone(),
            address,
            &self.hostname,
        )
        .await
        .map_err(|e| {
            protosocket_rpc::Error::IoFailure(
                std::io::Error::other(format!("could not connect {e:?}")).into(),
            )
        })?;
        let client = authenticate_protosocket_client(
            unauthenticated_client,
            self.credential_provider.clone(),
            0xDEADBEEF,
        )
        .await
        .map_err(|e| {
            protosocket_rpc::Error::IoFailure(
                std::io::Error::other(format!("could not authenticate {e:?}")).into(),
            )
        })?;
        log::debug!("successfully created and authenticated protosocket client");
        Ok(client)
    }
}

async fn create_protosocket_connection(
    credential_provider: CredentialProvider,
    runtime: tokio::runtime::Handle,
    address: SocketAddr,
    hostname: &str,
) -> MomentoResult<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>> {
    match credential_provider.endpoint_security {
        EndpointSecurity::Tls | EndpointSecurity::TlsOverride => {
            log::debug!("creating TLS connection to {address}");
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
            log::debug!("created TLS connector for server name: {}", hostname);
            create_connection_with_connector(address, connector, runtime).await
        }
        EndpointSecurity::Unverified => {
            log::debug!("creating unverified TLS connection to {address}");
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
            log::debug!("creating tcp connection to {address}");
            // TODO: seems to hang when credential provider uses insecure endpoint with server expecting one of the other options,
            // probably dropping an error or need to set a timeout somewhere
            let connector = TcpStreamConnector;
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
    log::debug!("connector: {:?}", connector);
    let (client, connection) = protosocket_rpc::client::connect::<Serializer, Serializer, C>(
        address,
        &protosocket_rpc::client::Configuration::new(connector),
    )
    .await?;
    log::debug!("created protosocket client connection");

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
