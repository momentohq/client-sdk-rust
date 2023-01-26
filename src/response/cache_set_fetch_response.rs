use std::collections::HashSet;

#[derive(Debug)]
#[non_exhaustive]
pub struct MomentoSetFetchResponse {
    pub value: Option<HashSet<Vec<u8>>>,
}
