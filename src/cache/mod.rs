mod messages;

pub use messages::MomentoRequest;

pub use messages::create_cache::{CreateCache, CreateCacheRequest};
pub use messages::delete_cache::{DeleteCache, DeleteCacheRequest};
pub use messages::flush_cache::{FlushCache, FlushCacheRequest};
pub use messages::list_caches::{
    CacheInfo, CacheLimits, ListCaches, ListCachesRequest, TopicLimits,
};

pub use messages::dictionary::dictionary_fetch::{
    DictionaryFetch, DictionaryFetchRequest, DictionaryFetchValue,
};
pub use messages::dictionary::dictionary_get_field::{
    DictionaryGetField, DictionaryGetFieldRequest,
};
pub use messages::dictionary::dictionary_get_fields::{
    DictionaryGetFields, DictionaryGetFieldsRequest,
};
pub use messages::dictionary::dictionary_increment::{
    DictionaryIncrement, DictionaryIncrementRequest,
};
pub use messages::dictionary::dictionary_length::{DictionaryLength, DictionaryLengthRequest};
pub use messages::dictionary::dictionary_remove_field::{
    DictionaryRemoveField, DictionaryRemoveFieldRequest,
};
pub use messages::dictionary::dictionary_remove_fields::{
    DictionaryRemoveFields, DictionaryRemoveFieldsRequest,
};
pub use messages::dictionary::dictionary_set_field::{
    DictionarySetField, DictionarySetFieldRequest,
};
pub use messages::dictionary::dictionary_set_fields::{
    DictionaryFieldValuePair, DictionarySetFields, DictionarySetFieldsRequest,
    IntoDictionaryFieldValuePairs,
};

pub use messages::scalar::decrease_ttl::{DecreaseTtl, DecreaseTtlRequest};
pub use messages::scalar::delete::{Delete, DeleteRequest};
pub use messages::scalar::get::{Get, GetRequest, GetValue};
pub use messages::scalar::increase_ttl::{IncreaseTtl, IncreaseTtlRequest};
pub use messages::scalar::increment::{Increment, IncrementRequest};
pub use messages::scalar::item_get_ttl::{ItemGetTtl, ItemGetTtlRequest};
pub use messages::scalar::item_get_type::{ItemGetType, ItemGetTypeRequest, ItemType};
pub use messages::scalar::key_exists::{KeyExists, KeyExistsRequest};
pub use messages::scalar::keys_exist::{KeysExist, KeysExistRequest};
pub use messages::scalar::set::{Set, SetRequest};
pub use messages::scalar::set_if_absent::{SetIfAbsent, SetIfAbsentRequest};
pub use messages::scalar::set_if_absent_or_equal::{SetIfAbsentOrEqual, SetIfAbsentOrEqualRequest};
pub use messages::scalar::set_if_equal::{SetIfEqual, SetIfEqualRequest};
pub use messages::scalar::set_if_not_equal::{SetIfNotEqual, SetIfNotEqualRequest};
pub use messages::scalar::set_if_present::{SetIfPresent, SetIfPresentRequest};
pub use messages::scalar::set_if_present_and_not_equal::{
    SetIfPresentAndNotEqual, SetIfPresentAndNotEqualRequest,
};
pub use messages::scalar::update_ttl::{UpdateTtl, UpdateTtlRequest};

pub use messages::set::set_add_elements::{SetAddElements, SetAddElementsRequest};
pub use messages::set::set_fetch::{SetFetch, SetFetchRequest, SetFetchValue};
pub use messages::set::set_remove_elements::{SetRemoveElements, SetRemoveElementsRequest};

pub use messages::sorted_set::sorted_set_fetch_by_rank::{
    SortedSetFetchByRankRequest, SortedSetOrder,
};
pub use messages::sorted_set::sorted_set_fetch_by_score::SortedSetFetchByScoreRequest;
pub use messages::sorted_set::sorted_set_fetch_response::{SortedSetElements, SortedSetFetch};
pub use messages::sorted_set::sorted_set_get_rank::{SortedSetGetRank, SortedSetGetRankRequest};
pub use messages::sorted_set::sorted_set_get_score::{SortedSetGetScore, SortedSetGetScoreRequest};
pub use messages::sorted_set::sorted_set_length::{SortedSetLength, SortedSetLengthRequest};
pub use messages::sorted_set::sorted_set_put_element::{
    SortedSetPutElement, SortedSetPutElementRequest,
};
pub use messages::sorted_set::sorted_set_put_elements::{
    IntoSortedSetElements, SortedSetElement, SortedSetPutElements, SortedSetPutElementsRequest,
};
pub use messages::sorted_set::sorted_set_remove_elements::{
    SortedSetRemoveElements, SortedSetRemoveElementsRequest,
};

pub use messages::list::list_concatenate_back::{ListConcatenateBack, ListConcatenateBackRequest};
pub use messages::list::list_concatenate_front::{
    ListConcatenateFront, ListConcatenateFrontRequest,
};
pub use messages::list::list_fetch::{ListFetch, ListFetchRequest, ListFetchValue};
pub use messages::list::list_length::{ListLength, ListLengthRequest};
pub use messages::list::list_pop_back::{ListPopBack, ListPopBackRequest, ListPopBackValue};
pub use messages::list::list_pop_front::{ListPopFront, ListPopFrontRequest, ListPopFrontValue};
pub use messages::list::list_remove_value::{ListRemoveValue, ListRemoveValueRequest};

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
