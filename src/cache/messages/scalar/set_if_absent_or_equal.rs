use momento_protos::cache_client::set_if_request::Condition::AbsentOrEqual;
use momento_protos::cache_client::set_if_response;

use crate::cache::messages::MomentoRequest;
use crate::utils::prep_request_with_timeout;
use crate::CacheClient;
use crate::{IntoBytes, MomentoError, MomentoResult};
use std::time::Duration;

/// Request to associate the given key with the given value if the key does not already
/// exist in the cache or the value in the cache is equal to the supplied `equal` value.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache to create.
/// * `key` - key of the item whose value we are setting
/// * `value` - data to store
/// * `equal` - data to compare to the cached value
///
/// # Optional Arguments
///
/// * `ttl` - The time-to-live for the item. If not provided, the client's default time-to-live is used.
///
/// # Example
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::time::Duration;
/// use momento::cache::{SetIfAbsentOrEqual, SetIfAbsentOrEqualRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
///
/// let set_request = SetIfAbsentOrEqualRequest::new(
///     &cache_name,
///     "key",
///     "new-value",
///     "cached-value"
/// ).ttl(Duration::from_secs(60));
///
/// match cache_client.send_request(set_request).await {
///     Ok(response) => match response {
///         SetIfAbsentOrEqual::Stored => println!("Value stored"),
///         SetIfAbsentOrEqual::NotStored => println!("Value not stored"),
///     }
///     Err(e) => if let MomentoErrorCode::NotFoundError = e.error_code {
///         println!("Cache not found: {}", &cache_name);
///     } else {
///         eprintln!("Error setting value in cache {}: {}", &cache_name, e);
///     }
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SetIfAbsentOrEqualRequest<K: IntoBytes, V: IntoBytes, E: IntoBytes> {
    cache_name: String,
    key: K,
    value: V,
    equal: E,
    ttl: Option<Duration>,
}

impl<K: IntoBytes, V: IntoBytes, E: IntoBytes> SetIfAbsentOrEqualRequest<K, V, E> {
    pub fn new(cache_name: impl Into<String>, key: K, value: V, equal: E) -> Self {
        let ttl = None;
        Self {
            cache_name: cache_name.into(),
            key,
            value,
            equal,
            ttl,
        }
    }

    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

impl<K: IntoBytes, V: IntoBytes, E: IntoBytes> MomentoRequest
    for SetIfAbsentOrEqualRequest<K, V, E>
{
    type Response = SetIfAbsentOrEqual;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SetIfAbsentOrEqual> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::SetIfRequest {
                cache_key: self.key.into_bytes(),
                cache_body: self.value.into_bytes(),
                ttl_milliseconds: cache_client.expand_ttl_ms(self.ttl)?,
                condition: Some(AbsentOrEqual(momento_protos::common::AbsentOrEqual {
                    value_to_check: self.equal.into_bytes(),
                })),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .set_if(request)
            .await?
            .into_inner();
        match response.result {
            Some(set_if_response::Result::Stored(_)) => Ok(SetIfAbsentOrEqual::Stored),
            Some(set_if_response::Result::NotStored(_)) => Ok(SetIfAbsentOrEqual::NotStored),
            _ => Err(MomentoError::unknown_error(
                "SetIfAbsentOrEqual",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SetIfAbsentOrEqual {
    Stored,
    NotStored,
}
