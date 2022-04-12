use serde_json::Value;
use std::convert::TryFrom;
use std::num::NonZeroU64;
use tonic::{
    codegen::InterceptedService,
    transport::{Channel, ClientTlsConfig, Uri},
    Request,
};

use crate::endpoint_resolver::MomentoEndpointsResolver;
use crate::grpc::cache_header_interceptor::CacheHeaderInterceptor;
use crate::{
    generated::control_client::{
        scs_control_client::ScsControlClient, CreateCacheRequest, CreateSigningKeyRequest,
        DeleteCacheRequest, ListCachesRequest, RevokeSigningKeyRequest,
    },
    grpc::auth_header_interceptor::AuthHeaderInterceptor,
    response::{
        create_signing_key_response::MomentoCreateSigningKeyResponse,
        error::MomentoError,
        list_cache_response::{MomentoCache, MomentoListCacheResult},
    },
};

use crate::response::{
    cache_get_response::MomentoGetStatus,
    cache_set_response::{MomentoSetResponse, MomentoSetStatus},
};
use crate::utils;
use crate::{
    generated::cache_client::{scs_client::ScsClient, ECacheResult, GetRequest, SetRequest},
    response::cache_get_response::MomentoGetResponse,
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

#[derive(Clone)]
pub struct SimpleCacheClientBuilder {
    data_endpoint: String,
    control_channel: Channel,
    data_channel: Channel,
    auth_token: String,
    default_ttl_seconds: NonZeroU64,
}

impl SimpleCacheClientBuilder {
    pub async fn new(
        auth_token: String,
        default_ttl_seconds: NonZeroU64,
    ) -> Result<Self, MomentoError> {
        let data_endpoint = utils::get_claims(&auth_token).c;

        let momento_endpoints = MomentoEndpointsResolver::resolve(&auth_token, &None);
        let control_endpoint_uri = Uri::try_from(&momento_endpoints.control_endpoint)?;
        let data_endpoint_uri = Uri::try_from(&momento_endpoints.data_endpoint)?;

        let control_channel = Channel::builder(control_endpoint_uri)
            .tls_config(ClientTlsConfig::default())
            .unwrap()
            .connect()
            .await?;

        let data_channel = Channel::builder(data_endpoint_uri)
            .tls_config(ClientTlsConfig::default())
            .unwrap()
            .connect()
            .await?;

        match utils::is_ttl_valid(&default_ttl_seconds) {
            Ok(_) => Ok(Self {
                data_endpoint,
                control_channel,
                data_channel,
                auth_token,
                default_ttl_seconds,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn default_ttl_seconds(mut self, seconds: NonZeroU64) -> Result<Self, MomentoError> {
        utils::is_ttl_valid(&seconds)?;
        self.default_ttl_seconds = seconds;
        Ok(self)
    }

    pub fn build(self) -> SimpleCacheClient {
        let control_interceptor = InterceptedService::new(
            self.control_channel,
            AuthHeaderInterceptor {
                auth_key: self.auth_token.clone(),
            },
        );
        let control_client = ScsControlClient::new(control_interceptor);

        let data_interceptor = InterceptedService::new(
            self.data_channel,
            CacheHeaderInterceptor {
                auth_key: self.auth_token.clone(),
            },
        );
        let data_client = ScsClient::new(data_interceptor);

        SimpleCacheClient {
            data_endpoint: self.data_endpoint,
            control_client,
            data_client,
            item_default_ttl_seconds: self.default_ttl_seconds,
        }
    }
}

pub struct SimpleCacheClient {
    data_endpoint: String,
    control_client: ScsControlClient<InterceptedService<Channel, AuthHeaderInterceptor>>,
    data_client: ScsClient<InterceptedService<Channel, CacheHeaderInterceptor>>,
    item_default_ttl_seconds: NonZeroU64,
}

impl SimpleCacheClient {
    /// Returns an instance of a Momento client
    ///
    /// # Arguments
    ///
    /// * `auth_token` - Momento Token
    /// * `item_default_ttl_seconds` - Default TTL for items put into a cache.
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use momento::simple_cache_client::SimpleCacheClient;
    ///     use std::env;
    ///     use std::num::NonZeroU64;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let default_ttl = 30;
    ///     let momento = SimpleCacheClient::new(auth_token, NonZeroU64::new(default_ttl).unwrap()).await;
    /// # })
    /// ```
    pub async fn new(
        auth_token: String,
        default_ttl_seconds: NonZeroU64,
    ) -> Result<Self, MomentoError> {
        let data_endpoint = utils::get_claims(&auth_token).c;

        let momento_endpoints = MomentoEndpointsResolver::resolve(&auth_token, &None);
        let control_client = SimpleCacheClient::build_control_client(
            auth_token.clone(),
            momento_endpoints.control_endpoint,
        )
        .await;
        let data_client = SimpleCacheClient::build_data_client(
            auth_token.clone(),
            momento_endpoints.data_endpoint,
        )
        .await;

        let simple_cache_client = Self {
            data_endpoint,
            control_client: control_client.unwrap(),
            data_client: data_client.unwrap(),
            item_default_ttl_seconds: default_ttl_seconds,
        };
        Ok(simple_cache_client)
    }

    async fn build_control_client(
        auth_token: String,
        endpoint: String,
    ) -> Result<ScsControlClient<InterceptedService<Channel, AuthHeaderInterceptor>>, MomentoError>
    {
        let uri = Uri::try_from(endpoint)?;
        let channel = Channel::builder(uri)
            .tls_config(ClientTlsConfig::default())
            .unwrap()
            .connect_lazy();

        let interceptor = InterceptedService::new(
            channel,
            AuthHeaderInterceptor {
                auth_key: auth_token,
            },
        );
        let client = ScsControlClient::new(interceptor);
        Ok(client)
    }

    async fn build_data_client(
        auth_token: String,
        endpoint: String,
    ) -> Result<ScsClient<InterceptedService<Channel, CacheHeaderInterceptor>>, MomentoError> {
        let uri = Uri::try_from(endpoint)?;
        let channel = Channel::builder(uri)
            .tls_config(ClientTlsConfig::default())
            .unwrap()
            .connect_lazy();

        let interceptor = InterceptedService::new(
            channel,
            CacheHeaderInterceptor {
                auth_key: auth_token,
            },
        );
        let client = ScsClient::new(interceptor);
        Ok(client)
    }

    /// Creates a new Momento cache
    ///
    /// # Arguments
    ///
    /// * `name` - name of cache to create
    pub async fn create_cache(&mut self, name: &str) -> Result<(), MomentoError> {
        utils::is_cache_name_valid(name)?;
        let request = Request::new(CreateCacheRequest {
            cache_name: name.to_string(),
        });

        self.control_client.create_cache(request).await?;
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
    /// use uuid::Uuid;
    /// use std::num::NonZeroU64;
    /// # tokio_test::block_on(async {
    ///     use momento::simple_cache_client::SimpleCacheClient;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClient::new(auth_token, NonZeroU64::new(5).unwrap()).await.unwrap();
    ///     momento.create_cache(&cache_name).await;
    ///     momento.delete_cache(&cache_name).await;
    /// # })
    /// ```
    pub async fn delete_cache(&mut self, name: &str) -> Result<(), MomentoError> {
        utils::is_cache_name_valid(name)?;
        let request = Request::new(DeleteCacheRequest {
            cache_name: name.to_string(),
        });
        self.control_client.delete_cache(request).await?;
        Ok(())
    }

    /// Lists all Momento caches
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use std::num::NonZeroU64;
    /// # tokio_test::block_on(async {
    ///     use momento::simple_cache_client::SimpleCacheClient;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClient::new(auth_token, NonZeroU64::new(5).unwrap()).await.unwrap();
    ///     momento.create_cache(&cache_name).await;
    ///     let caches = momento.list_caches(None).await;
    ///     momento.delete_cache(&cache_name).await;
    /// # })
    /// ```
    pub async fn list_caches(
        &mut self,
        next_token: Option<&str>,
    ) -> Result<MomentoListCacheResult, MomentoError> {
        let request = Request::new(ListCachesRequest {
            next_token: next_token.unwrap_or_default().to_string(),
        });
        let res = self.control_client.list_caches(request).await?.into_inner();
        let caches = res
            .cache
            .iter()
            .map(|cache| MomentoCache {
                cache_name: cache.cache_name.to_string(),
            })
            .collect();
        let response = MomentoListCacheResult {
            caches,
            next_token: res.next_token,
        };
        Ok(response)
    }

    /// Creates a new Momento signing key
    ///
    /// # Arguments
    ///
    /// * `ttl_minutes` - key's time-to-live in minutes
    pub async fn create_signing_key(
        &mut self,
        ttl_minutes: u32,
    ) -> Result<MomentoCreateSigningKeyResponse, MomentoError> {
        let request = Request::new(CreateSigningKeyRequest { ttl_minutes });
        let res = self
            .control_client
            .create_signing_key(request)
            .await?
            .into_inner();
        let key: Value = serde_json::from_str(&res.key).unwrap();
        let obj = key.as_object().unwrap();
        let kid = obj.get("kid").unwrap();
        let response = MomentoCreateSigningKeyResponse {
            key_id: kid.as_str().unwrap().to_owned(),
            key: res.key,
            expires_at: res.expires_at,
            endpoint: self.data_endpoint.clone(),
        };
        Ok(response)
    }

    /// Revokes a Momento signing key, all tokens signed by which will be invalid
    ///
    /// # Arguments
    ///
    /// * `key_id` - the ID of the key to revoke
    pub async fn revoke_signing_key(&mut self, key_id: &str) -> Result<(), MomentoError> {
        utils::is_key_id_valid(key_id)?;
        let request = Request::new(RevokeSigningKeyRequest {
            key_id: key_id.to_string(),
        });
        self.control_client.revoke_signing_key(request).await?;
        Ok(())
    }

    /// Sets an item in a Momento Cache
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache
    /// * `cache_key`
    /// * `cache_body`
    /// * `ttl_seconds` - If None is passed, uses the client's default ttl
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use std::num::NonZeroU64;
    /// # tokio_test::block_on(async {
    ///     use momento::simple_cache_client::SimpleCacheClient;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClient::new(auth_token, NonZeroU64::new(30).unwrap()).await.unwrap();
    ///     momento.create_cache(&cache_name).await;
    ///     momento.set(&cache_name, "cache_key", "cache_value", None).await;
    ///
    ///     // overriding default ttl
    ///     momento.set(&cache_name, "cache_key", "cache_value", Some(NonZeroU64::new(10).unwrap())).await;
    ///     momento.delete_cache(&cache_name).await;
    /// # })
    /// ```
    pub async fn set<I: MomentoRequest>(
        &mut self,
        cache_name: &str,
        key: I,
        body: I,
        ttl_seconds: Option<NonZeroU64>,
    ) -> Result<MomentoSetResponse, MomentoError> {
        utils::is_cache_name_valid(cache_name)?;
        let temp_ttl = ttl_seconds.unwrap_or(self.item_default_ttl_seconds);
        let ttl_to_use = match utils::is_ttl_valid(&temp_ttl) {
            Ok(_) => temp_ttl.get() * 1000_u64,
            Err(e) => return Err(e),
        };
        let mut request = tonic::Request::new(SetRequest {
            cache_key: key.into_bytes(),
            cache_body: body.into_bytes(),
            ttl_milliseconds: ttl_to_use,
        });
        request.metadata_mut().append(
            "cache",
            tonic::metadata::AsciiMetadataValue::from_str(cache_name).unwrap(),
        );
        let _ = self.data_client.set(request).await?;
        Ok(MomentoSetResponse {
            result: MomentoSetStatus::OK,
        })
    }

    /// Gets an item from a Momento Cache
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache
    /// * `key` - cache key
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use std::num::NonZeroU64;
    /// # tokio_test::block_on(async {
    ///     use std::env;
    ///     use momento::{response::cache_get_response::MomentoGetStatus, simple_cache_client::SimpleCacheClient};
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClient::new(auth_token, NonZeroU64::new(30).unwrap()).await.unwrap();
    ///     momento.create_cache(&cache_name).await;
    ///     let resp = momento.get(&cache_name, "cache_key").await.unwrap();
    ///     match resp.result {
    ///         MomentoGetStatus::HIT => println!("cache hit!"),
    ///         MomentoGetStatus::MISS => println!("cache miss"),
    ///         _ => println!("error occurred")
    ///     };
    ///
    ///     println!("cache value: {}", resp.as_string());
    ///     momento.delete_cache(&cache_name).await;
    /// # })
    /// ```
    pub async fn get<I: MomentoRequest>(
        &mut self,
        cache_name: &str,
        key: I,
    ) -> Result<MomentoGetResponse, MomentoError> {
        utils::is_cache_name_valid(cache_name)?;
        let mut request = tonic::Request::new(GetRequest {
            cache_key: key.into_bytes(),
        });
        request.metadata_mut().append(
            "cache",
            tonic::metadata::AsciiMetadataValue::from_str(cache_name).unwrap(),
        );
        let response = self.data_client.get(request).await?.into_inner();
        match response.result() {
            ECacheResult::Hit => Ok(MomentoGetResponse {
                result: MomentoGetStatus::HIT,
                value: response.cache_body,
            }),
            ECacheResult::Miss => Ok(MomentoGetResponse {
                result: MomentoGetStatus::MISS,
                value: response.cache_body,
            }),
            _ => todo!(),
        }
    }
}
