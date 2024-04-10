use crate::requests::cache::MomentoRequest;
use crate::simple_cache_client::prep_request_with_timeout;
use crate::{CacheClient, IntoBytes, MomentoResult};

/// Request to check if the provided keys exist in the cache.
/// Returns a list of booleans indicating whether each given key was found in the cache.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `keys` - list of keys to look up
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use momento::requests::cache::scalar::keys_exist::KeysExist;
/// use momento::requests::cache::scalar::keys_exist::KeysExistRequest;
///
/// let request = KeysExistRequest::new(
///     cache_name,
///     vec!["key1", "key2", "key3"]
/// );
///
/// let result = cache_client.send_request(request).await?;
/// if !result.exists.is_empty() {
///     println!("Processing list of booleans: {:#?}", result.exists);
/// } else {
///     println!("Received empty list of booleans!");
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct KeysExistRequest<K: IntoBytes> {
    cache_name: String,
    keys: Vec<K>,
}

impl<K: IntoBytes> KeysExistRequest<K> {
    pub fn new(cache_name: impl Into<String>, keys: Vec<K>) -> Self {
        Self {
            cache_name: cache_name.into(),
            keys,
        }
    }
}

impl<K: IntoBytes> MomentoRequest for KeysExistRequest<K> {
    type Response = KeysExist;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<KeysExist> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::KeysExistRequest {
                cache_keys: self.keys.into_iter().map(|key| key.into_bytes()).collect(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .keys_exist(request)
            .await?
            .into_inner();

        Ok(KeysExist {
            exists: response.exists,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct KeysExist {
    pub exists: Vec<bool>,
}

impl KeysExist {
    pub fn exists(self) -> Vec<bool> {
        self.exists
    }
}
