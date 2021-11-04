pub mod cache_client {
    tonic::include_proto!("cache_client");
}

use std::{
    convert::TryFrom,
    time::{Duration, SystemTime},
};

use tokio::time::sleep;

use tonic::{
    codegen::InterceptedService,
    transport::{Channel, ClientTlsConfig, Uri},
    Request,
};

use cache_client::{scs_client::ScsClient, ECacheResult, GetRequest, SetRequest};

use crate::response::cache_get_response::MomentoGetResponse;
use crate::{
    grpc::cache_header_interceptor::CacheHeaderInterceptor,
    response::{
        cache_get_response::MomentoGetStatus,
        cache_set_response::{MomentoSetResponse, MomentoSetStatus},
        error::MomentoError,
    },
};
pub trait MomentoRequest {
    fn into_bytes(self) -> Vec<u8>;
}

impl MomentoRequest for String {
    fn into_bytes(self) -> Vec<u8> {
        self.into_bytes()
    }
}

impl MomentoRequest for Vec<u8> {
    fn into_bytes(self) -> Vec<u8> {
        self
    }
}

impl MomentoRequest for &str {
    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

pub struct CacheClient {
    client: Option<ScsClient<InterceptedService<Channel, CacheHeaderInterceptor>>>,
    channel: Option<tonic::transport::Channel>,
    endpoint: String,
    auth_key: String,
    cache_name: String,
    default_ttl_seconds: u32,
}

impl CacheClient {
    pub fn new(
        cache_name: String,
        endpoint: String,
        auth_key: String,
        default_ttl_seconds: u32,
    ) -> Self {
        return Self {
            client: Option::None,
            endpoint: endpoint,
            channel: Option::None,
            auth_key: auth_key,
            cache_name: cache_name,
            default_ttl_seconds: default_ttl_seconds,
        };
    }

    pub async fn connect(&mut self) -> Result<(), MomentoError> {
        let uri = Uri::try_from(self.endpoint.as_str())?;
        let channel = Channel::builder(uri)
            .tls_config(ClientTlsConfig::default())
            .unwrap()
            .connect()
            .await?;

        self.channel = Some(channel);
        let interceptor = InterceptedService::new(
            self.channel.clone().unwrap(),
            CacheHeaderInterceptor {
                auth_key: self.auth_key.to_string(),
                cache_name: self.cache_name.to_string(),
            },
        );
        let client = ScsClient::new(interceptor);
        self.client = Some(client);
        self.wait_until_ready().await?;
        Ok(())
    }

    async fn wait_until_ready(&self) -> Result<(), MomentoError> {
        let backoff_millis = 50;
        let max_connect_time_seconds = 5;
        let start = SystemTime::now();
        while start.elapsed().unwrap().as_secs() < max_connect_time_seconds {
            let get_response = self.get("0000".to_string()).await;
            match get_response {
                Ok(_) => return Ok(()),
                Err(e) => match e {
                    MomentoError::InternalServerError => "",
                    _ => return Err(e),
                },
            };
            sleep(Duration::new(0, backoff_millis * 1000)).await;
        }
        Err(MomentoError::InternalServerError)
    }

    /// Gets an item from a Momento Cache
    ///
    /// # Arguments
    ///
    /// * `key` - cache key
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use momento::sdk::Momento;
    ///     let mut momento = Momento::new("auth_token".to_string()).await;
    ///     let mut cache = momento.get_cache("my_cache", 100).await;
    ///     let resp = cache.get("cache_key").await.unwrap();
    ///     match resp.result {
    ///         MomentoGetStatus::HIT => println!("cache hit!"),
    ///         MomentoGetStatus::MISS => println!("cache miss")
    ///     };
    ///
    ///     println!("cache value: {}", resp.as_string());
    /// # })
    /// ```
    pub async fn get<I: MomentoRequest>(&self, key: I) -> Result<MomentoGetResponse, MomentoError> {
        let get_request = Request::new(GetRequest {
            cache_key: key.into_bytes(),
        });

        let response = self
            .client
            .as_ref()
            .unwrap()
            .clone()
            .get(get_request)
            .await?;
        let get_response = response.into_inner();
        match get_response.result() {
            ECacheResult::Hit => Ok(MomentoGetResponse {
                result: MomentoGetStatus::HIT,
                value: get_response.cache_body,
            }),
            ECacheResult::Miss => Ok(MomentoGetResponse {
                result: MomentoGetStatus::MISS,
                value: get_response.cache_body,
            }),
            _ => todo!(),
        }
    }

    /// Sets an item in a Momento Cache
    ///
    /// # Arguments
    ///
    /// * `cache_key`
    /// * `cache_body`
    /// * `ttl_seconds` - If None is passed, uses the caches default ttl
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use momento::sdk::Momento;
    ///     let mut momento = Momento::new("auth_token".to_string()).await;
    ///     let mut cache = momento.get_cache("my_cache", 100).await;
    ///     cache.set("cache_key", "cache_value", None).await;
    ///
    ///     // overriding default ttl
    ///     cache.set("cache_key", "cache_value", 1).await;
    /// # })
    /// ```
    pub async fn set<I: MomentoRequest>(
        &self,
        key: I,
        body: I,
        ttl_seconds: Option<u32>,
    ) -> Result<MomentoSetResponse, MomentoError> {
        let request = tonic::Request::new(SetRequest {
            cache_key: key.into_bytes(),
            cache_body: body.into_bytes(),
            ttl_milliseconds: ttl_seconds.unwrap_or(self.default_ttl_seconds) * 1000,
        });
        let _ = self.client.as_ref().unwrap().clone().set(request).await?;
        Ok(MomentoSetResponse {
            result: MomentoSetStatus::OK,
        })
    }
}
