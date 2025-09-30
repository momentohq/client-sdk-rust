use momento_protos::protosocket::cache::{CacheCommand, CacheResponse};
use protosocket_rpc::client::{ConnectionPool, RpcClient};

use crate::cache::{GetRequest, SetRequest};
use crate::protosocket::cache::cache_client_builder::NeedsDefaultTtl;
use crate::protosocket::cache::utils::ProtosocketConnectionManager;
use crate::protosocket::cache::{Configuration, MomentoProtosocketRequest};
use crate::{utils, IntoBytes, MomentoError, MomentoResult, ProtosocketCacheClientBuilder};
use std::convert::TryInto;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

// TODO: remove `no_run` on doc examples to allow fully running them as doctests

/// A client for interacting with Momento Cache using the Protosocket protocol.
/// Client to work with Momento Cache, the serverless caching service, but using the protosocket protocol instead of gRPC.
///
/// # Example
/// To instantiate a [ProtosocketCacheClient], you need to provide a default TTL, a [Configuration](crate::protosocket::cache::Configuration), a [CredentialProvider](crate::CredentialProvider), and a [tokio::runtime::Handle].
/// Prebuilt configurations tuned for different environments are available in the [protosocket::cache::configurations](crate::protosocket::cache::configurations) module.
/// After building the client, make sure to authenticate with the server before sending any requests.
///
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use momento::protosocket::cache::configurations;
/// use momento::{CredentialProvider, ProtosocketCacheClient};
/// use std::time::Duration;
///
/// let cache_client = match ProtosocketCacheClient::builder()
///     .default_ttl(Duration::from_secs(60))
///     .configuration(configurations::Laptop::latest())
///     .credential_provider(
///         CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
///             .expect("auth token should be valid"),
///     )
///     .runtime(tokio::runtime::Handle::current())
///     .build()
///     .await
/// {
///     Ok(client) => client,
///     Err(err) => panic!("{err}"),
/// };
/// # Ok(())
/// # })
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct ProtosocketCacheClient {
    client_pool: std::sync::Arc<ConnectionPool<ProtosocketConnectionManager>>,
    message_id: std::sync::Arc<AtomicU64>,
    item_default_ttl: Duration,
    request_timeout: Duration,
}

impl ProtosocketCacheClient {
    pub(crate) fn new(
        client_pool: ConnectionPool<ProtosocketConnectionManager>,
        default_ttl: Duration,
        configuration: Configuration,
    ) -> Self {
        Self {
            client_pool: std::sync::Arc::new(client_pool),
            message_id: std::sync::Arc::new(AtomicU64::new(0)),
            item_default_ttl: default_ttl,
            request_timeout: configuration.timeout(),
        }
    }

    /// Constructs a ProtosocketCacheClient to use Momento Cache using the protosocket protocol.
    ///
    /// # Arguments
    /// - `default_ttl` - Default time-to-live for items in the cache.
    /// - `configuration` - Prebuilt configurations tuned for different environments are available in the [protosocket::cache::configurations](crate::protosocket::cache::configurations) module.
    /// - `credential_provider` - A [CredentialProvider](crate::CredentialProvider) to use for authenticating with Momento.
    /// - `runtime` - A [tokio::runtime::Handle] to use for running the client.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() -> anyhow::Result<()> {
    /// # tokio_test::block_on(async {
    /// use momento::protosocket::cache::configurations;
    /// use momento::{CredentialProvider, ProtosocketCacheClient};
    /// use std::time::Duration;
    ///
    /// let cache_client = match ProtosocketCacheClient::builder()
    ///     .default_ttl(Duration::from_secs(60))
    ///     .configuration(configurations::Laptop::latest())
    ///     .credential_provider(
    ///         CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
    ///             .expect("auth token should be valid"),
    ///     )
    ///     .runtime(tokio::runtime::Handle::current())
    ///     .build()
    ///     .await
    /// {
    ///     Ok(client) => client,
    ///     Err(err) => panic!("{err}"),
    /// };
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub fn builder() -> ProtosocketCacheClientBuilder<NeedsDefaultTtl> {
        ProtosocketCacheClientBuilder(NeedsDefaultTtl(()))
    }

    /// Gets an item from a Momento Cache
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache
    /// * `key` - key of entry within the cache.
    ///
    /// # Examples
    /// Assumes that a ProtosocketCacheClient named `cache_client` has been created and is available.
    /// ```no_run
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_protosocket_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_protosocket_cache_client().await;
    /// use std::convert::TryInto;
    /// use momento::cache::GetResponse;
    /// # cache_client.set(&cache_name, "key", "value").await?;
    ///
    /// let item: String = match(cache_client.get(&cache_name, "key").await?) {
    ///     GetResponse::Hit { value } => value.try_into().expect("I stored a string!"),
    ///     GetResponse::Miss => return Err(anyhow::Error::msg("cache miss"))
    /// };
    /// # assert_eq!(item, "value");
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](ProtosocketCacheClient::send_request) method to get an item using a [GetRequest].
    ///
    /// For more examples of handling the response, see [GetResponse](crate::cache::GetResponse).
    pub async fn get(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
    ) -> MomentoResult<crate::cache::GetResponse> {
        let request = GetRequest::new(cache_name, key);
        request.send(self, self.request_timeout).await
    }

    /// Sets an item in a Momento Cache
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache
    /// * `key` - key of the item whose value we are setting
    /// * `value` - data to stored in the cache item
    ///
    /// # Optional Arguments
    /// If you use [send_request](ProtosocketCacheClient::send_request) to set an item using a
    /// [SetRequest], you can also provide the following optional arguments:
    ///
    /// * `ttl` - The time-to-live for the item. If not provided, the client's default time-to-live is used.
    ///
    /// # Examples
    /// Assumes that a ProtosocketCacheClient named `cache_client` has been created and is available.
    /// ```no_run
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_protosocket_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_protosocket_cache_client().await;
    /// use momento::cache::SetResponse;
    /// use momento::MomentoErrorCode;
    ///
    /// match cache_client.set(&cache_name, "k1", "v1").await {
    ///     Ok(_) => println!("SetResponse successful"),
    ///     Err(e) => if let MomentoErrorCode::CacheNotFoundError = e.error_code {
    ///         println!("Cache not found: {}", &cache_name);
    ///     } else {
    ///         eprintln!("Error setting value in cache {}: {}", &cache_name, e);
    ///     }
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](ProtosocketCacheClient::send_request) method to get an item using a [SetRequest]
    /// which will allow you to set [optional arguments](crate::cache::SetRequest#optional-arguments) as well.
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

    pub(crate) fn message_id(&self) -> u64 {
        self.message_id.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) async fn protosocket_connection(
        &self,
    ) -> MomentoResult<RpcClient<CacheCommand, CacheResponse>> {
        let pooled_client = self.client_pool.get_connection().await.map_err(|e| {
            MomentoError::unknown_error("protosocket_connection", Some(e.to_string()))
        })?;
        Ok(pooled_client.clone())
    }

    pub(crate) fn expand_ttl_ms(&self, ttl: Option<Duration>) -> MomentoResult<u64> {
        let ttl = ttl.unwrap_or(self.item_default_ttl);
        utils::is_ttl_valid(ttl)?;

        Ok(ttl.as_millis().try_into().unwrap_or(i64::MAX as u64))
    }
}
