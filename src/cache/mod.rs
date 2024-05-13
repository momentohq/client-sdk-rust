mod messages;

pub use messages::MomentoRequest;

pub use messages::create_cache::{CreateCache, CreateCacheRequest};
pub use messages::delete_cache::{DeleteCache, DeleteCacheRequest};
pub use messages::flush_cache::{FlushCache, FlushCacheRequest};
pub use messages::list_caches::{
    CacheInfo, CacheLimits, ListCaches, ListCachesRequest, TopicLimits,
};

pub use messages::dictionary::dictionary_fetch::{
    DictionaryFetchRequest, DictionaryFetchResponse, DictionaryFetchValue,
};
pub use messages::dictionary::dictionary_get_field::{
    DictionaryGetFieldRequest, DictionaryGetFieldResponse,
};
pub use messages::dictionary::dictionary_get_fields::{
    DictionaryGetFieldsRequest, DictionaryGetFieldsResponse,
};
pub use messages::dictionary::dictionary_increment::{
    DictionaryIncrementRequest, DictionaryIncrementResponse,
};
pub use messages::dictionary::dictionary_length::{
    DictionaryLengthRequest, DictionaryLengthResponse,
};
pub use messages::dictionary::dictionary_remove_field::{
    DictionaryRemoveFieldRequest, DictionaryRemoveFieldResponse,
};
pub use messages::dictionary::dictionary_remove_fields::{
    DictionaryRemoveFieldsRequest, DictionaryRemoveFieldsResponse,
};
pub use messages::dictionary::dictionary_set_field::{
    DictionarySetFieldRequest, DictionarySetFieldResponse,
};
pub use messages::dictionary::dictionary_set_fields::{
    DictionaryFieldValuePair, DictionarySetFieldsRequest, DictionarySetFieldsResponse,
    IntoDictionaryFieldValuePairs,
};

pub use messages::scalar::decrease_ttl::{DecreaseTtlRequest, DecreaseTtlResponse};
pub use messages::scalar::delete::{DeleteRequest, DeleteResponse};
pub use messages::scalar::get::{GetRequest, GetResponse, GetValue};
pub use messages::scalar::increase_ttl::{IncreaseTtlRequest, IncreaseTtlResponse};
pub use messages::scalar::increment::{IncrementRequest, IncrementResponse};
pub use messages::scalar::item_get_ttl::{ItemGetTtlRequest, ItemGetTtlResponse};
pub use messages::scalar::item_get_type::{ItemGetTypeRequest, ItemGetTypeResponse, ItemType};
pub use messages::scalar::key_exists::{KeyExistsRequest, KeyExistsResponse};
pub use messages::scalar::keys_exist::{KeysExistRequest, KeysExistResponse};
pub use messages::scalar::set::{SetRequest, SetResponse};
pub use messages::scalar::set_if_absent::{SetIfAbsentRequest, SetIfAbsentResponse};
pub use messages::scalar::set_if_absent_or_equal::{
    SetIfAbsentOrEqualRequest, SetIfAbsentOrEqualResponse,
};
pub use messages::scalar::set_if_equal::{SetIfEqualRequest, SetIfEqualResponse};
pub use messages::scalar::set_if_not_equal::{SetIfNotEqualRequest, SetIfNotEqualResponse};
pub use messages::scalar::set_if_present::{SetIfPresentRequest, SetIfPresentResponse};
pub use messages::scalar::set_if_present_and_not_equal::{
    SetIfPresentAndNotEqualRequest, SetIfPresentAndNotEqualResponse,
};
pub use messages::scalar::update_ttl::{UpdateTtlRequest, UpdateTtlResponse};

pub use messages::set::set_add_elements::{SetAddElementsRequest, SetAddElementsResponse};
pub use messages::set::set_fetch::{SetFetchRequest, SetFetchResponse, SetFetchValue};
pub use messages::set::set_remove_elements::{SetRemoveElementsRequest, SetRemoveElementsResponse};

pub use messages::sorted_set::sorted_set_fetch_by_rank::{
    SortedSetFetchByRankRequest, SortedSetOrder,
};
pub use messages::sorted_set::sorted_set_fetch_by_score::SortedSetFetchByScoreRequest;
pub use messages::sorted_set::sorted_set_fetch_response::{
    SortedSetElements, SortedSetFetchResponse,
};
pub use messages::sorted_set::sorted_set_get_rank::{
    SortedSetGetRankRequest, SortedSetGetRankResponse,
};
pub use messages::sorted_set::sorted_set_get_score::{
    SortedSetGetScoreRequest, SortedSetGetScoreResponse,
};
pub use messages::sorted_set::sorted_set_length::{
    SortedSetLengthRequest, SortedSetLengthResponse,
};
pub use messages::sorted_set::sorted_set_put_element::{
    SortedSetPutElement, SortedSetPutElementRequest,
};
pub use messages::sorted_set::sorted_set_put_elements::{
    IntoSortedSetElements, SortedSetElement, SortedSetPutElements, SortedSetPutElementsRequest,
};
pub use messages::sorted_set::sorted_set_remove_elements::{
    SortedSetRemoveElements, SortedSetRemoveElementsRequest,
};

pub use messages::list::list_concatenate_back::{
    ListConcatenateBackRequest, ListConcatenateBackResponse,
};
pub use messages::list::list_concatenate_front::{
    ListConcatenateFrontRequest, ListConcatenateFrontResponse,
};
pub use messages::list::list_fetch::{ListFetchRequest, ListFetchResponse, ListFetchValue};
pub use messages::list::list_length::{ListLengthRequest, ListLengthResponse};
pub use messages::list::list_pop_back::{
    ListPopBackRequest, ListPopBackResponse, ListPopBackValue,
};
pub use messages::list::list_pop_front::{
    ListPopFrontRequest, ListPopFrontResponse, ListPopFrontValue,
};
pub use messages::list::list_push_back::{ListPushBackRequest, ListPushBackResponse};
pub use messages::list::list_push_front::{ListPushFrontRequest, ListPushFrontResponse};
pub use messages::list::list_remove_value::{ListRemoveValueRequest, ListRemoveValueResponse};

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
