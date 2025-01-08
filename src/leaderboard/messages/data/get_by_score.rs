use super::{Order, RankedElement};
use crate::leaderboard::MomentoRequest;
use crate::utils::prep_request_with_timeout;
use crate::{LeaderboardClient, MomentoResult};

use momento_protos::leaderboard::score_range::{Max, Min};

use std::ops::Range;

pub struct ScoreRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
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
        let max = val.min.map(Max::MaxExclusive);

        momento_protos::leaderboard::ScoreRange { min, max }
    }
}

pub struct GetByScoreRequest {
    cache_name: String,
    leaderboard: String,
    score_range: Option<ScoreRange>,
    offset: u32,
    limit_elements: u32,
    order: Order,
}

pub struct GetByScoreResponse {
    elements: Vec<RankedElement>,
}

impl GetByScoreRequest {
    /// Constructs a new SortedSetPutElementsRequest.
    pub fn new(
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
        score_range: impl Into<Option<ScoreRange>>,
        offset: u32,
        limit_elements: u32,
        order: Order,
    ) -> Self {
        Self {
            cache_name: cache_name.into(),
            leaderboard: leaderboard.into(),
            score_range: score_range.into(),
            offset,
            limit_elements,
            order,
        }
    }
}

/// The response type for a successful `GetByScoreRequest`
impl GetByScoreResponse {
    pub fn elements(&self) -> &[RankedElement] {
        &self.elements
    }
}

impl MomentoRequest for GetByScoreRequest {
    type Response = GetByScoreResponse;

    async fn send(self, leaderboard_client: &LeaderboardClient) -> MomentoResult<Self::Response> {
        let cache_name = self.cache_name.clone();
        let request = prep_request_with_timeout(
            &self.cache_name,
            leaderboard_client.deadline_millis(),
            momento_protos::leaderboard::GetByScoreRequest {
                cache_name,
                leaderboard: self.leaderboard,
                score_range: self.score_range.map(|v| v.into()),
                offset: self.offset,
                limit_elements: self.limit_elements,
                order: self.order as i32,
            },
        )?;

        let response = leaderboard_client
            .next_data_client()
            .get_by_score(request)
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
