use std::time::Duration;

use crate::{
    requests::cache::MomentoRequest, simple_cache_client::prep_request_with_timeout, CacheClient,
    IntoBytes, MomentoResult,
};

/// Adds an integer quantity to a field value.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `field` - the field to increment
/// * `amount` - the quantity to add to the value. May be positive, negative, or zero. Defaults to 1.
///
/// # Optional Arguments
///
/// * `ttl` - The time-to-live for the item. If not provided, the client's default time-to-live is used.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use momento::requests::cache::scalar::increment::Increment;
/// use momento::requests::cache::scalar::increment::IncrementRequest;
/// use std::time::Duration;
/// use momento::requests::MomentoErrorCode;
///
/// let increment_request = IncrementRequest::new(
///     &cache_name,
///     "key",
///     1
/// ).ttl(Duration::from_secs(60));
///
/// match cache_client.send_request(increment_request).await {
///     Ok(r) => println!("Incremented value: {}", r.value),
///     Err(e) => if let MomentoErrorCode::NotFoundError = e.error_code {
///         println!("Cache not found: {}", &cache_name);
///     } else {
///         eprintln!("Error incrementing value in cache {}: {}", &cache_name, e);
///     }
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct IncrementRequest<K: IntoBytes> {
    cache_name: String,
    field: K,
    amount: i64,
    ttl: Option<Duration>,
}

impl<K: IntoBytes> IncrementRequest<K> {
    pub fn new(cache_name: impl Into<String>, field: K, amount: i64) -> Self {
        let ttl = None;
        Self {
            cache_name: cache_name.into(),
            field,
            amount,
            ttl,
        }
    }

    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

impl<K: IntoBytes> MomentoRequest for IncrementRequest<K> {
    type Response = Increment;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<Increment> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::IncrementRequest {
                cache_key: self.field.into_bytes(),
                amount: self.amount,
                ttl_milliseconds: cache_client.expand_ttl_ms(self.ttl)?,
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .increment(request)
            .await?
            .into_inner();
        Ok(Increment {
            value: response.value,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Increment {
    pub value: i64,
}

impl Increment {
    pub fn value(self) -> i64 {
        self.value
    }
}
