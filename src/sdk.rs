pub mod control_client {
    tonic::include_proto!("control_client");
}

use std::convert::TryFrom;

use control_client::{
    scs_control_client::ScsControlClient, CreateCacheRequest, DeleteCacheRequest,
};
use tonic::{
    codegen::InterceptedService,
    transport::{Channel, ClientTlsConfig, Uri},
    Request,
};

use crate::{
    cache::CacheClient, grpc::auth_header_interceptor::AuthHeaderInterceptor, jwt::decode_jwt,
    response::error::MomentoError,
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
    ///     let momento = Momento::new("auth_token".to_string()).await;
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
    ///     let mut momento = Momento::new("auth_token".to_string()).await.unwrap();
    ///     let cache = momento.get_cache("my_cache", 1000).await.unwrap();
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
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use momento::sdk::Momento;
    ///     let mut momento = Momento::new("auth_token".to_string()).await.unwrap();
    ///     momento.create_cache("my_cache").await.unwrap();
    /// # })
    /// ```
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
    ///     let mut momento = Momento::new("auth_token".to_string()).await.unwrap();
    ///     momento.delete_cache("my_cache").await.unwrap();
    /// # })
    /// ```
    pub async fn delete_cache(&mut self, name: &str) -> Result<(), MomentoError> {
        let request = Request::new(DeleteCacheRequest {
            cache_name: name.to_string(),
        });
        self.client.delete_cache(request).await?;
        Ok(())
    }
}
