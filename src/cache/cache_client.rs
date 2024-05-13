use std::convert::TryInto;
use std::time::Duration;

use momento_protos::cache_client::scs_client::ScsClient;
use momento_protos::control_client::scs_control_client::ScsControlClient;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

use crate::cache::{
    Configuration, CreateCache, CreateCacheRequest, DecreaseTtlRequest, DecreaseTtlResponse,
    DeleteCache, DeleteCacheRequest, DeleteRequest, DeleteResponse, DictionaryFetchRequest,
    DictionaryFetchResponse, DictionaryGetFieldRequest, DictionaryGetFieldResponse,
    DictionaryGetFieldsRequest, DictionaryGetFieldsResponse, DictionaryIncrementRequest,
    DictionaryIncrementResponse, DictionaryLengthRequest, DictionaryLengthResponse,
    DictionaryRemoveFieldRequest, DictionaryRemoveFieldResponse, DictionaryRemoveFieldsRequest,
    DictionaryRemoveFieldsResponse, DictionarySetFieldRequest, DictionarySetFieldResponse,
    DictionarySetFieldsRequest, DictionarySetFieldsResponse, FlushCache, FlushCacheRequest,
    GetRequest, GetResponse, IncreaseTtlRequest, IncreaseTtlResponse, IncrementRequest,
    IncrementResponse, IntoDictionaryFieldValuePairs, IntoSortedSetElements, ItemGetTtlRequest,
    ItemGetTtlResponse, ItemGetTypeRequest, ItemGetTypeResponse, KeyExistsRequest,
    KeyExistsResponse, KeysExistRequest, KeysExistResponse, ListCaches, ListCachesRequest,
    ListConcatenateBackRequest, ListConcatenateBackResponse, ListConcatenateFrontRequest,
    ListConcatenateFrontResponse, ListFetchRequest, ListFetchResponse, ListLengthRequest,
    ListLengthResponse, ListPopBackRequest, ListPopBackResponse, ListPopFrontRequest,
    ListPopFrontResponse, ListPushBackRequest, ListPushBackResponse, ListPushFrontRequest,
    ListPushFrontResponse, ListRemoveValueRequest, ListRemoveValueResponse, MomentoRequest,
    SetAddElementsRequest, SetAddElementsResponse, SetFetchRequest, SetFetchResponse,
    SetIfAbsentOrEqualRequest, SetIfAbsentOrEqualResponse, SetIfAbsentRequest, SetIfAbsentResponse,
    SetIfEqualRequest, SetIfEqualResponse, SetIfNotEqualRequest, SetIfNotEqualResponse,
    SetIfPresentAndNotEqualRequest, SetIfPresentAndNotEqualResponse, SetIfPresentRequest,
    SetIfPresentResponse, SetRemoveElementsRequest, SetRemoveElementsResponse, SetRequest,
    SetResponse, SortedSetFetch, SortedSetFetchByRankRequest, SortedSetFetchByScoreRequest,
    SortedSetGetRank, SortedSetGetRankRequest, SortedSetGetScore, SortedSetGetScoreRequest,
    SortedSetLength, SortedSetLengthRequest, SortedSetOrder, SortedSetPutElement,
    SortedSetPutElementRequest, SortedSetPutElements, SortedSetPutElementsRequest,
    SortedSetRemoveElements, SortedSetRemoveElementsRequest, UpdateTtlRequest, UpdateTtlResponse,
};
use crate::grpc::header_interceptor::HeaderInterceptor;

use crate::cache::cache_client_builder::{CacheClientBuilder, NeedsDefaultTtl};
use crate::utils::IntoBytesIterable;
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
    /// use momento::cache::SetResponse;
    /// use momento::MomentoErrorCode;
    ///
    /// match cache_client.set(&cache_name, "k1", "v1").await {
    ///     Ok(_) => println!("SetResponse successful"),
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
    ) -> MomentoResult<SetResponse> {
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
    /// use momento::cache::GetResponse;
    /// # cache_client.set(&cache_name, "key", "value").await?;
    ///
    /// let item: String = match(cache_client.get(&cache_name, "key").await?) {
    ///     GetResponse::Hit { value } => value.try_into().expect("I stored a string!"),
    ///     GetResponse::Miss => return Err(anyhow::Error::msg("cache miss"))
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
    ) -> MomentoResult<GetResponse> {
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
    /// use momento::cache::DeleteResponse;
    /// use momento::MomentoErrorCode;
    ///
    /// match cache_client.delete(&cache_name, "key").await {
    ///     Ok(_) => println!("DeleteResponse successful"),
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
    ) -> MomentoResult<DeleteResponse> {
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
    /// use momento::cache::{DictionaryFetchRequest, DictionaryFetchResponse};
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
    ///    DictionaryFetchResponse::Hit{ value } => {
    ///       let dictionary: HashMap<String, String> = value.try_into().expect("I stored a dictionary!");
    ///       println!("Fetched dictionary: {:?}", dictionary);
    ///    }
    ///    DictionaryFetchResponse::Miss => println!("Cache miss"),
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
    ) -> MomentoResult<DictionaryFetchResponse> {
        let request = DictionaryFetchRequest::new(cache_name, dictionary_name);
        request.send(self).await
    }

    /// Gets a field from a dictionary.
    /// If the dictionary does not exist, a miss is returned.
    /// If the field does not exist, a miss is returned.
    ///
    /// # Arguments
    /// * `cache_name` - The name of the cache containing the dictionary.
    /// * `dictionary_name` - The name of the dictionary to get the field from.
    /// * `field` - The field to get.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # use std::convert::TryInto;
    /// # tokio_test::block_on(async {
    /// use momento::cache::DictionaryGetFieldResponse;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let dictionary_name = "dictionary";
    /// let field = "field";
    ///
    /// let set_response = cache_client.dictionary_set_field(
    ///   cache_name.to_string(),
    ///   dictionary_name,
    ///   field,
    ///   "value"
    /// ).await?;
    ///
    /// let response = cache_client.dictionary_get_field(cache_name, dictionary_name, field).await?;
    ///
    /// match response {
    ///   DictionaryGetFieldResponse::Hit { value } => {
    ///     let value: String = value.try_into().expect("I stored a string!");
    ///     println!("Fetched value: {}", value);
    ///   }
    ///   DictionaryGetFieldResponse::Miss => println!("Cache miss"),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get a field using a [DictionaryGetFieldRequest].
    pub async fn dictionary_get_field(
        &self,
        cache_name: impl Into<String>,
        dictionary_name: impl IntoBytes,
        field: impl IntoBytes,
    ) -> MomentoResult<DictionaryGetFieldResponse> {
        let request = DictionaryGetFieldRequest::new(cache_name, dictionary_name, field);
        request.send(self).await
    }

    /// Gets fields from a dictionary.
    ///
    /// If the dictionary does not exist, a miss is returned.
    /// If a field does not exist, it is not included in the response.
    ///
    /// # Arguments
    /// * `cache_name` - The name of the cache containing the dictionary.
    /// * `dictionary_name` - The name of the dictionary to get fields from.
    /// * `fields` - The fields to get.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use std::collections::HashMap;
    /// # use std::convert::TryInto;
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::{DictionaryGetFieldsResponse, DictionaryGetFieldsRequest};
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let dictionary_name = "dictionary";
    /// let fields = vec!["field1", "field2"];
    ///
    /// let set_response = cache_client.dictionary_set_fields(
    ///  cache_name.to_string(),
    /// dictionary_name,
    /// vec![("field1", "value1"), ("field2", "value2")]
    /// ).await?;
    ///
    /// let response = cache_client.dictionary_get_fields(cache_name, dictionary_name, fields).await?;
    ///
    /// match response {
    ///    DictionaryGetFieldsResponse::Hit { .. } => {
    ///      let dictionary: HashMap<String, String> = response.try_into().expect("I stored a dictionary of strings!");
    ///      println!("Fetched dictionary: {:?}", dictionary);
    ///    }
    ///    DictionaryGetFieldsResponse::Miss => println!("Cache miss"),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get fields using a [DictionaryGetFieldsRequest].
    pub async fn dictionary_get_fields<F: IntoBytesIterable + Clone>(
        &self,
        cache_name: impl Into<String>,
        dictionary_name: impl IntoBytes,
        fields: F,
    ) -> MomentoResult<DictionaryGetFieldsResponse<F>> {
        let request = DictionaryGetFieldsRequest::new(cache_name, dictionary_name, fields);
        request.send(self).await
    }

    /// Increments a field in a dictionary.
    /// If the dictionary does not exist, it is created and the field is set to the amount.
    /// If the field does not exist, it is created and set to the amount.
    /// If the value is not an integer, a type error is returned.
    ///
    /// # Arguments
    /// * `cache_name` - The name of the cache containing the dictionary.
    /// * `dictionary_name` - The name of the dictionary to increment the field in.
    /// * `field` - The field to increment.
    /// * `amount` - The amount to increment the field by.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::DictionaryIncrementResponse;
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let dictionary_name = "dictionary";
    /// let field = "field";
    /// let amount = 1;
    ///
    /// let response = cache_client.dictionary_increment(&cache_name, dictionary_name, field, amount).await;
    ///
    /// match response {
    ///   Ok(DictionaryIncrementResponse { value }) => println!("Incremented value: {}", value),
    ///   Err(e) => println!("Error incrementing value: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to increment a field using a [DictionaryIncrementRequest].
    pub async fn dictionary_increment(
        &self,
        cache_name: impl Into<String>,
        dictionary_name: impl IntoBytes,
        field: impl IntoBytes,
        amount: i64,
    ) -> MomentoResult<DictionaryIncrementResponse> {
        let request = DictionaryIncrementRequest::new(cache_name, dictionary_name, field, amount);
        request.send(self).await
    }

    /// Gets the number of elements in the given dictionary.
    /// If the dictionary does not exist, a miss is returned.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `dictionary_name` - name of the dictionary
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # use std::convert::TryInto;
    /// # tokio_test::block_on(async {
    /// use momento::cache::{DictionaryLengthResponse, DictionaryLengthRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let dictionary_name = "dictionary-name";
    /// # cache_client.dictionary_set_fields(&cache_name, dictionary_name, vec![("field1", "value1"), ("field2", "value2")]).await;
    ///
    /// let length: u32 = cache_client.dictionary_length(&cache_name, dictionary_name).await?.try_into().expect("Expected a dictionary length!");
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get the length of a dictionary using a [DictionaryLengthRequest].
    pub async fn dictionary_length(
        &self,
        cache_name: impl Into<String>,
        dictionary_name: impl IntoBytes,
    ) -> MomentoResult<DictionaryLengthResponse> {
        let request = DictionaryLengthRequest::new(cache_name, dictionary_name);
        request.send(self).await
    }

    /// Removes a field from a dictionary.
    /// If the dictionary or the field does not exist, a success response is returned.
    ///
    /// # Arguments
    /// * `cache_name` - The name of the cache containing the dictionary.
    /// * `dictionary_name` - The name of the dictionary to remove the field from.
    /// * `field` - The field to remove.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::{DictionaryRemoveFieldResponse, DictionaryRemoveFieldRequest};
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let dictionary_name = "dictionary";
    /// let field = "field";
    ///
    /// let remove_field_response = cache_client.dictionary_remove_field(cache_name, dictionary_name, field).await;
    ///
    /// match remove_field_response {
    ///   Ok(_) => println!("Field removed from dictionary"),
    ///   Err(e) => eprintln!("Error removing field from dictionary: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to remove a field using a [DictionaryRemoveFieldRequest].
    pub async fn dictionary_remove_field(
        &self,
        cache_name: impl Into<String>,
        dictionary_name: impl IntoBytes,
        field: impl IntoBytes,
    ) -> MomentoResult<DictionaryRemoveFieldResponse> {
        let request = DictionaryRemoveFieldRequest::new(cache_name, dictionary_name, field);
        request.send(self).await
    }

    /// Removes fields from a dictionary.
    /// If the dictionary or any field does not exist, a success response is returned.
    ///
    /// # Arguments
    /// * `cache_name` - The name of the cache containing the dictionary.
    /// * `dictionary_name` - The name of the dictionary to remove fields from.
    /// * `fields` - The fields to remove.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::{DictionaryRemoveFieldsResponse, DictionaryRemoveFieldsRequest};
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let dictionary_name = "dictionary";
    /// let fields = vec!["field1", "field2"];
    ///
    /// let set_response = cache_client.dictionary_set_fields(
    ///   cache_name.to_string(),
    ///   dictionary_name,
    ///   vec![("field1", "value1"), ("field2", "value2")]
    /// ).await?;
    ///
    /// let response = cache_client.dictionary_remove_fields(cache_name, dictionary_name, fields).await;
    ///
    /// match response {
    ///   Ok(_) => println!("Fields removed successfully"),
    ///   Err(e) => println!("Error removing fields: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to remove fields using a [DictionaryRemoveFieldsRequest].
    pub async fn dictionary_remove_fields<F: IntoBytesIterable>(
        &self,
        cache_name: impl Into<String>,
        dictionary_name: impl IntoBytes,
        fields: F,
    ) -> MomentoResult<DictionaryRemoveFieldsResponse> {
        let request = DictionaryRemoveFieldsRequest::new(cache_name, dictionary_name, fields);
        request.send(self).await
    }

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
    /// use momento::cache::{CollectionTtl, DictionarySetFieldResponse, DictionarySetFieldRequest};
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
    ) -> MomentoResult<DictionarySetFieldResponse> {
        let request = DictionarySetFieldRequest::new(cache_name, dictionary_name, field, value);
        request.send(self).await
    }

    /// Sets multiple fields in a dictionary. If the dictionary does not exist, it will be created.
    /// If the dictionary already exists, the fields will be updated.
    ///
    /// # Arguments
    /// * `cache_name` - The name of the cache containing the dictionary.
    /// * `dictionary_name` - The name of the dictionary to set fields in.
    /// * `elements` - The fields and values to set in the dictionary.
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to set fields using a
    /// [DictionarySetFieldsRequest], you can also provide the following optional arguments:
    ///
    /// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::{CollectionTtl, DictionarySetFieldsResponse, DictionarySetFieldsRequest};
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let dictionary_name = "dictionary";
    ///
    /// let set_fields_response = cache_client.dictionary_set_fields(
    ///    cache_name,
    ///    dictionary_name,
    ///    vec![("field1", "value1"), ("field2", "value2")]
    /// ).await;
    ///
    /// match set_fields_response {
    ///   Ok(_) => println!("Fields set successfully"),
    ///   Err(e) => eprintln!("Error setting fields: {:?}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to set fields using a [DictionarySetFieldsRequest]
    pub async fn dictionary_set_fields<F: IntoBytes, V: IntoBytes>(
        &self,
        cache_name: impl Into<String>,
        dictionary_name: impl IntoBytes,
        elements: impl IntoDictionaryFieldValuePairs<F, V>,
    ) -> MomentoResult<DictionarySetFieldsResponse> {
        let request = DictionarySetFieldsRequest::new(cache_name, dictionary_name, elements);
        request.send(self).await
    }

    /// Adds elements to the given set. Creates the set if it does not exist.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache containing the set.
    /// * `set_name` - The name of the set to add elements to.
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
    /// use momento::cache::SetAddElementsResponse;
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
    pub async fn set_add_elements<E: IntoBytesIterable>(
        &self,
        cache_name: impl Into<String>,
        set_name: impl IntoBytes,
        elements: E,
    ) -> MomentoResult<SetAddElementsResponse> {
        let request = SetAddElementsRequest::new(cache_name, set_name, elements);
        request.send(self).await
    }

    /// Fetch the elements in the given set.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache containing the set.
    /// * `set_name` - The name of the set to fetch.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento::MomentoResult;
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::SetFetchResponse;
    /// use std::convert::TryInto;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let set_name = "set";
    ///
    /// # let add_elements_response = cache_client.set_add_elements(&cache_name, set_name, vec!["value1", "value2"]).await?;
    ///
    /// let fetched_elements: Vec<String> = cache_client.set_fetch(cache_name, set_name).await?.try_into().expect("Expected a set!");
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to fetch elements using a [SetFetchRequest].
    pub async fn set_fetch(
        &self,
        cache_name: impl Into<String>,
        set_name: impl IntoBytes,
    ) -> MomentoResult<SetFetchResponse> {
        let request = SetFetchRequest::new(cache_name, set_name);
        request.send(self).await
    }

    /// Removes multiple elements from an existing set. If the set is emptied as a result, the set is deleted.
    /// If the set or any element does not exist, a success response is returned.
    ///
    /// # Arguments
    ///
    /// * `cache_name` - The name of the cache containing the set.
    /// * `set_name` - The name of the set to remove elements from.
    /// * `elements` - The elements to remove. Must be able to be converted to a `Vec<u8>`.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento::MomentoResult;
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::SetRemoveElementsResponse;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let set_name = "set";
    ///
    /// match cache_client.set_remove_elements(cache_name, set_name, vec!["element1", "element2"]).await {
    ///     Ok(_) => println!("Elements removed from set"),
    ///     Err(e) => eprintln!("Error removing elements from set: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to fetch elements using a [SetRemoveElementsRequest].
    pub async fn set_remove_elements<E: IntoBytes>(
        &self,
        cache_name: impl Into<String>,
        set_name: impl IntoBytes,
        elements: Vec<E>,
    ) -> MomentoResult<SetRemoveElementsResponse> {
        let request = SetRemoveElementsRequest::new(cache_name, set_name, elements);
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
    ///     SortedSetFetch::Hit{ value } => {
    ///         match value.into_strings() {
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
    /// You can also use the [send_request](CacheClient::send_request) method to fetch elements using a [SortedSetFetchByRankRequest].
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
    ///     SortedSetFetch::Hit{ value } => {
    ///         match value.into_strings() {
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

    /// Remove multiple elements from the sorted set.
    ///
    /// # Arguments
    /// * `cache_name` - The name of the cache containing the sorted set.
    /// * `sorted_set_name` - The name of the sorted set to remove elements from.
    /// * `values` - The values to remove. Must be able to be converted to a `Vec<u8>`.
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use momento::cache::SortedSetRemoveElements;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let sorted_set_name = "sorted_set";
    ///
    /// let remove_elements_response = cache_client.sorted_set_remove_elements(
    ///     cache_name,
    ///     sorted_set_name,
    ///     vec!["value1", "value2"]
    /// ).await;
    ///
    /// match remove_elements_response {
    ///     Ok(_) => println!("Elements removed from sorted set"),
    ///     Err(e) => eprintln!("Error removing elements from sorted set: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to remove elements using a [SortedSetRemoveElementsRequest].
    pub async fn sorted_set_remove_elements<V: IntoBytesIterable>(
        &self,
        cache_name: impl Into<String>,
        sorted_set_name: impl IntoBytes,
        values: V,
    ) -> MomentoResult<SortedSetRemoveElements> {
        let request = SortedSetRemoveElementsRequest::new(cache_name, sorted_set_name, values);
        request.send(self).await
    }

    /// GetResponse the number of entries in a sorted set collection.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `sorted_set_name` - name of the sorted set
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::convert::TryInto;
    /// use momento::cache::{SortedSetLength, SortedSetLengthRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let sorted_set_name = "sorted_set";
    ///
    /// # cache_client.sorted_set_put_elements(&cache_name, sorted_set_name.to_string(), vec![("value1", 1.0), ("value2", 2.0)]).await;
    ///
    /// let length: u32 = cache_client.sorted_set_length(cache_name, sorted_set_name).await?.try_into().expect("Expected a list length!");
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get sorted set length using a [SortedSetLengthRequest].
    pub async fn sorted_set_length(
        &self,
        cache_name: impl Into<String>,
        sorted_set_name: impl IntoBytes,
    ) -> MomentoResult<SortedSetLength> {
        let request = SortedSetLengthRequest::new(cache_name, sorted_set_name);
        request.send(self).await
    }

    /// GetResponse the rank (position) of a specific element in a sorted set.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `sorted_set_name` - name of the sorted set
    /// * `value` - the sorted set value to get the rank of
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::convert::TryInto;
    /// use momento::cache::{SortedSetGetRank, SortedSetGetRankRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let sorted_set_name = "sorted_set";
    ///
    /// # cache_client.sorted_set_put_elements(&cache_name, sorted_set_name.to_string(), vec![("value1", 1.0), ("value2", 2.0)]).await;
    ///
    /// let rank: u64 = cache_client.sorted_set_get_rank(cache_name, sorted_set_name, "value1").await?.try_into().expect("Expected a rank!");
    /// # assert_eq!(rank, 0);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get the rank of an element using a [SortedSetGetRankRequest].
    pub async fn sorted_set_get_rank(
        &self,
        cache_name: impl Into<String>,
        sorted_set_name: impl IntoBytes,
        value: impl IntoBytes,
    ) -> MomentoResult<SortedSetGetRank> {
        let request = SortedSetGetRankRequest::new(cache_name, sorted_set_name, value);
        request.send(self).await
    }

    /// GetResponse the score of a specific element in a sorted set.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `sorted_set_name` - name of the sorted set
    /// * `value` - the sorted set value to get the score of
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// use std::convert::TryInto;
    /// use momento::cache::{SortedSetGetScore, SortedSetGetScoreRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let sorted_set_name = "sorted_set";
    ///
    /// # cache_client.sorted_set_put_elements(&cache_name, sorted_set_name.to_string(), vec![("value1", 1.0), ("value2", 2.0)]).await;
    ///
    /// let score: f64 = cache_client.sorted_set_get_score(cache_name, sorted_set_name, "value1").await?.try_into().expect("Expected a score!");
    /// # assert_eq!(score, 1.0);
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to get the score of an element using a [SortedSetGetScoreRequest].
    pub async fn sorted_set_get_score(
        &self,
        cache_name: impl Into<String>,
        sorted_set_name: impl IntoBytes,
        value: impl IntoBytes,
    ) -> MomentoResult<SortedSetGetScore> {
        let request = SortedSetGetScoreRequest::new(cache_name, sorted_set_name, value);
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
    /// use momento::cache::KeyExistsResponse;
    ///
    /// let result = cache_client.key_exists(cache_name, "key").await?;
    /// if result.exists {
    ///     println!("Key exists!");
    /// } else {
    ///     println!("Key does not exist!");
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to check if a key exists using a [KeyExistsRequest].
    pub async fn key_exists(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
    ) -> MomentoResult<KeyExistsResponse> {
        let request = KeyExistsRequest::new(cache_name, key);
        request.send(self).await
    }

    /// Check if the provided keys exist in the cache.
    /// Returns an object that is accessible as a list or map of booleans indicating whether each given key was found in the cache.
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
    /// use momento::cache::KeysExistResponse;
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
    /// You can also use the [send_request](CacheClient::send_request) method to check if the keys exist using a [KeysExistRequest].
    pub async fn keys_exist(
        &self,
        cache_name: impl Into<String>,
        keys: impl IntoBytesIterable,
    ) -> MomentoResult<KeysExistResponse> {
        let request = KeysExistRequest::new(cache_name, keys);
        request.send(self).await
    }

    /// Adds an integer quantity to a cache item.
    /// Adds the quantity if and only if the existing value is a UTF-8 string representing a base 10 integer.
    /// If the item does not exist, this method creates it and sets the item's value to the amount to increment by.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `key` - the key to increment
    /// * `amount` - the quantity to add to the value. May be positive, negative, or zero. Defaults to 1.
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to increment a key using an
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
    /// use momento::cache::IncrementResponse;
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
    /// You can also use the [send_request](CacheClient::send_request) method to increment an item using an [IncrementRequest]
    /// which will allow you to set [optional arguments](IncrementRequest#optional-arguments) as well.
    pub async fn increment(
        &self,
        cache_name: impl Into<String>,
        key: impl IntoBytes,
        amount: i64,
    ) -> MomentoResult<IncrementResponse> {
        let request = IncrementRequest::new(cache_name, key, amount);
        request.send(self).await
    }

    /// Return the type of an item in the cache.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `key` - the key of the item to get the type of
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// use std::convert::TryInto;
    /// use momento::cache::{ItemGetTypeResponse, ItemType};
    /// # cache_client.set(&cache_name, "key1", "value").await?;
    ///
    /// let item: ItemType = match(cache_client.item_get_type(&cache_name, "key1").await?) {
    ///     ItemGetTypeResponse::Hit { key_type } => key_type.try_into().expect("Expected an item type!"),
    ///     ItemGetTypeResponse::Miss => return Err(anyhow::Error::msg("cache miss"))
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
    ) -> MomentoResult<ItemGetTypeResponse> {
        let request = ItemGetTypeRequest::new(cache_name, key);
        request.send(self).await
    }

    /// Return the remaining ttl of an item in the cache
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `key` - the key of the item for which the remaining ttl is requested
    ///
    /// # Examples
    /// Assumes that a CacheClient named `cache_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_cache_client;
    /// # tokio_test::block_on(async {
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// use std::convert::TryInto;
    /// use momento::cache::ItemGetTtlResponse;
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
    ) -> MomentoResult<ItemGetTtlResponse> {
        let request = ItemGetTtlRequest::new(cache_name, key);
        request.send(self).await
    }

    /// Update the ttl of an item in the cache.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `key` - the key of the item for which ttl is requested
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
    /// use momento::cache::UpdateTtlResponse;
    /// # cache_client.set(&cache_name, "key1", "value").await?;
    ///
    /// match(cache_client.update_ttl(&cache_name, "key1", Duration::from_secs(10)).await?) {
    ///     UpdateTtlResponse::Set => println!("TTL updated"),
    ///     UpdateTtlResponse::Miss => return Err(anyhow::Error::msg("cache miss"))
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
    ) -> MomentoResult<UpdateTtlResponse> {
        let request = UpdateTtlRequest::new(cache_name, key, ttl);
        request.send(self).await
    }

    /// Increase the ttl of an item in the cache.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `key` - the key of the item for which ttl is requested
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
    /// use momento::cache::IncreaseTtlResponse;
    /// # cache_client.set(&cache_name, "key1", "value").await?;
    ///
    /// match(cache_client.increase_ttl(&cache_name, "key1", Duration::from_secs(5)).await?) {
    ///     IncreaseTtlResponse::Set => println!("TTL updated"),
    ///     IncreaseTtlResponse::NotSet => println!("unable to increase TTL"),
    ///     IncreaseTtlResponse::Miss => return Err(anyhow::Error::msg("cache miss"))
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
    ) -> MomentoResult<IncreaseTtlResponse> {
        let request = IncreaseTtlRequest::new(cache_name, key, ttl);
        request.send(self).await
    }

    /// Decrease the ttl of an item in the cache.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `key` - the key of the item for which ttl is requested
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
    /// use momento::cache::DecreaseTtlResponse;
    /// # cache_client.set(&cache_name, "key1", "value").await?;
    ///
    /// match(cache_client.decrease_ttl(&cache_name, "key1", Duration::from_secs(3)).await?) {
    ///     DecreaseTtlResponse::Set => println!("TTL updated"),
    ///     DecreaseTtlResponse::NotSet => println!("unable to decrease TTL"),
    ///     DecreaseTtlResponse::Miss => return Err(anyhow::Error::msg("cache miss"))
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
    ) -> MomentoResult<DecreaseTtlResponse> {
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
    /// If you use [send_request](CacheClient::send_request) to conditionally set an item using an
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
    /// use momento::cache::{SetIfAbsentResponse, SetIfAbsentRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.set_if_absent(&cache_name, "key", "value1").await {
    ///     Ok(response) => match response {
    ///         SetIfAbsentResponse::Stored => println!("Value stored"),
    ///         SetIfAbsentResponse::NotStored => println!("Value not stored"),
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
    ) -> MomentoResult<SetIfAbsentResponse> {
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
    /// If you use [send_request](CacheClient::send_request) to conditionally set an item using an
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
    /// use momento::cache::{SetIfPresentResponse, SetIfPresentRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.set_if_present(&cache_name, "key", "value1").await {
    ///     Ok(response) => match response {
    ///         SetIfPresentResponse::Stored => println!("Value stored"),
    ///         SetIfPresentResponse::NotStored => println!("Value not stored"),
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
    ) -> MomentoResult<SetIfPresentResponse> {
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
    /// If you use [send_request](CacheClient::send_request) to conditionally set an item using an
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
    /// use momento::cache::{SetIfEqualResponse, SetIfEqualRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.set_if_equal(&cache_name, "key", "new-value", "cached-value").await {
    ///     Ok(response) => match response {
    ///         SetIfEqualResponse::Stored => println!("Value stored"),
    ///         SetIfEqualResponse::NotStored => println!("Value not stored"),
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
    ) -> MomentoResult<SetIfEqualResponse> {
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
    /// If you use [send_request](CacheClient::send_request) to conditionally set an item using an
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
    /// use momento::cache::{SetIfNotEqualResponse, SetIfNotEqualRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.set_if_not_equal(&cache_name, "key", "new-value", "cached-value").await {
    ///     Ok(response) => match response {
    ///         SetIfNotEqualResponse::Stored => println!("Value stored"),
    ///         SetIfNotEqualResponse::NotStored => println!("Value not stored"),
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
    ) -> MomentoResult<SetIfNotEqualResponse> {
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
    /// If you use [send_request](CacheClient::send_request) to conditionally set an item using an
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
    /// use momento::cache::{SetIfPresentAndNotEqualResponse, SetIfPresentAndNotEqualRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.set_if_present_and_not_equal(&cache_name, "key", "new-value", "cached-value").await {
    ///     Ok(response) => match response {
    ///         SetIfPresentAndNotEqualResponse::Stored => println!("Value stored"),
    ///         SetIfPresentAndNotEqualResponse::NotStored => println!("Value not stored"),
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
    ) -> MomentoResult<SetIfPresentAndNotEqualResponse> {
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
    /// If you use [send_request](CacheClient::send_request) to conditionally set an item using an
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
    /// use momento::cache::{SetIfAbsentOrEqualResponse, SetIfAbsentOrEqualRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    ///
    /// match cache_client.set_if_absent_or_equal(&cache_name, "key", "new-value", "cached-value").await {
    ///     Ok(response) => match response {
    ///         SetIfAbsentOrEqualResponse::Stored => println!("Value stored"),
    ///         SetIfAbsentOrEqualResponse::NotStored => println!("Value not stored"),
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
    ) -> MomentoResult<SetIfAbsentOrEqualResponse> {
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
    /// use momento::cache::ListLengthResponse;
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
    ) -> MomentoResult<ListLengthResponse> {
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
    /// use momento::cache::ListConcatenateFrontResponse;
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
        values: impl IntoBytesIterable,
    ) -> MomentoResult<ListConcatenateFrontResponse> {
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
    /// use momento::cache::ListConcatenateBackResponse;
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
        values: impl IntoBytesIterable,
    ) -> MomentoResult<ListConcatenateBackResponse> {
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
    /// use momento::cache::ListFetchResponse;
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
    ) -> MomentoResult<ListFetchResponse> {
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
    /// use momento::cache::{ListPopBackResponse, ListPopBackRequest};
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
    ) -> MomentoResult<ListPopBackResponse> {
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
    /// use momento::cache::{ListPopFrontResponse, ListPopFrontRequest};
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
    ) -> MomentoResult<ListPopFrontResponse> {
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
    /// use momento::cache::{ListRemoveValueResponse, ListRemoveValueRequest};
    /// use momento::MomentoErrorCode;
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let list_name = "list-name";
    /// # cache_client.list_concatenate_front(&cache_name, list_name, vec!["value1", "value2"]).await;
    ///
    /// match cache_client.list_remove_value(cache_name, list_name, "value1").await {
    ///     Ok(ListRemoveValueResponse {}) => println!("Successfully removed value"),
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
    ) -> MomentoResult<ListRemoveValueResponse> {
        let request = ListRemoveValueRequest::new(cache_name, list_name, value);
        request.send(self).await
    }

    /// Adds an element to the back of the given list. Creates the list if it does not already exist.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `list_name` - name of the list
    /// * `value` - value to append to list
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to add to a list using a [ListPushBackRequest],
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
    /// use momento::cache::{ListPushBackResponse, ListPushBackRequest};
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let list_name = "list-name";
    ///
    /// match cache_client.list_push_back(cache_name, list_name, "value").await {
    ///     Ok(_) => println!("Element added to list"),
    ///     Err(e) => eprintln!("Error adding element to list: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to add to a list using a [ListPushBackRequest]
    /// which will allow you to set [optional arguments](ListPushBackRequest#optional-arguments) as well.
    pub async fn list_push_back(
        &self,
        cache_name: impl Into<String>,
        list_name: impl IntoBytes,
        value: impl IntoBytes,
    ) -> MomentoResult<ListPushBackResponse> {
        let request = ListPushBackRequest::new(cache_name, list_name, value);
        request.send(self).await
    }

    /// Adds an element to the front of the given list. Creates the list if it does not already exist.
    ///
    /// # Arguments
    /// * `cache_name` - name of cache
    /// * `list_name` - name of the list
    /// * `value` - value to prepend to list
    ///
    /// # Optional Arguments
    /// If you use [send_request](CacheClient::send_request) to add to a list using a [ListPushFrontRequest],
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
    /// use momento::cache::{ListPushFrontResponse, ListPushFrontRequest};
    /// # let (cache_client, cache_name) = create_doctest_cache_client();
    /// let list_name = "list-name";
    ///
    /// match cache_client.list_push_front(cache_name, list_name, "value").await {
    ///     Ok(_) => println!("Element added to list"),
    ///     Err(e) => eprintln!("Error adding element to list: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](CacheClient::send_request) method to add to a list using a [ListPushFrontRequest]
    /// which will allow you to set [optional arguments](ListPushFrontRequest#optional-arguments) as well.
    pub async fn list_push_front(
        &self,
        cache_name: impl Into<String>,
        list_name: impl IntoBytes,
        value: impl IntoBytes,
    ) -> MomentoResult<ListPushFrontResponse> {
        let request = ListPushFrontRequest::new(cache_name, list_name, value);
        request.send(self).await
    }

    /// Lower-level API to send any type of MomentoRequest to the server. This is used for cases when
    /// you want to set optional fields on a request that are not supported by the short-hand API for
    /// that request type.
    ///
    /// See [SortedSetFetchByScoreRequest] for an example of creating a request with optional fields.
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
