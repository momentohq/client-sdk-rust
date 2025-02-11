use super::{fetch::FetchResponse, Order};
use crate::leaderboard::LeaderboardRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

use std::ops::{Range, RangeFrom, RangeInclusive, RangeTo};

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

impl From<RangeFrom<u32>> for RankRange {
    fn from(val: RangeFrom<u32>) -> Self {
        RankRange {
            start_inclusive: val.start,
            end_exclusive: u32::MAX,
        }
    }
}

impl From<RangeTo<u32>> for RankRange {
    fn from(val: RangeTo<u32>) -> Self {
        RankRange {
            start_inclusive: 0,
            end_exclusive: val.end,
        }
    }
}

impl From<RangeInclusive<u32>> for RankRange {
    /// Converts a range inclusive into a range exclusive.
    ///
    /// Clamps the end value to u32::MAX if it is u32::MAX.
    fn from(val: RangeInclusive<u32>) -> Self {
        let start_inclusive = *val.start();
        let end_exclusive = if *val.end() < u32::MAX {
            *val.end() + 1
        } else {
            *val.end()
        };
        RankRange {
            start_inclusive,
            end_exclusive,
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

impl LeaderboardRequest for FetchByRankRequest {
    type Response = FetchResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.client_timeout(),
            momento_protos::leaderboard::GetByRankRequest {
                leaderboard: leaderboard.leaderboard_name().to_string(),
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
            response.elements.iter().map(|v| v.into()).collect(),
        ))
    }
}
