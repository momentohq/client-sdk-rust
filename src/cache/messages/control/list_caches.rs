use momento_protos::control_client;
use tonic::Request;

use crate::cache::messages::MomentoRequest;
use crate::{CacheClient, MomentoResult};

/// Request to list all caches in your account.
///
/// # Example
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{ListCachesResponse, ListCachesRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
///
/// let list_caches_request = ListCachesRequest {};
///
/// match cache_client.send_request(list_caches_request).await {
///     Ok(response) => println!("Caches: {:#?}", response.caches),
///     Err(e) => eprintln!("Error listing caches: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListCachesRequest {}

impl MomentoRequest for ListCachesRequest {
    type Response = ListCachesResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ListCachesResponse> {
        let request = Request::new(control_client::ListCachesRequest {
            next_token: "".to_string(),
        });

        let response = cache_client
            .control_client()
            .list_caches(request)
            .await?
            .into_inner();

        Ok(ListCachesResponse::from_response(response))
    }
}

/// Limits for a cache.
#[derive(Debug, PartialEq, Eq)]
pub struct CacheLimits {
    /// The maximum traffic rate in requests per second.
    pub max_traffic_rate: u32,
    /// The maximum throughput in kilobytes per second.
    pub max_throughput_kbps: u32,
    /// The maximum item size in kilobytes.
    pub max_item_size_kb: u32,
    /// The maximum time-to-live in seconds.
    pub max_ttl_seconds: u64,
}

/// Limits for topics in a cache.
#[derive(Debug, PartialEq, Eq)]
pub struct TopicLimits {
    /// The maximum publish rate in requests per second.
    pub max_publish_rate: u32,
    /// The maximum number of subscriptions.
    pub max_subscription_count: u32,
    /// The maximum publish message size in kilobytes.
    pub max_publish_message_size_kb: u32,
}

/// Information about a cache.
#[derive(Debug, PartialEq, Eq)]
pub struct CacheInfo {
    /// The name of the cache.
    pub name: String,
    /// The limits for the cache.
    pub cache_limits: CacheLimits,
    /// The limits for topics in the cache.
    pub topic_limits: TopicLimits,
}

/// Response for a list caches operation.
///
/// You can cast your result directly into a `Result<Vec<CacheInfo>, MomentoError>` suitable for
/// ?-propagation if you know you are expecting a `Vec<CacheInfo>` item.
/// ```
/// # use momento::cache::{CacheInfo, ListCachesResponse};
/// # use momento::MomentoResult;
/// # let list_caches_response = ListCachesResponse { caches: vec![] };
/// let caches: Vec<CacheInfo> = list_caches_response.into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct ListCachesResponse {
    /// The caches in your account.
    pub caches: Vec<CacheInfo>,
}

impl ListCachesResponse {
    /// Convert a ListCachesResponse from the server into a ListCachesResponse.
    pub fn from_response(response: control_client::ListCachesResponse) -> ListCachesResponse {
        let mut caches = Vec::new();
        for cache in response.cache {
            let cache_limits = cache.cache_limits.unwrap_or_default();
            let topic_limits = cache.topic_limits.unwrap_or_default();
            caches.push(CacheInfo {
                name: cache.cache_name,
                cache_limits: CacheLimits {
                    max_traffic_rate: cache_limits.max_traffic_rate,
                    max_throughput_kbps: cache_limits.max_throughput_kbps,
                    max_item_size_kb: cache_limits.max_item_size_kb,
                    max_ttl_seconds: cache_limits.max_ttl_seconds,
                },
                topic_limits: TopicLimits {
                    max_publish_rate: topic_limits.max_publish_rate,
                    max_subscription_count: topic_limits.max_subscription_count,
                    max_publish_message_size_kb: topic_limits.max_publish_message_size_kb,
                },
            });
        }
        ListCachesResponse { caches }
    }
}

impl From<ListCachesResponse> for Vec<CacheInfo> {
    fn from(response: ListCachesResponse) -> Self {
        response.caches
    }
}
