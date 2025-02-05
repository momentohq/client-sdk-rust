use super::{IntoIds, Order, RankedElement};
use crate::leaderboard::LeaderboardRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

/// A request to get ranked elements by providing a list of element IDs.
pub struct GetRankRequest {
    ids: Vec<u32>,
    order: Order,
}

impl GetRankRequest {
    /// Constructs a new `GetRankRequest`.
    ///
    /// Defaults to ascending order, meaning that rank 0
    /// is the element with the lowest score.
    pub fn new(ids: impl IntoIds) -> Self {
        Self {
            ids: ids.into_ids(),
            order: Order::Ascending,
        }
    }

    /// Sets the order ranking.
    ///
    /// Defaults to ascending order.
    pub fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }
}

impl LeaderboardRequest for GetRankRequest {
    type Response = GetRankResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let ids = self.ids.into_ids();
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.client_timeout(),
            momento_protos::leaderboard::GetRankRequest {
                cache_name: cache_name.to_string(),
                leaderboard: leaderboard.leaderboard_name().to_string(),
                ids,
                order: self.order.into_proto() as i32,
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

    /// Consumes the response and returns the ranked elements.
    pub fn into_elements(self) -> Vec<RankedElement> {
        self.elements
    }
}
