use super::{Order, RankedElement};
use crate::leaderboard::LeaderboardRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

pub struct GetCompetitionRankRequest {
    ids: Vec<u32>,
    order: Order,
}

impl GetCompetitionRankRequest {
    pub fn new(ids: impl IntoIterator<Item = u32>) -> Self {
        Self {
            ids: ids.into_iter().collect(),
            order: Order::Descending,
        }
    }

    pub fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }
}

impl LeaderboardRequest for GetCompetitionRankRequest {
    type Response = GetCompetitionRankResponse;

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

        Ok(Self::Response {
            elements: response.elements.iter().map(|v| v.into()).collect(),
        })
    }
}

pub struct GetCompetitionRankResponse {
    elements: Vec<RankedElement>,
}

impl GetCompetitionRankResponse {
    pub fn elements(&self) -> &[RankedElement] {
        &self.elements
    }

    pub fn into_elements(self) -> Vec<RankedElement> {
        self.elements
    }
}