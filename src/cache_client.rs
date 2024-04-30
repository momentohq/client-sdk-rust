use std::convert::TryInto;
use std::time::Duration;

use momento_protos::cache_client::scs_client::ScsClient;
use momento_protos::control_client::scs_control_client::ScsControlClient;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

use crate::cache::{
    Configuration, CreateCache, CreateCacheRequest, DecreaseTtl, DecreaseTtlRequest, Delete,
    DeleteCache, DeleteCacheRequest, DeleteRequest, DictionaryFetch, DictionaryFetchRequest,
    DictionarySetField, DictionarySetFieldRequest, FlushCache, FlushCacheRequest, Get, GetRequest,
    IncreaseTtl, IncreaseTtlRequest, Increment, IncrementRequest, IntoSortedSetElements,
    ItemGetTtl, ItemGetTtlRequest, ItemGetType, ItemGetTypeRequest, KeyExists, KeyExistsRequest,
    KeysExist, KeysExistRequest, ListCaches, ListCachesRequest, ListConcatenateBack,
    ListConcatenateBackRequest, ListConcatenateFront, ListConcatenateFrontRequest, ListFetch,
    ListFetchRequest, ListLength, ListLengthRequest, ListPopBack, ListPopBackRequest, ListPopFront,
    ListPopFrontRequest, ListRemoveValue, ListRemoveValueRequest, MomentoRequest, Set,
    SetAddElements, SetAddElementsRequest, SetIfAbsent, SetIfAbsentOrEqual,
    SetIfAbsentOrEqualRequest, SetIfAbsentRequest, SetIfEqual, SetIfEqualRequest, SetIfNotEqual,
    SetIfNotEqualRequest, SetIfPresent, SetIfPresentAndNotEqual, SetIfPresentAndNotEqualRequest,
    SetIfPresentRequest, SetRequest, SortedSetFetch, SortedSetFetchByRankRequest,
    SortedSetFetchByScoreRequest, SortedSetOrder, SortedSetPutElement, SortedSetPutElementRequest,
    SortedSetPutElements, SortedSetPutElementsRequest, UpdateTtl, UpdateTtlRequest,
};
use crate::grpc::header_interceptor::HeaderInterceptor;

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
    /// use momento::cache::CreateCache;
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
    /// use momento::cache::DeleteCache;
    /// use momento::MomentoErrorCode;
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
    /// use momento::cache::ListCaches;
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
    /// use momento::cache::FlushCache;
    /// use momento::MomentoErrorCode;
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
    /// use momento::cache::Set;
    /// use momento::MomentoErrorCode;
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
    /// use momento::cache::Get;
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

    /// Deletes an item in a Momento Cache
    ///
    /// # Arguments
    ///
    /// * `cache_name` - name of cache
    /// * `key` - key of the item to delete
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// use momento::cache::Delete;
    /// use momento::MomentoErrorCode;
    ///
    /// match cache_client.delete(&cache_name, "key").await {
    ///     Ok(_) => println!("Delete successful"),
    ///     Err(e) => if let MomentoErrorCode::NotFoundError = e.error_code {
    ///         println!("Cache not found: {}", &cache_name);
    ///     } else {
    ///         eprintln!("Error deleting value in cache {}: {}", &cache_name, e);
    ///     }
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to delete an item using a [DeleteRequest].
    pub async fn delete(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
    ) -> MomentoResult<Delete> {
        let request = DeleteRequest::new(cache_name, key);
        request.send(self).await
    }

    /// Fetches a dictionary from a cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache containing the dictionary.
    /// * `dictionary_name` - The name of the dictionary to fetch.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use std::collections::HashMap;
    /// # use std::convert::TryInto;
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::{DictionaryFetchRequest, DictionaryFetch};
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let dictionary_name = "dictionary";
    ///
    /// let set_response = cache_client.dictionary_set_field(
    ///   cache_name.to_string(),
    ///   dictionary_name,
    ///   "field1",
    ///   "value1"
    /// ).await?;
    ///
    /// let fetch_response = cache_client.dictionary_fetch(cache_name, dictionary_name).await?;
    ///
    /// match fetch_response {
    ///    DictionaryFetch::Hit{ value } => {
    ///       let dictionary: HashMap<String, String> = value.try_into().expect("I stored a dictionary!");
    ///       println!("Fetched dictionary: {:?}", dictionary);
    ///    }
    ///    DictionaryFetch::Miss => println!("Cache miss"),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to fetch a dictionary using a [DictionaryFetchRequest].
    pub async fn dictionary_fetch(
        &self,
        cache_name: impl Into<String>,
        dictionary_name: impl IntoBytes,
    ) -> MomentoResult<DictionaryFetch> {
        let request = DictionaryFetchRequest::new(cache_name, dictionary_name);
        request.send(self).await
    }
    // dictionary_get_field
    // dictionary_get_fields
    // dictionary_increment
    // dictionary_length
    // dictionary_remove_field
    // dictionary_remove_fields

    /// Sets a field in a dictionary. If the field already exists, its value is updated.
    /// Creates the dictionary if it does not exist.
    ///
    /// # Arguments
    /// * `cache_name` - The name of the cache containing the dictionary.
    /// * `dictionary_name` - The name of the dictionary to set.
    /// * `field` - The field to set.
    /// * `value` - The value to set.
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to set a field using a
    /// [DictionarySetFieldRequest], you can also provide the following optional arguments:
    ///
    /// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::{CollectionTtl, DictionarySetField, DictionarySetFieldRequest};
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let dictionary_name = "dictionary";
    ///
    /// let set_field_response = cache_client.dictionary_set_field(
    ///    cache_name.to_string(),
    ///    dictionary_name,
    ///    "field",
    ///    "value"
    /// ).await;
    ///
    /// match set_field_response {
    ///   Ok(_) => println!("Field set in dictionary"),
    ///   Err(e) => eprintln!("Error setting field in dictionary: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to set a field using a [DictionarySetFieldRequest]
    /// which will allow you to set [optional arguments](DictionarySetFieldRequest#optional-arguments) as well.
    pub async fn dictionary_set_field(
        &self,
        cache_name: impl Into<String>,
        dictionary_name: impl IntoBytes,
        field: impl IntoBytes,
        value: impl IntoBytes,
    ) -> MomentoResult<DictionarySetField> {
        let request = DictionarySetFieldRequest::new(cache_name, dictionary_name, field, value);
        request.send(self).await
    }

    // dictionary_set_fields

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
    /// use momento::cache::SetAddElements;
    /// use momento::MomentoErrorCode;
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
    /// use momento::cache::SortedSetPutElement;
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
    /// use momento::cache::{SortedSetPutElements};
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
    /// You can also use the [send_request](CacheClient::send_request) method to put elements using a [SortedSetPutElementsRequest]
    /// which will allow you to set [optional arguments](SortedSetPutElementsRequest#optional-arguments) as well.
    pub async fn sorted_set_put_elements<V: IntoBytes>(
        &self,
        cache_name: impl Into<String>,
        sorted_set_name: impl IntoBytes,
        elements: impl IntoSortedSetElements<V>,
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
    /// * `order` - The order to sort the elements by. [SortedSetOrder::Ascending] or [SortedSetOrder::Descending].
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
    /// use momento::cache::{SortedSetOrder, SortedSetFetch};
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let sorted_set_name = "sorted_set";
    ///
    /// let fetch_response = cache_client.sorted_set_fetch_by_rank(
    ///     cache_name,
    ///     sorted_set_name,
    ///     SortedSetOrder::Ascending,
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
        order: SortedSetOrder,
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
    /// * `order` - The order to sort the elements by. [SortedSetOrder::Ascending] or [SortedSetOrder::Descending].
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
    /// use momento::cache::{SortedSetOrder, SortedSetFetch};
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let sorted_set_name = "sorted_set";
    ///
    /// let fetch_response = cache_client.sorted_set_fetch_by_score(
    ///     cache_name,
    ///     sorted_set_name,
    ///     SortedSetOrder::Ascending
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
        order: SortedSetOrder,
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
    /// use momento::cache::KeyExists;
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
    /// use momento::cache::KeysExist;
    /// use std::collections::HashMap;
    ///
    /// // Receive results as a HashMap
    /// let result_map: HashMap<String, bool> = cache_client.keys_exist(&cache_name, vec!["key1", "key2", "key3"]).await?.into();
    /// println!("Expecting all keys to exist: {:#?}", result_map);
    ///
    /// // Or receive results as a Vec
    /// let result_list: Vec<bool> = cache_client.keys_exist(&cache_name, vec!["key1", "key2", "key3"]).await?.into();
    /// println!("Expecting all keys to exist: {:#?}", result_list);
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

    /// Adds an integer quantity to a field value.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `field` - the field to increment
    /// * `amount` - the quantity to add to the value. May be positive, negative, or zero. Defaults to 1.
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to increment a field using an
    /// [IncrementRequest], you can also provide the following optional arguments:
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
    /// use momento::cache::Increment;
    /// use momento::MomentoErrorCode;
    ///
    /// match cache_client.increment(&cache_name, "key", 1).await {
    ///     Ok(r) => println!("Incremented value: {}", r.value),
    ///     Err(e) => if let MomentoErrorCode::NotFoundError = e.error_code {
    ///         println!("Cache not found: {}", &cache_name);
    ///     } else {
    ///         eprintln!("Error incrementing value in cache {}: {}", &cache_name, e);
    ///     }
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get an item using an [IncrementRequest]
    /// which will allow you to set [optional arguments](IncrementRequest#optional-arguments) as well.
    pub async fn increment(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        amount: i64,
    ) -> MomentoResult<Increment> {
        let request = IncrementRequest::new(cache_name, key, amount);
        request.send(self).await
    }

    /// Return the type of the key in the cache.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `key` - the key for which type is requested
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// use std::convert::TryInto;
    /// use momento::cache::{ItemGetType, ItemType};
    /// # cache_client.set(&cache_name, "key1", "value").await?;
    ///
    /// let item: ItemType = match(cache_client.item_get_type(&cache_name, "key1").await?) {
    ///     ItemGetType::Hit { key_type } => key_type.try_into().expect("Expected an item type!"),
    ///     ItemGetType::Miss => return Err(anyhow::Error::msg("cache miss"))
    /// };
    /// # assert_eq!(item, ItemType::Scalar);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get an item's type using a [ItemGetTypeRequest].
    pub async fn item_get_type(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
    ) -> MomentoResult<ItemGetType> {
        let request = ItemGetTypeRequest::new(cache_name, key);
        request.send(self).await
    }

    /// Return the remaining ttl of the key in the cache
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `key` - the key for which remaining ttl is requested
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// use std::convert::TryInto;
    /// use momento::cache::ItemGetTtl;
    /// use std::time::Duration;
    /// # cache_client.set(&cache_name, "key1", "value").await?;
    ///
    /// let remaining_ttl: Duration = cache_client.item_get_ttl(cache_name, "key1").await?.try_into().expect("Expected an item ttl!");
    /// # assert!(remaining_ttl <= Duration::from_secs(5));
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get an item's ttl using a [ItemGetTtlRequest].
    pub async fn item_get_ttl(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
    ) -> MomentoResult<ItemGetTtl> {
        let request = ItemGetTtlRequest::new(cache_name, key);
        request.send(self).await
    }

    /// Update the ttl of the key in the cache.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `key` - the key for which ttl is requested
    /// * `ttl` - The time-to-live that should overwrite the current ttl.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// use std::time::Duration;
    /// use momento::cache::UpdateTtl;
    /// # cache_client.set(&cache_name, "key1", "value").await?;
    ///
    /// match(cache_client.update_ttl(&cache_name, "key1", Duration::from_secs(10)).await?) {
    ///     UpdateTtl::Set => println!("TTL updated"),
    ///     UpdateTtl::Miss => return Err(anyhow::Error::msg("cache miss"))
    /// };
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to update an item's ttl using a [UpdateTtlRequest].
    pub async fn update_ttl(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        ttl: Duration,
    ) -> MomentoResult<UpdateTtl> {
        let request = UpdateTtlRequest::new(cache_name, key, ttl);
        request.send(self).await
    }

    /// Increase the ttl of the key in the cache.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `key` - the key for which ttl is requested
    /// * `ttl` - The time-to-live that should overwrite the current ttl. Should be greater than the current ttl.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// use std::time::Duration;
    /// use momento::cache::IncreaseTtl;
    /// # cache_client.set(&cache_name, "key1", "value").await?;
    ///
    /// match(cache_client.increase_ttl(&cache_name, "key1", Duration::from_secs(5)).await?) {
    ///     IncreaseTtl::Set => println!("TTL updated"),
    ///     IncreaseTtl::NotSet => println!("unable to increase TTL"),
    ///     IncreaseTtl::Miss => return Err(anyhow::Error::msg("cache miss"))
    /// };
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to increase an item's ttl using a [IncreaseTtlRequest].
    pub async fn increase_ttl(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        ttl: Duration,
    ) -> MomentoResult<IncreaseTtl> {
        let request = IncreaseTtlRequest::new(cache_name, key, ttl);
        request.send(self).await
    }

    /// Decrease the ttl of the key in the cache.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `key` - the key for which ttl is requested
    /// * `ttl` - The time-to-live that should overwrite the current ttl. Should be less than the current ttl.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// use std::time::Duration;
    /// use momento::cache::DecreaseTtl;
    /// # cache_client.set(&cache_name, "key1", "value").await?;
    ///
    /// match(cache_client.decrease_ttl(&cache_name, "key1", Duration::from_secs(3)).await?) {
    ///     DecreaseTtl::Set => println!("TTL updated"),
    ///     DecreaseTtl::NotSet => println!("unable to decrease TTL"),
    ///     DecreaseTtl::Miss => return Err(anyhow::Error::msg("cache miss"))
    /// };
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to decrease an item's ttl using a [DecreaseTtlRequest].
    pub async fn decrease_ttl(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        ttl: Duration,
    ) -> MomentoResult<DecreaseTtl> {
        let request = DecreaseTtlRequest::new(cache_name, key, ttl);
        request.send(self).await
    }

    /// Associate the given key with the given value if key is not already present in the cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache to create.
    /// * `key` - key of the item whose value we are setting
    /// * `value` - data to store
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to conditionally set a field using an
    /// [SetIfAbsentRequest], you can also provide the following optional arguments:
    ///
    /// * `ttl` - The time-to-live for the item. If not provided, the client's default time-to-live is used.
    ///
    /// # Example
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use momento::cache::{SetIfAbsent, SetIfAbsentRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.set_if_absent(&cache_name, "key", "value1").await {
    ///     Ok(response) => match response {
    ///         SetIfAbsent::Stored => println!("Value stored"),
    ///         SetIfAbsent::NotStored => println!("Value not stored"),
    ///     }
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
    /// You can also use the [send_request](CacheClient::send_request) method to conditionally set an item using a [SetIfAbsentRequest].
    pub async fn set_if_absent(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        value: impl IntoBytes,
    ) -> MomentoResult<SetIfAbsent> {
        let request = SetIfAbsentRequest::new(cache_name, key, value);
        request.send(self).await
    }

    /// Associate the given key with the given value if key is present in the cache.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache to create.
    /// * `key` - key of the item whose value we are setting
    /// * `value` - data to store
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to conditionally set a field using an
    /// [SetIfPresentRequest], you can also provide the following optional arguments:
    ///
    /// * `ttl` - The time-to-live for the item. If not provided, the client's default time-to-live is used.
    ///
    /// # Example
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use momento::cache::{SetIfPresent, SetIfPresentRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.set_if_present(&cache_name, "key", "value1").await {
    ///     Ok(response) => match response {
    ///         SetIfPresent::Stored => println!("Value stored"),
    ///         SetIfPresent::NotStored => println!("Value not stored"),
    ///     }
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
    /// You can also use the [send_request](CacheClient::send_request) method to conditionally set an item using a [SetIfPresentRequest].
    pub async fn set_if_present(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        value: impl IntoBytes,
    ) -> MomentoResult<SetIfPresent> {
        let request = SetIfPresentRequest::new(cache_name, key, value);
        request.send(self).await
    }

    /// Associates the given key with the given value if the key is present
    /// in the cache and its value is equal to the supplied `equal` value.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache to create.
    /// * `key` - key of the item whose value we are setting
    /// * `value` - data to store
    /// * `equal` - data to compare to the cached value
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to conditionally set a field using an
    /// [SetIfEqualRequest], you can also provide the following optional arguments:
    ///
    /// * `ttl` - The time-to-live for the item. If not provided, the client's default time-to-live is used.
    ///
    /// # Example
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use momento::cache::{SetIfEqual, SetIfEqualRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.set_if_equal(&cache_name, "key", "new-value", "cached-value").await {
    ///     Ok(response) => match response {
    ///         SetIfEqual::Stored => println!("Value stored"),
    ///         SetIfEqual::NotStored => println!("Value not stored"),
    ///     }
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
    /// You can also use the [send_request](CacheClient::send_request) method to conditionally set an item using a [SetIfEqualRequest].
    pub async fn set_if_equal(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        value: impl IntoBytes,
        equal: impl IntoBytes,
    ) -> MomentoResult<SetIfEqual> {
        let request = SetIfEqualRequest::new(cache_name, key, value, equal);
        request.send(self).await
    }

    /// Associates the given key with the given value if the key does not already exist in the
    /// cache or the value in the cache is not equal to the value supplied `not_equal` value.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache to create.
    /// * `key` - key of the item whose value we are setting
    /// * `value` - data to store
    /// * `not_equal` - data to compare to the cached value
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to conditionally set a field using an
    /// [SetIfNotEqualRequest], you can also provide the following optional arguments:
    ///
    /// * `ttl` - The time-to-live for the item. If not provided, the client's default time-to-live is used.
    ///
    /// # Example
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use momento::cache::{SetIfNotEqual, SetIfNotEqualRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.set_if_not_equal(&cache_name, "key", "new-value", "cached-value").await {
    ///     Ok(response) => match response {
    ///         SetIfNotEqual::Stored => println!("Value stored"),
    ///         SetIfNotEqual::NotStored => println!("Value not stored"),
    ///     }
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
    /// You can also use the [send_request](CacheClient::send_request) method to conditionally set an item using a [SetIfNotEqualRequest].
    pub async fn set_if_not_equal(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        value: impl IntoBytes,
        not_equal: impl IntoBytes,
    ) -> MomentoResult<SetIfNotEqual> {
        let request = SetIfNotEqualRequest::new(cache_name, key, value, not_equal);
        request.send(self).await
    }

    /// Associates the given key with the given value if the key exists in the cache
    /// and the value in the cache is not equal to the value supplied `not_equal` value.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache to create.
    /// * `key` - key of the item whose value we are setting
    /// * `value` - data to store
    /// * `not_equal` - data to compare to the cached value
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to conditionally set a field using an
    /// [SetIfPresentAndNotEqualRequest], you can also provide the following optional arguments:
    ///
    /// * `ttl` - The time-to-live for the item. If not provided, the client's default time-to-live is used.
    ///
    /// # Example
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use momento::cache::{SetIfPresentAndNotEqual, SetIfPresentAndNotEqualRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.set_if_present_and_not_equal(&cache_name, "key", "new-value", "cached-value").await {
    ///     Ok(response) => match response {
    ///         SetIfPresentAndNotEqual::Stored => println!("Value stored"),
    ///         SetIfPresentAndNotEqual::NotStored => println!("Value not stored"),
    ///     }
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
    /// You can also use the [send_request](CacheClient::send_request) method to conditionally set an item using a [SetIfPresentAndNotEqualRequest].
    pub async fn set_if_present_and_not_equal(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        value: impl IntoBytes,
        not_equal: impl IntoBytes,
    ) -> MomentoResult<SetIfPresentAndNotEqual> {
        let request = SetIfPresentAndNotEqualRequest::new(cache_name, key, value, not_equal);
        request.send(self).await
    }

    /// Associate the given key with the given value if the key does not already
    /// exist in the cache or the value in the cache is equal to the supplied `equal` value.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache to create.
    /// * `key` - key of the item whose value we are setting
    /// * `value` - data to store
    /// * `equal` - data to compare to the cached value
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to conditionally set a field using an
    /// [SetIfAbsentOrEqualRequest], you can also provide the following optional arguments:
    ///
    /// * `ttl` - The time-to-live for the item. If not provided, the client's default time-to-live is used.
    ///
    /// # Example
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use momento::cache::{SetIfAbsentOrEqual, SetIfAbsentOrEqualRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.set_if_absent_or_equal(&cache_name, "key", "new-value", "cached-value").await {
    ///     Ok(response) => match response {
    ///         SetIfAbsentOrEqual::Stored => println!("Value stored"),
    ///         SetIfAbsentOrEqual::NotStored => println!("Value not stored"),
    ///     }
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
    /// You can also use the [send_request](CacheClient::send_request) method to conditionally set an item using a [SetIfAbsentOrEqualRequest].
    pub async fn set_if_absent_or_equal(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        value: impl IntoBytes,
        equal: impl IntoBytes,
    ) -> MomentoResult<SetIfAbsentOrEqual> {
        let request = SetIfAbsentOrEqualRequest::new(cache_name, key, value, equal);
        request.send(self).await
    }

    /// Gets the number of elements in the given list.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `list_name` - name of the list
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::convert::TryInto;
    /// use momento::cache::ListLength;
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let list_name = "list-name";
    /// # cache_client.list_concatenate_front(&cache_name, list_name, vec!["value1", "value2"]).await;
    ///
    /// let length: u32 = cache_client.list_length(&cache_name, list_name).await?.try_into().expect("Expected a list length!");
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get a list's length using a [ListLengthRequest].
    pub async fn list_length(
        &self,
        cache_name: impl Into<String>,
        list_name: impl IntoBytes,
    ) -> MomentoResult<ListLength> {
        let request = ListLengthRequest::new(cache_name, list_name);
        request.send(self).await
    }

    /// Adds multiple elements to the front of the given list. Creates the list if it does not already exist.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `list_name` - name of the list
    /// * `values` - list of values to add to the front of the list
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to add elements to the front of a list using a [ListConcatenateFrontRequest],
    /// you can also provide the following optional arguments:
    ///
    /// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
    /// * `truncate_back_to_size` - If the list exceeds this length, remove excess from the back of the list.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::ListConcatenateFront;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let list_name = "list-name";
    ///
    /// let concat_front_response = cache_client.list_concatenate_front(
    ///     cache_name,
    ///     list_name,
    ///     vec!["value1", "value2"]
    /// ).await;
    ///
    /// match concat_front_response {
    ///     Ok(_) => println!("Elements added to list"),
    ///     Err(e) => eprintln!("Error adding elements to list: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to concatenate an item using a [ListConcatenateFrontRequest]
    /// which will allow you to set [optional arguments](ListConcatenateFrontRequest#optional-arguments) as well.
    pub async fn list_concatenate_front(
        &self,
        cache_name: impl Into<String>,
        list_name: impl IntoBytes,
        values: Vec<impl IntoBytes>,
    ) -> MomentoResult<ListConcatenateFront> {
        let request = ListConcatenateFrontRequest::new(cache_name, list_name, values);
        request.send(self).await
    }

    /// Adds multiple elements to the back of the given list. Creates the list if it does not already exist.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `list_name` - name of the list
    /// * `values` - list of values to add to the back of the list
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to add elements to the back of a list using a [ListConcatenateBackRequest],
    /// you can also provide the following optional arguments:
    ///
    /// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
    /// * `truncate_front_to_size` - If the list exceeds this length, remove excess from the front of the list.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::ListConcatenateBack;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let list_name = "list-name";
    ///
    /// let concat_front_response = cache_client.list_concatenate_back(
    ///     cache_name,
    ///     list_name,
    ///     vec!["value1", "value2"]
    /// ).await;
    ///
    /// match concat_front_response {
    ///     Ok(_) => println!("Elements added to list"),
    ///     Err(e) => eprintln!("Error adding elements to list: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to concatenate an item using a [ListConcatenateBackRequest]
    /// which will allow you to set [optional arguments](ListConcatenateBackRequest#optional-arguments) as well.
    pub async fn list_concatenate_back(
        &self,
        cache_name: impl Into<String>,
        list_name: impl IntoBytes,
        values: Vec<impl IntoBytes>,
    ) -> MomentoResult<ListConcatenateBack> {
        let request = ListConcatenateBackRequest::new(cache_name, list_name, values);
        request.send(self).await
    }

    /// Gets a list item from a cache with optional slices.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `list_name` - name of the list
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to fetch a list using a [ListFetchRequest],
    /// you can also provide the following optional arguments:
    ///
    /// * `start_index` - The starting inclusive element of the list to fetch. Default is 0.
    /// * `end_index` - The ending exclusive element of the list to fetch. Default is end of list.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::convert::TryInto;
    /// use momento::cache::ListFetch;
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let list_name = "list-name";
    /// # cache_client.list_concatenate_front(&cache_name, list_name, vec!["value1", "value2"]).await;
    ///
    /// let fetched_values: Vec<String> = cache_client.list_fetch(cache_name, list_name).await?.try_into().expect("Expected a list fetch!");
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to fetch a list using a [ListFetchRequest]
    /// which will allow you to set [optional arguments](ListFetchRequest#optional-arguments) as well.
    pub async fn list_fetch(
        &self,
        cache_name: impl Into<String>,
        list_name: impl IntoBytes,
    ) -> MomentoResult<ListFetch> {
        let request = ListFetchRequest::new(cache_name, list_name);
        request.send(self).await
    }

    /// Remove and return the last element from a list item.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `list_name` - name of the list
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::convert::TryInto;
    /// use momento::cache::{ListPopBack, ListPopBackRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let list_name = "list-name";
    /// # cache_client.list_concatenate_front(&cache_name, list_name, vec!["value1", "value2"]).await;
    ///
    /// let popped_value: String = cache_client.list_pop_back(cache_name, list_name).await?.try_into().expect("Expected a popped list value!");
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to pop a value off the list using a [ListPopBackRequest].
    pub async fn list_pop_back(
        &self,
        cache_name: impl Into<String>,
        list_name: impl IntoBytes,
    ) -> MomentoResult<ListPopBack> {
        let request = ListPopBackRequest::new(cache_name, list_name);
        request.send(self).await
    }

    /// Remove and return the first element from a list item.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `list_name` - name of the list
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::convert::TryInto;
    /// use momento::cache::{ListPopFront, ListPopFrontRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let list_name = "list-name";
    /// # cache_client.list_concatenate_front(&cache_name, list_name, vec!["value1", "value2"]).await;
    ///
    /// let popped_value: String = cache_client.list_pop_front(cache_name, list_name).await?.try_into().expect("Expected a popped list value!");
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to pop a value off the list using a [ListPopFrontRequest].
    pub async fn list_pop_front(
        &self,
        cache_name: impl Into<String>,
        list_name: impl IntoBytes,
    ) -> MomentoResult<ListPopFront> {
        let request = ListPopFrontRequest::new(cache_name, list_name);
        request.send(self).await
    }

    /// Remove all elements in a list item equal to a particular value.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `list_name` - name of the list
    /// * `value` - value to remove
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::{ListRemoveValue, ListRemoveValueRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let list_name = "list-name";
    /// # cache_client.list_concatenate_front(&cache_name, list_name, vec!["value1", "value2"]).await;
    ///
    /// match cache_client.list_remove_value(cache_name, list_name, "value1").await {
    ///     Ok(ListRemoveValue {}) => println!("Successfully removed value"),
    ///     Err(e) => eprintln!("Error removing value: {:?}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to remove a value from the list using a [ListRemoveValueRequest].
    pub async fn list_remove_value(
        &self,
        cache_name: impl Into<String>,
        list_name: impl IntoBytes,
        value: impl IntoBytes,
    ) -> MomentoResult<ListRemoveValue> {
        let request = ListRemoveValueRequest::new(cache_name, list_name, value);
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
