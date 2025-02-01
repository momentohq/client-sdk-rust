use crate::leaderboard::messages::MomentoRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

/// A request to get the number of elements in a leaderboard.
pub struct GetLeaderboardLengthRequest {}

impl GetLeaderboardLengthRequest {
    /// Constructs a new `GetLeaderboardLengthRequest`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GetLeaderboardLengthRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl MomentoRequest for GetLeaderboardLengthRequest {
    type Response = GetLeaderboardLengthResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.deadline(),
            momento_protos::leaderboard::GetLeaderboardLengthRequest {
                cache_name: cache_name.clone(),
                leaderboard: leaderboard.leaderboard_name().clone(),
            },
        )?;

        let response = leaderboard
            .next_data_client()
            .get_leaderboard_length(request)
            .await?
            .into_inner();

        Ok(Self::Response {
            count: response.count,
        })
    }
}

/// The response type for a successful `GetLeaderboardLengthRequest`
#[derive(Debug, PartialEq, Eq)]
pub struct GetLeaderboardLengthResponse {
    count: u32,
}

impl GetLeaderboardLengthResponse {
    /// Returns the number of elements that were in the leaderboard.
    pub fn count(&self) -> u32 {
        self.count
    }
}
