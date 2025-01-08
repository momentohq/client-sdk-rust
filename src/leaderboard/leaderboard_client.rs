use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::leaderboard::leaderboard_client_builder::{
    LeaderboardClientBuilder, NeedsConfiguration,
};
use crate::leaderboard::messages::data::delete_leaderboard::{
    DeleteLeaderboardRequest, DeleteLeaderboardResponse,
};
use crate::leaderboard::messages::data::get_by_rank::{
    GetByRankRequest, GetByRankResponse, RankRange,
};
use crate::leaderboard::messages::data::get_by_score::{
    GetByScoreRequest, GetByScoreResponse, ScoreRange,
};
use crate::leaderboard::messages::data::get_leaderboard_length::{
    GetLeaderboardLengthRequest, GetLeaderboardLengthResponse,
};
use crate::leaderboard::messages::data::get_rank::{GetRankRequest, GetRankResponse};
use crate::leaderboard::messages::data::remove_elements::{
    RemoveElementsRequest, RemoveElementsResponse,
};
use crate::leaderboard::messages::data::upsert_elements::{
    IntoElements, UpsertElementsRequest, UpsertElementsResponse,
};
use crate::leaderboard::{Configuration, MomentoRequest};
use crate::MomentoResult;

use momento_protos::control_client::scs_control_client::ScsControlClient;
use momento_protos::leaderboard::leaderboard_client::LeaderboardClient as SLbClient;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static NEXT_DATA_CLIENT_INDEX: AtomicUsize = AtomicUsize::new(0);

pub use crate::leaderboard::messages::data::Order;

#[derive(Clone, Debug)]
pub struct LeaderboardClient {
    data_clients: Vec<SLbClient<InterceptedService<Channel, HeaderInterceptor>>>,
    #[allow(dead_code)]
    control_client: ScsControlClient<InterceptedService<Channel, HeaderInterceptor>>,
    configuration: Configuration,
}

impl LeaderboardClient {
    pub fn builder() -> LeaderboardClientBuilder<NeedsConfiguration> {
        LeaderboardClientBuilder(NeedsConfiguration {})
    }

    pub async fn delete_leaderboard(
        &self,
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
    ) -> MomentoResult<DeleteLeaderboardResponse> {
        let request = DeleteLeaderboardRequest::new(cache_name, leaderboard);
        request.send(self).await
    }

    pub async fn get_by_rank<T: Into<RankRange>>(
        &self,
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
        rank_range: Option<T>,
        order: Order,
    ) -> MomentoResult<GetByRankResponse> {
        let request = GetByRankRequest::new(cache_name, leaderboard, rank_range, order);
        request.send(self).await
    }

    pub async fn get_by_score(
        &self,
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
        score_range: impl Into<Option<ScoreRange>>,
        offset: u32,
        limit_elements: u32,
        order: Order,
    ) -> MomentoResult<GetByScoreResponse> {
        let request = GetByScoreRequest::new(
            cache_name,
            leaderboard,
            score_range,
            offset,
            limit_elements,
            order,
        );
        request.send(self).await
    }

    pub async fn get_leaderboard_length(
        &self,
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
    ) -> MomentoResult<GetLeaderboardLengthResponse> {
        let request = GetLeaderboardLengthRequest::new(cache_name, leaderboard);
        request.send(self).await
    }

    pub async fn get_rank<T: Into<Vec<u32>>>(
        &self,
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
        ids: T,
        order: Order,
    ) -> MomentoResult<GetRankResponse> {
        let request = GetRankRequest::new(cache_name, leaderboard, ids, order);
        request.send(self).await
    }

    pub async fn remove_elements<T: Into<Vec<u32>>>(
        &self,
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
        ids: T,
    ) -> MomentoResult<RemoveElementsResponse> {
        let request = RemoveElementsRequest::new(cache_name, leaderboard, ids);
        request.send(self).await
    }

    pub async fn upsert_elements<E: IntoElements>(
        &self,
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
        elements: E,
    ) -> MomentoResult<UpsertElementsResponse> {
        let request = UpsertElementsRequest::new(cache_name, leaderboard, elements);
        request.send(self).await
    }

    /* helper fns */
    pub(crate) fn new(
        data_clients: Vec<SLbClient<InterceptedService<Channel, HeaderInterceptor>>>,
        control_client: ScsControlClient<InterceptedService<Channel, HeaderInterceptor>>,
        configuration: Configuration,
    ) -> Self {
        Self {
            data_clients,
            control_client,
            configuration,
        }
    }

    pub(crate) fn deadline_millis(&self) -> Duration {
        self.configuration.deadline_millis()
    }

    #[allow(dead_code)]
    pub(crate) fn control_client(
        &self,
    ) -> ScsControlClient<InterceptedService<Channel, HeaderInterceptor>> {
        self.control_client.clone()
    }

    pub(crate) fn next_data_client(
        &self,
    ) -> SLbClient<InterceptedService<Channel, HeaderInterceptor>> {
        let next_index =
            NEXT_DATA_CLIENT_INDEX.fetch_add(1, Ordering::Relaxed) % self.data_clients.len();
        self.data_clients[next_index].clone()
    }
}
