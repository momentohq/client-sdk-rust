use std::collections::HashMap;
use std::marker::PhantomData;

use crate::cache::messages::MomentoRequest;
use crate::cache::CollectionTtl;
use crate::utils::prep_request_with_timeout;
use crate::IntoBytes;
use crate::{CacheClient, MomentoResult};
use momento_protos::cache_client::{
    DictionaryFieldValuePair as DictionaryFieldValuePairProto,
    DictionarySetRequest as DictionarySetFieldRequestProto,
};

/// This trait defines an interface for converting a type into a vector of [DictionaryFieldValuePair].
pub trait IntoDictionaryFieldValuePairs<F: IntoBytes, V: IntoBytes>: Send {
    /// Converts the type into a vector of [DictionaryFieldValuePair].
    fn into_dictionary_field_value_pairs(self) -> Vec<DictionaryFieldValuePair<F, V>>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct DictionaryFieldValuePair<F: IntoBytes, V: IntoBytes> {
    pub field: F,
    pub value: V,
}

impl<F: IntoBytes, V: IntoBytes> IntoDictionaryFieldValuePairs<F, V>
    for Vec<DictionaryFieldValuePair<F, V>>
{
    fn into_dictionary_field_value_pairs(self) -> Vec<DictionaryFieldValuePair<F, V>> {
        self
    }
}

impl<F: IntoBytes, V: IntoBytes> IntoDictionaryFieldValuePairs<F, V> for Vec<(F, V)> {
    fn into_dictionary_field_value_pairs(self) -> Vec<DictionaryFieldValuePair<F, V>> {
        self.into_iter()
            .map(|(field, value)| DictionaryFieldValuePair { field, value })
            .collect()
    }
}

impl<F: IntoBytes, V: IntoBytes> IntoDictionaryFieldValuePairs<F, V> for HashMap<F, V> {
    fn into_dictionary_field_value_pairs(self) -> Vec<DictionaryFieldValuePair<F, V>> {
        self.into_iter()
            .map(|(field, value)| DictionaryFieldValuePair { field, value })
            .collect()
    }
}

/// Request to set multiple fields in a dictionary. If the dictionary does not exist, it will be
/// created. If the dictionary already exists, the fields will be updated.
///
/// # Arguments
///
/// - `cache_name`: The name of the cache where the dictionary is stored.
/// - `dictionary_name`: The name of the dictionary to set fields in.
/// - `elements`: The fields and values to set in the dictionary.
///
/// # Optional Arguments
///
/// - `collection_ttl`: The time-to-live for the collection. If not provided, the client's default time-to-live is used.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{CollectionTtl, DictionarySetFieldsResponse, DictionarySetFieldsRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let dictionary_name = "dictionary";
///
/// let set_fields_request = DictionarySetFieldsRequest::new(
///     cache_name,
///     dictionary_name,
///     vec![("field1", "value1"), ("field2", "value2")]
/// ).ttl(CollectionTtl::default());
///
/// match cache_client.send_request(set_fields_request).await {
///     Ok(_) => println!("Fields set successfully"),
///     Err(e) => eprintln!("Error setting fields: {:?}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct DictionarySetFieldsRequest<D, F, V, E>
where
    D: IntoBytes,
    F: IntoBytes,
    V: IntoBytes,
    E: IntoDictionaryFieldValuePairs<F, V>,
{
    cache_name: String,
    dictionary_name: D,
    elements: E,
    collection_ttl: Option<CollectionTtl>,
    // F and V are only used for the [IntoDictionaryFieldValuePairs] generic type parameter.
    // See the [PhantomData] documentation for more information.
    _field_marker: PhantomData<F>,
    _value_marker: PhantomData<V>,
}

impl<D, F, V, E> DictionarySetFieldsRequest<D, F, V, E>
where
    D: IntoBytes,
    F: IntoBytes,
    V: IntoBytes,
    E: IntoDictionaryFieldValuePairs<F, V>,
{
    pub fn new(cache_name: impl Into<String>, dictionary_name: D, elements: E) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name: cache_name.into(),
            dictionary_name,
            elements,
            collection_ttl: Some(collection_ttl),
            _field_marker: PhantomData,
            _value_marker: PhantomData,
        }
    }

    pub fn ttl(mut self, collection_ttl: CollectionTtl) -> Self {
        self.collection_ttl = Some(collection_ttl);
        self
    }
}

impl<D, F, V, E> MomentoRequest for DictionarySetFieldsRequest<D, F, V, E>
where
    D: IntoBytes,
    F: IntoBytes,
    V: IntoBytes,
    E: IntoDictionaryFieldValuePairs<F, V>,
{
    type Response = DictionarySetFieldsResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<Self::Response> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            DictionarySetFieldRequestProto {
                dictionary_name: self.dictionary_name.into_bytes(),
                items: self
                    .elements
                    .into_dictionary_field_value_pairs()
                    .into_iter()
                    .map(|pair| DictionaryFieldValuePairProto {
                        field: pair.field.into_bytes(),
                        value: pair.value.into_bytes(),
                    })
                    .collect(),
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
            },
        )?;

        cache_client
            .data_client
            .clone()
            .dictionary_set(request)
            .await?;

        Ok(DictionarySetFieldsResponse {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DictionarySetFieldsResponse {}
