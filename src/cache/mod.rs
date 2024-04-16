// Make module "requests" private and re-export its contents so users can
// import using "momento::cache::{Get, Set, ...}"
mod requests;

pub use requests::MomentoRequest;

pub use requests::create_cache::{CreateCache, CreateCacheRequest};
pub use requests::delete_cache::{DeleteCache, DeleteCacheRequest};
pub use requests::flush_cache::{FlushCache, FlushCacheRequest};
pub use requests::list_caches::{ListCaches, ListCachesRequest};

pub use requests::scalar::get::{Get, GetRequest, GetValue};
pub use requests::scalar::set::{Set, SetRequest};

pub use requests::set::set_add_elements::{SetAddElements, SetAddElementsRequest};

pub use requests::sorted_set::sorted_set_fetch_by_rank::{SortOrder, SortedSetFetchByRankRequest};
pub use requests::sorted_set::sorted_set_fetch_by_score::SortedSetFetchByScoreRequest;
pub use requests::sorted_set::sorted_set_fetch_response::{SortedSetElements, SortedSetFetch};
pub use requests::sorted_set::sorted_set_put_element::{
    SortedSetPutElement, SortedSetPutElementRequest,
};
pub use requests::sorted_set::sorted_set_put_elements::{
    IntoSortedSetElements, SortedSetElement, SortedSetPutElements, SortedSetPutElementsRequest,
};
