use std::{
    convert::TryFrom,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use crate::{
    credential_provider::EndpointSecurity, protosocket::cache::cache_client_builder::Serializer,
    CredentialProvider,
};
use momento_protos::protosocket::cache::{
    cache_command::RpcKind, cache_response::Kind, unary::Command, AuthenticateCommand,
    AuthenticateResponse, CacheCommand, CacheResponse, Unary,
};
use protosocket_rpc::{
    client::{TcpStreamConnector, UnverifiedTlsStreamConnector, WebpkiTlsStreamConnector},
    ProtosocketControlCode,
};
use rustls_pki_types::ServerName;
use tokio::sync::Mutex;

use crate::{MomentoError, MomentoResult};

#[derive(Clone, Debug)]
pub(crate) struct HealthyProtosocket {
    client: Arc<Mutex<Option<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>>>>,
    message_id: Arc<AtomicU64>,
    credential_provider: CredentialProvider,
    runtime: tokio::runtime::Handle,
}

#[allow(clippy::expect_used)]
impl HealthyProtosocket {
    pub fn new(
        client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
        message_id: AtomicU64,
        credential_provider: CredentialProvider,
        runtime: tokio::runtime::Handle,
    ) -> Self {
        Self {
            client: Arc::new(Mutex::new(Some(client))),
            message_id: Arc::new(message_id),
            credential_provider,
            runtime,
        }
    }

    pub fn message_id(&self) -> u64 {
        self.message_id.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn get_client(
        &self,
    ) -> MomentoResult<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>> {
        let mut client_guard = self.client.lock().await;
        match &*client_guard {
            Some(client) => {
                if client.is_alive() {
                    Ok(client.clone())
                } else {
                    *client_guard = None;
                    let endpoint = self.credential_provider.clone().cache_endpoint;
                    log::info!("getting protosocket client for endpoint {endpoint}");
                    let client_result = create_protosocket_connection(
                        self.credential_provider.clone(),
                        self.runtime.clone(),
                    )
                    .await?;
                    let (client, message_id) = authenticate_protosocket_client(
                        client_result,
                        self.credential_provider.clone(),
                    )
                    .await?;
                    *client_guard = Some(client.clone());
                    self.message_id
                        .store(message_id.load(Ordering::Relaxed), Ordering::Relaxed);
                    Ok(client)
                }
            }
            None => {
                let endpoint = self.credential_provider.clone().cache_endpoint;
                log::info!("getting protosocket client for endpoint {endpoint}");
                let client_result = create_protosocket_connection(
                    self.credential_provider.clone(),
                    self.runtime.clone(),
                )
                .await?;
                let (client, message_id) = authenticate_protosocket_client(
                    client_result,
                    self.credential_provider.clone(),
                )
                .await?;
                *client_guard = Some(client.clone());
                self.message_id
                    .store(message_id.load(Ordering::Relaxed), Ordering::Relaxed);
                Ok(client)
            }
        }
    }
}

pub(crate) async fn create_protosocket_connection(
    credential_provider: CredentialProvider,
    runtime: tokio::runtime::Handle,
) -> MomentoResult<protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>> {
    let endpoint = credential_provider.clone().cache_endpoint;
    let address = endpoint
        .to_string()
        .parse()
        .map_err(|e: std::net::AddrParseError| {
            MomentoError::unknown_error("build", Some(e.to_string()))
        })?;

    match credential_provider.endpoint_security {
        EndpointSecurity::Tls => {
            // TODO: use a default value or panic here?
            let hostname = endpoint
                .split(":")
                .next()
                .unwrap_or("localhost")
                .to_string();
            let server_name = ServerName::try_from(hostname.clone()).map_err(|_| {
                MomentoError::unknown_error(
                    "build",
                    Some(format!(
                        "Error creating server name from hostname: {}",
                        hostname
                    )),
                )
            })?;
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
            runtime.spawn(connection);

            Ok(client)
        }
        EndpointSecurity::Unverified => {
            let hostname = endpoint
                .split(":")
                .next()
                .unwrap_or("localhost")
                .to_string();
            let server_name = ServerName::try_from(hostname.clone()).map_err(|_| {
                MomentoError::unknown_error(
                    "build",
                    Some(format!(
                        "Error creating server name from hostname: {}",
                        hostname
                    )),
                )
            })?;
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
            runtime.spawn(connection);

            Ok(client)
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
            runtime.spawn(connection);

            Ok(client)
        }
    }
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
        Some(Kind::Auth(AuthenticateResponse {})) => Ok((client, message_id)),
        Some(Kind::Error(error)) => Err(MomentoError::protosocket_command_error(error)),
        _ => Err(MomentoError::protosocket_unexpected_kind_error()),
    }
}
