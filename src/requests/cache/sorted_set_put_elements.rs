use momento_protos::cache_client::{SortedSetElement, SortedSetPutRequest};

use crate::requests::cache::MomentoRequest;
use crate::simple_cache_client::prep_request_with_timeout;
use crate::{CacheClient, CollectionTtl, IntoBytes, MomentoResult};

pub struct SortedSetPutElementsRequest<S: IntoBytes, E: IntoBytes> {
    cache_name: String,
    sorted_set_name: S,
    elements: Vec<(E, f64)>,
    collection_ttl: Option<CollectionTtl>,
}

impl<S: IntoBytes, E: IntoBytes> SortedSetPutElementsRequest<S, E> {
    pub fn new(cache_name: String, sorted_set_name: S, elements: Vec<(E, f64)>) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name,
            sorted_set_name,
            elements,
            collection_ttl: Some(collection_ttl),
        }
    }

    pub fn with_ttl(mut self, collection_ttl: CollectionTtl) -> Self {
        self.collection_ttl = Some(collection_ttl);
        self
    }
}

impl<S: IntoBytes, E: IntoBytes> MomentoRequest for SortedSetPutElementsRequest<S, E> {
    type Response = SortedSetPutElements;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetPutElements> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let elements = self
            .elements
            .into_iter()
            .map(|e| SortedSetElement {
                value: e.0.into_bytes(),
                score: e.1,
            })
            .collect();
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            SortedSetPutRequest {
                set_name,
                elements,
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
            },
        )?;

        let _ = cache_client
            .data_client
            .clone()
            .sorted_set_put(request)
            .await?;
        Ok(SortedSetPutElements {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SortedSetPutElements {}
