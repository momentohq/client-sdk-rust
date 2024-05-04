use crate::{
    cache::{CollectionTtl, MomentoRequest},
    utils::prep_request_with_timeout,
    CacheClient, IntoBytes, MomentoResult,
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
/// * `collection_ttl`: The time-to-live for the collection. If not provided, the client's default time-to-live is used.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use momento::cache::{CollectionTtl, DictionaryIncrement, DictionaryIncrementRequest};
/// use momento::MomentoErrorCode;
///
/// let dictionary_name = "dictionary";
/// let field = "field";
/// let increment_request = DictionaryIncrementRequest::new(
///   &cache_name,
///   dictionary_name,
///   field,
///   1
/// ).ttl(CollectionTtl::default());
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
pub struct DictionaryIncrementRequest<D: IntoBytes, F: IntoBytes> {
    cache_name: String,
    dictionary_name: D,
    field: F,
    amount: i64,
    collection_ttl: Option<CollectionTtl>,
}

impl<D: IntoBytes, F: IntoBytes> DictionaryIncrementRequest<D, F> {
    pub fn new(cache_name: impl Into<String>, dictionary_name: D, field: F, amount: i64) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name: cache_name.into(),
            dictionary_name,
            field,
            amount,
            collection_ttl: Some(collection_ttl),
        }
    }

    pub fn ttl(mut self, collection_ttl: CollectionTtl) -> Self {
        self.collection_ttl = Some(collection_ttl);
        self
    }
}

impl<D: IntoBytes, F: IntoBytes> MomentoRequest for DictionaryIncrementRequest<D, F> {
    type Response = DictionaryIncrement;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<DictionaryIncrement> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::DictionaryIncrementRequest {
                dictionary_name: self.dictionary_name.into_bytes(),
                field: self.field.into_bytes(),
                amount: self.amount,
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .dictionary_increment(request)
            .await?
            .into_inner();
        Ok(DictionaryIncrement {
            value: response.value,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DictionaryIncrement {
    pub value: i64,
}

impl DictionaryIncrement {
    pub fn value(self) -> i64 {
        self.value
    }
}
