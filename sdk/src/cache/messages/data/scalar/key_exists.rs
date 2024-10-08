use crate::cache::MomentoRequest;
use crate::utils::prep_request_with_timeout;
use crate::{CacheClient, IntoBytes, MomentoError, MomentoResult};

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
/// use momento::cache::{KeyExistsResponse, KeyExistsRequest};
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
    /// Constructs a new KeyExistsRequest.
    pub fn new(cache_name: impl Into<String>, key: K) -> Self {
        Self {
            cache_name: cache_name.into(),
            key,
        }
    }
}

impl<K: IntoBytes> MomentoRequest for KeyExistsRequest<K> {
    type Response = KeyExistsResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<KeyExistsResponse> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::KeysExistRequest {
                cache_keys: vec![self.key.into_bytes()],
            },
        )?;

        let response = cache_client
            .next_data_client()
            .keys_exist(request)
            .await?
            .into_inner();

        match response.exists.first() {
            Some(exists) => Ok(KeyExistsResponse { exists: *exists }),
            _ => Err(MomentoError::unknown_error(
                "KeyExists",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response for a key exists operation.
#[derive(Debug, PartialEq, Eq)]
pub struct KeyExistsResponse {
    /// True if the key exists in the cache.
    pub exists: bool,
}

impl KeyExistsResponse {
    /// Returns true if the key exists in the cache.
    pub fn exists(self) -> bool {
        self.exists
    }
}
