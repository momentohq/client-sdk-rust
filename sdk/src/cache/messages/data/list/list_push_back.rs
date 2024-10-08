use crate::{
    cache::{CollectionTtl, MomentoRequest},
    utils::prep_request_with_timeout,
    CacheClient, IntoBytes, MomentoResult,
};

/// Adds an element to the back of the given list. Creates the list if it does not already exist.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `list_name` - name of the list
/// * `value` - value to append to list
///
/// # Optional Arguments
///
/// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
/// * `truncate_front_to_size` - If the list exceeds this length, remove excess from the front of the list.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{ListPushBackResponse, ListPushBackRequest, CollectionTtl};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let list_name = "list-name";
/// let push_back_request = ListPushBackRequest::new(cache_name, list_name, "value1")
///     .ttl(CollectionTtl::default())
///     .truncate_front_to_size(10);
///
/// match cache_client.send_request(push_back_request).await {
///     Ok(_) => println!("Element added to list"),
///     Err(e) => eprintln!("Error adding element to list: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListPushBackRequest<L: IntoBytes, V: IntoBytes> {
    cache_name: String,
    list_name: L,
    value: V,
    collection_ttl: Option<CollectionTtl>,
    truncate_front_to_size: Option<u32>,
}

impl<L: IntoBytes, V: IntoBytes> ListPushBackRequest<L, V> {
    /// Constructs a new ListPushBackRequest.
    pub fn new(cache_name: impl Into<String>, list_name: L, value: V) -> Self {
        Self {
            cache_name: cache_name.into(),
            list_name,
            value,
            collection_ttl: None,
            truncate_front_to_size: None,
        }
    }

    /// Set the time-to-live for the collection.
    pub fn ttl(mut self, collection_ttl: impl Into<Option<CollectionTtl>>) -> Self {
        self.collection_ttl = collection_ttl.into();
        self
    }

    /// If the list exceeds this length, remove excess from the front of the list.
    pub fn truncate_front_to_size(
        mut self,
        truncate_front_to_size: impl Into<Option<u32>>,
    ) -> Self {
        self.truncate_front_to_size = truncate_front_to_size.into();
        self
    }
}

impl<L: IntoBytes, V: IntoBytes> MomentoRequest for ListPushBackRequest<L, V> {
    type Response = ListPushBackResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ListPushBackResponse> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let value = self.value.into_bytes();
        let list_name = self.list_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::ListPushBackRequest {
                list_name,
                value,
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
                truncate_front_to_size: self.truncate_front_to_size.unwrap_or(0),
            },
        )?;

        let _ = cache_client
            .next_data_client()
            .list_push_back(request)
            .await?;
        Ok(ListPushBackResponse {})
    }
}

/// The response type for a successful list push back request.
#[derive(Debug, PartialEq, Eq)]
pub struct ListPushBackResponse {}
