use momento_protos::cache_client::sorted_set_fetch_request::by_score::Score;
use momento_protos::cache_client::sorted_set_fetch_request::{by_score, ByScore, Range};
use momento_protos::cache_client::{SortedSetFetchRequest, Unbounded};

use crate::requests::cache::sorted_set_fetch_by_rank::SortOrder;
use crate::requests::cache::sorted_set_fetch_by_rank::SortOrder::Ascending;
use crate::requests::cache::MomentoRequest;
use crate::response::cache::sorted_set_fetch::SortedSetFetch;
use crate::simple_cache_client::prep_request_with_timeout;
use crate::{CacheClient, IntoBytes, MomentoResult};

pub struct SortedSetFetchByScoreRequest<S: IntoBytes> {
    cache_name: String,
    sorted_set_name: S,
    min_score: Option<f64>,
    max_score: Option<f64>,
    order: SortOrder,
    offset: Option<u32>,
    count: Option<i32>,
}

impl<S: IntoBytes> SortedSetFetchByScoreRequest<S> {
    pub fn new(cache_name: String, sorted_set_name: S) -> Self {
        Self {
            cache_name,
            sorted_set_name,
            min_score: None,
            max_score: None,
            order: Ascending,
            offset: None,
            count: None,
        }
    }

    pub fn with_min_score(mut self, min_score: f64) -> Self {
        self.min_score = Some(min_score);
        self
    }

    pub fn with_max_score(mut self, max_score: f64) -> Self {
        self.max_score = Some(max_score);
        self
    }

    pub fn with_order(mut self, order: SortOrder) -> Self {
        self.order = order;
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_count(mut self, count: i32) -> Self {
        self.count = Some(count);
        self
    }
}

impl<S: IntoBytes> MomentoRequest for SortedSetFetchByScoreRequest<S> {
    type Response = SortedSetFetch;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetFetch> {
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;

        let by_score = ByScore {
            min: if self.min_score.is_some() {
                Some(by_score::Min::MinScore(Score {
                    score: self.min_score.unwrap(),
                    exclusive: false,
                }))
            } else {
                Some(by_score::Min::UnboundedMin(Unbounded {}))
            },
            max: if self.max_score.is_some() {
                Some(by_score::Max::MaxScore(Score {
                    score: self.max_score.unwrap(),
                    exclusive: false,
                }))
            } else {
                Some(by_score::Max::UnboundedMax(Unbounded {}))
            },
            offset: self.offset.unwrap_or(0),
            count: self.count.unwrap_or(-1),
        };

        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            SortedSetFetchRequest {
                set_name,
                order: self.order as i32,
                with_scores: true,
                range: Some(Range::ByScore(by_score)),
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
