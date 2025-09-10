use crate::cache::SetRequest;
use crate::protosocket::cache::MomentoProtosocketRequest;
use crate::{IntoBytes, MomentoError, MomentoResult, ProtosocketCacheClient};
use momento_protos::protosocket::cache::cache_command::RpcKind;
use momento_protos::protosocket::cache::cache_response::Kind;
use momento_protos::protosocket::cache::unary::Command;
use momento_protos::protosocket::cache::{CacheCommand, SetCommand, SetResponse, Unary};
use protosocket_rpc::ProtosocketControlCode;

impl<K: IntoBytes, V: IntoBytes> MomentoProtosocketRequest for SetRequest<K, V> {
    type Response = crate::cache::SetResponse;

    async fn send(self, client: &ProtosocketCacheClient) -> MomentoResult<Self::Response> {
        let completion = client
            .protosocket_client()
            .send_unary(CacheCommand {
                message_id: client.message_id(),
                control_code: ProtosocketControlCode::Normal as u32,
                rpc_kind: Some(RpcKind::Unary(Unary {
                    command: Some(Command::Set(SetCommand {
                        namespace: self.cache_name.to_string(),
                        key: self.key.into_bytes(),
                        value: self.value.into_bytes(),
                        ttl_milliseconds: client.expand_ttl_ms(self.ttl)?,
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
