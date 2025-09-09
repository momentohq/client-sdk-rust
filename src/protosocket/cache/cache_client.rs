use crate::cache::messages::data::scalar::get::Value;
use crate::protosocket::cache::cache_client_builder::NeedsDefaultTtl;
use crate::{utils, IntoBytes, MomentoError, MomentoResult, ProtosocketCacheClientBuilder};
use momento_protos::protosocket::cache::cache_command::RpcKind;
use momento_protos::protosocket::cache::cache_response::Kind;
use momento_protos::protosocket::cache::unary::Command;
use momento_protos::protosocket::cache::{
    CacheCommand, CacheResponse, GetCommand, GetResponse, SetCommand, SetResponse, Unary,
};
use momento_protos::protosocket::common::Status;
use protosocket_rpc::ProtosocketControlCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// A client for interacting with Momento Cache using the Protosocket protocol.
// TODO: complete docs
pub struct ProtosocketCacheClient {
    message_id: AtomicU64,
    client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
}

impl ProtosocketCacheClient {
    pub(crate) fn new(
        message_id: AtomicU64,
        client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
    ) -> Self {
        Self { message_id, client }
    }

    /// Constructs a new ProtosocketCacheClientBuilder.
    // TODO: complete docs
    pub fn builder() -> ProtosocketCacheClientBuilder<NeedsDefaultTtl> {
        ProtosocketCacheClientBuilder(NeedsDefaultTtl(()))
    }

    /// Gets an item from a Momento Cache
    // TODO: request timeout, request building pattern, docs
    pub async fn get(
        &self,
        namespace: impl ToString,
        key: impl IntoBytes,
    ) -> MomentoResult<crate::cache::GetResponse> {
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
                _ => Err(MomentoError::protosocket_command_error(error)),
            },
            _ => Err(MomentoError::protosocket_unexpected_kind_error()),
        }
    }

    /// Sets an item in a Momento Cache
    // TODO: request timeout, default ttl, request building pattern, docs
    pub async fn set(
        &self,
        namespace: impl ToString,
        key: impl IntoBytes,
        value: impl IntoBytes,
        ttl: Duration,
    ) -> MomentoResult<crate::cache::SetResponse> {
        utils::is_ttl_valid(ttl)?;
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
            Some(Kind::Error(error)) => Err(MomentoError::protosocket_command_error(error)),
            _ => Err(MomentoError::protosocket_unexpected_kind_error()),
        }
    }
}
