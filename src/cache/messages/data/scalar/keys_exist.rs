use std::collections::HashMap;

use crate::cache::MomentoRequest;
use crate::utils::parse_string;
use crate::utils::prep_request_with_timeout;
use crate::IntoBytesIterable;
use crate::{CacheClient, MomentoResult};

/// Request to check if the provided keys exist in the cache.
/// Returns an object that is accessible as a list or map of booleans indicating whether each given key was found in the cache.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `keys` - list of keys to check for existence
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
///
/// You can receive the results as a `HashMap<String, bool>`:
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use momento::cache::{KeysExistResponse, KeysExistRequest};
/// use std::collections::HashMap;
///
/// let request = KeysExistRequest::new(
///     cache_name,
///     vec!["key1", "key2", "key3"]
/// );
///
/// let result_map: HashMap<String, bool> = cache_client.send_request(request).await?.into();
/// println!("Expecting all keys to exist: {:#?}", result_map);
/// # Ok(())
/// # })
/// # }
/// ```
///
/// Or you can receive the results as a `Vec<bool>`:
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use momento::cache::{KeysExistResponse, KeysExistRequest};
/// use std::collections::HashMap;
///
/// let request = KeysExistRequest::new(
///     cache_name,
///     vec!["key1", "key2", "key3"]
/// );
///
/// let result_list: Vec<bool> = cache_client.send_request(request).await?.into();
/// println!("Expecting all keys to exist: {:#?}", result_list);
/// # Ok(())
/// # })
/// # }
/// ```
pub struct KeysExistRequest<K: IntoBytesIterable> {
    cache_name: String,
    keys: K,
}

impl<K: IntoBytesIterable> KeysExistRequest<K> {
    /// Constructs a new KeysExistRequest.
    pub fn new(cache_name: impl Into<String>, keys: K) -> Self {
        Self {
            cache_name: cache_name.into(),
            keys,
        }
    }
}

impl<K: IntoBytesIterable> MomentoRequest for KeysExistRequest<K> {
    type Response = KeysExistResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<KeysExistResponse> {
        // consume self.keys once to convert all keys to bytes
        let byte_keys: Vec<Vec<u8>> = self.keys.into_bytes();

        // convert keys to strings for the response exists_dictionary because HashMap<IntoBytes, bool> is not allowed
        let string_keys: Vec<String> = byte_keys
            .iter()
            .map(|key| parse_string(key.clone()))
            .collect::<MomentoResult<Vec<String>>>()?;

        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::KeysExistRequest {
                cache_keys: byte_keys,
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .keys_exist(request)
            .await?
            .into_inner();

        Ok(KeysExistResponse {
            exists: response.exists.clone(),
            exists_dictionary: string_keys
                .into_iter()
                .zip(response.exists.clone())
                .collect(),
        })
    }
}

/// Response for a keys exist operation.
///
/// You can use `into()` to convert a `KeysExist` response into a `Vec<bool>` or a `HashMap<String, bool>`.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use momento::cache::KeysExistResponse;
/// use std::collections::HashMap;
///
/// let result_list: Vec<bool> = cache_client.keys_exist(&cache_name, vec!["key1", "key2", "key3"]).await?.into();
///
/// let result_map: HashMap<String, bool> = cache_client.keys_exist(&cache_name, vec!["key1", "key2", "key3"]).await?.into();
/// # Ok(())
/// # })
/// # }
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct KeysExistResponse {
    exists: Vec<bool>,
    exists_dictionary: HashMap<String, bool>,
}

impl From<KeysExistResponse> for Vec<bool> {
    fn from(response: KeysExistResponse) -> Self {
        response.exists
    }
}

impl From<KeysExistResponse> for HashMap<String, bool> {
    fn from(response: KeysExistResponse) -> Self {
        response.exists_dictionary
    }
}
