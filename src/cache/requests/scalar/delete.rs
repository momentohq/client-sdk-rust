use crate::{
    requests::cache::MomentoRequest, simple_cache_client::prep_request_with_timeout, CacheClient,
    IntoBytes, MomentoResult,
};

/// Deletes an item in a Momento Cache
///
/// # Arguments
///
/// * `cache_name` - name of cache
/// * `key` - key of the item to delete
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use momento::requests::cache::basic::delete::Delete;
/// use momento::requests::cache::basic::delete::DeleteRequest;
/// use momento::requests::MomentoErrorCode;
///
/// let delete_request = DeleteRequest::new(
///     &cache_name,
///     "key"
/// );
///
/// match cache_client.send_request(delete_request).await {
///     Ok(_) => println!("Delete successful"),
///     Err(e) => if let MomentoErrorCode::NotFoundError = e.error_code {
///         println!("Cache not found: {}", &cache_name);
///     } else {
///         eprintln!("Error deleting value in cache {}: {}", &cache_name, e);
///     }
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct DeleteRequest<K: IntoBytes> {
    cache_name: String,
    key: K,
}

impl<K: IntoBytes> DeleteRequest<K> {
    pub fn new(cache_name: impl Into<String>, key: K) -> Self {
        Self {
            cache_name: cache_name.into(),
            key,
        }
    }
}

impl<K: IntoBytes> MomentoRequest for DeleteRequest<K> {
    type Response = Delete;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<Delete> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::DeleteRequest {
                cache_key: self.key.into_bytes(),
            },
        )?;

        let _ = cache_client.data_client.clone().delete(request).await?;
        Ok(Delete {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Delete {}
