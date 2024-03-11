use std::convert::TryInto;
use std::time::Duration;

use momento_protos::cache_client::scs_client::ScsClient;
use momento_protos::control_client::scs_control_client::ScsControlClient;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

use crate::config::configuration::Configuration;
use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::requests::cache::create_cache::{CreateCache, CreateCacheRequest};
use crate::requests::cache::delete_cache::{DeleteCache, DeleteCacheRequest};
use crate::requests::cache::set_add_elements::{SetAddElements, SetAddElementsRequest};
use crate::requests::cache::sorted_set_fetch_by_rank::{SortOrder, SortedSetFetchByRankRequest};
use crate::requests::cache::sorted_set_fetch_by_score::SortedSetFetchByScoreRequest;
use crate::requests::cache::sorted_set_put_element::{
    SortedSetPutElement, SortedSetPutElementRequest,
};
use crate::requests::cache::sorted_set_put_elements::{
    SortedSetPutElements, SortedSetPutElementsRequest,
};
use crate::requests::cache::MomentoRequest;
use crate::response::cache::sorted_set_fetch::SortedSetFetch;
use crate::utils::user_agent;
use crate::{utils, CredentialProvider, IntoBytes, MomentoResult};

#[derive(Clone)]
pub struct CacheClient {
    pub(crate) data_client: ScsClient<InterceptedService<Channel, HeaderInterceptor>>,
    pub(crate) control_client: ScsControlClient<InterceptedService<Channel, HeaderInterceptor>>,
    pub(crate) configuration: Configuration,
    item_default_ttl: Duration,
}

impl CacheClient {
    /* constructor */
    pub fn new(
        credential_provider: CredentialProvider,
        configuration: Configuration,
        default_ttl: Duration,
    ) -> MomentoResult<Self> {
        let agent_value = &user_agent("sdk");

        let data_channel = utils::connect_channel_lazily_configurable(
            &credential_provider.cache_endpoint,
            configuration.transport_strategy.grpc_configuration.clone(),
        )?;
        let control_channel = utils::connect_channel_lazily_configurable(
            &credential_provider.control_endpoint,
            configuration.transport_strategy.grpc_configuration.clone(),
        )?;

        let data_interceptor = InterceptedService::new(
            data_channel,
            HeaderInterceptor::new(&credential_provider.auth_token, agent_value),
        );
        let control_interceptor = InterceptedService::new(
            control_channel,
            HeaderInterceptor::new(&credential_provider.auth_token, agent_value),
        );

        let data_client = ScsClient::new(data_interceptor);
        let control_client = ScsControlClient::new(control_interceptor);

        Ok(CacheClient {
            data_client,
            control_client,
            configuration,
            item_default_ttl: default_ttl,
        })
    }

    /* public API */

    pub async fn create_cache(&self, cache_name: String) -> MomentoResult<CreateCache> {
        let request = CreateCacheRequest::new(cache_name);
        request.send(self).await
    }

    pub async fn delete_cache(&self, cache_name: String) -> MomentoResult<DeleteCache> {
        let request = DeleteCacheRequest::new(cache_name);
        request.send(self).await
    }

    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use momento::config::configurations;
    /// use momento::CredentialProviderBuilder;
    /// use momento::requests::cache::set_add_elements::SetAddElements;
    ///
    /// let credential_provider = CredentialProviderBuilder::from_environment_variable("MOMENTO_API_KEY".to_string())
    ///     .build()?;
    /// let cache_name = "cache";
    ///
    /// let cache_client = momento::CacheClient::new(
    ///    credential_provider,
    ///    configurations::laptop::latest(),
    ///    Duration::from_secs(5),
    ///)?;
    ///
    /// let set_add_elements_response = cache_client.set_add_elements(cache_name.to_string(), "set", vec!["element1", "element2"]).await?;
    /// assert_eq!(set_add_elements_response, SetAddElements {});
    /// # Ok(())
    /// # })
    /// #
    /// }
    /// ```
    pub async fn set_add_elements<E: IntoBytes>(
        &self,
        cache_name: String,
        set_name: impl IntoBytes,
        elements: Vec<E>,
    ) -> MomentoResult<SetAddElements> {
        let request = SetAddElementsRequest::new(cache_name, set_name, elements);
        request.send(self).await
    }

    pub async fn sorted_set_put_element<E: IntoBytes>(
        &self,
        cache_name: String,
        sorted_set_name: impl IntoBytes,
        value: E,
        score: f64,
    ) -> MomentoResult<SortedSetPutElement> {
        let request = SortedSetPutElementRequest::new(cache_name, sorted_set_name, value, score);
        request.send(self).await
    }

    pub async fn sorted_set_put_elements<E: IntoBytes>(
        &self,
        cache_name: String,
        sorted_set_name: impl IntoBytes,
        elements: Vec<(E, f64)>,
    ) -> MomentoResult<SortedSetPutElements> {
        let request = SortedSetPutElementsRequest::new(cache_name, sorted_set_name, elements);
        request.send(self).await
    }

    pub async fn sorted_set_fetch_by_rank<S: IntoBytes>(
        &self,
        cache_name: String,
        sorted_set_name: S,
        order: SortOrder,
    ) -> MomentoResult<SortedSetFetch> {
        let request =
            SortedSetFetchByRankRequest::new(cache_name, sorted_set_name).with_order(order);
        request.send(self).await
    }

    pub async fn sorted_set_fetch_by_score<S: IntoBytes>(
        &self,
        cache_name: String,
        sorted_set_name: S,
        order: SortOrder,
    ) -> MomentoResult<SortedSetFetch> {
        let request =
            SortedSetFetchByScoreRequest::new(cache_name, sorted_set_name).with_order(order);
        request.send(self).await
    }

    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_protos::cache_client::update_ttl_response::Result::Set;
    /// use momento::requests::cache::set_add_elements::SetAddElementsRequest;
    /// tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use momento::config::configurations;
    /// use momento::CredentialProviderBuilder;
    /// use momento::requests::cache::set_add_elements::SetAddElements;
    ///
    /// let credential_provider = CredentialProviderBuilder::from_environment_variable("MOMENTO_API_KEY".to_string())
    ///     .build()?;
    /// let cache_name = "cache";
    ///
    /// let cache_client = momento::CacheClient::new(
    ///    credential_provider,
    ///    configurations::laptop::latest(),
    ///    Duration::from_secs(5),
    ///)?;
    ///
    /// let set_add_elements_response = cache_client.send_request(
    ///     SetAddElementsRequest::new(cache_name.to_string(), "set", vec!["element1", "element2"])
    /// ).await?;
    /// assert_eq!(set_add_elements_response, SetAddElements {});
    /// # Ok(())
    /// # })
    /// #
    /// }
    /// ```
    pub async fn send_request<R: MomentoRequest>(&self, request: R) -> MomentoResult<R::Response> {
        request.send(self).await
    }

    /* helper fns */
    pub(crate) fn expand_ttl_ms(&self, ttl: Option<Duration>) -> MomentoResult<u64> {
        let ttl = ttl.unwrap_or(self.item_default_ttl);
        utils::is_ttl_valid(ttl)?;

        Ok(ttl.as_millis().try_into().unwrap_or(i64::MAX as u64))
    }
}
