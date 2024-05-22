/// Contains the request and response types for cache operations.
pub mod messages;

pub use messages::MomentoRequest;

pub use messages::control::create_cache::{CreateCacheRequest, CreateCacheResponse};
pub use messages::control::delete_cache::{DeleteCacheRequest, DeleteCacheResponse};
pub use messages::control::flush_cache::{FlushCacheRequest, FlushCacheResponse};
pub use messages::control::list_caches::{
    CacheInfo, CacheLimits, ListCachesRequest, ListCachesResponse, TopicLimits,
};

pub use messages::data::dictionary::dictionary_fetch::{
    DictionaryFetchRequest, DictionaryFetchResponse,
};
pub use messages::data::dictionary::dictionary_get_field::{
    DictionaryGetFieldRequest, DictionaryGetFieldResponse,
};
pub use messages::data::dictionary::dictionary_get_fields::{
    DictionaryGetFieldsRequest, DictionaryGetFieldsResponse,
};
pub use messages::data::dictionary::dictionary_increment::{
    DictionaryIncrementRequest, DictionaryIncrementResponse,
};
pub use messages::data::dictionary::dictionary_length::{
    DictionaryLengthRequest, DictionaryLengthResponse,
};
pub use messages::data::dictionary::dictionary_remove_field::{
    DictionaryRemoveFieldRequest, DictionaryRemoveFieldResponse,
};
pub use messages::data::dictionary::dictionary_remove_fields::{
    DictionaryRemoveFieldsRequest, DictionaryRemoveFieldsResponse,
};
pub use messages::data::dictionary::dictionary_set_field::{
    DictionarySetFieldRequest, DictionarySetFieldResponse,
};
pub use messages::data::dictionary::dictionary_set_fields::{
    DictionaryFieldValuePair, DictionarySetFieldsRequest, DictionarySetFieldsResponse,
    IntoDictionaryFieldValuePairs,
};

pub use messages::data::scalar::decrease_ttl::{DecreaseTtlRequest, DecreaseTtlResponse};
pub use messages::data::scalar::delete::{DeleteRequest, DeleteResponse};
pub use messages::data::scalar::get::{GetRequest, GetResponse};
pub use messages::data::scalar::increase_ttl::{IncreaseTtlRequest, IncreaseTtlResponse};
pub use messages::data::scalar::increment::{IncrementRequest, IncrementResponse};
pub use messages::data::scalar::item_get_ttl::{ItemGetTtlRequest, ItemGetTtlResponse};
pub use messages::data::scalar::item_get_type::{
    ItemGetTypeRequest, ItemGetTypeResponse, ItemType,
};
pub use messages::data::scalar::key_exists::{KeyExistsRequest, KeyExistsResponse};
pub use messages::data::scalar::keys_exist::{KeysExistRequest, KeysExistResponse};
pub use messages::data::scalar::set::{SetRequest, SetResponse};
pub use messages::data::scalar::set_if_absent::{SetIfAbsentRequest, SetIfAbsentResponse};
pub use messages::data::scalar::set_if_absent_or_equal::{
    SetIfAbsentOrEqualRequest, SetIfAbsentOrEqualResponse,
};
pub use messages::data::scalar::set_if_equal::{SetIfEqualRequest, SetIfEqualResponse};
pub use messages::data::scalar::set_if_not_equal::{SetIfNotEqualRequest, SetIfNotEqualResponse};
pub use messages::data::scalar::set_if_present::{SetIfPresentRequest, SetIfPresentResponse};
pub use messages::data::scalar::set_if_present_and_not_equal::{
    SetIfPresentAndNotEqualRequest, SetIfPresentAndNotEqualResponse,
};
pub use messages::data::scalar::update_ttl::{UpdateTtlRequest, UpdateTtlResponse};

pub use messages::data::set::set_add_elements::{SetAddElementsRequest, SetAddElementsResponse};
pub use messages::data::set::set_fetch::{SetFetchRequest, SetFetchResponse};
pub use messages::data::set::set_remove_elements::{
    SetRemoveElementsRequest, SetRemoveElementsResponse,
};

pub use messages::data::sorted_set::sorted_set_fetch_by_rank::{
    SortedSetFetchByRankRequest, SortedSetOrder,
};
pub use messages::data::sorted_set::sorted_set_fetch_by_score::SortedSetFetchByScoreRequest;
pub use messages::data::sorted_set::sorted_set_fetch_response::{
    SortedSetElements, SortedSetFetchResponse,
};
pub use messages::data::sorted_set::sorted_set_get_rank::{
    SortedSetGetRankRequest, SortedSetGetRankResponse,
};
pub use messages::data::sorted_set::sorted_set_get_score::{
    SortedSetGetScoreRequest, SortedSetGetScoreResponse,
};
pub use messages::data::sorted_set::sorted_set_length::{
    SortedSetLengthRequest, SortedSetLengthResponse,
};
pub use messages::data::sorted_set::sorted_set_put_element::{
    SortedSetPutElementRequest, SortedSetPutElementResponse,
};
pub use messages::data::sorted_set::sorted_set_put_elements::{
    IntoSortedSetElements, SortedSetElement, SortedSetPutElementsRequest,
    SortedSetPutElementsResponse,
};
pub use messages::data::sorted_set::sorted_set_remove_elements::{
    SortedSetRemoveElementsRequest, SortedSetRemoveElementsResponse,
};

pub use messages::data::list::list_concatenate_back::{
    ListConcatenateBackRequest, ListConcatenateBackResponse,
};
pub use messages::data::list::list_concatenate_front::{
    ListConcatenateFrontRequest, ListConcatenateFrontResponse,
};
pub use messages::data::list::list_fetch::{ListFetchRequest, ListFetchResponse};
pub use messages::data::list::list_length::{ListLengthRequest, ListLengthResponse};
pub use messages::data::list::list_pop_back::{ListPopBackRequest, ListPopBackResponse};
pub use messages::data::list::list_pop_front::{ListPopFrontRequest, ListPopFrontResponse};
pub use messages::data::list::list_push_back::{ListPushBackRequest, ListPushBackResponse};
pub use messages::data::list::list_push_front::{ListPushFrontRequest, ListPushFrontResponse};
pub use messages::data::list::list_remove_value::{
    ListRemoveValueRequest, ListRemoveValueResponse,
};

// Similar re-exporting with config::configuration and config::configurations
// so import paths can be simpmlified to "momento::cache::Configuration" and
// "use momento::cache::configurations::laptop"
mod config;

pub use config::configuration::Configuration;
pub use config::configurations;

mod collection_ttl;
pub use collection_ttl::CollectionTtl;

mod cache_client;
mod cache_client_builder;
pub use cache_client::CacheClient;
