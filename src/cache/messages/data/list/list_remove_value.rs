use momento_protos::cache_client::list_remove_request::Remove;

use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoResult,
};

/// Remove all elements in a list item equal to a particular value.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `list_name` - name of the list
/// * `value` - value to remove
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{ListRemoveValueResponse, ListRemoveValueRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let list_name = "list-name";
/// # cache_client.list_concatenate_front(&cache_name, list_name, vec!["value1", "value2"]).await;
///
/// let remove_request = ListRemoveValueRequest::new(cache_name, list_name, "value1");
/// match cache_client.send_request(remove_request).await {
///     Ok(ListRemoveValueResponse {}) => println!("Successfully removed value"),
///     Err(e) => eprintln!("Error removing value: {:?}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListRemoveValueRequest<L: IntoBytes, V: IntoBytes> {
    cache_name: String,
    list_name: L,
    value: V,
}

impl<L: IntoBytes, V: IntoBytes> ListRemoveValueRequest<L, V> {
    /// Constructs a new ListRemoveValueRequest.
    pub fn new(cache_name: impl Into<String>, list_name: L, value: V) -> Self {
        Self {
            cache_name: cache_name.into(),
            list_name,
            value,
        }
    }
}

impl<L: IntoBytes, V: IntoBytes> MomentoRequest for ListRemoveValueRequest<L, V> {
    type Response = ListRemoveValueResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ListRemoveValueResponse> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::ListRemoveRequest {
                list_name: self.list_name.into_bytes(),
                remove: Some(Remove::AllElementsWithValue(self.value.into_bytes())),
            },
        )?;

        cache_client
            .data_client
            .clone()
            .list_remove(request)
            .await?
            .into_inner();
        Ok(ListRemoveValueResponse {})
    }
}

/// The response type for a successful list remove value request.
#[derive(Debug, PartialEq, Eq)]
pub struct ListRemoveValueResponse {}
