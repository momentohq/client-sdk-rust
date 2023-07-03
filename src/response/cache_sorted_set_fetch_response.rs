use momento_protos::cache_client::sorted_set_fetch_response::SortedSet;

#[derive(Debug)]
#[non_exhaustive]
pub struct MomentoSortedSetFetchResponse {
    pub value: Option<SortedSet>,
}
