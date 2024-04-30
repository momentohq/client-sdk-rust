use crate::cache::requests::MomentoRequest;
use crate::cache::CollectionTtl;
use crate::utils::prep_request_with_timeout;
use crate::IntoBytes;
use crate::{CacheClient, MomentoResult};
use momento_protos::cache_client::{
    DictionaryFieldValuePair as DictionaryFieldValuePairProto,
    DictionarySetRequest as DictionarySetFieldRequestProto,
};

/// Request to set a field in a dictionary.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the dictionary.
/// * `dictionary_name` - The name of the dictionary to set.
/// * `field` - The field to set.
/// * `value` - The value to set.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{CollectionTtl, DictionarySetField, DictionarySetFieldRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let dictionary_name = "dictionary";
///
/// let set_field_request = DictionarySetFieldRequest::new(
///    cache_name.to_string(),
///    dictionary_name,
///    "field",
///    "value"
/// ).ttl(CollectionTtl::default());
///
/// match cache_client.send_request(set_field_request).await {
///    Ok(_) => println!("Field set in dictionary"),
///    Err(e) => eprintln!("Error setting field in dictionary: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
pub struct DictionarySetFieldRequest<D, F, V>
where
    D: IntoBytes,
    F: IntoBytes,
    V: IntoBytes,
{
    cache_name: String,
    dictionary_name: D,
    field: F,
    value: V,
    collection_ttl: Option<CollectionTtl>,
}

impl<D, F, V> DictionarySetFieldRequest<D, F, V>
where
    D: IntoBytes,
    F: IntoBytes,
    V: IntoBytes,
{
    pub fn new(cache_name: impl Into<String>, dictionary_name: D, field: F, value: V) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name: cache_name.into(),
            dictionary_name,
            field,
            value,
            collection_ttl: Some(collection_ttl),
        }
    }

    pub fn ttl(mut self, collection_ttl: CollectionTtl) -> Self {
        self.collection_ttl = Some(collection_ttl);
        self
    }
}

impl<D, F, V> MomentoRequest for DictionarySetFieldRequest<D, F, V>
where
    D: IntoBytes,
    F: IntoBytes,
    V: IntoBytes,
{
    type Response = DictionarySetField;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<Self::Response> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            DictionarySetFieldRequestProto {
                dictionary_name: self.dictionary_name.into_bytes(),
                items: vec![DictionaryFieldValuePairProto {
                    field: self.field.into_bytes(),
                    value: self.value.into_bytes(),
                }],
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
            },
        )?;

        cache_client
            .data_client
            .clone()
            .dictionary_set(request)
            .await?;

        Ok(DictionarySetField {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DictionarySetField {}
