use crate::cache::messages::data::scalar::get::Value;
use crate::{utils, CredentialProvider, IntoBytes};
use momento_protos::protosocket::cache::cache_command::RpcKind;
use momento_protos::protosocket::cache::cache_response::Kind;
use momento_protos::protosocket::cache::unary::Command;
use momento_protos::protosocket::cache::{
    AuthenticateCommand, AuthenticateResponse, CacheCommand, CacheResponse, GetCommand,
    GetResponse, SetCommand, SetResponse, Unary,
};
use momento_protos::protosocket::common::{CommandError, Status};
use protosocket_prost::ProstSerializer;
use protosocket_rpc::ProtosocketControlCode;
use std::future::Future;
use std::net::AddrParseError;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum ProtosocketCacheError {
    #[error("Failed to parse address: {cause:?}")]
    UnparsableAddr {
        #[from]
        cause: AddrParseError,
    },
    #[error("Failed to connect to protosocket: {cause:?}")]
    Protosocket {
        #[from]
        cause: protosocket_rpc::Error,
    },
    #[error("Command error: {cause:?}")]
    CommandError { cause: CommandError },
    #[error("Unexpected return kind!")]
    UnexpectedKind, // TODO better info
    #[error("Invalid TTL")]
    InvalidTTL,
}

type Serializer = ProstSerializer<CacheResponse, CacheCommand>;

pub struct UnauthenticatedClient {
    client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
}

pub struct ProtosocketCacheClient {
    message_id: AtomicU64,
    client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
}

impl UnauthenticatedClient {
    pub async fn authenticate(
        self,
        credential_provider: CredentialProvider,
    ) -> Result<ProtosocketCacheClient, ProtosocketCacheError> {
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
            Some(Kind::Auth(AuthenticateResponse {})) => Ok(ProtosocketCacheClient {
                message_id,
                client: self.client,
            }),
            Some(Kind::Error(error)) => Err(ProtosocketCacheError::CommandError { cause: error }),
            _ => Err(ProtosocketCacheError::UnexpectedKind),
        }
    }
}

impl ProtosocketCacheClient {
    pub async fn new_unauthenticated(
        address: impl ToString,
    ) -> Result<(UnauthenticatedClient, impl Future<Output = ()>), ProtosocketCacheError> {
        let address = address.to_string().parse()?;
        let (client, connection) = protosocket_rpc::client::connect::<Serializer, Serializer>(
            address,
            &protosocket_rpc::client::Configuration::default(),
        )
        .await?;
        // Obscure the connection types, since they're private and we can't reference them.
        // Now it's just a generic unit future.
        let connection_future = async move { connection.await };

        Ok((UnauthenticatedClient { client }, connection_future))
    }

    pub async fn get(
        &self,
        namespace: impl ToString,
        key: impl IntoBytes,
    ) -> Result<crate::cache::GetResponse, ProtosocketCacheError> {
        let completion = self
            .client
            .send_unary(CacheCommand {
                message_id: self.message_id.fetch_add(1, Ordering::Relaxed),
                control_code: ProtosocketControlCode::Normal as u32,
                rpc_kind: Some(RpcKind::Unary(Unary {
                    command: Some(Command::Get(GetCommand {
                        namespace: namespace.to_string(),
                        key: key.into_bytes(),
                    })),
                })),
            })
            .await?;
        let response = completion.await?;
        match response.kind {
            Some(Kind::Get(GetResponse { value })) => Ok(crate::cache::GetResponse::Hit {
                value: Value::new(value),
            }),
            Some(Kind::Error(error)) => match error.code() {
                Status::NotFound => Ok(crate::cache::GetResponse::Miss),
                _ => Err(ProtosocketCacheError::CommandError { cause: error }),
            },
            _ => Err(ProtosocketCacheError::UnexpectedKind),
        }
    }

    pub async fn set(
        &self,
        namespace: impl ToString,
        key: impl IntoBytes,
        value: impl IntoBytes,
        ttl: Duration,
    ) -> Result<crate::cache::SetResponse, ProtosocketCacheError> {
        utils::is_ttl_valid(ttl).map_err(|_| ProtosocketCacheError::InvalidTTL)?;
        let completion = self
            .client
            .send_unary(CacheCommand {
                message_id: self.message_id.fetch_add(1, Ordering::Relaxed),
                control_code: ProtosocketControlCode::Normal as u32,
                rpc_kind: Some(RpcKind::Unary(Unary {
                    command: Some(Command::Set(SetCommand {
                        namespace: namespace.to_string(),
                        key: key.into_bytes(),
                        value: value.into_bytes(),
                        ttl_milliseconds: ttl.as_millis() as u64,
                    })),
                })),
            })
            .await?;
        let response = completion.await?;
        match response.kind {
            Some(Kind::Set(SetResponse {})) => Ok(crate::cache::SetResponse {}),
            Some(Kind::Error(error)) => Err(ProtosocketCacheError::CommandError { cause: error }),
            _ => Err(ProtosocketCacheError::UnexpectedKind),
        }
    }
}
