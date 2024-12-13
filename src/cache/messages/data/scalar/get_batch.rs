use momento_protos::cache_client::ECacheResult;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
};

use crate::{
    cache::MomentoRequest,
    utils::{self, parse_string},
    CacheClient, IntoBytesIterable, MomentoError, MomentoResult,
};

use crate::cache::messages::data::scalar::get::{GetResponse, Value};

/// Request to get a batch of items from a Momento Cache
///
/// # Arguments
///
/// * `cache_name` - name of cache
/// * `keys` - list of keys to fetch
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use std::convert::TryInto;
/// use std::collections::HashMap;
/// use momento::cache::{GetBatchRequest, GetResponse};
/// # cache_client.set(&cache_name, "key1", "value1").await?;
/// # cache_client.set(&cache_name, "key2", "value2").await?;
///
/// let get_batch_request = GetBatchRequest::new(cache_name, vec!["key1", "key2"]);
/// let response = cache_client.send_request(get_batch_request).await?;
/// let results_map: HashMap<String, GetResponse> = response.try_into().expect("stored string keys");
/// # assert_eq!(results_map.clone().len(), 2);
///
/// for (key, response) in results_map {
///     match response {
///         GetResponse::Hit { value } => {
///             let value: String = value.try_into().expect("I stored a string!");
///             println!("Fetched value for key {}: {}", key, value);
///         }
///         GetResponse::Miss => println!("Cache miss for key {}", key),
///     }
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct GetBatchRequest<K: IntoBytesIterable> {
    cache_name: String,
    keys: K,
}

impl<K: IntoBytesIterable> GetBatchRequest<K> {
    /// Constructs a new GetBatchRequest.
    pub fn new(cache_name: impl Into<String>, keys: K) -> Self {
        Self {
            cache_name: cache_name.into(),
            keys,
        }
    }
}

impl<K: IntoBytesIterable> MomentoRequest for GetBatchRequest<K> {
    type Response = GetBatchResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<GetBatchResponse> {
        // Convert all keys to bytes so they can be sent over the wire
        // and create a HashMap of (key, GetResponse) pairs for the response
        let byte_keys: Vec<Vec<u8>> = self.keys.into_bytes();

        let get_requests = byte_keys
            .iter()
            .map(|key| momento_protos::cache_client::GetRequest {
                cache_key: key.clone(),
            })
            .collect();

        let get_batch_request = utils::prep_request_with_timeout(
            &self.cache_name,
            cache_client.deadline_millis(),
            momento_protos::cache_client::GetBatchRequest {
                items: get_requests,
            },
        )?;

        let mut response_stream = cache_client
            .next_data_client()
            .get_batch(get_batch_request)
            .await?
            .into_inner();

        // receive stream of get responses
        let mut responses: HashMap<Vec<u8>, GetResponse> = HashMap::new();
        let mut byte_keys_iter = byte_keys.into_iter();
        while let Some(get_response) = response_stream.message().await? {
            let sdk_get_response = match get_response.result() {
                ECacheResult::Hit => GetResponse::Hit {
                    value: Value {
                        raw_item: get_response.cache_body,
                    },
                },
                ECacheResult::Miss => GetResponse::Miss,
                _ => {
                    return Err(MomentoError::unknown_error(
                        "GetBatch",
                        Some(format!("{:#?}", get_response)),
                    ))
                }
            };
            let key = match byte_keys_iter.next() {
                Some(key) => key,
                None => {
                    return Err(MomentoError::unknown_error(
                        "GetBatch",
                        Some("Received more responses than expected".to_string()),
                    ))
                }
            };
            responses.insert(key, sdk_get_response);
        }

        Ok(GetBatchResponse {
            results_dictionary: responses,
        })
    }
}

/// Response for a cache get batch operation.
///
/// You can use `into()` to convert a `GetBatchResponse` into one of the following:
/// - `HashMap<Vec<u8>, GetResponse>`
/// - `HashMap<Vec<u8>, Value>`
/// - `HashMap<Vec<u8>, Vec<u8>>`
///
/// You can use `try_into()` to convert a `GetBatchResponse` into one of the following:
/// - `HashMap<Vec<u8>, String>`
/// - `HashMap<String, GetResponse>`
/// - `HashMap<String, Value>`
/// - `HashMap<String, String>`
/// - `HashMap<String, Vec<u8>>`
///
/// The `HashMap<_, GetResponse>` maps will return all the keys and get responses, whether they are a hit or miss.
///
/// The other conversions will filter out `GetResponse::Miss` responses because only `GetResponse::Hit` objects contain a `Value` fetched from the cache and can be converted to `String` or `Vec<u8>`.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use std::convert::TryInto;
/// use std::collections::HashMap;
/// use momento::cache::GetResponse;
/// use momento::cache::messages::data::scalar::get::Value;
///
/// let keys = vec!["key1", "key2", "key3"];
/// let get_batch_response = cache_client.get_batch(&cache_name, keys.clone()).await?;
///
/// let byte_keys_get_responses: HashMap<Vec<u8>, GetResponse> = get_batch_response.clone().into();
/// let str_keys_get_responses: HashMap<String, GetResponse> = get_batch_response.clone().try_into().expect("stored string keys");
///
/// let byte_keys_hit_values: HashMap<Vec<u8>, Value> = get_batch_response.clone().into();
/// let str_keys_hit_values: HashMap<String, Value> = get_batch_response.clone().try_into().expect("stored string keys");
///
/// let byte_keys_hit_bytes: HashMap<Vec<u8>, Vec<u8>> = get_batch_response.clone().into();
/// let str_keys_hit_bytes: HashMap<String, Vec<u8>> = get_batch_response.clone().try_into().expect("stored string keys");
///
/// let byte_keys_hit_strings: HashMap<Vec<u8>, String> = get_batch_response.clone().try_into().expect("stored string values");
/// let str_keys_hit_strings: HashMap<String, String> = get_batch_response.clone().try_into().expect("stored string keys and values");
/// # Ok(())
/// # })
/// # }
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GetBatchResponse {
    results_dictionary: HashMap<Vec<u8>, GetResponse>,
}

// (Bytes key, GetResponse) pairs -- does NOT filter out Miss responses
impl From<GetBatchResponse> for HashMap<Vec<u8>, GetResponse> {
    fn from(response: GetBatchResponse) -> Self {
        response.results_dictionary
    }
}

// (Bytes key, Value) pairs -- filters out Miss responses
impl From<GetBatchResponse> for HashMap<Vec<u8>, Value> {
    fn from(response: GetBatchResponse) -> Self {
        response
            .results_dictionary
            .into_iter()
            .filter_map(|(key, get_response)| match get_response {
                GetResponse::Hit { value } => Some((key, value)),
                GetResponse::Miss => None,
            })
            .collect()
    }
}

// (Bytes key, Bytes value) pairs -- filters out Miss responses
impl From<GetBatchResponse> for HashMap<Vec<u8>, Vec<u8>> {
    fn from(response: GetBatchResponse) -> Self {
        response
            .results_dictionary
            .into_iter()
            .filter_map(|(key, get_response)| match get_response {
                GetResponse::Hit { value } => Some((key, value.into())),
                GetResponse::Miss => None,
            })
            .collect()
    }
}

// (Bytes key, String value) pairs -- filters out Miss responses
impl TryFrom<GetBatchResponse> for HashMap<Vec<u8>, String> {
    type Error = MomentoError;

    fn try_from(response: GetBatchResponse) -> Result<Self, Self::Error> {
        response
            .results_dictionary
            .into_iter()
            .filter(|(_, get_response)| match get_response {
                GetResponse::Hit { value: _ } => true,
                GetResponse::Miss => false,
            })
            .map(|(key, hit_response)| match hit_response.try_into() {
                Ok(value) => Ok((key, value)),
                Err(e) => Err(e),
            })
            .collect()
    }
}

// (String key, GetResponse) pairs -- does NOT filter out Miss responses
impl TryFrom<GetBatchResponse> for HashMap<String, GetResponse> {
    type Error = MomentoError;

    fn try_from(response: GetBatchResponse) -> Result<Self, Self::Error> {
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

// (String key, Value) pairs -- filters out Miss responses
impl TryFrom<GetBatchResponse> for HashMap<String, Value> {
    type Error = MomentoError;

    fn try_from(response: GetBatchResponse) -> Result<Self, Self::Error> {
        response
            .results_dictionary
            .into_iter()
            .filter_map(|(key, get_response)| match get_response {
                GetResponse::Hit { value } => Some((key, value)),
                GetResponse::Miss => None,
            })
            .map(|(key, value)| match parse_string(key) {
                Ok(str_key) => Ok((str_key, value)),
                Err(e) => Err(e),
            })
            .collect()
    }
}

// (String key, String value) pairs -- filters out Miss responses
impl TryFrom<GetBatchResponse> for HashMap<String, String> {
    type Error = MomentoError;

    fn try_from(response: GetBatchResponse) -> Result<Self, Self::Error> {
        let values_map: HashMap<String, Value> = response.try_into()?;
        values_map
            .into_iter()
            .try_fold(HashMap::new(), |mut acc, (key, value)| {
                match value.try_into() {
                    Ok(value) => {
                        acc.insert(key, value);
                        Ok(acc)
                    }
                    Err(e) => Err(e),
                }
            })
    }
}

// (String key, Bytes value) pairs -- filters out Miss responses
impl TryFrom<GetBatchResponse> for HashMap<String, Vec<u8>> {
    type Error = MomentoError;

    fn try_from(response: GetBatchResponse) -> Result<Self, Self::Error> {
        let values_map: HashMap<String, Value> = response.try_into()?;
        Ok(values_map
            .into_iter()
            .fold(HashMap::new(), |mut acc, (key, value)| {
                acc.insert(key, value.into());
                acc
            }))
    }
}
