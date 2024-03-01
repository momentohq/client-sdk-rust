use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::requests::cache::set_add_elements::{SetAddElements, SetAddElementsRequest};
use crate::requests::cache::MomentoRequest;
use crate::utils::user_agent;
use crate::{utils, CredentialProvider, IntoBytes, MomentoResult};
use momento_protos::cache_client::scs_client::ScsClient;
use std::convert::TryInto;
use std::time::Duration;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

pub struct CacheClient {
    pub(crate) data_client: ScsClient<InterceptedService<Channel, HeaderInterceptor>>,
    item_default_ttl: Duration,
}

impl CacheClient {
    /* constructor */
    pub fn new(
        credential_provider: CredentialProvider,
        default_ttl: Duration,
    ) -> MomentoResult<Self> {
        let data_channel = utils::connect_channel_lazily(&credential_provider.cache_endpoint)?;

        let data_interceptor = InterceptedService::new(
            data_channel,
            HeaderInterceptor::new(&credential_provider.auth_token, &user_agent("sdk")),
        );
        let data_client = ScsClient::new(data_interceptor);
        Ok(CacheClient {
            data_client,
            item_default_ttl: default_ttl,
        })
    }

    /* public API */

    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use momento::{CredentialProviderBuilder};
    /// use momento::requests::cache::set_add_elements::SetAddElements;
    ///
    /// let credential_provider = CredentialProviderBuilder::from_environment_variable("MOMENTO_API_KEY".to_string())
    ///     .build()?;
    /// let cache_name = "cache";
    ///
    /// let cache_client = momento::CacheClient::new(credential_provider, Duration::from_secs(5))?;
    ///
    /// let set_add_elements_response = cache_client.set_add_elements(cache_name.to_string(), "set", vec!["element1", "element2"]).await?;
    /// assert_eq!(set_add_elements_response, SetAddElements {});
    /// # Ok(())
    /// # })
    /// #
    /// }
    /// ```
    pub async fn set_add_elements<E: IntoBytes>(
        self,
        cache_name: String,
        set_name: impl IntoBytes,
        elements: Vec<E>,
    ) -> MomentoResult<SetAddElements> {
        let request = SetAddElementsRequest::new(cache_name, set_name, elements);
        request.send(&self).await
    }

    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_protos::cache_client::update_ttl_response::Result::Set;
    /// use momento::requests::cache::set_add_elements::SetAddElementsRequest;
    /// tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use momento::{CredentialProviderBuilder};
    /// use momento::requests::cache::set_add_elements::SetAddElements;
    ///
    /// let credential_provider = CredentialProviderBuilder::from_environment_variable("MOMENTO_API_KEY".to_string())
    ///     .build()?;
    /// let cache_name = "cache";
    ///
    /// let cache_client = momento::CacheClient::new(credential_provider, Duration::from_secs(5))?;
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
    pub async fn send_request<R: MomentoRequest>(self, request: R) -> MomentoResult<R::Response> {
        request.send(&self).await
    }

    /* helper fns */
    pub(crate) fn expand_ttl_ms(&self, ttl: Option<Duration>) -> MomentoResult<u64> {
        let ttl = ttl.unwrap_or(self.item_default_ttl);
        utils::is_ttl_valid(ttl)?;

        Ok(ttl.as_millis().try_into().unwrap_or(i64::MAX as u64))
    }
}
