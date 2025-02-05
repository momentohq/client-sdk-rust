use crate::leaderboard::MomentoRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

use super::IntoIds;

/// A request to remove a set of elements from a leaderboard using their element
/// ids.
pub struct RemoveElementsRequest {
    ids: Vec<u32>,
}

impl RemoveElementsRequest {
    /// Constructs a new `RemoveElementsRequest`.
    pub fn new(ids: impl IntoIds) -> Self {
        Self {
            ids: ids.into_ids(),
        }
    }
}

impl MomentoRequest for RemoveElementsRequest {
    type Response = RemoveElementsResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.client_timeout(),
            momento_protos::leaderboard::RemoveElementsRequest {
                cache_name: cache_name.clone(),
                leaderboard: leaderboard.leaderboard_name().clone(),
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
