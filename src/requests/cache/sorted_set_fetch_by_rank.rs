use momento_protos::cache_client::sorted_set_fetch_request::{by_index, ByIndex, Range};
use momento_protos::cache_client::{SortedSetFetchRequest, Unbounded};

use crate::requests::cache::sorted_set_fetch_by_rank::SortOrder::Ascending;
use crate::requests::cache::MomentoRequest;
use crate::response::cache::sorted_set_fetch::SortedSetFetch;
use crate::simple_cache_client::prep_request_with_timeout;
use crate::{CacheClient, IntoBytes, MomentoResult};

#[repr(i32)]
pub enum SortOrder {
    Ascending = 0,
    Descending = 1,
}

pub struct SortedSetFetchByRankRequest<S: IntoBytes> {
    cache_name: String,
    sorted_set_name: S,
    start_rank: Option<i32>,
    end_rank: Option<i32>,
    order: SortOrder,
}

impl<S: IntoBytes> SortedSetFetchByRankRequest<S> {
    pub fn new(cache_name: String, sorted_set_name: S) -> Self {
        Self {
            cache_name,
            sorted_set_name,
            start_rank: None,
            end_rank: None,
            order: Ascending,
        }
    }

    pub fn with_start_rank(mut self, start_rank: i32) -> Self {
        self.start_rank = Some(start_rank);
        self
    }

    pub fn with_end_rank(mut self, end_rank: i32) -> Self {
        self.end_rank = Some(end_rank);
        self
    }

    pub fn with_order(mut self, order: SortOrder) -> Self {
        self.order = order;
        self
    }
}

impl<S: IntoBytes> MomentoRequest for SortedSetFetchByRankRequest<S> {
    type Response = SortedSetFetch;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetFetch> {
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;

        let by_index = ByIndex {
            start: if self.start_rank.is_some() {
                Some(by_index::Start::InclusiveStartIndex(
                    self.start_rank.unwrap(),
                ))
            } else {
                Some(by_index::Start::UnboundedStart(Unbounded {}))
            },
            end: if self.end_rank.is_some() {
                Some(by_index::End::ExclusiveEndIndex(self.end_rank.unwrap()))
            } else {
                Some(by_index::End::UnboundedEnd(Unbounded {}))
            },
        };

        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            SortedSetFetchRequest {
                set_name,
                order: self.order as i32,
                with_scores: true,
                range: Some(Range::ByIndex(by_index)),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .sorted_set_fetch(request)
            .await?
            .into_inner();

        SortedSetFetch::from_fetch_response(response)
    }
}
