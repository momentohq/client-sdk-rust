use crate::leaderboard::MomentoRequest;
use crate::utils::prep_request_with_timeout;
use crate::{LeaderboardClient, MomentoResult};

pub struct RemoveElementsRequest {
    cache_name: String,
    leaderboard: String,
    ids: Vec<u32>,
}

impl RemoveElementsRequest {
    /// Constructs a new SortedSetPutElementsRequest.
    pub fn new(
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
        ids: impl Into<Vec<u32>>,
    ) -> Self {
        Self {
            cache_name: cache_name.into(),
            leaderboard: leaderboard.into(),
            ids: ids.into(),
        }
    }
}

impl MomentoRequest for RemoveElementsRequest {
    type Response = RemoveElementsResponse;

    async fn send(self, leaderboard_client: &LeaderboardClient) -> MomentoResult<Self::Response> {
        let cache_name = self.cache_name.clone();
        let request = prep_request_with_timeout(
            &self.cache_name,
            leaderboard_client.deadline_millis(),
            momento_protos::leaderboard::RemoveElementsRequest {
                cache_name,
                leaderboard: self.leaderboard,
                ids: self.ids,
            },
        )?;

        let _ = leaderboard_client
            .next_data_client()
            .remove_elements(request)
            .await?;
        Ok(Self::Response {})
    }
}

/// The response type for a successful `RemoveElementsRequest`
#[derive(Debug, PartialEq, Eq)]
pub struct RemoveElementsResponse {}
