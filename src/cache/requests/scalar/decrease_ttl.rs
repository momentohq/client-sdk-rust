use std::time::Duration;

use momento_protos::cache_client::update_ttl_request::UpdateTtl::DecreaseToMilliseconds;
use momento_protos::cache_client::update_ttl_response::{self};

use crate::MomentoError;
use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoResult,
};

/// Decrease the ttl of the key in the cache.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `key` - the key for which ttl is requested
/// * `ttl` - The time-to-live that should overwrite the current ttl. Should be less than the current ttl.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use std::time::Duration;
/// use momento::cache::{DecreaseTtl, DecreaseTtlRequest};
/// # cache_client.set(&cache_name, "key1", "value").await?;
///
/// let request = DecreaseTtlRequest::new(&cache_name, "key1", Duration::from_secs(3));
///
/// match(cache_client.send_request(request).await?) {
///     DecreaseTtl::Set => println!("TTL updated"),
///     DecreaseTtl::NotSet => println!("unable to decrease TTL"),
///     DecreaseTtl::Miss => return Err(anyhow::Error::msg("cache miss"))
/// };
/// # Ok(())
/// # })
/// # }
/// ```
pub struct DecreaseTtlRequest<K: IntoBytes> {
    cache_name: String,
    key: K,
    ttl: Duration,
}

impl<K: IntoBytes> DecreaseTtlRequest<K> {
    pub fn new(cache_name: impl Into<String>, key: K, ttl: Duration) -> Self {
        Self {
            cache_name: cache_name.into(),
            key,
            ttl,
        }
    }
}

impl<K: IntoBytes> MomentoRequest for DecreaseTtlRequest<K> {
    type Response = DecreaseTtl;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<DecreaseTtl> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::UpdateTtlRequest {
                cache_key: self.key.into_bytes(),
                update_ttl: Some(DecreaseToMilliseconds(
                    cache_client.expand_ttl_ms(Some(self.ttl))?,
                )),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .update_ttl(request)
            .await?
            .into_inner();

        match response.result {
            Some(update_ttl_response::Result::Missing(_)) => Ok(DecreaseTtl::Miss),
            Some(update_ttl_response::Result::Set(_)) => Ok(DecreaseTtl::Set),
            Some(update_ttl_response::Result::NotSet(_)) => Ok(DecreaseTtl::NotSet),
            _ => Err(MomentoError::unknown_error(
                "DecreaseTtl",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecreaseTtl {
    Set,
    NotSet,
    Miss,
}
