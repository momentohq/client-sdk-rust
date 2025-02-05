use crate::leaderboard::messages::MomentoRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

/// Request to delete a leaderboard
pub struct DeleteRequest {}

impl DeleteRequest {
    /// Constructs a new `DeleteRequest`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DeleteRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl MomentoRequest for DeleteRequest {
    type Response = DeleteResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.client_timeout(),
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

/// The response type for a successful `DeleteRequest`
#[derive(Debug, PartialEq, Eq)]
pub struct DeleteResponse {}
