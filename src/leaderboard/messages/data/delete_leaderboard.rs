use crate::leaderboard::messages::MomentoRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

/// Request to delete a leaderboard
pub struct DeleteLeaderboardRequest {}

impl DeleteLeaderboardRequest {
    /// Constructs a new `DeleteLeaderboardRequest`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DeleteLeaderboardRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl MomentoRequest for DeleteLeaderboardRequest {
    type Response = DeleteLeaderboardResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.deadline(),
            momento_protos::leaderboard::DeleteLeaderboardRequest {
                cache_name: cache_name.clone(),
                leaderboard: leaderboard.leaderboard_name().clone(),
            },
        )?;

        leaderboard
            .next_data_client()
            .delete_leaderboard(request)
            .await?;
        Ok(Self::Response {})
    }
}

/// The response type for a successful `DeleteLeaderboardRequest`
#[derive(Debug, PartialEq, Eq)]
pub struct DeleteLeaderboardResponse {}
