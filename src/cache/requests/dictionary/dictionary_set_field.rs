use crate::cache::requests::MomentoRequest;
use crate::cache::CollectionTtl;
use crate::utils::prep_request_with_timeout;
use crate::IntoBytes;
use crate::{CacheClient, MomentoResult};
use momento_protos::cache_client::{
    DictionaryFieldValuePair as DictionaryFieldValuePairProto,
    DictionarySetRequest as DictionarySetFieldRequestProto,
};

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
