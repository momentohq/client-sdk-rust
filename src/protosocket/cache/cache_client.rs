use crate::cache::{GetRequest, SetRequest};
use crate::protosocket::cache::cache_client_builder::NeedsDefaultTtl;
use crate::protosocket::cache::{Configuration, MomentoProtosocketRequest};
use crate::{utils, IntoBytes, MomentoResult, ProtosocketCacheClientBuilder};
use momento_protos::protosocket::cache::{CacheCommand, CacheResponse};
use std::convert::TryInto;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// A client for interacting with Momento Cache using the Protosocket protocol.
// TODO: complete docs
pub struct ProtosocketCacheClient {
    message_id: AtomicU64,
    client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
    item_default_ttl: Duration,
    request_timeout: Duration,
}

impl ProtosocketCacheClient {
    pub(crate) fn new(
        message_id: AtomicU64,
        client: protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse>,
        default_ttl: Duration,
        configuration: Configuration,
    ) -> Self {
        Self {
            message_id,
            client,
            item_default_ttl: default_ttl,
            request_timeout: configuration.timeout(),
        }
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
        cache_name: impl Into<String>,
        key: impl IntoBytes,
    ) -> MomentoResult<crate::cache::GetResponse> {
        let request = GetRequest::new(cache_name, key);
        request.send(self, self.request_timeout).await
    }

    /// Sets an item in a Momento Cache
    // TODO: request timeout, default ttl, request building pattern, docs
    pub async fn set(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        value: impl IntoBytes,
    ) -> MomentoResult<crate::cache::SetResponse> {
        let request = SetRequest::new(cache_name, key, value);
        request.send(self, self.request_timeout).await
    }

    /// Lower-level API to send any type of MomentoProtosocketRequest to the server. This is used for cases when
    /// you want to set optional fields on a request that are not supported by the short-hand API for
    /// that request type.
    pub async fn send_request<R: MomentoProtosocketRequest>(
        &self,
        request: R,
    ) -> MomentoResult<R::Response> {
        request.send(self, self.request_timeout).await
    }

    pub(crate) fn protosocket_client(
        &self,
    ) -> &protosocket_rpc::client::RpcClient<CacheCommand, CacheResponse> {
        &self.client
    }

    pub(crate) fn message_id(&self) -> u64 {
        self.message_id.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) fn expand_ttl_ms(&self, ttl: Option<Duration>) -> MomentoResult<u64> {
        let ttl = ttl.unwrap_or(self.item_default_ttl);
        utils::is_ttl_valid(ttl)?;

        Ok(ttl.as_millis().try_into().unwrap_or(i64::MAX as u64))
    }
}
