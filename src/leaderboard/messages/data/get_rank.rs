use super::{IntoIds, Order, RankedElement};
use crate::leaderboard::MomentoRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

/// A request to get ranked elements by providing a list of element IDs.
pub struct GetRankRequest {
    ids: Vec<u32>,
    order: Order,
}

impl GetRankRequest {
    /// Constructs a new `GetRankRequest`.
    pub fn new(ids: impl Into<Vec<u32>>, order: Order) -> Self {
        Self {
            ids: ids.into(),
            order,
        }
    }
}

impl MomentoRequest for GetRankRequest {
    type Response = GetRankResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let ids = self.ids.into_ids();
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.deadline(),
            momento_protos::leaderboard::GetRankRequest {
                cache_name: cache_name.clone(),
                leaderboard: leaderboard.leaderboard_name().clone(),
                ids,
                order: self.order as i32,
            },
        )?;

        let response = leaderboard
            .next_data_client()
            .get_rank(request)
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

/// The response type for a successful `GetRankRequest`
pub struct GetRankResponse {
    elements: Vec<RankedElement>,
}

impl GetRankResponse {
    /// Returns the ranked elements in the response.
    pub fn elements(&self) -> &[RankedElement] {
        &self.elements
    }
}
