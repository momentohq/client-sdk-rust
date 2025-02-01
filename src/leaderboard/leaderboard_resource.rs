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

use momento_protos::leaderboard::leaderboard_client::LeaderboardClient as SLbClient;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static NEXT_DATA_CLIENT_INDEX: AtomicUsize = AtomicUsize::new(0);

pub use crate::leaderboard::messages::data::Order;

/// Represents a remote leaderboard resource.
pub struct Leaderboard {
    data_clients: Vec<SLbClient<InterceptedService<Channel, HeaderInterceptor>>>,
    deadline: Duration,
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
    pub async fn fetch_by_rank<T: Into<RankRange>>(
        &self,
        rank_range: Option<T>,
        order: Order,
    ) -> MomentoResult<FetchResponse> {
        let request = FetchByRankRequest::new(rank_range, order);
        request.send(self).await
    }

    /// Get elements from a leaderboard by score.
    pub async fn fetch_by_score(
        &self,
        score_range: impl Into<Option<ScoreRange>>,
        offset: u32,
        limit_elements: u32,
        order: Order,
    ) -> MomentoResult<FetchResponse> {
        let request = FetchByScoreRequest::new(score_range, offset, limit_elements, order);
        request.send(self).await
    }

    /// Get the length of a leaderboard.
    pub async fn length(&self) -> MomentoResult<LengthResponse> {
        let request = LengthRequest::new();
        request.send(self).await
    }

    /// Get elements from a leaderboard using their element ids.
    pub async fn get_rank<T: Into<Vec<u32>>>(
        &self,
        ids: T,
        order: Order,
    ) -> MomentoResult<GetRankResponse> {
        let request = GetRankRequest::new(ids, order);
        request.send(self).await
    }

    /// Remove elements from a leaderboard using their element ids.
    pub async fn remove_elements<T: Into<Vec<u32>>>(
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
        data_clients: Vec<SLbClient<InterceptedService<Channel, HeaderInterceptor>>>,
        deadline: Duration,
        cache_name: impl Into<String>,
        leaderboard_name: impl Into<String>,
    ) -> Self {
        Self {
            data_clients,
            deadline,
            cache_name: cache_name.into(),
            leaderboard_name: leaderboard_name.into(),
        }
    }

    pub(crate) fn next_data_client(
        &self,
    ) -> SLbClient<InterceptedService<Channel, HeaderInterceptor>> {
        let next_index =
            NEXT_DATA_CLIENT_INDEX.fetch_add(1, Ordering::Relaxed) % self.data_clients.len();
        self.data_clients[next_index].clone()
    }

    pub(crate) fn deadline(&self) -> Duration {
        self.deadline
    }

    pub(crate) fn cache_name(&self) -> &String {
        &self.cache_name
    }

    pub(crate) fn leaderboard_name(&self) -> &String {
        &self.leaderboard_name
    }
}
