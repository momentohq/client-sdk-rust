use super::{Order, RankedElement};
use crate::leaderboard::MomentoRequest;
use crate::utils::prep_request_with_timeout;
use crate::{LeaderboardClient, MomentoResult};

use std::ops::Range;

pub struct RankRange {
    pub start_inclusive: u32,
    pub end_exclusive: u32,
}

impl From<Range<u32>> for RankRange {
    fn from(val: std::ops::Range<u32>) -> Self {
        RankRange {
            start_inclusive: val.start,
            end_exclusive: val.end,
        }
    }
}

impl From<RankRange> for momento_protos::leaderboard::RankRange {
    fn from(val: RankRange) -> Self {
        momento_protos::leaderboard::RankRange {
            start_inclusive: val.start_inclusive,
            end_exclusive: val.end_exclusive,
        }
    }
}

pub struct GetByRankRequest {
    cache_name: String,
    leaderboard: String,
    rank_range: Option<RankRange>,
    order: Order,
}

pub struct GetByRankResponse {
    elements: Vec<RankedElement>,
}

impl GetByRankRequest {
    /// Constructs a new SortedSetPutElementsRequest.
    pub fn new<T: Into<RankRange>>(
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
        rank_range: Option<T>,
        order: Order,
    ) -> Self {
        Self {
            cache_name: cache_name.into(),
            leaderboard: leaderboard.into(),
            rank_range: rank_range.map(|v| v.into()),
            order,
        }
    }
}

impl GetByRankResponse {
    pub fn elements(&self) -> &[RankedElement] {
        &self.elements
    }
}

impl MomentoRequest for GetByRankRequest {
    type Response = GetByRankResponse;

    async fn send(self, leaderboard_client: &LeaderboardClient) -> MomentoResult<Self::Response> {
        let cache_name = self.cache_name.clone();
        let request = prep_request_with_timeout(
            &self.cache_name,
            leaderboard_client.deadline_millis(),
            momento_protos::leaderboard::GetByRankRequest {
                cache_name,
                leaderboard: self.leaderboard,
                rank_range: self.rank_range.map(|v| v.into()),
                order: self.order as i32,
            },
        )?;

        let response = leaderboard_client
            .next_data_client()
            .get_by_rank(request)
            .await?
            .into_inner();

        Ok(Self::Response {
            elements: response
                .elements
                .iter()
                .map(|v| RankedElement {
                    id: v.id,
                    rank: v.rank,
                    score: v.score,
                })
                .collect(),
        })
    }
}
