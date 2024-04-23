use momento_protos::cache_client::set_if_request::Condition::Absent;
use momento_protos::cache_client::set_if_response;

use crate::cache::requests::MomentoRequest;
use crate::cache_client::CacheClient;
use crate::utils::prep_request_with_timeout;
use crate::{IntoBytes, MomentoResult};
use std::time::Duration;

/// Request to set associate the given key with the given value if key is not already present in the cache.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache to create.
/// * `key` - key of the item whose value we are setting
/// * `value` - data to store
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
/// use momento::cache::{SetIfAbsent, SetIfAbsentRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
///
/// let set_request = SetIfAbsentRequest::new(
///     &cache_name,
///     "key",
///     "value1"
/// ).ttl(Duration::from_secs(60));
///
/// match cache_client.send_request(set_request).await {
///     Ok(response) => match response {
///         SetIfAbsent::Stored => println!("Value stored"),
///         SetIfAbsent::NotStored => println!("Value not stored"),
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
pub struct SetIfAbsentRequest<K: IntoBytes, V: IntoBytes> {
    cache_name: String,
    key: K,
    value: V,
    ttl: Option<Duration>,
}

impl<K: IntoBytes, V: IntoBytes> SetIfAbsentRequest<K, V> {
    pub fn new(cache_name: impl Into<String>, key: K, value: V) -> Self {
        let ttl = None;
        Self {
            cache_name: cache_name.into(),
            key,
            value,
            ttl,
        }
    }

    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

impl<K: IntoBytes, V: IntoBytes> MomentoRequest for SetIfAbsentRequest<K, V> {
    type Response = SetIfAbsent;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SetIfAbsent> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::SetIfRequest {
                cache_key: self.key.into_bytes(),
                cache_body: self.value.into_bytes(),
                ttl_milliseconds: cache_client.expand_ttl_ms(self.ttl)?,
                condition: Some(Absent(momento_protos::common::Absent {})),
            },
        )?;

        let response = cache_client.data_client.clone().set_if(request).await?.into_inner();
        match response.result {
            Some(set_if_response::Result::Stored(_)) => Ok(SetIfAbsent::Stored),
            Some(set_if_response::Result::NotStored(_)) => Ok(SetIfAbsent::NotStored),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SetIfAbsent {
    Stored,
    NotStored,
}
