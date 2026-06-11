use std::time::Duration;

use crate::cache::DeleteRequest;
use crate::protosocket::cache::MomentoProtosocketRequest;
use crate::{utils, IntoBytes, MomentoError, MomentoResult, ProtosocketCacheClient};
use momento_protos::protosocket::cache::cache_command::RpcKind;
use momento_protos::protosocket::cache::cache_response::Kind;
use momento_protos::protosocket::cache::unary::Command;
use momento_protos::protosocket::cache::{CacheCommand, DeleteCommand, DeleteResponse, Unary};
use protosocket_rpc::ProtosocketControlCode;

impl<K: IntoBytes> MomentoProtosocketRequest for DeleteRequest<K> {
    type Response = crate::cache::DeleteResponse;

    async fn send(
        self,
        client: &ProtosocketCacheClient,
        timeout: Duration,
    ) -> MomentoResult<Self::Response> {
        utils::execute_protosocket_request_with_timeout(|| self.send_delete(client), timeout).await
    }
}

impl<K: IntoBytes> DeleteRequest<K> {
    async fn send_delete(
        self,
        client: &ProtosocketCacheClient,
    ) -> MomentoResult<crate::cache::DeleteResponse> {
        let completion = client
            .protosocket_connection()
            .await?
            .send_unary(CacheCommand {
                message_id: client.message_id(),
                control_code: ProtosocketControlCode::Normal as u32,
                rpc_kind: Some(RpcKind::Unary(Unary {
                    command: Some(Command::Delete(DeleteCommand {
                        namespace: self.cache_name.to_string(),
                        key: self.key.into_bytes(),
                    })),
                })),
            })
            .await?;
        let response = completion.await?;
        match response.kind {
            Some(Kind::Delete(DeleteResponse {})) => Ok(crate::cache::DeleteResponse {}),
            Some(Kind::Error(error)) => Err(MomentoError::protosocket_command_error(error)),
            _ => Err(MomentoError::protosocket_unexpected_kind_error()),
        }
    }
}
