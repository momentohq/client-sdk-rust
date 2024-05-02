use momento_protos::cache_client::{
    set_difference_request::{
        subtrahend::{Set, SubtrahendSet},
        Difference, Subtrahend,
    },
    SetDifferenceRequest,
};

use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoResult,
};

/// Removes multiple elements from an existing set. If the set is emptied as a result, the set is deleted.
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
/// use momento::cache::SetRemoveElementsRequest;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let set_name = "set";
///
/// let request = SetRemoveElementsRequest::new(cache_name, set_name, vec!["element1", "element2"]);
///
/// match cache_client.send_request(request).await {
///     Ok(_) => println!("Elements removed from set"),
///     Err(e) => eprintln!("Error removing elements from set: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SetRemoveElementsRequest<S: IntoBytes, E: IntoBytes> {
    cache_name: String,
    set_name: S,
    elements: Vec<E>,
}

impl<S: IntoBytes, E: IntoBytes> SetRemoveElementsRequest<S, E> {
    pub fn new(cache_name: impl Into<String>, set_name: S, elements: Vec<E>) -> Self {
        Self {
            cache_name: cache_name.into(),
            set_name,
            elements,
        }
    }
}

impl<S: IntoBytes, E: IntoBytes> MomentoRequest for SetRemoveElementsRequest<S, E> {
    type Response = SetRemoveElements;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SetRemoveElements> {
        let elements = self.elements.into_iter().map(|e| e.into_bytes()).collect();
        let set_name = self.set_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            SetDifferenceRequest {
                set_name,
                difference: Some(Difference::Subtrahend(Subtrahend {
                    subtrahend_set: Some(SubtrahendSet::Set(Set { elements })),
                })),
            },
        )?;

        let _ = cache_client
            .data_client
            .clone()
            .set_difference(request)
            .await?;
        Ok(SetRemoveElements {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SetRemoveElements {}
