use std::collections::HashMap;
use std::convert::TryFrom;
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
pub struct SetBatchRequest<K: IntoBytes, V: IntoBytes> {
    cache_name: String,
    items: Vec<(K, V)>,
    ttl: Option<Duration>,
}

impl<K: IntoBytes, V: IntoBytes> SetBatchRequest<K, V> {
    /// Construct a new SetBatchRequest.
    pub fn new(cache_name: impl Into<String>, items: impl IntoIterator<Item = (K, V)>) -> Self {
        Self {
            cache_name: cache_name.into(),
            items: items.into_iter().collect(),
            ttl: None,
        }
    }

    /// Set the time-to-live for the batch of items.
    pub fn ttl(mut self, ttl: impl Into<Option<Duration>>) -> Self {
        self.ttl = ttl.into();
        self
    }
}

impl<K: IntoBytes, V: IntoBytes> MomentoRequest for SetBatchRequest<K, V> {
    type Response = SetBatchResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SetBatchResponse> {
        // Turn map of items into a vector of keys and vector of SetRequest objects
        // so we can map keys to the correct SetResponse objects later
        let mut set_requests: Vec<momento_protos::cache_client::SetRequest> = vec![];
        let mut set_request_keys: Vec<Vec<u8>> = vec![];
        for (key, value) in self.items.into_iter() {
            let byte_key = key.into_bytes();
            let set_request = momento_protos::cache_client::SetRequest {
                cache_key: byte_key.clone(),
                cache_body: value.into_bytes(),
                ttl_milliseconds: cache_client.expand_ttl_ms(self.ttl)?,
            };
            set_requests.push(set_request);
            set_request_keys.push(byte_key);
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

        // receive stream of set responses
        let mut responses: HashMap<Vec<u8>, SetResponse> = HashMap::new();
        let mut set_request_keys_iter = set_request_keys.into_iter();
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
            responses.insert(key, sdk_set_response);
        }

        Ok(SetBatchResponse {
            results_dictionary: responses,
        })
    }
}

/// The response type for a successful set request.
///
/// You can use `into()` to convert a `SetBatchResponse` into a `HashMap<Vec<u8>, SetResponse>`.
///
/// You can use `try_into()` to convert a `SetBatchResponse` into a `HashMap<String, SetResponse>`.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use std::convert::TryInto;
/// use std::collections::HashMap;
/// use momento::cache::SetResponse;
///
/// let items = HashMap::from([("k1", "v1"), ("k2", "v2"), ("k3", "v3")]);
/// let set_batch_response = cache_client.set_batch(&cache_name, items).await?;
///
/// let byte_keys_set_responses: HashMap<Vec<u8>, SetResponse> = set_batch_response.clone().into();
///
/// let str_keys_set_responses: HashMap<String, SetResponse> = set_batch_response.clone().try_into().expect("stored string keys");
/// # Ok(())
/// # })
/// # }
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SetBatchResponse {
    results_dictionary: HashMap<Vec<u8>, SetResponse>,
}

impl From<SetBatchResponse> for HashMap<Vec<u8>, SetResponse> {
    fn from(response: SetBatchResponse) -> Self {
        response.results_dictionary
    }
}

impl TryFrom<SetBatchResponse> for HashMap<String, SetResponse> {
    type Error = MomentoError;

    fn try_from(response: SetBatchResponse) -> Result<Self, Self::Error> {
        response
            .results_dictionary
            .into_iter()
            .map(|(key, response)| match parse_string(key) {
                Ok(str_key) => Ok((str_key, response)),
                Err(e) => Err(e),
            })
            .collect()
    }
}
