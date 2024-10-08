use momento_protos::cache_client::set_if_request::Condition::Present;
use momento_protos::cache_client::set_if_response;

use crate::cache::messages::MomentoRequest;
use crate::utils::prep_request_with_timeout;
use crate::CacheClient;
use crate::{IntoBytes, MomentoError, MomentoResult};
use std::time::Duration;

/// Request to set associate the given key with the given value if key is present in the cache.
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
/// use momento::cache::{SetIfPresentResponse, SetIfPresentRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
///
/// let set_request = SetIfPresentRequest::new(
///     &cache_name,
///     "key",
///     "value1"
/// ).ttl(Duration::from_secs(60));
///
/// match cache_client.send_request(set_request).await {
///     Ok(response) => match response {
///         SetIfPresentResponse::Stored => println!("Value stored"),
///         SetIfPresentResponse::NotStored => println!("Value not stored"),
///     }
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
pub struct SetIfPresentRequest<K: IntoBytes, V: IntoBytes> {
    cache_name: String,
    key: K,
    value: V,
    ttl: Option<Duration>,
}

impl<K: IntoBytes, V: IntoBytes> SetIfPresentRequest<K, V> {
    /// Constructs a new SetIfPresentRequest.
    pub fn new(cache_name: impl Into<String>, key: K, value: V) -> Self {
        let ttl = None;
        Self {
            cache_name: cache_name.into(),
            key,
            value,
            ttl,
        }
    }

    /// Set the time-to-live for the item.
    pub fn ttl(mut self, ttl: impl Into<Option<Duration>>) -> Self {
        self.ttl = ttl.into();
        self
    }
}

impl<K: IntoBytes, V: IntoBytes> MomentoRequest for SetIfPresentRequest<K, V> {
    type Response = SetIfPresentResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SetIfPresentResponse> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::SetIfRequest {
                cache_key: self.key.into_bytes(),
                cache_body: self.value.into_bytes(),
                ttl_milliseconds: cache_client.expand_ttl_ms(self.ttl)?,
                condition: Some(Present(momento_protos::common::Present {})),
            },
        )?;

        let response = cache_client
.next_data_client()
            .set_if(request)
            .await?
            .into_inner();
        match response.result {
            Some(set_if_response::Result::Stored(_)) => Ok(SetIfPresentResponse::Stored),
            Some(set_if_response::Result::NotStored(_)) => Ok(SetIfPresentResponse::NotStored),
            _ => Err(MomentoError::unknown_error(
                "SetIfPresent",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response for a set if present operation.
#[derive(Debug, PartialEq, Eq)]
pub enum SetIfPresentResponse {
    /// The value was successfully stored.
    Stored,
    /// The key was not present in the cache.
    NotStored,
}
