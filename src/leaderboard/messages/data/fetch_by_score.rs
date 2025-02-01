use super::{fetch::FetchResponse, Order, RankedElement};
use crate::leaderboard::MomentoRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

use momento_protos::leaderboard::score_range::{Max, Min};

use std::ops::Range;

/// Represents a range of scores used to request elements by score.
pub struct ScoreRange {
    min: Option<f64>,
    max: Option<f64>,
}

impl From<Range<f64>> for ScoreRange {
    fn from(val: std::ops::Range<f64>) -> Self {
        let min = if val.start.is_finite() {
            Some(val.start)
        } else {
            None
        };

        let max = if val.end.is_finite() {
            Some(val.start)
        } else {
            None
        };

        ScoreRange { min, max }
    }
}

impl From<ScoreRange> for momento_protos::leaderboard::ScoreRange {
    fn from(val: ScoreRange) -> Self {
        let min = val.min.map(Min::MinInclusive);
        let max = val.max.map(Max::MaxExclusive);

        momento_protos::leaderboard::ScoreRange { min, max }
    }
}

/// A request to retrieve ranked elements by score.
pub struct FetchByScoreRequest {
    score_range: Option<ScoreRange>,
    offset: u32,
    limit_elements: u32,
    order: Order,
}

impl FetchByScoreRequest {
    /// Constructs a new `FetchByScoreRequest`.
    pub fn new(
        score_range: impl Into<Option<ScoreRange>>,
        offset: u32,
        limit_elements: u32,
        order: Order,
    ) -> Self {
        Self {
            score_range: score_range.into(),
            offset,
            limit_elements,
            order,
        }
    }
}

impl MomentoRequest for FetchByScoreRequest {
    type Response = FetchResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.deadline(),
            momento_protos::leaderboard::GetByScoreRequest {
                cache_name: cache_name.clone(),
                leaderboard: leaderboard.leaderboard_name().clone(),
                score_range: self.score_range.map(|v| v.into()),
                offset: self.offset,
                limit_elements: self.limit_elements,
                order: self.order as i32,
            },
        )?;

        let response = leaderboard
            .next_data_client()
            .get_by_score(request)
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
