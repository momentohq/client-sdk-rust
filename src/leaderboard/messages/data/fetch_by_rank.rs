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
    rank_range: RankRange,
    order: Order,
}

impl FetchByRankRequest {
    /// Constructs a new `FetchByRankRequest`.
    ///
    /// Defaults to ascending order, meaning rank 0 is the element
    /// with the lowest score.
    pub fn new<T: Into<RankRange>>(rank_range: T) -> Self {
        Self {
            rank_range: rank_range.into(),
            order: Order::Ascending,
        }
    }

    /// Sets the order of the elements to be fetched.
    ///
    /// Otherwise the default is ascending order.
    pub fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }
}

impl MomentoRequest for FetchByRankRequest {
    type Response = FetchResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.client_timeout(),
            momento_protos::leaderboard::GetByRankRequest {
                cache_name: cache_name.clone(),
                leaderboard: leaderboard.leaderboard_name().clone(),
                rank_range: Some(self.rank_range.into()),
                order: self.order.into_proto() as i32,
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
