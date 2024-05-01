use std::collections::HashMap;
use std::marker::PhantomData;

use crate::cache::requests::MomentoRequest;
use crate::cache::CollectionTtl;
use crate::utils::prep_request_with_timeout;
use crate::IntoBytes;
use crate::{CacheClient, MomentoResult};
use momento_protos::cache_client::{
    DictionaryFieldValuePair as DictionaryFieldValuePairProto,
    DictionarySetRequest as DictionarySetFieldRequestProto,
};

pub trait IntoDictionaryFieldValuePairs<F: IntoBytes, V: IntoBytes>: Send {
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
    type Response = DictionarySetFields;

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

        Ok(DictionarySetFields {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DictionarySetFields {}
