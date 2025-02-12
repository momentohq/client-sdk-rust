use crate::leaderboard::LeaderboardRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

/// A request to remove a set of elements from a leaderboard using their element
/// ids.
pub struct RemoveElementsRequest {
    ids: Vec<u32>,
}

impl RemoveElementsRequest {
    /// Constructs a new `RemoveElementsRequest`.
    pub fn new(ids: impl IntoIterator<Item = u32>) -> Self {
        Self {
            ids: ids.into_iter().collect(),
        }
    }
}

impl LeaderboardRequest for RemoveElementsRequest {
    type Response = RemoveElementsResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.client_timeout(),
            momento_protos::leaderboard::RemoveElementsRequest {
                leaderboard: leaderboard.leaderboard_name().to_string(),
                ids: self.ids,
            },
        )?;

        let _ = leaderboard
            .next_data_client()
            .remove_elements(request)
            .await?;
        Ok(Self::Response {})
    }
}

/// The response type for a successful `RemoveElementsRequest`
#[derive(Debug, PartialEq, Eq)]
pub struct RemoveElementsResponse {}
