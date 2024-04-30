// Make module "requests" private and re-export its contents so users can
// import using "momento::cache::{Get, Set, ...}"
mod requests;

pub use requests::MomentoRequest;

pub use requests::create_cache::{CreateCache, CreateCacheRequest};
pub use requests::delete_cache::{DeleteCache, DeleteCacheRequest};
pub use requests::flush_cache::{FlushCache, FlushCacheRequest};
pub use requests::list_caches::{
    CacheInfo, CacheLimits, ListCaches, ListCachesRequest, TopicLimits,
};

pub use requests::dictionary::dictionary_fetch::{DictionaryFetch, DictionaryFetchRequest};
pub use requests::dictionary::dictionary_set_field::{
    DictionarySetField, DictionarySetFieldRequest,
};

pub use requests::scalar::decrease_ttl::{DecreaseTtl, DecreaseTtlRequest};
pub use requests::scalar::delete::{Delete, DeleteRequest};
pub use requests::scalar::get::{Get, GetRequest, GetValue};
pub use requests::scalar::increase_ttl::{IncreaseTtl, IncreaseTtlRequest};
pub use requests::scalar::increment::{Increment, IncrementRequest};
pub use requests::scalar::item_get_ttl::{ItemGetTtl, ItemGetTtlRequest};
pub use requests::scalar::item_get_type::{ItemGetType, ItemGetTypeRequest, ItemType};
pub use requests::scalar::key_exists::{KeyExists, KeyExistsRequest};
pub use requests::scalar::keys_exist::{KeysExist, KeysExistRequest};
pub use requests::scalar::set::{Set, SetRequest};
pub use requests::scalar::set_if_absent::{SetIfAbsent, SetIfAbsentRequest};
pub use requests::scalar::set_if_absent_or_equal::{SetIfAbsentOrEqual, SetIfAbsentOrEqualRequest};
pub use requests::scalar::set_if_equal::{SetIfEqual, SetIfEqualRequest};
pub use requests::scalar::set_if_not_equal::{SetIfNotEqual, SetIfNotEqualRequest};
pub use requests::scalar::set_if_present::{SetIfPresent, SetIfPresentRequest};
pub use requests::scalar::set_if_present_and_not_equal::{
    SetIfPresentAndNotEqual, SetIfPresentAndNotEqualRequest,
};
pub use requests::scalar::update_ttl::{UpdateTtl, UpdateTtlRequest};

pub use requests::set::set_add_elements::{SetAddElements, SetAddElementsRequest};

pub use requests::sorted_set::sorted_set_fetch_by_rank::{
    SortedSetFetchByRankRequest, SortedSetOrder,
};
pub use requests::sorted_set::sorted_set_fetch_by_score::SortedSetFetchByScoreRequest;
pub use requests::sorted_set::sorted_set_fetch_response::{SortedSetElements, SortedSetFetch};
pub use requests::sorted_set::sorted_set_put_element::{
    SortedSetPutElement, SortedSetPutElementRequest,
};
pub use requests::sorted_set::sorted_set_put_elements::{
    IntoSortedSetElements, SortedSetElement, SortedSetPutElements, SortedSetPutElementsRequest,
};

// Similar re-exporting with config::configuration and config::configurations
// so import paths can be simpmlified to "momento::cache::Configuration" and
// "use momento::cache::configurations::laptop"
mod config;

pub use config::configuration::Configuration;
pub use config::configurations;

mod collection_ttl;

pub use collection_ttl::CollectionTtl;
