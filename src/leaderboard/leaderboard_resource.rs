/// Represents a remote leaderboard resource.
use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::leaderboard::messages::data::delete::{DeleteRequest, DeleteResponse};
use crate::leaderboard::messages::data::fetch::FetchResponse;
use crate::leaderboard::messages::data::fetch_by_rank::{FetchByRankRequest, RankRange};
use crate::leaderboard::messages::data::fetch_by_score::{FetchByScoreRequest, ScoreRange};
use crate::leaderboard::messages::data::get_rank::{GetRankRequest, GetRankResponse};
use crate::leaderboard::messages::data::length::{LengthRequest, LengthResponse};
use crate::leaderboard::messages::data::remove_elements::{
    RemoveElementsRequest, RemoveElementsResponse,
};
use crate::leaderboard::messages::data::upsert::{IntoElements, UpsertRequest, UpsertResponse};
use crate::leaderboard::MomentoRequest;
use crate::MomentoResult;

use momento_protos::leaderboard::leaderboard_client as leaderboard_proto;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static NEXT_DATA_CLIENT_INDEX: AtomicUsize = AtomicUsize::new(0);

use super::messages::data::IntoIds;

/// Represents a remote leaderboard resource.
pub struct Leaderboard {
    data_clients:
        Vec<leaderboard_proto::LeaderboardClient<InterceptedService<Channel, HeaderInterceptor>>>,
    client_timeout: Duration,
    cache_name: String,
    leaderboard_name: String,
}

impl Leaderboard {
    /// Delete a leaderboard.
    pub async fn delete(&self) -> MomentoResult<DeleteResponse> {
        let request = DeleteRequest::new();
        request.send(self).await
    }

    /// Fetch elements from a leaderboard by rank.
    ///
    /// Note: can fetch a maximum of 8192 elements at a time and rank
    /// is 0-based (index begins at 0).
    ///
    /// Defaults to ascending order, meaning rank 0 is the element with
    /// the lowest score.
    pub async fn fetch_by_rank(
        &self,
        rank_range: impl Into<RankRange>,
    ) -> MomentoResult<FetchResponse> {
        let request = FetchByRankRequest::new(rank_range);
        request.send(self).await
    }

    /// Get elements from a leaderboard by score.
    ///
    /// Note: can fetch a maximum of 8192 elements at a time.
    ///
    /// Defaults to ascending order, meaning the results will be
    /// ordered from lowest to highest score.
    pub async fn fetch_by_score(
        &self,
        score_range: impl Into<ScoreRange>,
    ) -> MomentoResult<FetchResponse> {
        let request = FetchByScoreRequest::new(score_range);
        request.send(self).await
    }

    /// Get the length of a leaderboard.
    pub async fn len(&self) -> MomentoResult<LengthResponse> {
        let request = LengthRequest::new();
        request.send(self).await
    }

    /// Get rank of elements from a leaderboard using their element ids.
    ///
    /// Defaults to ascending order rank, meaning rank 0 is the element with
    /// the lowest score.
    pub async fn get_rank<T: IntoIds>(&self, ids: T) -> MomentoResult<GetRankResponse> {
        let request = GetRankRequest::new(ids);
        request.send(self).await
    }

    /// Remove elements from a leaderboard using their element ids.
    pub async fn remove_elements<T: IntoIds>(
        &self,
        ids: T,
    ) -> MomentoResult<RemoveElementsResponse> {
        let request = RemoveElementsRequest::new(ids);
        request.send(self).await
    }

    /// Upsert (update/insert) elements into a leaderboard.
    pub async fn upsert<E: IntoElements>(&self, elements: E) -> MomentoResult<UpsertResponse> {
        let request = UpsertRequest::new(elements);
        request.send(self).await
    }

    /* helper fns */
    pub(crate) fn new(
        data_clients: Vec<
            leaderboard_proto::LeaderboardClient<InterceptedService<Channel, HeaderInterceptor>>,
        >,
        client_timeout: Duration,
        cache_name: impl Into<String>,
        leaderboard_name: impl Into<String>,
    ) -> Self {
        Self {
            data_clients,
            client_timeout,
            cache_name: cache_name.into(),
            leaderboard_name: leaderboard_name.into(),
        }
    }

    pub(crate) fn next_data_client(
        &self,
    ) -> leaderboard_proto::LeaderboardClient<InterceptedService<Channel, HeaderInterceptor>> {
        let next_index =
            NEXT_DATA_CLIENT_INDEX.fetch_add(1, Ordering::Relaxed) % self.data_clients.len();
        self.data_clients[next_index].clone()
    }

    pub(crate) fn client_timeout(&self) -> Duration {
        self.client_timeout
    }

    pub(crate) fn cache_name(&self) -> &str {
        &self.cache_name
    }

    pub(crate) fn leaderboard_name(&self) -> &str {
        &self.leaderboard_name
    }

    /// Lower-level API to send any type of MomentoRequest to the server. This is used for cases when
    /// you want to set optional fields on a request that are not supported by the short-hand API for
    /// that request type.
    pub async fn send_request<R: MomentoRequest>(&self, request: R) -> MomentoResult<R::Response> {
        request.send(self).await
    }
}
