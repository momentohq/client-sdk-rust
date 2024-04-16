use std::convert::TryInto;
use std::time::Duration;

use momento_protos::cache_client::scs_client::ScsClient;
use momento_protos::control_client::scs_control_client::ScsControlClient;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

use crate::config::configuration::Configuration;
use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::requests::cache::basic::get::{Get, GetRequest};
use crate::requests::cache::basic::set::{Set, SetRequest};
use crate::requests::cache::create_cache::{CreateCache, CreateCacheRequest};
use crate::requests::cache::delete_cache::{DeleteCache, DeleteCacheRequest};
use crate::requests::cache::flush_cache::{FlushCache, FlushCacheRequest};
use crate::requests::cache::list_caches::{ListCaches, ListCachesRequest};
use crate::requests::cache::scalar::key_exists::{KeyExists, KeyExistsRequest};
use crate::requests::cache::scalar::keys_exist::{KeysExist, KeysExistRequest};
use crate::requests::cache::set::set_add_elements::{SetAddElements, SetAddElementsRequest};
use crate::requests::cache::sorted_set::sorted_set_fetch_by_rank::{
    SortOrder, SortedSetFetchByRankRequest,
};
use crate::requests::cache::sorted_set::sorted_set_fetch_by_score::SortedSetFetchByScoreRequest;
use crate::requests::cache::sorted_set::sorted_set_fetch_response::SortedSetFetch;
use crate::requests::cache::sorted_set::sorted_set_put_element::{
    SortedSetPutElement, SortedSetPutElementRequest,
};
use crate::requests::cache::sorted_set::sorted_set_put_elements::{
    SortedSetPutElements, SortedSetPutElementsRequest,
};
use crate::requests::cache::MomentoRequest;

use crate::cache_client_builder::{CacheClientBuilder, NeedsDefaultTtl};
use crate::{utils, IntoBytes, MomentoResult};

/// Client to perform operations on a Momento cache.
#[derive(Clone, Debug)]
pub struct CacheClient {
    pub(crate) data_client: ScsClient<InterceptedService<Channel, HeaderInterceptor>>,
    pub(crate) control_client: ScsControlClient<InterceptedService<Channel, HeaderInterceptor>>,
    pub(crate) configuration: Configuration,
    pub(crate) item_default_ttl: Duration,
}

impl CacheClient {
    /* constructor */
    pub fn builder() -> CacheClientBuilder<NeedsDefaultTtl> {
        CacheClientBuilder(NeedsDefaultTtl(()))
    }

    /* public API */

    /// Creates a cache with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the cache to be created.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```no_run
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::requests::cache::create_cache::CreateCache;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.create_cache(&cache_name).await? {
    ///     CreateCache::Created => println!("Cache {} created", &cache_name),
    ///     CreateCache::AlreadyExists => println!("Cache {} already exists", &cache_name),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to create a cache using a [CreateCacheRequest].
    pub async fn create_cache(&self, cache_name: impl Into<String>) -> MomentoResult<CreateCache> {
        let request = CreateCacheRequest::new(cache_name);
        request.send(self).await
    }

    /// Deletes the cache with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the cache to be deleted.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```no_run
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::requests::cache::delete_cache::DeleteCache;
    /// use momento::requests::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.delete_cache(&cache_name).await {
    ///     Ok(_) => println!("Cache deleted: {}", &cache_name),
    ///     Err(e) => if let MomentoErrorCode::NotFoundError = e.error_code {
    ///         println!("Cache not found: {}", &cache_name);
    ///     } else {
    ///         eprintln!("Error deleting cache {}: {}", &cache_name, e);
    ///     }
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to delete a cache using a [DeleteCacheRequest].
    pub async fn delete_cache(&self, cache_name: impl Into<String>) -> MomentoResult<DeleteCache> {
        let request = DeleteCacheRequest::new(cache_name);
        request.send(self).await
    }

    /// Lists all caches in your account.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::requests::cache::list_caches::ListCaches;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.list_caches().await {
    ///     Ok(response) => println!("Caches: {:#?}", response.caches),
    ///     Err(e) => eprintln!("Error listing caches: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to list caches using a [ListCachesRequest].
    pub async fn list_caches(&self) -> MomentoResult<ListCaches> {
        let request = ListCachesRequest {};
        request.send(self).await
    }

    /// Flushes the cache with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the cache to be flushed of data.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::requests::cache::flush_cache::FlushCache;
    /// use momento::requests::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.flush_cache(cache_name.to_string()).await {
    ///     Ok(_) => println!("Flushed cache: {}", cache_name),
    ///     Err(e) => {
    ///         if let MomentoErrorCode::NotFoundError = e.error_code {
    ///             println!("Cache not found: {}", cache_name);
    ///         } else {
    ///            eprintln!("Error flushing cache: {}", e);
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to delete a cache using a [FlushCacheRequest].
    pub async fn flush_cache(&self, cache_name: impl Into<String>) -> MomentoResult<FlushCache> {
        let request = FlushCacheRequest::new(cache_name);
        request.send(self).await
    }

    /// Sets an item in a Momento Cache
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache
    /// * `key` - key of the item whose value we are setting
    /// * `value` - data to stored in the cache item
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to set an item using a
    /// [SetRequest], you can also provide the following optional arguments:
    ///
    /// * `ttl` - The time-to-live for the item. If not provided, the client's default time-to-live is used.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// use momento::requests::cache::basic::set::Set;
    /// use momento::requests::MomentoErrorCode;
    ///
    /// match cache_client.set(&cache_name, "k1", "v1").await {
    ///     Ok(_) => println!("Set successful"),
    ///     Err(e) => if let MomentoErrorCode::NotFoundError = e.error_code {
    ///         println!("Cache not found: {}", &cache_name);
    ///     } else {
    ///         eprintln!("Error setting value in cache {}: {}", &cache_name, e);
    ///     }
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get an item using a [SetRequest]
    /// which will allow you to set [optional arguments](SetRequest#optional-arguments) as well.
    pub async fn set(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        value: impl IntoBytes,
    ) -> MomentoResult<Set> {
        let request = SetRequest::new(cache_name, key, value);
        request.send(self).await
    }

    /// Gets an item from a Momento Cache
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache
    /// * `key` - key of entry within the cache.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// use std::convert::TryInto;
    /// use momento::requests::cache::basic::get::Get;
    /// # cache_client.set(&cache_name, "key", "value").await?;
    ///
    /// let item: String = match(cache_client.get(&cache_name, "key").await?) {
    ///     Get::Hit { value } => value.try_into().expect("I stored a string!"),
    ///     Get::Miss => return Err(anyhow::Error::msg("cache miss"))
    /// };
    /// # assert_eq!(item, "value");
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get an item using a [GetRequest].
    pub async fn get(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
    ) -> MomentoResult<Get> {
        let request = GetRequest::new(cache_name, key);
        request.send(self).await
    }

    /// Adds elements to the given set. Creates the set if it does not exist.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache containing the sorted set.
    /// * `set_name` - The name of the sorted set to add an element to.
    /// * `elements` - The elements to add. Must be able to be converted to a `Vec<u8>`.
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to add an element using a
    /// [SetAddElementsRequest], you can also provide the following optional arguments:
    ///
    /// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::requests::cache::set::set_add_elements::SetAddElements;
    /// use momento::requests::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let set_name = "set";
    ///
    /// let add_elements_response = cache_client.set_add_elements(
    ///     cache_name,
    ///     set_name,
    ///     vec!["value1", "value2"]
    /// ).await;
    ///
    /// match add_elements_response {
    ///     Ok(_) => println!("Elements added to set"),
    ///     Err(e) => eprintln!("Error adding elements to set: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get an item using a [SetAddElementsRequest]
    /// which will allow you to set [optional arguments](SetAddElementsRequest#optional-arguments) as well.
    pub async fn set_add_elements<E: IntoBytes>(
        &self,
        cache_name: impl Into<String>,
        set_name: impl IntoBytes,
        elements: Vec<E>,
    ) -> MomentoResult<SetAddElements> {
        let request = SetAddElementsRequest::new(cache_name, set_name, elements);
        request.send(self).await
    }

    /// Adds an element to the given sorted set. If the element already exists, its score is updated.
    /// Creates the sorted set if it does not exist.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache containing the sorted set.
    /// * `sorted_set_name` - The name of the sorted set to add an element to.
    /// * `value` - The value of the element to add. Must be able to be converted to a `Vec<u8>`.
    /// * `score` - The score of the element to add.
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to add an element using a
    /// [SortedSetPutElementRequest], you can also provide the following optional arguments:
    ///
    /// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::requests::cache::sorted_set::sorted_set_put_element::SortedSetPutElement;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let sorted_set_name = "sorted_set";
    ///
    /// let put_element_response = cache_client.sorted_set_put_element(
    ///     cache_name,
    ///     sorted_set_name,
    ///     "value",
    ///     1.0
    /// ).await;
    ///
    /// match put_element_response {
    ///     Ok(_) => println!("Element added to sorted set"),
    ///     Err(e) => eprintln!("Error adding element to sorted set: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get an item using a [SortedSetPutElementRequest]
    /// which will allow you to set [optional arguments](SortedSetPutElementRequest#optional-arguments) as well.
    pub async fn sorted_set_put_element(
        &self,
        cache_name: impl Into<String>,
        sorted_set_name: impl IntoBytes,
        value: impl IntoBytes,
        score: f64,
    ) -> MomentoResult<SortedSetPutElement> {
        let request = SortedSetPutElementRequest::new(cache_name, sorted_set_name, value, score);
        request.send(self).await
    }

    /// Adds elements to the given sorted set. If an element already exists, its score is updated.
    /// Creates the sorted set if it does not exist.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache containing the sorted set.
    /// * `sorted_set_name` - The name of the sorted set to add an element to.
    /// * `elements` - The values and scores to add. The values must be able to be converted to a `Vec<u8>`.
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to add elements using a
    /// [SortedSetPutElementsRequest], you can also provide the following optional arguments:
    ///
    /// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::requests::cache::sorted_set::sorted_set_put_elements::SortedSetPutElements;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let sorted_set_name = "sorted_set";
    ///
    /// let put_element_response = cache_client.sorted_set_put_elements(
    ///     cache_name,
    ///     sorted_set_name,
    ///     vec![("value1", 1.0), ("value2", 2.0)]
    /// ).await;
    ///
    /// match put_element_response {
    ///     Ok(_) => println!("Elements added to sorted set"),
    ///     Err(e) => eprintln!("Error adding elements to sorted set: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get an item using a [SortedSetPutElementsRequest]
    /// which will allow you to set [optional arguments](SortedSetPutElementsRequest#optional-arguments) as well.
    pub async fn sorted_set_put_elements<E: IntoBytes>(
        &self,
        cache_name: impl Into<String>,
        sorted_set_name: impl IntoBytes,
        elements: Vec<(E, f64)>,
    ) -> MomentoResult<SortedSetPutElements> {
        let request = SortedSetPutElementsRequest::new(cache_name, sorted_set_name, elements);
        request.send(self).await
    }

    /// Fetch the elements in the given sorted set by their rank.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache containing the sorted set.
    /// * `sorted_set_name` - The name of the sorted set to add an element to.
    /// * `order` - The order to sort the elements by. [SortOrder::Ascending] or [SortOrder::Descending].
    /// * `start_rank` - The rank of the first element to fetch. Defaults to 0. This rank is
    /// inclusive, i.e. the element at this rank will be fetched.
    /// * `end_rank` - The rank of the last element to fetch. This rank is exclusive, i.e. the
    /// element at this rank will not be fetched. Defaults to -1, which fetches up until and
    /// including the last element.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento::MomentoResult;
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::requests::cache::sorted_set::sorted_set_fetch_by_rank::SortOrder;
    /// use momento::requests::cache::sorted_set::sorted_set_fetch_response::SortedSetFetch;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let sorted_set_name = "sorted_set";
    ///
    /// let fetch_response = cache_client.sorted_set_fetch_by_rank(
    ///     cache_name,
    ///     sorted_set_name,
    ///     SortOrder::Ascending,
    ///     None,
    ///     None
    /// ).await?;
    ///
    /// match fetch_response {
    ///     SortedSetFetch::Hit{ elements } => {
    ///         match elements.into_strings() {
    ///             Ok(vec) => {
    ///                 println!("Fetched elements: {:?}", vec);
    ///             }
    ///             Err(error) => {
    ///                 eprintln!("Error: {}", error);
    ///             }
    ///         }
    ///     }
    ///     SortedSetFetch::Miss => println!("Cache miss"),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to fetch elements using a [SortedSetFetchByRankRequest]..
    pub async fn sorted_set_fetch_by_rank(
        &self,
        cache_name: impl Into<String>,
        sorted_set_name: impl IntoBytes,
        order: SortOrder,
        start_rank: Option<i32>,
        end_rank: Option<i32>,
    ) -> MomentoResult<SortedSetFetch> {
        let mut request =
            SortedSetFetchByRankRequest::new(cache_name, sorted_set_name).order(order);

        if let Some(start) = start_rank {
            request = request.start_rank(start);
        }
        if let Some(end) = end_rank {
            request = request.end_rank(end);
        }
        request.send(self).await
    }

    /// Fetch the elements in the given sorted set by their score.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache containing the sorted set.
    /// * `sorted_set_name` - The name of the sorted set to add an element to.
    /// * `order` - The order to sort the elements by. [SortOrder::Ascending] or [SortOrder::Descending].
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to fetch elements using a
    /// [SortedSetFetchByScoreRequest], you can also provide the following optional arguments:
    ///
    /// * `min_score` - The minimum score (inclusive) of the elements to fetch. Defaults to negative
    /// infinity.
    /// * `max_score` - The maximum score (inclusive) of the elements to fetch. Defaults to positive
    /// infinity.
    /// * `offset` - The number of elements to skip before returning the first element. Defaults to
    /// 0. Note: this is not the rank of the first element to return, but the number of elements of
    /// the result set to skip before returning the first element.
    /// * `count` - The maximum number of elements to return. Defaults to all elements.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento::MomentoResult;
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::requests::cache::sorted_set::sorted_set_fetch_by_rank::SortOrder;
    /// use momento::requests::cache::sorted_set::sorted_set_fetch_response::SortedSetFetch;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let sorted_set_name = "sorted_set";
    ///
    /// let fetch_response = cache_client.sorted_set_fetch_by_score(
    ///     cache_name,
    ///     sorted_set_name,
    ///     SortOrder::Ascending
    /// ).await?;
    ///
    /// match fetch_response {
    ///     SortedSetFetch::Hit{ elements } => {
    ///         match elements.into_strings() {
    ///             Ok(vec) => {
    ///                 println!("Fetched elements: {:?}", vec);
    ///             }
    ///             Err(error) => {
    ///                 eprintln!("Error: {}", error);
    ///             }
    ///         }
    ///     }
    ///     SortedSetFetch::Miss => println!("Cache miss"),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get an item using a [SortedSetFetchByScoreRequest]
    /// which will allow you to set [optional arguments](SortedSetFetchByScoreRequest#optional-arguments) as well.
    pub async fn sorted_set_fetch_by_score(
        &self,
        cache_name: impl Into<String>,
        sorted_set_name: impl IntoBytes,
        order: SortOrder,
    ) -> MomentoResult<SortedSetFetch> {
        let request = SortedSetFetchByScoreRequest::new(cache_name, sorted_set_name).order(order);
        request.send(self).await
    }

    /// Check if the provided key exists in the cache
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `key` - key to check for existence
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// use momento::requests::cache::scalar::key_exists::KeyExists;
    ///
    /// let result = cache_client.key_exists(&cache_name, "key").await?;
    /// if result.exists {
    ///     println!("Key exists!");
    /// } else {
    ///     println!("Key does not exist!");
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get an item using a [KeyExistsRequest].
    pub async fn key_exists(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
    ) -> MomentoResult<KeyExists> {
        let request = KeyExistsRequest::new(cache_name, key);
        request.send(self).await
    }

    /// Check if the provided keys exist in the cache.
    /// Returns a list of booleans indicating whether each given key was found in the cache.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `keys` - list of keys to check for existence
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// use momento::requests::cache::scalar::keys_exist::KeysExist;
    /// use std::collections::HashMap;
    ///
    /// let result = cache_client.keys_exist(&cache_name, vec!["key1", "key2", "key3"]).await?;
    /// let exists_map: HashMap<String, bool> = result.into();
    /// println!("Expecting all keys to exist: {:#?}", exists_map);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get an item using a [KeysExistRequest].
    pub async fn keys_exist(
        &self,
        cache_name: impl Into<String>,
        keys: Vec<impl IntoBytes>,
    ) -> MomentoResult<KeysExist> {
        let request = KeysExistRequest::new(cache_name, keys);
        request.send(self).await
    }

    /// Lower-level API to send any type of MomentoRequest to the server. This is used for cases when
    /// you want to set optional fields on a request that are not supported by the short-hand API for
    /// that request type.
    ///
    /// See [SortedSetFetchByRankRequest] for an example of creating a request with optional fields.
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
