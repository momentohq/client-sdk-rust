use crate::requests::cache::MomentoRequest;
use crate::simple_cache_client::prep_request_with_timeout;
use crate::{CacheClient, IntoBytes, MomentoResult};

/// Request to check if a key exists in a cache.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `key` - key to check for existence
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use momento::requests::cache::scalar::key_exists::KeyExists;
/// use momento::requests::cache::scalar::key_exists::KeyExistsRequest;
///
/// let request = KeyExistsRequest::new(
///     cache_name,
///     "key"
/// );
///
/// let result = cache_client.send_request(request).await?;
/// if result.exists {
///     println!("Key exists!");
/// } else {
///     println!("Key does not exist!");
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct KeyExistsRequest<K: IntoBytes> {
    cache_name: String,
    key: K,
}

impl<K: IntoBytes> KeyExistsRequest<K> {
    pub fn new(cache_name: impl Into<String>, key: K) -> Self {
        Self {
            cache_name: cache_name.into(),
            key,
        }
    }
}

impl<K: IntoBytes> MomentoRequest for KeyExistsRequest<K> {
    type Response = KeyExists;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<KeyExists> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::KeysExistRequest {
                cache_keys: vec![self.key.into_bytes()],
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .keys_exist(request)
            .await?
            .into_inner();

        match response.exists.first() {
            Some(exists) => Ok(KeyExists { exists: *exists }),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct KeyExists {
    pub exists: bool,
}

impl KeyExists {
    pub fn exists(self) -> bool {
        self.exists
    }
}
