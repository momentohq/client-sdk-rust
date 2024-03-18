use crate::cache_client::CacheClient;
use crate::requests::cache::MomentoRequest;
use crate::simple_cache_client::prep_request_with_timeout;
use crate::{IntoBytes, MomentoResult};
use std::time::Duration;

/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_client();
/// use momento::requests::cache::basic::set::Set;
///
/// let set_response = cache_client.set(&cache_name, "key", "value").await?;
/// assert_eq!(set_response, Set {});
/// # Ok(())
/// # })
/// #
/// }
/// ```
pub struct SetRequest<K: IntoBytes, V: IntoBytes> {
    cache_name: String,
    key: K,
    value: V,
    ttl: Option<Duration>,
}

impl<K: IntoBytes, V: IntoBytes> SetRequest<K, V> {
    pub fn new(cache_name: String, key: K, value: V) -> Self {
        let ttl = None;
        Self {
            cache_name,
            key,
            value,
            ttl,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

impl<K: IntoBytes, V: IntoBytes> MomentoRequest for SetRequest<K, V> {
    type Response = Set;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<Set> {
        // let ttl = self.ttl.unwrap_or_default();
        // let elements = self.elements.into_iter().map(|e| e.into_bytes()).collect();
        // let set_name = self.set_name.into_bytes();
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::SetRequest {
                cache_key: self.key.into_bytes(),
                cache_body: self.value.into_bytes(),
                ttl_milliseconds: cache_client.expand_ttl_ms(self.ttl)?,
            },
        )?;

        let _ = cache_client.data_client.clone().set(request).await?;
        Ok(Set {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Set {}
