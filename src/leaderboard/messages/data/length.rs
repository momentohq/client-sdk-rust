use crate::leaderboard::messages::LeaderboardRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

/// A request to get the number of elements in a leaderboard.
pub struct LengthRequest {}

impl LengthRequest {
    /// Constructs a new `LengthRequest`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for LengthRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl LeaderboardRequest for LengthRequest {
    type Response = LengthResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.client_timeout(),
            momento_protos::leaderboard::GetLeaderboardLengthRequest {
                leaderboard: leaderboard.leaderboard_name().to_string(),
            },
        )?;

        let response = leaderboard
            .next_data_client()
            .get_leaderboard_length(request)
            .await?
            .into_inner();

        Ok(Self::Response {
            length: response.count,
        })
    }
}

/// The response type for a successful `LengthRequest`
#[derive(Debug, PartialEq, Eq)]
pub struct LengthResponse {
    length: u32,
}

impl LengthResponse {
    /// Returns the number of elements that were in the leaderboard.
    pub fn length(&self) -> u32 {
        self.length
    }
}
