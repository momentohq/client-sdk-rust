use crate::leaderboard::messages::MomentoRequest;
use crate::{utils, LeaderboardClient, MomentoResult};

use tonic::Request;

/// A request to get the number of elements in a leaderboard.
pub struct GetLeaderboardLengthRequest {
    cache_name: String,
    leaderboard: String,
}

impl GetLeaderboardLengthRequest {
    /// Constructs a new `GetLeaderboardLengthRequest`.
    pub fn new(cache_name: impl Into<String>, leaderboard: impl Into<String>) -> Self {
        Self {
            cache_name: cache_name.into(),
            leaderboard: leaderboard.into(),
        }
    }
}

impl MomentoRequest for GetLeaderboardLengthRequest {
    type Response = GetLeaderboardLengthResponse;

    async fn send(self, leaderboard_client: &LeaderboardClient) -> MomentoResult<Self::Response> {
        let cache_name = &self.cache_name;

        utils::is_cache_name_valid(cache_name)?;
        let request = Request::new(momento_protos::leaderboard::GetLeaderboardLengthRequest {
            cache_name: cache_name.to_string(),
            leaderboard: self.leaderboard.to_string(),
        });

        let response = leaderboard_client
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
