use std::convert::TryFrom;
use tonic::{
    codegen::InterceptedService,
    transport::{Channel, ClientTlsConfig, Uri},
    Request,
};

use crate::{
    cache::CacheClient,
    generated::control_client::{
        scs_control_client::ScsControlClient, CreateCacheRequest, DeleteCacheRequest,
        ListCachesRequest,
    },
    grpc::auth_header_interceptor::AuthHeaderInterceptor,
    jwt::decode_jwt,
    response::{
        error::MomentoError,
        list_cache_response::{MomentoCache, MomentoListCacheResult},
    },
};

pub struct Momento {
    client: ScsControlClient<InterceptedService<Channel, AuthHeaderInterceptor>>,
    cache_endpoint: String,
    auth_key: String,
}

impl Momento {
    /// Returns an instance of a Momento client
    ///
    /// # Arguments
    ///
    /// * `auth_key` - Momento jwt
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use momento::sdk::Momento;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let momento = Momento::new(auth_token).await;
    /// # })
    /// ```
    pub async fn new(auth_key: String) -> Result<Self, MomentoError> {
        let claims = decode_jwt(&auth_key)?;
        let formatted_cp_endpoint = format!("https://{}:443", claims.cp);
        let uri = Uri::try_from(formatted_cp_endpoint.as_str())?;
        let channel = Channel::builder(uri)
            .tls_config(ClientTlsConfig::default())
            .unwrap()
            .connect()
            .await?;
        let interceptor = InterceptedService::new(
            channel.clone(),
            AuthHeaderInterceptor {
                auth_key: auth_key.clone(),
            },
        );
        let client = ScsControlClient::new(interceptor);
        return Ok(Self {
            auth_key: auth_key,
            cache_endpoint: format!("https://{}:443", claims.c),
            client: client,
        });
    }

    /// Gets a MomentoCache to perform gets and sets on
    ///
    /// # Arguments
    ///
    /// * `name` - name of cache to get
    /// * `default_ttl_seconds` - the default number of seconds for items to live in the Momento cache, can be overriden
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use momento::sdk::Momento;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = env::var("TEST_CACHE_NAME").expect("TEST_CACHE_NAME must be set");
    ///     let mut momento = Momento::new(auth_token).await.unwrap();
    ///     let cache = momento.get_cache(&cache_name, 1000).await.unwrap();
    /// # })
    /// ```
    pub async fn get_cache(
        &mut self,
        name: &str,
        default_ttl_seconds: u32,
    ) -> Result<CacheClient, MomentoError> {
        let mut client = CacheClient::new(
            name.to_string(),
            self.cache_endpoint.clone(),
            self.auth_key.clone(),
            default_ttl_seconds,
        );
        client.connect().await?;
        return Ok(client);
    }

    /// Creates a new Momento cache
    ///
    /// # Arguments
    ///
    /// * `name` - name of cache to create
    pub async fn create_cache(&mut self, name: &str) -> Result<(), MomentoError> {
        let request = Request::new(CreateCacheRequest {
            cache_name: name.to_string(),
        });

        self.client.create_cache(request).await?;
        Ok(())
    }

    /// Deletes a Momento cache, and all of its contents
    ///
    /// # Arguments
    ///
    /// * `name` - name of cache to delete
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use momento::sdk::Momento;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let mut momento = Momento::new(auth_token).await.unwrap();
    ///     momento.delete_cache("my_cache").await;
    /// # })
    /// ```
    pub async fn delete_cache(&mut self, name: &str) -> Result<(), MomentoError> {
        let request = Request::new(DeleteCacheRequest {
            cache_name: name.to_string(),
        });
        self.client.delete_cache(request).await?;
        Ok(())
    }

    /// Lists all Momento caches
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use momento::sdk::Momento;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let mut momento = Momento::new(auth_token).await.unwrap();
    ///     let caches = momento.list_caches(None).await;
    /// # })
    /// ```
    pub async fn list_caches(
        &mut self,
        next_token: Option<&str>,
    ) -> Result<MomentoListCacheResult, MomentoError> {
        let request = Request::new(ListCachesRequest {
            next_token: next_token.unwrap_or_default().to_string(),
        });
        let res = self.client.list_caches(request).await?.into_inner();
        let caches = res
            .cache
            .iter()
            .map(|cache| MomentoCache {
                cache_name: cache.cache_name.to_string(),
            })
            .collect();
        let response = MomentoListCacheResult {
            caches: caches,
            next_token: res.next_token.to_string(),
        };
        Ok(response)
    }
}
