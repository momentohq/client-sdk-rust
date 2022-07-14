use chrono::{DateTime, NaiveDateTime, Utc};
use momento_protos::{
    cache_client::{scs_client::ScsClient, DeleteRequest, ECacheResult, GetRequest, SetRequest},
    control_client::{
        scs_control_client::ScsControlClient, CreateCacheRequest, CreateSigningKeyRequest,
        DeleteCacheRequest, ListCachesRequest, ListSigningKeysRequest, RevokeSigningKeyRequest,
    },
};
use serde_json::Value;
use std::convert::TryFrom;
use std::num::NonZeroU64;
use tonic::{codegen::InterceptedService, transport::Channel, Request};

use crate::endpoint_resolver::MomentoEndpointsResolver;
use crate::{grpc::header_interceptor::HeaderInterceptor, utils::connect_channel_lazily};

use crate::response::{
    cache_get_response::MomentoGetResponse,
    cache_get_response::MomentoGetStatus,
    cache_set_response::{MomentoSetResponse, MomentoSetStatus},
    create_signing_key_response::MomentoCreateSigningKeyResponse,
    error::MomentoError,
    list_cache_response::{MomentoCache, MomentoListCacheResult},
    list_signing_keys_response::{MomentoListSigningKeyResult, MomentoSigningKey},
};
use crate::utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    user_agent_name: String,
}

pub fn request_meta_data<T>(
    request: &mut tonic::Request<T>,
    cache_name: &str,
) -> Result<(), MomentoError> {
    tonic::metadata::AsciiMetadataValue::try_from(cache_name)
        .map(|value| {
            request.metadata_mut().append("cache", value);
        })
        .map_err(|e| {
            MomentoError::InvalidArgument(format!(
                "Could not treat cache name as a header value: {}",
                e
            ))
        })
}

impl SimpleCacheClientBuilder {
    /// Returns a builder which can produce an instance of a Momento client
    ///
    /// # Arguments
    ///
    /// * `auth_token` - Momento Token
    /// * `item_default_ttl_seconds` - Default TTL for items put into a cache.
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use momento::simple_cache_client::SimpleCacheClientBuilder;
    ///     use std::env;
    ///     use std::num::NonZeroU64;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let default_ttl = 30;
    ///     let momento = SimpleCacheClientBuilder::new(auth_token, NonZeroU64::new(default_ttl).unwrap())
    ///         .expect("could not create a client")
    ///         .build();
    /// # })
    /// ```
    pub fn new(auth_token: String, default_ttl_seconds: NonZeroU64) -> Result<Self, MomentoError> {
        SimpleCacheClientBuilder::new_with_explicit_agent_name(
            auth_token,
            default_ttl_seconds,
            "sdk",
            None,
        )
    }

    /// Like new() above, but requires a momento_endpoint.
    // TODO: Update the documentation and tests and deprecate the existing new method. This will be
    // done once we start vending out tokens with no endpoints and have published the new momento
    // endpoints.
    pub fn new_with_endpoint(
        auth_token: String,
        default_ttl_seconds: NonZeroU64,
        momento_endpoint: String,
    ) -> Result<Self, MomentoError> {
        SimpleCacheClientBuilder::new_with_explicit_agent_name(
            auth_token,
            default_ttl_seconds,
            "sdk",
            Some(momento_endpoint),
        )
    }

    /// Like new() above, but used for naming integrations.
    pub fn new_with_explicit_agent_name(
        auth_token: String,
        default_ttl_seconds: NonZeroU64,
        user_agent_name: &str,
        momento_endpoint: Option<String>,
    ) -> Result<Self, MomentoError> {
        let momento_endpoints =
            match MomentoEndpointsResolver::resolve(&auth_token, momento_endpoint) {
                Ok(endpoints) => endpoints,
                Err(e) => return Err(e),
            };
        log::debug!("connecting to endpoints: {:?}", momento_endpoints);

        let control_channel = connect_channel_lazily(&momento_endpoints.control_endpoint.url)?;
        let data_channel = connect_channel_lazily(&momento_endpoints.data_endpoint.url)?;

        match utils::is_ttl_valid(&default_ttl_seconds) {
            Ok(_) => Ok(Self {
                data_endpoint: momento_endpoints.data_endpoint.hostname,
                control_channel,
                data_channel,
                auth_token,
                default_ttl_seconds,
                user_agent_name: user_agent_name.to_string(),
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
        let agent_value = format!(
            "rust-{agent_name}:{version}",
            agent_name = self.user_agent_name,
            version = VERSION
        );
        let control_interceptor = InterceptedService::new(
            self.control_channel,
            HeaderInterceptor::new(&self.auth_token, &agent_value),
        );
        let control_client = ScsControlClient::new(control_interceptor);

        let data_interceptor = InterceptedService::new(
            self.data_channel,
            HeaderInterceptor::new(&self.auth_token, &agent_value),
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
    control_client: ScsControlClient<InterceptedService<Channel, HeaderInterceptor>>,
    data_client: ScsClient<InterceptedService<Channel, HeaderInterceptor>>,
    item_default_ttl_seconds: NonZeroU64,
}

impl SimpleCacheClient {
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
    ///     use momento::simple_cache_client::SimpleCacheClientBuilder;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, NonZeroU64::new(5).unwrap())
    ///         .expect("could not create a client")
    ///         .build();
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
    ///     use momento::simple_cache_client::SimpleCacheClientBuilder;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, NonZeroU64::new(5).unwrap())
    ///         .expect("could not create a client")
    ///         .build();
    ///     momento.create_cache(&cache_name).await;
    ///     let caches = momento.list_caches(None).await;
    ///     momento.delete_cache(&cache_name).await;
    /// # })
    /// ```
    pub async fn list_caches(
        &mut self,
        next_token: Option<String>,
    ) -> Result<MomentoListCacheResult, MomentoError> {
        let request = Request::new(ListCachesRequest {
            next_token: next_token.unwrap_or_else(|| "".to_string()),
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
            next_token: match res.next_token.is_empty() {
                true => None,
                false => Some(res.next_token),
            },
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
            expires_at: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(res.expires_at as i64, 0),
                Utc,
            ),
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

    /// Lists all Momento signing keys for a user
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use std::num::NonZeroU64;
    /// # tokio_test::block_on(async {
    ///     use momento::simple_cache_client::SimpleCacheClientBuilder;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, NonZeroU64::new(5).unwrap())
    ///         .expect("could not create a client")
    ///         .build();
    ///     let ttl_minutes = 10;
    ///     momento.create_signing_key(ttl_minutes).await;
    ///     let keys = momento.list_signing_keys(None).await;
    /// # })
    /// ```
    pub async fn list_signing_keys(
        &mut self,
        next_token: Option<&str>,
    ) -> Result<MomentoListSigningKeyResult, MomentoError> {
        let request = Request::new(ListSigningKeysRequest {
            next_token: next_token.unwrap_or_default().to_string(),
        });
        let res = self
            .control_client
            .list_signing_keys(request)
            .await?
            .into_inner();
        let signing_keys = res
            .signing_key
            .iter()
            .map(|signing_key| MomentoSigningKey {
                key_id: signing_key.key_id.to_string(),
                expires_at: DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(signing_key.expires_at as i64, 0),
                    Utc,
                ),
                endpoint: self.data_endpoint.clone(),
            })
            .collect();
        let response = MomentoListSigningKeyResult {
            signing_keys,
            next_token: res.next_token,
        };
        Ok(response)
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
    ///     use momento::simple_cache_client::SimpleCacheClientBuilder;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, NonZeroU64::new(30).unwrap())
    ///         .expect("could not create a client")
    ///         .build();
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
        request_meta_data(&mut request, cache_name)?;
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
    ///     use momento::{response::cache_get_response::MomentoGetStatus, simple_cache_client::SimpleCacheClientBuilder};
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, NonZeroU64::new(30).unwrap())
    ///         .expect("could not create a client")
    ///         .build();
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
        request_meta_data(&mut request, cache_name)?;
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

    /// Deletes an item from a Momento Cache
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
    ///     use momento::{response::cache_get_response::MomentoGetStatus, simple_cache_client::SimpleCacheClientBuilder};
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, NonZeroU64::new(30).unwrap())
    ///         .expect("could not create a client")
    ///         .build();
    ///     momento.create_cache(&cache_name).await;
    ///     let result = momento.set(&cache_name, "cache_key", "cache_value", None).await;
    ///     momento.delete(&cache_name, "cache_key").await.unwrap();
    ///     momento.delete_cache(&cache_name).await;
    /// # })
    /// ```
    pub async fn delete<I: MomentoRequest>(
        &mut self,
        cache_name: &str,
        key: I,
    ) -> Result<(), MomentoError> {
        utils::is_cache_name_valid(cache_name)?;
        let mut request = tonic::Request::new(DeleteRequest {
            cache_key: key.into_bytes(),
        });
        request_meta_data(&mut request, cache_name)?;
        self.data_client.delete(request).await?.into_inner();
        Ok(())
    }
}
