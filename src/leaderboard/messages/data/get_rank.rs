use super::{IntoIds, Order, RankedElement};
use crate::leaderboard::MomentoRequest;
use crate::utils::prep_request_with_timeout;
use crate::{LeaderboardClient, MomentoResult};

pub struct GetRankRequest {
    cache_name: String,
    leaderboard: String,
    ids: Vec<u32>,
    order: Order,
}

impl GetRankRequest {
    /// Constructs a new SortedSetPutElementsRequest.
    pub fn new(
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
        ids: impl Into<Vec<u32>>,
        order: Order,
    ) -> Self {
        Self {
            cache_name: cache_name.into(),
            leaderboard: leaderboard.into(),
            ids: ids.into(),
            order,
        }
    }
}

impl MomentoRequest for GetRankRequest {
    type Response = GetRankResponse;

    async fn send(self, leaderboard_client: &LeaderboardClient) -> MomentoResult<Self::Response> {
        let ids = self.ids.into_ids();
        let cache_name = self.cache_name.clone();
        let request = prep_request_with_timeout(
            &self.cache_name,
            leaderboard_client.deadline_millis(),
            momento_protos::leaderboard::GetRankRequest {
                cache_name,
                leaderboard: self.leaderboard,
                ids,
                order: self.order as i32,
            },
        )?;

        let response = leaderboard_client
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
    pub fn elements(&self) -> &[RankedElement] {
        &self.elements
    }
}
