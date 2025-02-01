use super::{fetch::FetchResponse, Order, RankedElement};
use crate::leaderboard::MomentoRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

use std::ops::Range;

/// Represents a range of ranks used to request elements by rank.
pub struct RankRange {
    start_inclusive: u32,
    end_exclusive: u32,
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

/// A request to get ranked elements by rank.
pub struct FetchByRankRequest {
    rank_range: Option<RankRange>,
    order: Order,
}

impl FetchByRankRequest {
    /// Constructs a new `FetchByRankRequest`.
    pub fn new<T: Into<RankRange>>(rank_range: Option<T>, order: Order) -> Self {
        Self {
            rank_range: rank_range.map(|v| v.into()),
            order,
        }
    }
}

impl MomentoRequest for FetchByRankRequest {
    type Response = FetchResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.deadline(),
            momento_protos::leaderboard::GetByRankRequest {
                cache_name: cache_name.clone(),
                leaderboard: leaderboard.leaderboard_name().clone(),
                rank_range: self.rank_range.map(|v| v.into()),
                order: self.order as i32,
            },
        )?;

        let response = leaderboard
            .next_data_client()
            .get_by_rank(request)
            .await?
            .into_inner();

        Ok(Self::Response::new(
            response
                .elements
                .iter()
                .map(|v| RankedElement {
                    id: v.id,
                    rank: v.rank,
                    score: v.score,
                })
                .collect(),
        ))
    }
}
