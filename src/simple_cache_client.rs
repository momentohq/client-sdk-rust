use core::num::NonZeroU32;
use momento_protos::control_client::generate_api_token_request::Expiry;
use momento_protos::control_client::{FlushCacheRequest, GenerateApiTokenRequest};
use momento_protos::{
    cache_client::scs_client::*,
    cache_client::*,
    control_client::{
        scs_control_client::ScsControlClient, CreateCacheRequest, CreateSigningKeyRequest,
        DeleteCacheRequest, ListCachesRequest, ListSigningKeysRequest, RevokeSigningKeyRequest,
    },
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;
use std::ops::RangeBounds;
use std::time::{Duration, UNIX_EPOCH};
use tonic::{codegen::InterceptedService, transport::Channel, Request};

use crate::credential_provider::CredentialProvider;
use crate::requests::generate_api_token_request::TokenExpiry;
use crate::response::{
    ApiToken, DictionaryFetch, DictionaryGet, DictionaryPairs, Get, GetValue, ListCacheEntry,
    MomentoCache, MomentoCreateSigningKeyResponse, MomentoDeleteResponse,
    MomentoDictionaryDeleteResponse, MomentoDictionaryIncrementResponse,
    MomentoDictionarySetResponse, MomentoError, MomentoFlushCacheResponse,
    MomentoGenerateApiTokenResponse, MomentoListCacheResponse, MomentoListFetchResponse,
    MomentoListSigningKeyResult, MomentoSetDifferenceResponse, MomentoSetFetchResponse,
    MomentoSetResponse, MomentoSigningKey, MomentoSortedSetFetchResponse,
};
use crate::sorted_set;
use crate::utils::{self, base64_encode};
use crate::{grpc::header_interceptor::HeaderInterceptor, utils::connect_channel_lazily};
use crate::{utils::user_agent, MomentoResult};

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

fn request_meta_data<T>(request: &mut tonic::Request<T>, cache_name: &str) -> MomentoResult<()> {
    tonic::metadata::AsciiMetadataValue::try_from(cache_name)
        .map(|value| {
            request.metadata_mut().append("cache", value);
        })
        .map_err(|e| MomentoError::InvalidArgument {
            description: format!("Could not treat cache name as a header value: {e}").into(),
            source: Some(crate::ErrorSource::Unknown(Box::new(e))),
        })
}

impl SimpleCacheClientBuilder {
    /// Returns a builder which can produce an instance of a Momento client
    ///
    /// # Arguments
    ///
    /// * `credential_provider` - Momento Credential Provider
    /// * `default_ttl` - Default TTL for items put into a cache.
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use momento::{CredentialProviderBuilder, SimpleCacheClientBuilder};
    ///     use std::env;
    ///     use std::time::Duration;
    ///     let credential_provider = CredentialProviderBuilder::new_from_environment_variable("TEST_AUTH_TOKEN")
    ///         .build()
    ///         .expect("TEST_AUTH_TOKEN must be set");
    ///     let momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))
    ///         .expect("could not create a client")
    ///         .build();
    /// # })
    /// ```
    pub fn new(
        credential_provider: CredentialProvider,
        default_ttl: Duration,
    ) -> MomentoResult<Self> {
        SimpleCacheClientBuilder::new_with_explicit_agent_name(
            credential_provider,
            default_ttl,
            "sdk",
        )
    }

    /// Like new() above, but used for naming integrations.
    pub fn new_with_explicit_agent_name(
        credential_provider: CredentialProvider,
        default_ttl: Duration,
        user_agent_name: &str,
    ) -> MomentoResult<Self> {
        log::debug!(
            "connecting to cache endpoint: {:?}, connecting to control endpoint: {:?}",
            &credential_provider.cache_endpoint,
            &credential_provider.control_endpoint
        );

        let control_channel = connect_channel_lazily(&credential_provider.control_endpoint)?;
        let data_channel = connect_channel_lazily(&credential_provider.cache_endpoint)?;

        match utils::is_ttl_valid(default_ttl) {
            Ok(_) => Ok(Self {
                data_endpoint: credential_provider.cache_endpoint,
                control_channel,
                data_channel,
                auth_token: credential_provider.auth_token,
                default_ttl,
                user_agent_name: user_agent_name.to_string(),
            }),
            Err(e) => Err(e),
        }
    }

    pub fn default_ttl(mut self, ttl: Duration) -> MomentoResult<Self> {
        utils::is_ttl_valid(ttl)?;
        self.default_ttl = ttl;
        Ok(self)
    }

    pub fn build(self) -> SimpleCacheClient {
        let agent_value = user_agent(&self.user_agent_name);
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
    pub async fn create_cache(&mut self, name: &str) -> MomentoResult<()> {
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
    /// # fn main() -> anyhow::Result<()> {
    /// # tokio_test::block_on(async {
    /// use uuid::Uuid;
    /// use std::time::Duration;
    /// use momento::{CredentialProviderBuilder, SimpleCacheClientBuilder};
    ///
    /// let credential_provider = CredentialProviderBuilder::new_from_environment_variable("TEST_AUTH_TOKEN")
    ///     .build()
    ///     .expect("TEST_AUTH_TOKEN must be set");
    /// let cache_name = "rust-sdk-".to_string() + &Uuid::new_v4().to_string();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(5))?
    ///     .build();
    ///
    /// momento.create_cache(&cache_name).await?;
    /// momento.delete_cache(&cache_name).await?;
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn delete_cache(&mut self, name: &str) -> MomentoResult<()> {
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
    /// # fn main() -> anyhow::Result<()> {
    /// # tokio_test::block_on(async {
    /// # use futures::FutureExt;
    /// use uuid::Uuid;
    /// use std::time::Duration;
    /// use momento::{CredentialProviderBuilder, SimpleCacheClientBuilder};
    ///
    /// let credential_provider = CredentialProviderBuilder::new_from_environment_variable("TEST_AUTH_TOKEN")
    ///     .build()
    ///     .expect("TEST_AUTH_TOKEN must be set");
    /// let cache_name = "rust-sdk-".to_string() + &Uuid::new_v4().to_string();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(5))?
    ///     .build();
    ///
    /// momento.create_cache(&cache_name).await?;
    /// # let result = std::panic::AssertUnwindSafe(async {
    /// let resp = momento.list_caches(None).await?;
    ///
    /// assert!(resp.caches.iter().any(|cache| cache.cache_name == cache_name));
    /// # Ok(())
    /// # }).catch_unwind().await;
    /// # momento.delete_cache(&cache_name).await?;
    /// # result.unwrap_or_else(|e| std::panic::resume_unwind(e))
    /// # })
    /// # }
    /// ```
    pub async fn list_caches(
        &mut self,
        next_token: Option<String>,
    ) -> MomentoResult<MomentoListCacheResponse> {
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
        let response = MomentoListCacheResponse {
            caches,
            next_token: match res.next_token.is_empty() {
                true => None,
                false => Some(res.next_token),
            },
        };
        Ok(response)
    }

    pub async fn flush_cache(
        &mut self,
        cache_name: &str,
    ) -> MomentoResult<MomentoFlushCacheResponse> {
        let request = Request::new(FlushCacheRequest {
            cache_name: cache_name.to_string(),
        });
        self.control_client.flush_cache(request).await?;
        let response = MomentoFlushCacheResponse {};
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
    ) -> MomentoResult<MomentoCreateSigningKeyResponse> {
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
            expires_at: UNIX_EPOCH + Duration::from_secs(res.expires_at),
            endpoint: self.data_endpoint.clone(),
        };
        Ok(response)
    }

    /// Revokes a Momento signing key, all tokens signed by which will be invalid
    ///
    /// # Arguments
    ///
    /// * `key_id` - the ID of the key to revoke
    pub async fn revoke_signing_key(&mut self, key_id: &str) -> MomentoResult<()> {
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
    /// # fn main() -> anyhow::Result<()> {
    /// # tokio_test::block_on(async {
    /// # use futures::FutureExt;
    /// use uuid::Uuid;
    /// use std::time::Duration;
    /// use momento::{CredentialProviderBuilder, SimpleCacheClientBuilder};
    ///
    /// let ttl_minutes = 10;
    /// let credential_provider = CredentialProviderBuilder::new_from_environment_variable("TEST_AUTH_TOKEN")
    ///     .build()
    ///     .expect("TEST_AUTH_TOKEN must be set");
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(5))?
    ///     .build();
    ///
    /// let key = momento.create_signing_key(ttl_minutes).await?;
    /// # let result = std::panic::AssertUnwindSafe(async {
    /// let list = momento.list_signing_keys(None).await?.signing_keys;
    /// assert!(list.iter().any(|sk| sk.key_id == key.key_id));
    /// # Ok(())
    /// # }).catch_unwind().await;
    ///
    /// momento.revoke_signing_key(&key.key_id).await?;
    /// # result.unwrap_or_else(|e| std::panic::resume_unwind(e))
    /// # })
    /// # }
    /// ```
    pub async fn list_signing_keys(
        &mut self,
        next_token: Option<&str>,
    ) -> MomentoResult<MomentoListSigningKeyResult> {
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
                expires_at: UNIX_EPOCH + Duration::from_secs(signing_key.expires_at),
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
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::SimpleCacheClientBuilder;
    ///
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// // Use client default TTL: 30 seconds, as specified above.
    /// momento.set(&cache_name, "k1", "v1", None).await?;
    ///
    /// // Use a custom TTL of 10 minutes for this entry.
    /// momento.set(&cache_name, "k2", "v2", Some(Duration::from_secs(600))).await?;
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn set(
        &mut self,
        cache_name: &str,
        key: impl IntoBytes,
        body: impl IntoBytes,
        ttl: impl Into<Option<Duration>>,
    ) -> MomentoResult<MomentoSetResponse> {
        let request = self.prep_request(
            cache_name,
            SetRequest {
                cache_key: key.into_bytes(),
                cache_body: body.into_bytes(),
                ttl_milliseconds: self.expand_ttl_ms(ttl.into())?,
            },
        )?;
        let _ = self.data_client.set(request).await?;
        Ok(MomentoSetResponse::new())
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
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::SimpleCacheClientBuilder;
    /// use momento::response::{Get, GetValue};
    ///
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.set(&cache_name, "present", "value", None).await?;
    ///
    /// let present = momento.get(&cache_name, "present").await?;
    /// let missing = momento.get(&cache_name, "missing").await?;
    ///
    /// assert_eq!(present, Get::Hit { value: GetValue::new(b"value".to_vec()) });
    /// assert_eq!(missing, Get::Miss);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn get(&mut self, cache_name: &str, key: impl IntoBytes) -> MomentoResult<Get> {
        let request = self.prep_request(
            cache_name,
            GetRequest {
                cache_key: key.into_bytes(),
            },
        )?;

        let response = self.data_client.get(request).await?.into_inner();
        match response.result() {
            ECacheResult::Hit => Ok(Get::Hit {
                value: GetValue {
                    raw_item: response.cache_body,
                },
            }),
            ECacheResult::Miss => Ok(Get::Miss),
            _ => unreachable!(),
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
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use std::iter::FromIterator;
    /// use std::collections::HashMap;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// let dict = HashMap::from_iter([("k1", "v1"), ("k2", "v2")]);
    /// momento.dictionary_set(&cache_name, "dict", dict, ttl).await?;
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn dictionary_set<K: IntoBytes, V: IntoBytes>(
        &mut self,
        cache_name: &str,
        dictionary_name: impl IntoBytes,
        dictionary: HashMap<K, V>,
        policy: CollectionTtl,
    ) -> MomentoResult<MomentoDictionarySetResponse> {
        utils::is_cache_name_valid(cache_name)?;

        let items = dictionary
            .into_iter()
            .map(|(k, v)| DictionaryFieldValuePair {
                field: k.into_bytes(),
                value: v.into_bytes(),
            })
            .collect();

        let request = self.prep_request(
            cache_name,
            DictionarySetRequest {
                dictionary_name: dictionary_name.into_bytes(),
                items,
                ttl_milliseconds: self.expand_ttl_ms(policy.ttl())?,
                refresh_ttl: policy.refresh(),
            },
        )?;

        let _ = self.data_client.dictionary_set(request).await?;
        Ok(MomentoDictionarySetResponse::new())
    }

    /// Get a subset of the fields in a dictionary from the Momento cache.
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
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use std::iter::FromIterator;
    /// use std::collections::HashMap;
    /// use std::convert::TryInto;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// let dict = HashMap::from_iter([("k1", "v1"), ("k2", "v2")]);
    /// momento.dictionary_set(&cache_name, "dict", dict, ttl).await?;
    ///
    /// let dict: HashMap<String, String> = momento
    ///     .dictionary_get(&cache_name, "dict", vec!["k1"])
    ///     .await?
    ///     .try_into()?;
    ///
    /// assert_eq!(dict.get("k1"), Some(&"v1".to_string()));
    /// assert_eq!(dict.get("k2"), None);
    /// assert_eq!(dict.len(), 1);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn dictionary_get<K: IntoBytes>(
        &mut self,
        cache_name: &str,
        dictionary: impl IntoBytes,
        fields: Vec<K>,
    ) -> MomentoResult<DictionaryGet> {
        use dictionary_get_response::Dictionary;

        let fields = convert_vec(fields);
        let request = self.prep_request(
            cache_name,
            DictionaryGetRequest {
                dictionary_name: dictionary.into_bytes(),
                fields: fields.clone(),
            },
        )?;

        let response = self.data_client.dictionary_get(request).await?.into_inner();
        match response.dictionary {
            Some(Dictionary::Found(response)) => {
                // map the dictionary response parts to get responses
                // Defer paying to turn these into a map in case the user has a different hasher or data structure in mind.
                let pairs: Vec<(Vec<u8>, Vec<u8>)> = fields
                    .into_iter()
                    .zip(response.items.into_iter())
                    .filter(|(_, item)| item.result() == ECacheResult::Hit)
                    .map(|(field, item)| (field, item.cache_body))
                    .collect();

                Ok(DictionaryGet::Hit {
                    value: DictionaryPairs { raw_value: pairs },
                })
            }
            Some(Dictionary::Missing(_)) | None => Ok(DictionaryGet::Miss),
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
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use std::iter::FromIterator;
    /// use std::collections::HashMap;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder, response::DictionaryFetch};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// let dict = HashMap::from_iter([("key", "value")]);
    /// momento.dictionary_set(&cache_name, "dict", dict, ttl).await?;
    ///
    /// let dict = match momento
    ///     .dictionary_fetch(&cache_name, "dict")
    ///     .await? {
    ///     DictionaryFetch::Hit { value } => value.into_strings()?,
    ///     DictionaryFetch::Miss => panic!("dictionary does not exist"),
    /// };
    ///
    /// assert_eq!(dict.get("key"), Some(&"value".to_string()));
    /// assert_eq!(dict.len(), 1);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn dictionary_fetch(
        &mut self,
        cache_name: &str,
        dictionary: impl IntoBytes,
    ) -> MomentoResult<DictionaryFetch> {
        use dictionary_fetch_response::Dictionary;

        let request = self.prep_request(
            cache_name,
            DictionaryFetchRequest {
                dictionary_name: dictionary.into_bytes(),
            },
        )?;

        let response = self
            .data_client
            .dictionary_fetch(request)
            .await?
            .into_inner();
        match response.dictionary {
            Some(Dictionary::Found(response)) => {
                Ok(DictionaryFetch::Hit {
                    value: DictionaryPairs {
                        raw_value: response
                            .items
                            // Consume the payload vectors by value to avoid extra copies
                            .into_iter()
                            .map(|pair| (pair.field, pair.value))
                            .collect(),
                    },
                })
            }
            Some(Dictionary::Missing(_)) | None => Ok(DictionaryFetch::Miss),
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
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use std::iter::FromIterator;
    /// use std::collections::HashMap;
    /// use momento::{CollectionTtl, Fields, SimpleCacheClientBuilder, response::DictionaryFetch};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// let dict = HashMap::from_iter([("a", "b"), ("c", "d"), ("e", "f")]);
    /// momento.dictionary_set(&cache_name, "dict", dict, ttl).await?;
    ///
    /// momento.dictionary_delete(&cache_name, "dict", Fields::Some(vec!["a"]));
    /// let dict1 = match momento.dictionary_fetch(&cache_name, "dict").await? {
    ///     DictionaryFetch::Hit { value } => value.into_strings().unwrap(),
    ///     DictionaryFetch::Miss => panic!("it should exist"),
    /// };
    /// momento.dictionary_delete::<Vec<u8>>(&cache_name, "dict", Fields::All).await?;
    ///
    /// assert!(dict1.contains_key("c"));
    /// assert!(dict1.contains_key("e"));
    /// assert_eq!(momento.dictionary_fetch(&cache_name, "dict").await?, DictionaryFetch::Miss);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn dictionary_delete<K: IntoBytes>(
        &mut self,
        cache_name: &str,
        dictionary: impl IntoBytes,
        fields: Fields<K>,
    ) -> MomentoResult<MomentoDictionaryDeleteResponse> {
        use dictionary_delete_request::{All, Delete};

        let request = match fields {
            Fields::Some(fields) => DictionaryDeleteRequest {
                dictionary_name: dictionary.into_bytes(),
                delete: Some(Delete::Some(dictionary_delete_request::Some {
                    fields: convert_vec(fields),
                })),
            },
            Fields::All => DictionaryDeleteRequest {
                dictionary_name: dictionary.into_bytes(),
                delete: Some(Delete::All(All::default())),
            },
        };

        self.data_client
            .dictionary_delete(self.prep_request(cache_name, request)?)
            .await?;
        Ok(MomentoDictionaryDeleteResponse::new())
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
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// let resp = momento.dictionary_increment(&cache_name, "dict", "key", 10, ttl).await?;
    ///
    /// // key was empty before, now it is 10
    /// assert_eq!(resp.value, 10);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn dictionary_increment(
        &mut self,
        cache_name: &str,
        dictionary: impl IntoBytes,
        field: impl IntoBytes,
        amount: i64,
        policy: CollectionTtl,
    ) -> MomentoResult<MomentoDictionaryIncrementResponse> {
        let request = self.prep_request(
            cache_name,
            DictionaryIncrementRequest {
                dictionary_name: dictionary.into_bytes(),
                field: field.into_bytes(),
                amount,
                ttl_milliseconds: self.expand_ttl_ms(policy.ttl())?,
                refresh_ttl: policy.refresh(),
            },
        )?;

        let response = self
            .data_client
            .dictionary_increment(request)
            .await?
            .into_inner();

        Ok(MomentoDictionaryIncrementResponse {
            value: response.value,
        })
    }

    /// Push multiple values to the beginning of a list.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of the cache to store the list in.
    /// * `list_name` - list to modify.
    /// * `values` - values to push to the front of the list.
    /// * `truncate_to` - if set, indicates the maximum number of elements the
    ///   list may contain before truncating elements from the back.
    /// * `policy` - TTL policy to use.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// assert_eq!(momento.list_concat_front(&cache_name, "list", ["a", "b"], 3, ttl).await?, 2);
    /// assert_eq!(momento.list_concat_front(&cache_name, "list", ["c", "d"], 3, ttl).await?, 3);
    ///
    /// let entry = momento
    ///     .list_fetch(&cache_name, "list")
    ///     .await?
    ///     .expect("list was missing within the cache");
    /// let values: Vec<_> = entry.value().iter().map(|v| &v[..]).collect();
    /// let expected: Vec<&[u8]> = vec![b"c", b"d", b"a"];
    /// assert_eq!(values, expected);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn list_concat_front<V: IntoBytes>(
        &mut self,
        cache_name: &str,
        list_name: impl IntoBytes,
        values: impl IntoIterator<Item = V>,
        truncate_to: impl Into<Option<u32>>,
        policy: CollectionTtl,
    ) -> MomentoResult<u32> {
        let request = self.prep_request(
            cache_name,
            ListConcatenateFrontRequest {
                list_name: list_name.into_bytes(),
                values: convert_vec(values),
                ttl_milliseconds: self.expand_ttl_ms(policy.ttl())?,
                refresh_ttl: policy.refresh(),
                truncate_back_to_size: truncate_to.into().unwrap_or(u32::MAX),
            },
        )?;

        self.data_client
            .list_concatenate_front(request)
            .await
            .map(|resp| resp.into_inner().list_length)
            .map_err(From::from)
    }

    /// Push multiple values to the end of a list.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of the cache to store the list in.
    /// * `list_name` - list to modify.
    /// * `values` - values to push to the front of the list.
    /// * `truncate_to` - if set, indicates the maximum number of elements the
    ///   list may contain before truncating elements from the front.
    /// * `policy` - TTL policy to use.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// assert_eq!(momento.list_concat_back(&cache_name, "list", ["a", "b"], 3, ttl).await?, 2);
    /// assert_eq!(momento.list_concat_back(&cache_name, "list", ["c", "d"], 3, ttl).await?, 3);
    ///
    /// let entry = momento
    ///     .list_fetch(&cache_name, "list")
    ///     .await?
    ///     .expect("list was missing within the cache");
    /// let values: Vec<_> = entry.value().iter().map(|v| &v[..]).collect();
    /// let expected: Vec<&[u8]> = vec![b"b", b"c", b"d"];
    /// assert_eq!(values, expected);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn list_concat_back<V: IntoBytes>(
        &mut self,
        cache_name: &str,
        list_name: impl IntoBytes,
        values: impl IntoIterator<Item = V>,
        truncate_to: impl Into<Option<u32>>,
        policy: CollectionTtl,
    ) -> MomentoResult<u32> {
        let request = self.prep_request(
            cache_name,
            ListConcatenateBackRequest {
                list_name: list_name.into_bytes(),
                values: convert_vec(values),
                ttl_milliseconds: self.expand_ttl_ms(policy.ttl())?,
                refresh_ttl: policy.refresh(),
                truncate_front_to_size: truncate_to.into().unwrap_or(u32::MAX),
            },
        )?;

        self.data_client
            .list_concatenate_back(request)
            .await
            .map(|resp| resp.into_inner().list_length)
            .map_err(From::from)
    }

    /// Inserts a value at the start of a list.
    ///
    /// A missing entry is treated as a list of length 0.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of the cache to store the list in.
    /// * `list_name` - the list to insert the value in to.
    /// * `value` - value to insert at the front of the list.
    /// * `policy` - the TTL policy to use for this operation.
    /// * `truncate_to` - should the list exceed this length, it will be truncated.
    ///   If `None`, no truncation will be done. Truncation occurs from the front.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.list_set(&cache_name, "list", ["a", "b"], ttl).await?;
    /// momento.list_push_front(&cache_name, "list", "!", None, ttl).await?;
    ///
    /// let entry = momento.list_fetch(&cache_name, "list").await?.unwrap();
    /// let values: Vec<_> = entry.value().iter().map(|v| &v[..]).collect();
    /// let expected: Vec<&[u8]> = vec![b"!", b"a", b"b"];
    /// assert_eq!(values, expected);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn list_push_front(
        &mut self,
        cache_name: &str,
        list_name: impl IntoBytes,
        value: impl IntoBytes,
        truncate_to: impl Into<Option<u32>>,
        policy: CollectionTtl,
    ) -> MomentoResult<u32> {
        let request = self.prep_request(
            cache_name,
            ListPushFrontRequest {
                list_name: list_name.into_bytes(),
                value: value.into_bytes(),
                ttl_milliseconds: self.expand_ttl_ms(policy.ttl())?,
                refresh_ttl: policy.refresh(),
                truncate_back_to_size: truncate_to.into().unwrap_or(u32::MAX),
            },
        )?;

        Ok(self
            .data_client
            .list_push_front(request)
            .await?
            .into_inner()
            .list_length)
    }

    /// Inserts a value at the end of a list.
    ///
    /// A missing entry is treated as a list of length 0.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of the cache to store the list in.
    /// * `list_name` - the list to insert the value in to.
    /// * `value` - value to insert at the end of the list.
    /// * `policy` - the TTL policy to use for this operation.
    /// * `truncate_to` - should the list exceed this length, it will be truncated.
    ///   If `None`, no truncation will be done. Truncation occurs from the front.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.list_set(&cache_name, "list", ["a", "b"], ttl).await?;
    /// momento.list_push_back(&cache_name, "list", "!", None, ttl).await?;
    ///
    /// let entry = momento.list_fetch(&cache_name, "list").await?.unwrap();
    /// let values: Vec<_> = entry.value().iter().map(|v| &v[..]).collect();
    /// let expected: Vec<&[u8]> = vec![b"a", b"b", b"!"];
    /// assert_eq!(values, expected);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn list_push_back(
        &mut self,
        cache_name: &str,
        list_name: impl IntoBytes,
        value: impl IntoBytes,
        truncate_to: impl Into<Option<u32>>,
        policy: CollectionTtl,
    ) -> MomentoResult<u32> {
        let request = self.prep_request(
            cache_name,
            ListPushBackRequest {
                list_name: list_name.into_bytes(),
                value: value.into_bytes(),
                ttl_milliseconds: self.expand_ttl_ms(policy.ttl())?,
                refresh_ttl: policy.refresh(),
                truncate_front_to_size: truncate_to.into().unwrap_or(u32::MAX),
            },
        )?;

        Ok(self
            .data_client
            .list_push_back(request)
            .await?
            .into_inner()
            .list_length)
    }

    /// Set a list within the cache.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of the cache to store the list in.
    /// * `list_name` - list to be set in the cache.
    /// * `values` - the values that make up the list.
    /// * `policy` - TTL policy to use for this operation.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.list_set(&cache_name, "list", ["a", "b"], ttl).await?;
    /// momento.list_set(&cache_name, "list", ["c", "d"], ttl).await?;
    ///
    /// let entry = momento.list_fetch(&cache_name, "list").await?.unwrap();
    /// let values: Vec<_> = entry.value().iter().map(|v| &v[..]).collect();
    /// let expected: Vec<&[u8]> = vec![b"c", b"d"];
    /// assert_eq!(values, expected);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn list_set<V: IntoBytes>(
        &mut self,
        cache_name: &str,
        list_name: impl IntoBytes,
        values: impl IntoIterator<Item = V>,
        policy: CollectionTtl,
    ) -> MomentoResult<()> {
        // We're exposing this as a set so updating an existing entry should count as if
        // we are creating a new entry.
        let policy = policy.with_refresh_on_update();
        let values: Vec<_> = values.into_iter().map(|v| v.into_bytes()).collect();
        let count = values.len();

        self.list_concat_front(cache_name, list_name, values, count as u32, policy)
            .await?;

        Ok(())
    }

    /// Fetch the entire list from the cache.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of the cache to fetch list from.
    /// * `list_name` - the list to fetch.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.list_set(&cache_name, "list", ["a", "b"], ttl).await?;
    ///
    /// let entry = momento.list_fetch(&cache_name, "list").await?.unwrap();
    /// let values: Vec<_> = entry.value().iter().map(|v| &v[..]).collect();
    /// let expected: Vec<&[u8]> = vec![b"a", b"b"];
    /// assert_eq!(values, expected);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn list_fetch(
        &mut self,
        cache_name: &str,
        list_name: impl IntoBytes,
    ) -> MomentoResult<MomentoListFetchResponse> {
        use list_fetch_response::List;

        let request = self.prep_request(
            cache_name,
            ListFetchRequest {
                list_name: list_name.into_bytes(),
                start_index: None,
                end_index: None,
            },
        )?;

        let response = self.data_client.list_fetch(request).await?.into_inner();
        Ok(match response.list {
            Some(List::Found(found)) => Some(ListCacheEntry::new(found.values)),
            _ => None,
        })
    }

    /// Retrieve and remove the first item from a list.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of the cache in which the list is stored.
    /// * `list_name` - name of the list to pop from.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.list_set(&cache_name, "list", ["a", "b"], ttl).await?;
    ///
    /// assert!(matches!(
    ///     momento.list_pop_front(&cache_name, "list").await?,
    ///     Some(v) if v == b"a"
    /// ));
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn list_pop_front(
        &mut self,
        cache_name: &str,
        list_name: impl IntoBytes,
    ) -> MomentoResult<Option<Vec<u8>>> {
        use momento_protos::cache_client::list_pop_front_response::List;

        let request = self.prep_request(
            cache_name,
            ListPopFrontRequest {
                list_name: list_name.into_bytes(),
            },
        )?;

        Ok(
            match self
                .data_client
                .list_pop_front(request)
                .await?
                .into_inner()
                .list
            {
                Some(List::Found(list)) => Some(list.front),
                _ => None,
            },
        )
    }

    /// Retrieve and remove the last item from a list.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of the cache in which the list is stored.
    /// * `list_name` - name of the list to pop from.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.list_set(&cache_name, "list", ["a", "b"], ttl).await?;
    ///
    /// assert!(matches!(
    ///     momento.list_pop_back(&cache_name, "list").await?,
    ///     Some(v) if v == b"b"
    /// ));
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn list_pop_back(
        &mut self,
        cache_name: &str,
        list_name: impl IntoBytes,
    ) -> MomentoResult<Option<Vec<u8>>> {
        use momento_protos::cache_client::list_pop_back_response::List;

        let request = self.prep_request(
            cache_name,
            ListPopBackRequest {
                list_name: list_name.into_bytes(),
            },
        )?;

        Ok(
            match self
                .data_client
                .list_pop_back(request)
                .await?
                .into_inner()
                .list
            {
                Some(List::Found(list)) => Some(list.back),
                _ => None,
            },
        )
    }

    /// Remove all elements in a list matching a particular value.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of the cache in which to look for the list.
    /// * `list_name` - name of the list from which to remove elements.
    /// * `value` - the value to remove from the list.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.list_set(&cache_name, "list", ["a", "b", "c", "a"], ttl).await?;
    /// momento.list_remove_value(&cache_name, "list", "a").await?;
    ///
    /// let entry = momento.list_fetch(&cache_name, "list").await?.unwrap();
    /// let values: Vec<_> = entry.value().iter().map(|v| &v[..]).collect();
    /// let expected: Vec<&[u8]> = vec![b"b", b"c"];
    /// assert_eq!(values, expected);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn list_remove_value(
        &mut self,
        cache_name: &str,
        list_name: impl IntoBytes,
        value: impl IntoBytes,
    ) -> MomentoResult<()> {
        use list_remove_request::Remove;

        let request = self.prep_request(
            cache_name,
            ListRemoveRequest {
                list_name: list_name.into_bytes(),
                remove: Some(Remove::AllElementsWithValue(value.into_bytes())),
            },
        )?;

        self.data_client.list_remove(request).await?;
        Ok(())
    }

    /// Erase a range of elements from a list.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - the name of the cache in which to look for the list.
    /// * `list_name` - name of the list from which to remove elements.
    /// * `range` - range of indices to erase from the list.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.list_set(&cache_name, "list", ["a", "b", "c", "d"], ttl).await?;
    /// momento.list_erase(&cache_name, "list", 1..=2).await?;
    ///
    /// let entry = momento.list_fetch(&cache_name, "list").await?.unwrap();
    /// let values: Vec<_> = entry.value().iter().map(|v| &v[..]).collect();
    /// let expected: Vec<&[u8]> = vec![b"a", b"d"];
    /// assert_eq!(values, expected);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn list_erase(
        &mut self,
        cache_name: &str,
        list_name: impl IntoBytes,
        range: impl RangeBounds<u32>,
    ) -> MomentoResult<()> {
        self.list_erase_many(cache_name, list_name, [range]).await
    }

    /// Erase multiple ranges of elements from a list.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - the name of the cache in which to look for the list.
    /// * `list_name` - the name of the list from which to remove elements.
    /// * `ranges` - ranges of indices to erase from the list.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.list_set(&cache_name, "list", ["a", "b", "c", "d", "e", "f"], ttl).await?;
    /// momento.list_erase_many(&cache_name, "list", vec![1..=2, 4..=4]).await?;
    ///
    /// let entry = momento.list_fetch(&cache_name, "list").await?.unwrap();
    /// let values: Vec<_> = entry.value().iter().map(|v| &v[..]).collect();
    /// let expected: Vec<&[u8]> = vec![b"a", b"d", b"f"];
    /// assert_eq!(values, expected);
    /// # Ok(())
    /// # })
    /// # }
    ///
    pub async fn list_erase_many<R: RangeBounds<u32>>(
        &mut self,
        cache_name: &str,
        list_name: impl IntoBytes,
        ranges: impl IntoIterator<Item = R>,
    ) -> MomentoResult<()> {
        use list_erase_request::{Erase, ListRanges};
        use std::ops::Bound;

        let list_name = list_name.into_bytes();
        let mut req_ranges = Vec::new();
        for range in ranges {
            let start = match range.start_bound() {
                Bound::Unbounded => 0,
                Bound::Included(&start) => start,
                Bound::Excluded(&u32::MAX) => return Ok(()),
                Bound::Excluded(&start) => start + 1,
            };
            let count = match range.end_bound() {
                Bound::Unbounded if start == 0 => {
                    return self.delete(cache_name, list_name).await.map(|_| ())
                }
                Bound::Unbounded => u32::MAX - start,
                Bound::Included(&end) if end < start => return Ok(()),
                Bound::Included(&end) => end - start + 1,
                Bound::Excluded(&end) if end <= start => return Ok(()),
                Bound::Excluded(&end) => end - start,
            };

            req_ranges.push(ListRange {
                begin_index: start,
                count,
            })
        }

        let request = self.prep_request(
            cache_name,
            ListEraseRequest {
                list_name: list_name.into_bytes(),
                erase: Some(Erase::Some(ListRanges { ranges: req_ranges })),
            },
        )?;

        let _ = self.data_client.list_erase(request).await?;
        Ok(())
    }

    /// Fetch the length of a list from the cache.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - the name of the cache in which to look for the list.
    /// * `list_name` - name of the list.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.list_set(&cache_name, "present", ["a", "b"], ttl).await?;
    ///
    /// assert_eq!(momento.list_length(&cache_name, "present").await?, Some(2));
    /// assert_eq!(momento.list_length(&cache_name, "missing").await?, None);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn list_length(
        &mut self,
        cache_name: &str,
        list_name: impl IntoBytes,
    ) -> MomentoResult<Option<u32>> {
        use list_length_response::List;

        let request = self.prep_request(
            cache_name,
            ListLengthRequest {
                list_name: list_name.into_bytes(),
            },
        )?;

        let response = self.data_client.list_length(request).await?.into_inner();

        Ok(match response.list {
            Some(List::Found(found)) => Some(found.length),
            _ => None,
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
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::SimpleCacheClientBuilder;
    ///
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// match momento.set_fetch(&cache_name, "test set").await?.value {
    ///     Some(set) => {
    ///         println!("set entries:");
    ///         for entry in &set {
    ///             println!("{:?}", entry);
    ///         }
    ///     },
    ///     None => println!("set not found!"),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn set_fetch(
        &mut self,
        cache_name: &str,
        set_name: impl IntoBytes,
    ) -> MomentoResult<MomentoSetFetchResponse> {
        use set_fetch_response::Set;

        let request = self.prep_request(
            cache_name,
            SetFetchRequest {
                set_name: set_name.into_bytes(),
            },
        )?;

        let response = self.data_client.set_fetch(request).await?.into_inner();
        Ok(MomentoSetFetchResponse {
            value: response.set.and_then(|set| match set {
                Set::Found(found) => Some(HashSet::from_iter(found.elements)),
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
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.set_union(&cache_name, "myset", vec!["c", "d"], ttl).await?;
    /// momento.set_union(&cache_name, "myset", vec!["a", "b", "c"], ttl).await?;
    ///
    /// let set = momento.set_fetch(&cache_name, "myset").await?.value.unwrap();
    ///
    /// assert!(set.contains("a".as_bytes()));
    /// assert!(set.contains("b".as_bytes()));
    /// assert!(set.contains("c".as_bytes()));
    /// assert!(set.contains("d".as_bytes()));
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn set_union<E: IntoBytes>(
        &mut self,
        cache_name: &str,
        set_name: impl IntoBytes,
        elements: Vec<E>,
        policy: CollectionTtl,
    ) -> MomentoResult<()> {
        let request = self.prep_request(
            cache_name,
            SetUnionRequest {
                set_name: set_name.into_bytes(),
                elements: convert_vec(elements),
                ttl_milliseconds: self.expand_ttl_ms(policy.ttl())?,
                refresh_ttl: policy.refresh(),
            },
        )?;

        let _ = self.data_client.set_union(request).await?.into_inner();
        Ok(())
    }

    /// Remove items from an existing set in the cache.
    ///
    /// After this operation the set will contain any elements in the original set that
    /// were not contained in this request. Note that this is a no-op there is no such
    /// set in the cache already.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - the name of the cache.
    /// * `set_name` - the name of the set.
    /// * `elements` - elements to remove from the set in the cache.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    /// use momento::response::MomentoSetDifferenceResponse;
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.set_union(&cache_name, "test set", vec!["a", "b", "c", "d"], ttl).await?;
    /// momento.set_difference(&cache_name, "test set", vec!["b", "d"]).await?;
    ///
    /// let set = momento.set_fetch(&cache_name, "test set").await?.value.unwrap();
    ///
    /// assert!(set.contains("a".as_bytes()));
    /// assert!(set.contains("c".as_bytes()));
    /// assert!(!set.contains("b".as_bytes()));
    /// assert!(!set.contains("d".as_bytes()));
    /// assert_eq!(set.len(), 2);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn set_difference<E: IntoBytes>(
        &mut self,
        cache_name: &str,
        set_name: impl IntoBytes,
        elements: Vec<E>,
    ) -> MomentoResult<MomentoSetDifferenceResponse> {
        use set_difference_request::subtrahend::{Set as SubtrahendSetSet, SubtrahendSet};
        use set_difference_request::{Difference, Subtrahend};
        use set_difference_response::Set;

        let request = self.prep_request(
            cache_name,
            SetDifferenceRequest {
                set_name: set_name.into_bytes(),
                difference: Some(Difference::Subtrahend(Subtrahend {
                    subtrahend_set: Some(SubtrahendSet::Set(SubtrahendSetSet {
                        elements: elements.into_iter().map(|e| e.into_bytes()).collect(),
                    })),
                })),
            },
        )?;

        let response = self.data_client.set_difference(request).await?.into_inner();

        Ok(match response.set {
            Some(Set::Found(_)) => MomentoSetDifferenceResponse::Found,
            _ => MomentoSetDifferenceResponse::Missing,
        })
    }

    /// This is an alias for [`set_union`].
    ///
    /// [`set_union`]: SimpleCacheClient::set_union
    pub async fn set_add_elements<E: IntoBytes>(
        &mut self,
        cache_name: &str,
        set_name: impl IntoBytes,
        elements: Vec<E>,
        policy: CollectionTtl,
    ) -> MomentoResult<()> {
        self.set_union(cache_name, set_name, elements, policy).await
    }

    /// This is an alias for [`set_difference`].
    ///
    /// [`set_difference`]: SimpleCacheClient::set_difference
    pub async fn set_remove_elements<E: IntoBytes>(
        &mut self,
        cache_name: &str,
        set_name: impl IntoBytes,
        elements: Vec<E>,
    ) -> MomentoResult<MomentoSetDifferenceResponse> {
        self.set_difference(cache_name, set_name, elements).await
    }

    /// Fetches a sorted set from a Momento Cache.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `_cache_name` - name of cache.
    /// * `_set_name` - name of the set.
    /// * `_order` - specify ascending or descending order
    /// * `_limit` - optionally limit the number of results returned
    /// * `_range` - constrain to a range of elements by index or by score
    pub async fn sorted_set_fetch(
        &mut self,
        _cache_name: &str,
        _set_name: impl IntoBytes,
        _order: sorted_set::Order,
        _limit: impl Into<Option<NonZeroU32>>,
        _range: Option<sorted_set::Range>,
    ) -> MomentoResult<MomentoSortedSetFetchResponse> {
        todo!("This api was reworked and is pending implementation in the rust sdk: https://github.com/momentohq/client-sdk-rust/issues/135");
    }

    /// Gets the rank of an element in a sorted set in a Momento Cache.
    ///
    /// The return result is a `MomentoResult` which on success contains an
    /// option. If the sorted set or element is not found the `None` variant is
    /// returned. Otherwise, the `Some` variant contains the rank of the item
    /// within the sorted set.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache.
    /// * `set_name` - name of the set.
    /// * `element_name` - name of the element.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::SimpleCacheClientBuilder;
    /// use momento::sorted_set::Order;
    ///
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// match momento.sorted_set_get_rank(&cache_name, "test sorted set", "element a").await? {
    ///     Some(rank) => {
    ///         println!("element has rank: {rank}");
    ///     },
    ///     None => println!("sorted set or element not found!"),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn sorted_set_get_rank(
        &mut self,
        cache_name: &str,
        set_name: impl IntoBytes,
        element_name: impl IntoBytes,
    ) -> MomentoResult<Option<u64>> {
        use momento_protos::cache_client::sorted_set_get_rank_request::Order::Ascending;
        use momento_protos::cache_client::sorted_set_get_rank_response::Rank;

        let request = self.prep_request(
            cache_name,
            SortedSetGetRankRequest {
                set_name: set_name.into_bytes(),
                value: element_name.into_bytes(),
                order: Ascending.into(),
            },
        )?;

        let response = self
            .data_client
            .sorted_set_get_rank(request)
            .await?
            .into_inner();

        let rank = match response.rank {
            Some(Rank::ElementRank(r)) => {
                if r.result() == ECacheResult::Ok {
                    Some(r.rank)
                } else {
                    None
                }
            }
            _ => None,
        };

        Ok(rank)
    }

    /// Gets the score associated with one or more elements in a sorted set in a
    /// Momento Cache.
    ///
    /// The return result is a `MomentoResult` which on success contains a vec
    /// of options. The ordering matches the order in which the element names
    /// were provided. If the element was found, the `Some` variant contains the
    /// score of the element within the sorted set. Otherwise, if the sorted set
    /// or element was not found, the `None` variant will be in that position.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache.
    /// * `set_name` - name of the set.
    /// * `element_names` - names of the elements.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::SimpleCacheClientBuilder;
    ///
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// let elements = vec!["a", "b", "c"];
    ///
    /// let scores = momento.sorted_set_get_score(
    ///     &cache_name,
    ///     "test sorted set",
    ///     elements.clone()
    /// ).await?;
    ///
    /// println!("element\tscore");
    /// for (element, score) in elements.iter().zip(scores.iter()) {
    ///     let score = score.map(|s| format!("{s}")).unwrap_or("None".to_string());
    ///     println!("{element}\t{score}");
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn sorted_set_get_score(
        &mut self,
        cache_name: &str,
        set_name: impl IntoBytes,
        element_names: Vec<impl IntoBytes>,
    ) -> MomentoResult<Vec<Option<f64>>> {
        use momento_protos::cache_client::sorted_set_get_score_response::SortedSet;

        let len = element_names.len();

        let request = self.prep_request(
            cache_name,
            SortedSetGetScoreRequest {
                set_name: set_name.into_bytes(),
                values: convert_vec(element_names),
            },
        )?;

        let response = self
            .data_client
            .sorted_set_get_score(request)
            .await?
            .into_inner();

        match response.sorted_set {
            Some(SortedSet::Found(s)) => Ok(s
                .elements
                .iter()
                .map(|e| {
                    if e.result() == ECacheResult::Ok {
                        Some(e.score)
                    } else {
                        None
                    }
                })
                .collect()),
            _ => Ok(vec![None; len]),
        }
    }

    /// Increments the score for an element in a sorted set in a Momento Cache.
    ///
    /// The return type is a `MomentoResult` where on success the element has
    /// been incremented and the new score is returned.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache.
    /// * `set_name` - name of the set.
    /// * `element_name` - name of the element.
    /// * `amount` - the amount to be added to the score.
    /// * `policy` - TTL policy to use.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.sorted_set_increment(&cache_name, "test sorted set", "a", 50.0, ttl).await?;
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn sorted_set_increment(
        &mut self,
        cache_name: &str,
        set_name: impl IntoBytes,
        element_name: impl IntoBytes,
        amount: f64,
        policy: CollectionTtl,
    ) -> MomentoResult<f64> {
        let request = self.prep_request(
            cache_name,
            SortedSetIncrementRequest {
                set_name: set_name.into_bytes(),
                value: element_name.into_bytes(),
                amount,
                ttl_milliseconds: self.expand_ttl_ms(policy.ttl())?,
                refresh_ttl: policy.refresh(),
            },
        )?;

        let response = self
            .data_client
            .sorted_set_increment(request)
            .await?
            .into_inner();

        Ok(response.score)
    }

    /// Adds elements to a sorted set in a Momento Cache.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache.
    /// * `set_name` - name of the set.
    /// * `elements` - the elements to be added.
    /// * `policy` - TTL policy to use.
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::{CollectionTtl, SimpleCacheClientBuilder};
    /// use momento::sorted_set::SortedSetElement;
    ///
    /// let ttl = CollectionTtl::default();
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.sorted_set_put(&cache_name, "test sorted set", vec![
    ///     SortedSetElement { value: "a".into(), score: 50.0 },
    ///     SortedSetElement { value: "b".into(), score: 60.0 },
    /// ], ttl).await?;
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn sorted_set_put(
        &mut self,
        cache_name: &str,
        set_name: impl IntoBytes,
        elements: Vec<SortedSetElement>,
        policy: CollectionTtl,
    ) -> MomentoResult<()> {
        let request = self.prep_request(
            cache_name,
            SortedSetPutRequest {
                set_name: set_name.into_bytes(),
                elements,
                ttl_milliseconds: self.expand_ttl_ms(policy.ttl())?,
                refresh_ttl: policy.refresh(),
            },
        )?;

        let _ = self.data_client.sorted_set_put(request).await?.into_inner();

        Ok(())
    }

    /// Removes elements from a sorted set from a Momento Cache.
    ///
    /// *NOTE*: This is preview functionality and requires that you contact
    /// Momento Support to enable these APIs for your cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache.
    /// * `set_name` - name of the set.
    /// * `element_names` - the names of the elements to be removed
    ///
    /// # Example
    /// ```
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::SimpleCacheClientBuilder;
    ///
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.sorted_set_remove(&cache_name, "test sorted set", vec!["a", "b", "c"]).await?;
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn sorted_set_remove(
        &mut self,
        cache_name: &str,
        set_name: impl IntoBytes,
        mut element_names: Vec<impl IntoBytes>,
    ) -> MomentoResult<()> {
        use momento_protos::cache_client::sorted_set_remove_request::{RemoveElements, Some};

        let request = self.prep_request(
            cache_name,
            SortedSetRemoveRequest {
                set_name: set_name.into_bytes(),
                remove_elements: Some(RemoveElements::Some(Some {
                    values: element_names.drain(..).map(|v| v.into_bytes()).collect(),
                })),
            },
        )?;

        let _ = self
            .data_client
            .sorted_set_remove(request)
            .await?
            .into_inner();

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
    /// # fn main() -> momento_test_util::DoctestResult {
    /// # momento_test_util::doctest(|cache_name, credential_provider| async move {
    /// use std::time::Duration;
    /// use momento::SimpleCacheClientBuilder;
    ///
    /// let mut momento = SimpleCacheClientBuilder::new(credential_provider, Duration::from_secs(30))?
    ///     .build();
    ///
    /// momento.set(&cache_name, "key", "value", None).await?;
    /// momento.delete(&cache_name, "key").await?;
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn delete(
        &mut self,
        cache_name: &str,
        key: impl IntoBytes,
    ) -> MomentoResult<MomentoDeleteResponse> {
        let request = self.prep_request(
            cache_name,
            DeleteRequest {
                cache_key: key.into_bytes(),
            },
        )?;
        self.data_client.delete(request).await?.into_inner();
        Ok(MomentoDeleteResponse::new())
    }

    /// Generates an api token for Momento
    ///
    /// # Arguments
    ///
    /// * `token_expiry` - when should the token expire, can be set to Never to never expire
    pub async fn generate_api_token(
        &mut self,
        token_expiry: TokenExpiry,
    ) -> MomentoResult<MomentoGenerateApiTokenResponse> {
        let expiry = match token_expiry {
            TokenExpiry::Never => {
                Expiry::Never(momento_protos::control_client::generate_api_token_request::Never {})
            }
            TokenExpiry::Expires { valid_for_seconds } => Expiry::Expires(
                momento_protos::control_client::generate_api_token_request::Expires {
                    valid_for_seconds,
                },
            ),
        };
        let request = GenerateApiTokenRequest {
            expiry: Some(expiry),
        };
        let resp = self
            .control_client
            .generate_api_token(request)
            .await?
            .into_inner();

        let api_key_with_endpoint = ApiToken {
            api_key: resp.api_key,
            endpoint: resp.endpoint,
        };

        Ok(MomentoGenerateApiTokenResponse {
            api_token: base64_encode(api_key_with_endpoint),
            refresh_token: resp.refresh_token,
            valid_until: UNIX_EPOCH + Duration::from_secs(resp.valid_until),
        })
    }

    fn expand_ttl_ms(&self, ttl: Option<Duration>) -> MomentoResult<u64> {
        let ttl = ttl.unwrap_or(self.item_default_ttl);
        utils::is_ttl_valid(ttl)?;

        Ok(ttl.as_millis().try_into().unwrap_or(i64::MAX as u64))
    }

    fn prep_request<R>(&self, cache_name: &str, request: R) -> MomentoResult<tonic::Request<R>> {
        utils::is_cache_name_valid(cache_name)?;

        let mut request = tonic::Request::new(request);
        request_meta_data(&mut request, cache_name)?;
        Ok(request)
    }
}

/// An enum that is used to indicate if an operation should apply to all fields
/// or just some fields of a dictionary.
pub enum Fields<K> {
    All,
    Some(Vec<K>),
}

fn convert_vec<E: IntoBytes>(vec: impl IntoIterator<Item = E>) -> Vec<Vec<u8>> {
    vec.into_iter().map(|e| e.into_bytes()).collect()
}
