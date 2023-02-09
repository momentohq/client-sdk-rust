use momento_protos::cache_client::SortedSetElement;

#[derive(Debug)]
#[non_exhaustive]
pub struct MomentoSortedSetFetchResponse {
    pub value: Option<Vec<SortedSetElement>>,
}
