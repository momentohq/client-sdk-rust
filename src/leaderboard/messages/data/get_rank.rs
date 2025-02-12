use super::fetch::FetchResponse;
use super::Order;
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
    pub fn new(ids: impl IntoIterator<Item = u32>) -> Self {
        Self {
            ids: ids.into_iter().collect(),
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
    type Response = FetchResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.client_timeout(),
            momento_protos::leaderboard::GetRankRequest {
                leaderboard: leaderboard.leaderboard_name().to_string(),
                ids: self.ids,
                order: self.order.into_proto() as i32,
            },
        )?;

        let response = leaderboard
            .next_data_client()
            .get_rank(request)
            .await?
            .into_inner();

        Ok(FetchResponse::new(
            response.elements.iter().map(|v| v.into()).collect(),
        ))
    }
}
