use std::convert::TryFrom;
use std::time::Duration;

use momento_protos::cache_client::item_get_ttl_response::{self};

use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoError,
    MomentoResult,
};

/// Return the remaining ttl of an item in the cache
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `key` - the key of the item for which the remaining ttl is requested
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use std::convert::TryInto;
/// use std::time::Duration;
/// use momento::cache::{ItemGetTtlResponse, ItemGetTtlRequest};
/// # cache_client.set(&cache_name, "key1", "value").await?;
///
/// let request = ItemGetTtlRequest::new(&cache_name, "key1");
///
/// let remaining_ttl: Duration = cache_client.send_request(request).await?.try_into().expect("Expected an item ttl!");
/// # assert!(remaining_ttl <= Duration::from_secs(5));
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ItemGetTtlRequest<K: IntoBytes> {
    cache_name: String,
    key: K,
}

impl<K: IntoBytes> ItemGetTtlRequest<K> {
    /// Constructs a new ItemGetTtlRequest.
    pub fn new(cache_name: impl Into<String>, key: K) -> Self {
        Self {
            cache_name: cache_name.into(),
            key,
        }
    }
}

impl<K: IntoBytes> MomentoRequest for ItemGetTtlRequest<K> {
    type Response = ItemGetTtlResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ItemGetTtlResponse> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::ItemGetTtlRequest {
                cache_key: self.key.into_bytes(),
            },
        )?;

        let response = cache_client
.next_data_client()
            .item_get_ttl(request)
            .await?
            .into_inner();

        match response.result {
            Some(item_get_ttl_response::Result::Missing(_)) => Ok(ItemGetTtlResponse::Miss),
            Some(item_get_ttl_response::Result::Found(found)) => Ok(ItemGetTtlResponse::Hit {
                remaining_ttl: Duration::from_millis(found.remaining_ttl_millis),
            }),
            _ => Err(MomentoError::unknown_error(
                "ItemGetTtl",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response for a get item ttl operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// # use momento::cache::ItemGetTtlResponse;
/// use std::convert::TryInto;
/// use std::time::Duration;
/// # let response = ItemGetTtlResponse::Hit { remaining_ttl: Duration::from_secs(5) };
/// let remaining_ttl: Duration = match response {
///     ItemGetTtlResponse::Hit { remaining_ttl } => remaining_ttl.try_into().expect("Expected an item ttl!"),
///     ItemGetTtlResponse::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<ItemType, MomentoError> suitable for
/// ?-propagation if you know you are expecting a GetItemType::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// # use momento::cache::ItemGetTtlResponse;
/// use std::convert::TryInto;
/// use std::time::Duration;
/// # let response = ItemGetTtlResponse::Hit { remaining_ttl: Duration::from_secs(1) };
/// let remaining_ttl: MomentoResult<Duration> = response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum ItemGetTtlResponse {
    /// The item was found in the cache.
    Hit {
        /// The remaining time-to-live of the item.
        remaining_ttl: Duration,
    },
    /// The item was not found in the cache.
    Miss,
}

impl TryFrom<ItemGetTtlResponse> for Duration {
    type Error = MomentoError;

    fn try_from(value: ItemGetTtlResponse) -> Result<Self, Self::Error> {
        match value {
            ItemGetTtlResponse::Hit { remaining_ttl } => Ok(remaining_ttl),
            ItemGetTtlResponse::Miss => Err(MomentoError::miss("ItemGetTtl")),
        }
    }
}
