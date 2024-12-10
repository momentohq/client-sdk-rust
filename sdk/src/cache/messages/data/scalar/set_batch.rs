use std::collections::HashMap;
use std::time::Duration;

use momento_protos::cache_client::ECacheResult;

use crate::cache::MomentoRequest;
use crate::utils::{parse_string, prep_request_with_timeout};
use crate::{CacheClient, IntoBytes, MomentoError, MomentoResult};

use crate::cache::messages::data::scalar::set::SetResponse;

/// Request to set a batch of items in a cache.
///
/// # Arguments
///
/// * `cache_name` - name of the cache
/// * `items` - HashMap of (key, value) pairs to set
///
/// # Optional Arguments
///
/// * `ttl` - The time-to-live for the items. If not provided, the client's default time-to-live is used.
///
/// # Example
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::time::Duration;
/// use std::collections::HashMap;
/// use momento::cache::{SetResponse, SetBatchRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
///
/// let items = HashMap::from([("k1", "v1"), ("k2", "v2"), ("k3", "v3")]);
/// let set_batch_request = SetBatchRequest::new(
///     &cache_name,
///     items
/// ).ttl(Duration::from_secs(60));
///
/// match cache_client.send_request(set_batch_request).await {
///     Ok(_) => println!("SetBatchResponse successful"),
///     Err(e) => if let MomentoErrorCode::CacheNotFoundError = e.error_code {
///         println!("Cache not found: {}", &cache_name);
///     } else {
///         eprintln!("Error setting values in cache {}: {}", &cache_name, e);
///     }
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SetBatchRequest<K: IntoBytes + Copy, V: IntoBytes + Copy> {
    cache_name: String,
    items: HashMap<K, V>,
    ttl: Option<Duration>,
}

impl<K: IntoBytes + Copy, V: IntoBytes + Copy> SetBatchRequest<K, V> {
    /// Construct a new SetBatchRequest.
    pub fn new(cache_name: impl Into<String>, items: HashMap<K, V>) -> Self {
        Self {
            cache_name: cache_name.into(),
            items,
            ttl: None,
        }
    }

    /// Set the time-to-live for the batch of items.
    pub fn ttl(mut self, ttl: impl Into<Option<Duration>>) -> Self {
        self.ttl = ttl.into();
        self
    }
}

impl<K: IntoBytes + Copy, V: IntoBytes + Copy> MomentoRequest for SetBatchRequest<K, V> {
    type Response = SetBatchResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SetBatchResponse> {
        // Turn map of items into a vector of keys and vector of SetRequest objects
        // so we can map keys to the correct SetResponse objects later
        let mut set_requests: Vec<momento_protos::cache_client::SetRequest> = vec![];
        let mut set_request_keys: Vec<String> = vec![];
        for (key, value) in self.items.iter() {
            let byte_key = key.into_bytes();
            let set_request = momento_protos::cache_client::SetRequest {
                cache_key: byte_key.clone(),
                cache_body: value.into_bytes(),
                ttl_milliseconds: cache_client.expand_ttl_ms(self.ttl)?,
            };
            set_requests.push(set_request);
            set_request_keys.push(parse_string(byte_key.clone())?);
        }

        let set_batch_request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.deadline_millis(),
            momento_protos::cache_client::SetBatchRequest {
                items: set_requests,
            },
        )?;

        let mut response_stream = cache_client
            .next_data_client()
            .set_batch(set_batch_request)
            .await?
            .into_inner();

        // receive stream of get responses
        let mut responses: HashMap<String, SetResponse> = HashMap::new();
        let mut set_request_keys_iter = set_request_keys.iter();
        while let Some(set_response) = response_stream.message().await? {
            let sdk_set_response = match set_response.result() {
                ECacheResult::Ok => SetResponse {},
                _ => {
                    return Err(MomentoError::unknown_error(
                        "Set",
                        Some(format!("{:#?}", set_response)),
                    ))
                }
            };
            let key = match set_request_keys_iter.next() {
                Some(key) => key,
                None => {
                    return Err(MomentoError::unknown_error(
                        "SetBatch",
                        Some("Received more responses than expected".to_string()),
                    ))
                }
            };
            responses.insert(key.to_string(), sdk_set_response);
        }

        Ok(SetBatchResponse {
            results_dictionary: responses,
        })
    }
}

/// The response type for a successful set request.
///
/// You can use `into()` to convert a `SetBatchResponse` into a `HashMap<String, SetResponse>` or a `Vec<SetResponse>`.
#[derive(Debug, PartialEq, Eq)]
pub struct SetBatchResponse {
    results_dictionary: HashMap<String, SetResponse>,
}

impl From<SetBatchResponse> for HashMap<String, SetResponse> {
    fn from(response: SetBatchResponse) -> Self {
        response.results_dictionary
    }
}

impl From<SetBatchResponse> for Vec<SetResponse> {
    fn from(response: SetBatchResponse) -> Self {
        response.results_dictionary.into_values().collect()
    }
}
