use chrono::{DateTime, NaiveDateTime, Utc};
use momento_protos::{
    cache_client::scs_client::*,
    cache_client::*,
    control_client::{
        scs_control_client::ScsControlClient, CreateCacheRequest, CreateSigningKeyRequest,
        DeleteCacheRequest, ListCachesRequest, ListSigningKeysRequest, RevokeSigningKeyRequest,
    },
};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::time::Duration;
use tonic::{codegen::InterceptedService, transport::Channel, Request};

use crate::{endpoint_resolver::MomentoEndpointsResolver, MomentoResult};
use crate::{grpc::header_interceptor::HeaderInterceptor, utils::connect_channel_lazily};

use crate::response::{
    MomentoCache, MomentoCreateSigningKeyResponse, MomentoDictionaryFetchResponse,
    MomentoDictionaryFetchStatus, MomentoDictionaryGetResponse, MomentoDictionaryGetStatus,
    MomentoDictionaryIncrementResponse, MomentoDictionarySetResponse, MomentoDictionarySetStatus,
    MomentoError, MomentoGetResponse, MomentoGetStatus, MomentoListCacheResult,
    MomentoListSigningKeyResult, MomentoSetFetchResponse, MomentoSetResponse, MomentoSetStatus,
    MomentoSigningKey,
};
use crate::utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
}

impl<T> IntoBytes for T
where
    T: Into<Vec<u8>>,
{
    fn into_bytes(self) -> Vec<u8> {
        self.into()
    }
}

/// Represents the desired behavior for managing the TTL on collection objects.
///
/// For cache operations that modify a collection (dictionaries, lists, or sets), there
/// are a few things to consider. The first time the collection is created, we need to
/// set a TTL on it. For subsequent operations that modify the collection you may choose
/// to update the TTL in order to prolong the life of the cached collection object, or
/// you may choose to leave the TTL unmodified in order to ensure that the collection
/// expires at the original TTL.
///
/// The default behaviour is to refresh the TTL (to prolong the life of the collection)
/// each time it is written using the client's default item TTL.
#[derive(Copy, Clone, Debug)]
pub struct CollectionTtl {
    ttl: Option<Duration>,
    refresh: bool,
}

impl CollectionTtl {
    /// Create a collection TTL with the provided `ttl` and `refresh` settings.
    pub const fn new(ttl: Option<Duration>, refresh: bool) -> Self {
        Self { ttl, refresh }
    }

    /// Create a collection TTL that updates the TTL for the collection any time it is
    /// modified.
    ///
    /// If `ttl` is `None` then the default item TTL for the client will be used.
    pub fn refresh_on_update(ttl: impl Into<Option<Duration>>) -> Self {
        Self::new(ttl.into(), true)
    }

    /// Create a collection TTL that will not refresh the TTL for the collection when
    /// it is updated.
    ///
    /// Use this if you want to be sure that the collection expires at the originally
    /// specified time, even if you make modifications to the value of the collection.
    ///
    /// The TTL will still be used when a new collection is created. If `ttl` is `None`
    /// then the default item TTL for the client will be used.
    pub fn initialize_only(ttl: impl Into<Option<Duration>>) -> Self {
        Self::new(ttl.into(), false)
    }

    /// Create a collection TTL that updates the TTL for the collection only if an
    /// explicit `ttl` is provided here.
    pub fn refresh_if_provided(ttl: impl Into<Option<Duration>>) -> Self {
        let ttl = ttl.into();
        Self::new(ttl, ttl.is_some())
    }

    /// Return a new collection TTL which uses the same TTL but refreshes on updates.
    pub fn with_refresh_on_update(self) -> Self {
        Self::new(self.ttl(), true)
    }

    /// Return a new collection TTL which uses the same TTL but does not refresh on
    /// updates.
    pub fn with_no_refresh_on_update(self) -> Self {
        Self::new(self.ttl(), false)
    }

    /// Return a new collecton TTL which has the same refresh behaviour but uses the
    /// provided TTL.
    pub fn with_ttl(self, ttl: impl Into<Option<Duration>>) -> Self {
        Self::new(ttl.into(), self.refresh())
    }

    /// The [`Duration`] after which the cached collection should be expired from the
    /// cache.
    ///
    /// If `None`, the default item TTL for the client will be used.
    pub fn ttl(&self) -> Option<Duration> {
        self.ttl
    }

    /// Whether the collection's TTL will be refreshed on every update.
    ///
    /// If true, this will extend the time at which the collection would expire when
    /// an update operation happens. Otherwise, the collection's TTL will only be set
    /// when it is initially created.
    pub fn refresh(&self) -> bool {
        self.refresh
    }
}

impl Default for CollectionTtl {
    fn default() -> Self {
        Self::new(None, true)
    }
}

#[derive(Clone)]
pub struct SimpleCacheClientBuilder {
    data_endpoint: String,
    control_channel: Channel,
    data_channel: Channel,
    auth_token: String,
    default_ttl: Duration,
    user_agent_name: String,
}

fn request_meta_data<T>(
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
    /// * `default_ttl` - Default TTL for items put into a cache.
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use momento::SimpleCacheClientBuilder;
    ///     use std::env;
    ///     use std::time::Duration;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(30))
    ///         .expect("could not create a client")
    ///         .build();
    /// # })
    /// ```
    pub fn new(auth_token: String, default_ttl: Duration) -> Result<Self, MomentoError> {
        SimpleCacheClientBuilder::new_with_explicit_agent_name(auth_token, default_ttl, "sdk", None)
    }

    /// Like new() above, but requires a momento_endpoint.
    // TODO: Update the documentation and tests and deprecate the existing new method. This will be
    // done once we start vending out tokens with no endpoints and have published the new momento
    // endpoints.
    pub fn new_with_endpoint(
        auth_token: String,
        default_ttl: Duration,
        momento_endpoint: String,
    ) -> Result<Self, MomentoError> {
        SimpleCacheClientBuilder::new_with_explicit_agent_name(
            auth_token,
            default_ttl,
            "sdk",
            Some(momento_endpoint),
        )
    }

    /// Like new() above, but used for naming integrations.
    pub fn new_with_explicit_agent_name(
        auth_token: String,
        default_ttl: Duration,
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

        match utils::is_ttl_valid(default_ttl) {
            Ok(_) => Ok(Self {
                data_endpoint: momento_endpoints.data_endpoint.hostname,
                control_channel,
                data_channel,
                auth_token,
                default_ttl,
                user_agent_name: user_agent_name.to_string(),
            }),
            Err(e) => Err(e),
        }
    }

    pub fn default_ttl(mut self, ttl: Duration) -> Result<Self, MomentoError> {
        utils::is_ttl_valid(ttl)?;
        self.default_ttl = ttl;
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
            item_default_ttl: self.default_ttl,
        }
    }
}

#[derive(Clone)]
pub struct SimpleCacheClient {
    data_endpoint: String,
    control_client: ScsControlClient<InterceptedService<Channel, HeaderInterceptor>>,
    data_client: ScsClient<InterceptedService<Channel, HeaderInterceptor>>,
    item_default_ttl: Duration,
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
    /// use std::time::Duration;
    /// # tokio_test::block_on(async {
    ///     use momento::SimpleCacheClientBuilder;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(5))
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
    /// use std::time::Duration;
    /// # tokio_test::block_on(async {
    ///     use momento::SimpleCacheClientBuilder;
    ///     let auth_token = std::env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(5))
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
            next_token: next_token.unwrap_or_default(),
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
    /// * `ttl_minutes` - key's time-to-live.
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
        let key: Value =
            serde_json::from_str(&res.key).expect("failed to parse key from json string");
        let obj = key
            .as_object()
            .expect("couldn't cast json value to a Map<String, Value>");
        let kid = obj
            .get("kid")
            .expect("object didn't contain key 'kid', this is required");
        let response = MomentoCreateSigningKeyResponse {
            key_id: kid.as_str().expect("'kid' not a valid str").to_owned(),
            key: res.key,
            expires_at: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(res.expires_at as i64, 0)
                    .expect("couldn't parse from timestamp"),
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
    /// use std::time::Duration;
    /// # tokio_test::block_on(async {
    ///     use momento::SimpleCacheClientBuilder;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(5))
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
                    NaiveDateTime::from_timestamp_opt(signing_key.expires_at as i64, 0)
                        .expect("couldn't parse timestamp from signing key"),
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
    /// * `cache_key` - key of entry within the cache.
    /// * `cache_body` - data stored within the cache entry.
    /// * `ttl` - The TTL to use for the
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use std::time::Duration;
    /// # tokio_test::block_on(async {
    ///     use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(30))
    ///         .expect("could not create a client")
    ///         .build();
    ///     momento.create_cache(&cache_name).await;
    ///     momento.set(&cache_name, "cache_key", "cache_value", None).await;
    ///
    ///     // overriding default ttl
    ///     momento.set(&cache_name, "cache_key", "cache_value", Duration::from_secs(10)).await;
    ///     momento.delete_cache(&cache_name).await;
    /// # })
    /// ```
    pub async fn set(
        &mut self,
        cache_name: &str,
        key: impl IntoBytes,
        body: impl IntoBytes,
        ttl: impl Into<Option<Duration>>,
    ) -> Result<MomentoSetResponse, MomentoError> {
        utils::is_cache_name_valid(cache_name)?;

        let mut request = tonic::Request::new(SetRequest {
            cache_key: key.into_bytes(),
            cache_body: body.into_bytes(),
            ttl_milliseconds: self.expand_ttl_ms(ttl.into())?,
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
    /// use std::time::Duration;
    /// # tokio_test::block_on(async {
    ///     use std::env;
    ///     use momento::{response::MomentoGetStatus, SimpleCacheClientBuilder};
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(30))
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
    pub async fn get(
        &mut self,
        cache_name: &str,
        key: impl IntoBytes,
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

    /// Sets dictionary items in a Momento Cache
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache
    /// * `dictionary_name` - name of the dictionary
    /// * `dictionary` - hashmap of dictionary key-value pairs
    /// * `policy` - TTL policy to use.
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use std::time::Duration;
    /// # tokio_test::block_on(async {
    ///     use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///     use std::collections::HashMap;
    ///     use std::env;
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(30))
    ///         .expect("could not create a client")
    ///         .build();
    ///     momento.create_cache(&cache_name).await;
    ///
    ///     let mut dictionary = HashMap::new();
    ///     dictionary.insert("key1".to_string(), "value1".to_string());
    ///     dictionary.insert("key2".to_string(), "value2".to_string());
    ///
    ///     let dictionary_name = Uuid::new_v4().to_string();
    ///
    ///     momento.dictionary_set(&cache_name, &*dictionary_name, dictionary, CollectionTtl::default()).await;
    ///     momento.delete_cache(&cache_name).await;
    /// # })
    /// ```
    pub async fn dictionary_set<K: IntoBytes, V: IntoBytes>(
        &mut self,
        cache_name: &str,
        dictionary_name: impl IntoBytes,
        dictionary: HashMap<K, V>,
        policy: CollectionTtl,
    ) -> Result<MomentoDictionarySetResponse, MomentoError> {
        utils::is_cache_name_valid(cache_name)?;

        let mut dictionary = dictionary;
        let items = dictionary
            .drain()
            .map(|(k, v)| DictionaryFieldValuePair {
                field: k.into_bytes(),
                value: v.into_bytes(),
            })
            .collect();

        let mut request = tonic::Request::new(DictionarySetRequest {
            dictionary_name: dictionary_name.into_bytes(),
            items,
            ttl_milliseconds: self.expand_ttl_ms(policy.ttl())?,
            refresh_ttl: policy.refresh(),
        });
        request_meta_data(&mut request, cache_name)?;
        let _ = self.data_client.dictionary_set(request).await?;

        Ok(MomentoDictionarySetResponse {
            result: MomentoDictionarySetStatus::OK,
        })
    }

    /// Gets dictionary fields from a Momento Cache
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache
    /// * `dictionary_name` - name of dictionary
    /// * `fields` - dictionary keys to read
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use std::time::Duration;
    /// # tokio_test::block_on(async {
    ///     use std::env;
    ///     use momento::{
    ///         response::MomentoDictionaryGetStatus,
    ///         SimpleCacheClientBuilder,
    ///     };
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(30))
    ///         .expect("could not create a client")
    ///         .build();
    ///     momento.create_cache(&cache_name).await;
    ///
    ///     let dictionary_name = Uuid::new_v4().to_string();
    ///
    ///     let resp = momento
    ///         .dictionary_get(&cache_name, &*dictionary_name, vec!["key1", "key2"])
    ///         .await
    ///         .unwrap();
    ///
    ///     match resp.result {
    ///         MomentoDictionaryGetStatus::FOUND => println!("dictionary found!"),
    ///         MomentoDictionaryGetStatus::MISSING => println!("dictionary missing!"),
    ///         _ => println!("error occurred")
    ///     };
    ///
    ///
    ///     if let Some(dictionary) = resp.dictionary {
    ///         println!("dictionary entries:");
    ///         for (key, value) in dictionary.iter() {
    ///             println!("{:?} => {:?}", key, value);
    ///         }
    ///     }
    ///
    ///     momento.delete_cache(&cache_name).await;
    /// # })
    /// ```
    pub async fn dictionary_get<K: IntoBytes>(
        &mut self,
        cache_name: &str,
        dictionary: impl IntoBytes,
        fields: Vec<K>,
    ) -> Result<MomentoDictionaryGetResponse, MomentoError> {
        utils::is_cache_name_valid(cache_name)?;
        let mut fields = fields;

        let mut fields: Vec<Vec<u8>> = fields.drain(..).map(|f| f.into_bytes()).collect();

        let mut request = tonic::Request::new(DictionaryGetRequest {
            dictionary_name: dictionary.into_bytes(),
            fields: fields.clone(),
        });
        request_meta_data(&mut request, cache_name)?;
        let response = self.data_client.dictionary_get(request).await?.into_inner();
        match response.dictionary {
            Some(dictionary_get_response::Dictionary::Found(mut response)) => {
                let mut dictionary = HashMap::new();

                // map the dictionary response parts to get responses
                for (field, item) in fields.drain(..).zip(response.items.drain(..)) {
                    if item.result() == ECacheResult::Hit {
                        dictionary.insert(field, item.cache_body);
                    }
                }

                Ok(MomentoDictionaryGetResponse {
                    result: MomentoDictionaryGetStatus::FOUND,
                    dictionary: Some(dictionary),
                })
            }
            Some(dictionary_get_response::Dictionary::Missing(_)) | None => {
                Ok(MomentoDictionaryGetResponse {
                    result: MomentoDictionaryGetStatus::MISSING,
                    dictionary: None,
                })
            }
        }
    }

    /// Fetches a dictionary from a Momento Cache
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache
    /// * `dictionary_name` - name of dictionary
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use std::time::Duration;
    /// # tokio_test::block_on(async {
    ///     use momento::{
    ///         response::MomentoDictionaryFetchStatus,
    ///         SimpleCacheClientBuilder,
    ///     };
    ///     let auth_token = std::env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(30))
    ///         .expect("could not create a client")
    ///         .build();
    ///     momento.create_cache(&cache_name).await;
    ///
    ///     let dictionary_name = Uuid::new_v4().to_string();
    ///
    ///     let resp = momento.dictionary_fetch(&cache_name, &*dictionary_name).await.unwrap();
    ///
    ///     match resp.result {
    ///         MomentoDictionaryFetchStatus::FOUND => println!("dictionary found!"),
    ///         MomentoDictionaryFetchStatus::MISSING => println!("dictionary missing!"),
    ///         _ => println!("error occurred")
    ///     };
    ///
    ///
    ///     if let Some(dictionary) = resp.dictionary {
    ///         println!("dictionary entries:");
    ///         for (key, value) in dictionary.iter() {
    ///             println!("{:?} => {:?}", key, value);
    ///         }
    ///     }
    ///
    ///     momento.delete_cache(&cache_name).await;
    /// # })
    /// ```
    pub async fn dictionary_fetch(
        &mut self,
        cache_name: &str,
        dictionary: impl IntoBytes,
    ) -> Result<MomentoDictionaryFetchResponse, MomentoError> {
        utils::is_cache_name_valid(cache_name)?;

        let mut request = tonic::Request::new(DictionaryFetchRequest {
            dictionary_name: dictionary.into_bytes(),
        });
        request_meta_data(&mut request, cache_name)?;
        let response = self
            .data_client
            .dictionary_fetch(request)
            .await?
            .into_inner();
        match response.dictionary {
            Some(dictionary_fetch_response::Dictionary::Found(response)) => {
                Ok(MomentoDictionaryFetchResponse {
                    result: MomentoDictionaryFetchStatus::FOUND,
                    dictionary: Some(
                        response
                            .items
                            // Consume the payload vectors by value to avoid extra copies
                            .into_iter()
                            .map(|pair| (pair.field, pair.value))
                            .collect(),
                    ),
                })
            }
            Some(dictionary_fetch_response::Dictionary::Missing(_)) | None => {
                Ok(MomentoDictionaryFetchResponse {
                    result: MomentoDictionaryFetchStatus::MISSING,
                    dictionary: None,
                })
            }
        }
    }

    /// Delete entire dictionary or some dictionary fields from a Momento Cache
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache
    /// * `dictionary_name` - name of dictionary
    /// * `fields` - dictionary keys to delete
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use std::time::Duration;
    /// # tokio_test::block_on(async {
    ///     use std::env;
    ///     use momento::{Fields, SimpleCacheClientBuilder};
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(30))
    ///         .expect("could not create a client")
    ///         .build();
    ///     momento.create_cache(&cache_name).await;
    ///
    ///     let dictionary_name = Uuid::new_v4().to_string();
    ///
    ///     // remove some fields
    ///     let resp = momento.dictionary_delete(
    ///         &cache_name,
    ///         &*dictionary_name,
    ///         Fields::Some(vec!["field_1"]),
    ///     ).await.unwrap();
    ///
    ///     // remove entire dictionary
    ///     let resp = momento.dictionary_delete(
    ///         &cache_name,
    ///         &*dictionary_name,
    ///         Fields::<Vec<u8>>::All,
    ///     ).await.unwrap();
    ///
    ///     momento.delete_cache(&cache_name).await;
    /// # })
    /// ```
    pub async fn dictionary_delete<K: IntoBytes>(
        &mut self,
        cache_name: &str,
        dictionary: impl IntoBytes,
        fields: Fields<K>,
    ) -> Result<(), MomentoError> {
        utils::is_cache_name_valid(cache_name)?;

        let mut request = match fields {
            Fields::Some(mut fields) => {
                let fields: Vec<Vec<u8>> = fields.drain(..).map(|f| f.into_bytes()).collect();
                tonic::Request::new(DictionaryDeleteRequest {
                    dictionary_name: dictionary.into_bytes(),
                    delete: Some(dictionary_delete_request::Delete::Some(
                        dictionary_delete_request::Some { fields },
                    )),
                })
            }
            Fields::All => tonic::Request::new(DictionaryDeleteRequest {
                dictionary_name: dictionary.into_bytes(),
                delete: Some(dictionary_delete_request::Delete::All(
                    dictionary_delete_request::All {},
                )),
            }),
        };
        request_meta_data(&mut request, cache_name)?;
        self.data_client.dictionary_delete(request).await?;
        Ok(())
    }

    /// Increment a value within a dictionary.
    ///
    /// If the dictionary already exists, then the value will be incremented. If either
    /// of the dictionary or field do not exist, then they will be created and initialized
    /// to 0, before being incremented by `amount`.
    ///
    /// Returns the current value of the field within the dictionary after being incremented.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache.
    /// * `dictionary` - name of dictionary.
    /// * `field` - name of the field to increment from the dictionary.
    /// * `amount` - quantity to add to the value.
    /// * `policy` - the TTL policy to use.
    ///
    /// # Example
    /// ```
    /// # tokio_test::block_on(async {
    /// use uuid::Uuid;
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let auth_token = std::env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    /// let cache_name = Uuid::new_v4().to_string();
    /// let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(30))
    ///     .expect("unable to create momento client")
    ///     .build();
    /// momento.create_cache(&cache_name)
    ///     .await
    ///     .expect("Failed to create cache");
    ///
    /// let dictionary = Uuid::new_v4().to_string();
    /// let field = Uuid::new_v4().to_string();
    ///
    /// let value = momento
    ///     .dictionary_increment(&cache_name, dictionary, field, 10, CollectionTtl::default())
    ///     .await
    ///     .expect("Failed to increment dictionary key")
    ///     .value;
    ///
    /// println!("Dicationary key has been incremented to {}", value);
    ///
    /// momento.delete_cache(&cache_name)
    ///     .await
    ///     .expect("Failed to delete cache");
    /// # })
    /// ```
    pub async fn dictionary_increment(
        &mut self,
        cache_name: &str,
        dictionary: impl IntoBytes,
        field: impl IntoBytes,
        amount: i64,
        policy: CollectionTtl,
    ) -> Result<MomentoDictionaryIncrementResponse, MomentoError> {
        utils::is_cache_name_valid(cache_name)?;

        let mut request = tonic::Request::new(DictionaryIncrementRequest {
            dictionary_name: dictionary.into_bytes(),
            field: field.into_bytes(),
            amount,
            ttl_milliseconds: self.expand_ttl_ms(policy.ttl())?,
            refresh_ttl: policy.refresh(),
        });
        request_meta_data(&mut request, cache_name)?;

        let response = self
            .data_client
            .dictionary_increment(request)
            .await?
            .into_inner();

        Ok(MomentoDictionaryIncrementResponse {
            value: response.value,
        })
    }

    /// Fetches a set from a Momento Cache.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache.
    /// * `set_name` - name of the set.
    ///
    /// # Example
    /// ```
    /// # tokio_test::block_on(async {
    /// use uuid::Uuid;
    /// use std::time::Duration;
    /// use momento::SimpleCacheClientBuilder;
    ///
    /// let auth_token = std::env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be defined");
    /// let cache_name = Uuid::new_v4().to_string();
    /// let set_name = Uuid::new_v4().to_string();
    ///
    /// let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(30))
    ///     .expect("could not create a client")
    ///     .build();
    ///
    /// momento.create_cache(&cache_name).await;
    ///
    /// let response = momento
    ///     .set_fetch(&cache_name, set_name)
    ///     .await
    ///     .expect("Failed to fetch the set");
    /// if let Some(set) = response.value {
    ///     println!("set entries:");
    ///     for entry in &set {
    ///         println!("{:?}", entry);
    ///     }
    /// } else {
    ///     println!("set not found!");
    /// }
    ///
    /// momento
    ///     .delete_cache(&cache_name)
    ///     .await
    ///     .expect("Failed to delete the cache");
    /// # })
    /// ```
    pub async fn set_fetch(
        &mut self,
        cache_name: &str,
        set_name: impl IntoBytes,
    ) -> Result<MomentoSetFetchResponse, MomentoError> {
        use set_fetch_response::Set;

        utils::is_cache_name_valid(cache_name)?;

        let mut request = tonic::Request::new(SetFetchRequest {
            set_name: set_name.into_bytes(),
        });
        request_meta_data(&mut request, cache_name)?;

        let response = self.data_client.set_fetch(request).await?.into_inner();
        Ok(MomentoSetFetchResponse {
            value: response.set.and_then(|set| match set {
                Set::Found(found) => Some(found.elements.into_iter().collect()),
                Set::Missing(_) => None,
            }),
        })
    }

    /// Unions a set with one present within a Momento cache.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache.
    /// * `set_name` - name of the set.
    /// * `elements` - elements to be unioned with the existing set within the cache.
    /// * `policy` - the TTL policy to use.
    ///
    /// # Example
    /// ```
    /// # tokio_test::block_on(async {
    /// use uuid::Uuid;
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let auth_token = std::env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be defined");
    /// let cache_name = Uuid::new_v4().to_string();
    /// let set_name = Uuid::new_v4().to_string();
    ///
    /// let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(30))
    ///     .expect("could not create a client")
    ///     .build();
    ///
    /// momento
    ///     .create_cache(&cache_name)
    ///     .await
    ///     .expect("unable to create the cache");
    ///
    /// momento
    ///     .set_union(&cache_name, set_name, vec!["a", "b", "c"], CollectionTtl::default())
    ///     .await
    ///     .expect("Failed to run a set union");
    ///
    /// momento
    ///     .delete_cache(&cache_name)
    ///     .await
    ///     .expect("Failed to delete the cache");
    /// # });
    /// ```
    pub async fn set_union<E: IntoBytes>(
        &mut self,
        cache_name: &str,
        set_name: impl IntoBytes,
        elements: Vec<E>,
        policy: CollectionTtl,
    ) -> Result<(), MomentoError> {
        utils::is_cache_name_valid(cache_name)?;

        let mut request = tonic::Request::new(SetUnionRequest {
            set_name: set_name.into_bytes(),
            elements: elements.into_iter().map(|e| e.into_bytes()).collect(),
            ttl_milliseconds: self.expand_ttl_ms(policy.ttl())?,
            refresh_ttl: policy.refresh(),
        });
        request_meta_data(&mut request, cache_name)?;

        let _ = self.data_client.set_union(request).await?.into_inner();
        Ok(())
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
    /// use std::time::Duration;
    /// # tokio_test::block_on(async {
    ///     use std::env;
    ///     use momento::{response::MomentoGetStatus, CollectionTtl, SimpleCacheClientBuilder};
    ///     let auth_token = env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");
    ///     let cache_name = Uuid::new_v4().to_string();
    ///     let mut momento = SimpleCacheClientBuilder::new(auth_token, Duration::from_secs(30))
    ///         .expect("could not create a client")
    ///         .build();
    ///     momento.create_cache(&cache_name).await;
    ///     let result = momento.set(&cache_name, "cache_key", "cache_value", None).await;
    ///     momento.delete(&cache_name, "cache_key").await.unwrap();
    ///     momento.delete_cache(&cache_name).await;
    /// # })
    /// ```
    pub async fn delete(
        &mut self,
        cache_name: &str,
        key: impl IntoBytes,
    ) -> Result<(), MomentoError> {
        utils::is_cache_name_valid(cache_name)?;
        let mut request = tonic::Request::new(DeleteRequest {
            cache_key: key.into_bytes(),
        });
        request_meta_data(&mut request, cache_name)?;
        self.data_client.delete(request).await?.into_inner();
        Ok(())
    }

    fn expand_ttl_ms(&self, ttl: Option<Duration>) -> MomentoResult<u64> {
        let ttl = ttl.unwrap_or(self.item_default_ttl);
        utils::is_ttl_valid(ttl)?;

        Ok(ttl.as_millis().try_into().unwrap_or(i64::MAX as u64))
    }
}

/// An enum that is used to indicate if an operation should apply to all fields
/// or just some fields of a dictionary.
pub enum Fields<K> {
    All,
    Some(Vec<K>),
}
