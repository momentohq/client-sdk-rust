use crate::leaderboard::messages::MomentoRequest;
use crate::{utils, LeaderboardClient, MomentoResult};

use tonic::Request;

/// Request to delete a leaderboard
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the leaderboard.
/// * `leaderboard` - The name of the leaderboard.
pub struct DeleteLeaderboardRequest {
    /// The name of the cache containing the leaderboard.
    pub cache_name: String,
    /// The leaderboard to be deleted.
    pub leaderboard: String,
}

impl DeleteLeaderboardRequest {
    /// Constructs a new `DeleteLeaderboardRequest`.
    pub fn new(cache_name: impl Into<String>, leaderboard: impl Into<String>) -> Self {
        Self {
            cache_name: cache_name.into(),
            leaderboard: leaderboard.into(),
        }
    }
}

impl MomentoRequest for DeleteLeaderboardRequest {
    type Response = DeleteLeaderboardResponse;

    async fn send(self, leaderboard_client: &LeaderboardClient) -> MomentoResult<Self::Response> {
        let cache_name = &self.cache_name;

        utils::is_cache_name_valid(cache_name)?;
        let request = Request::new(momento_protos::leaderboard::DeleteLeaderboardRequest {
            cache_name: cache_name.to_string(),
            leaderboard: self.leaderboard.to_string(),
        });

        let _ = leaderboard_client
            .next_data_client()
            .delete_leaderboard(request)
            .await?;
        Ok(Self::Response {})
    }
}

/// The response type for a successful `DeleteLeaderboardRequest`
#[derive(Debug, PartialEq, Eq)]
pub struct DeleteLeaderboardResponse {}
