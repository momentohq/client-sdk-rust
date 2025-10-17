use std::time::Duration;

use crate::cache::messages::data::scalar::get::Value;
use crate::cache::GetRequest;
use crate::protosocket::cache::MomentoProtosocketRequest;
use crate::{utils, IntoBytes, MomentoError, MomentoResult, ProtosocketCacheClient};
use momento_protos::protosocket::cache::cache_command::RpcKind;
use momento_protos::protosocket::cache::cache_response::Kind;
use momento_protos::protosocket::cache::unary::Command;
use momento_protos::protosocket::cache::{CacheCommand, GetCommand, GetResponse, Unary};
use momento_protos::protosocket::common::Status;
use protosocket_rpc::ProtosocketControlCode;

impl<K: IntoBytes> MomentoProtosocketRequest for GetRequest<K> {
    type Response = crate::cache::GetResponse;

    async fn send(
        self,
        client: &ProtosocketCacheClient,
        timeout: Duration,
    ) -> MomentoResult<Self::Response> {
        utils::execute_protosocket_request_with_timeout(|| self.send_get(client), timeout).await
    }
}

impl<K: IntoBytes> GetRequest<K> {
    async fn send_get(
        self,
        client: &ProtosocketCacheClient,
    ) -> MomentoResult<crate::cache::GetResponse> {
        let completion = client
            .protosocket_connection()
            .await?
            .send_unary(CacheCommand {
                message_id: client.message_id(),
                control_code: ProtosocketControlCode::Normal as u32,
                rpc_kind: Some(RpcKind::Unary(Unary {
                    command: Some(Command::Get(GetCommand {
                        namespace: self.cache_name.to_string(),
                        key: self.key.into_bytes(),
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
                Status::NotFound => match error.message.as_str() {
                    "Key not found" => Ok(crate::cache::GetResponse::Miss),
                    _ => Err(MomentoError::protosocket_command_error(error)),
                },
                _ => Err(MomentoError::protosocket_command_error(error)),
            },
            _ => Err(MomentoError::protosocket_unexpected_kind_error()),
        }
    }
}
