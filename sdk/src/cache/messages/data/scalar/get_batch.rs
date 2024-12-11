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
/// let results_map: HashMap<String, GetResponse> = cache_client.send_request(get_batch_request).await?.into();
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
        // consume self.keys once to convert all keys to bytes
        let byte_keys: Vec<Vec<u8>> = self.keys.into_bytes();

        // convert keys to strings for the HashMap keys
        let string_keys: Vec<String> = byte_keys
            .iter()
            .map(|key| parse_string(key.clone()))
            .collect::<MomentoResult<Vec<String>>>()?;

        let get_requests = byte_keys
            .into_iter()
            .map(|key| momento_protos::cache_client::GetRequest { cache_key: key })
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
        let mut responses: HashMap<String, GetResponse> = HashMap::new();
        let mut string_keys_iter = string_keys.into_iter();
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
            let key = match string_keys_iter.next() {
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
/// You can use `into()` to convert a `GetBatchResponse` into a `HashMap<String, GetResponse>`, a `HashMap<String, Value>`, or a `HashMap<String, Vec<u8>>`.
/// You can use `try_into()` to convert a `GetBatchResponse` into a `HashMap<String, String>`.
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
///
/// let result_map: HashMap<String, GetResponse> = cache_client.get_batch(&cache_name, keys.clone()).await?.into();
///
/// let result_map_values: HashMap<String, Value> = cache_client.get_batch(&cache_name, keys.clone()).await?.into();
///
/// let result_map_string_values: HashMap<String, Vec<u8>> = cache_client.get_batch(&cache_name, keys.clone()).await?.into();
///
/// let result_map_byte_values: HashMap<String, String> = cache_client.get_batch(&cache_name, keys.clone()).await?.try_into().expect("expecting strings");
/// # Ok(())
/// # })
/// # }
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GetBatchResponse {
    results_dictionary: HashMap<String, GetResponse>,
}

impl From<GetBatchResponse> for HashMap<String, GetResponse> {
    fn from(response: GetBatchResponse) -> Self {
        response.results_dictionary
    }
}

impl From<GetBatchResponse> for HashMap<String, Value> {
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

impl TryFrom<GetBatchResponse> for HashMap<String, String> {
    type Error = MomentoError;

    fn try_from(response: GetBatchResponse) -> Result<Self, Self::Error> {
        let values_map: HashMap<String, Value> = response.into();
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

impl From<GetBatchResponse> for HashMap<String, Vec<u8>> {
    fn from(response: GetBatchResponse) -> Self {
        let values_map: HashMap<String, Value> = response.into();
        values_map
            .into_iter()
            .map(|(key, value)| (key, value.raw_item))
            .collect()
    }
}
