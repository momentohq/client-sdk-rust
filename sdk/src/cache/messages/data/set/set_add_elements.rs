use momento_protos::cache_client::SetUnionRequest;

use crate::cache::messages::MomentoRequest;
use crate::cache::CollectionTtl;
use crate::utils::prep_request_with_timeout;
use crate::CacheClient;
use crate::{IntoBytes, IntoBytesIterable, MomentoResult};

/// Request to add elements to the given set. Creates the set if it does not exist.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the set.
/// * `set_name` - The name of the set to add an element to.
/// * `elements` - The elements to add. Must be able to be converted to a `Vec<u8>`.
///
/// # Optional Arguments
///
/// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{CollectionTtl, SetAddElementsResponse, SetAddElementsRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let set_name = "set";
///
/// let add_elements_request = SetAddElementsRequest::new(
///     cache_name,
///     set_name,
///     vec!["value1", "value2"]
/// ).ttl(CollectionTtl::default());
///
/// match cache_client.send_request(add_elements_request).await {
///     Ok(_) => println!("Elements added to set"),
///     Err(e) => eprintln!("Error adding elements to set: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SetAddElementsRequest<S: IntoBytes, E: IntoBytesIterable> {
    cache_name: String,
    set_name: S,
    elements: E,
    collection_ttl: Option<CollectionTtl>,
}

impl<S: IntoBytes, E: IntoBytesIterable> SetAddElementsRequest<S, E> {
    /// Constructs a new SetAddElementsRequest.
    pub fn new(cache_name: impl Into<String>, set_name: S, elements: E) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name: cache_name.into(),
            set_name,
            elements,
            collection_ttl: Some(collection_ttl),
        }
    }

    /// Set the time-to-live for the collection.
    pub fn ttl(mut self, collection_ttl: impl Into<Option<CollectionTtl>>) -> Self {
        self.collection_ttl = collection_ttl.into();
        self
    }
}

impl<S: IntoBytes, E: IntoBytesIterable> MomentoRequest for SetAddElementsRequest<S, E> {
    type Response = SetAddElementsResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SetAddElementsResponse> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let elements = self.elements.into_bytes();
        let set_name = self.set_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            SetUnionRequest {
                set_name,
                elements,
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
            },
        )?;

        let _ = cache_client.data_client.clone().set_union(request).await?;
        Ok(SetAddElementsResponse {})
    }
}

/// The response type for a successful set add elements request.
#[derive(Debug, PartialEq, Eq)]
pub struct SetAddElementsResponse {}
