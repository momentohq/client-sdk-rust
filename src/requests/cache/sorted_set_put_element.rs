use momento_protos::cache_client::{SortedSetElement, SortedSetPutRequest};

use crate::requests::cache::MomentoRequest;
use crate::simple_cache_client::prep_request_with_timeout;
use crate::{CacheClient, CollectionTtl, IntoBytes, MomentoResult};

pub struct SortedSetPutElementRequest<S: IntoBytes, E: IntoBytes> {
    cache_name: String,
    sorted_set_name: S,
    value: E,
    score: f64,
    collection_ttl: Option<CollectionTtl>,
}

impl<S: IntoBytes, E: IntoBytes> SortedSetPutElementRequest<S, E> {
    pub fn new(cache_name: String, sorted_set_name: S, value: E, score: f64) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name,
            sorted_set_name,
            value,
            score,
            collection_ttl: Some(collection_ttl),
        }
    }

    pub fn with_ttl(mut self, collection_ttl: CollectionTtl) -> Self {
        self.collection_ttl = Some(collection_ttl);
        self
    }
}

impl<S: IntoBytes, E: IntoBytes> MomentoRequest for SortedSetPutElementRequest<S, E> {
    type Response = SortedSetPutElement;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetPutElement> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let element = SortedSetElement {
            value: self.value.into_bytes(),
            score: self.score,
        };
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            SortedSetPutRequest {
                set_name,
                elements: vec![element],
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
            },
        )?;

        let _ = cache_client
            .data_client
            .clone()
            .sorted_set_put(request)
            .await?;
        Ok(SortedSetPutElement {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SortedSetPutElement {}
