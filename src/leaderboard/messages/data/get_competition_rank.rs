use super::fetch::FetchResponse;
use super::Order;
use crate::leaderboard::LeaderboardRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

pub struct GetCompetitionRankRequest {
    ids: Vec<u32>,
    order: Order,
}

impl GetCompetitionRankRequest {
    pub fn new(ids: impl IntoIterator<Item = u32>, order: Option<Order>) -> Self {
        Self {
            ids: ids.into_iter().collect(),
            order: order.unwrap_or(Order::Descending),
        }
    }
}

impl LeaderboardRequest for GetCompetitionRankRequest {
    type Response = FetchResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.client_timeout(),
            momento_protos::leaderboard::GetCompetitionRankRequest {
                leaderboard: leaderboard.leaderboard_name().to_string(),
                ids: self.ids,
                order: Some(self.order.into_proto() as i32),
            },
        )?;

        let response = leaderboard
            .next_data_client()
            .get_competition_rank(request)
            .await?
            .into_inner();

        Ok(FetchResponse::new(
            response.elements.iter().map(|v| v.into()).collect(),
        ))
    }
}
