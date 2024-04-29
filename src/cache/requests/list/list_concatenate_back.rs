use crate::{
    cache::{CollectionTtl, MomentoRequest},
    utils::prep_request_with_timeout,
    CacheClient, IntoBytes, MomentoResult,
};

/// Adds multiple elements to the back of the given list. Creates the list if it does not already exist.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `list_name` - name of the list
/// * `values` - list of values to add to the back of the list
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
/// use momento::cache::{ListConcatenateBack, ListConcatenateBackRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let list_name = "list-name";
/// let concat_back_request = ListConcatenateBackRequest::new(cache_name, list_name, vec!["value1", "value2"]);
///
/// match cache_client.send_request(concat_back_request).await {
///     Ok(_) => println!("Elements added to list"),
///     Err(e) => eprintln!("Error adding elements to list: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListConcatenateBackRequest<L: IntoBytes, V: IntoBytes> {
    cache_name: String,
    list_name: L,
    values: Vec<V>,
    collection_ttl: Option<CollectionTtl>,
    truncate_front_to_size: Option<u32>,
}

impl<L: IntoBytes, V: IntoBytes> ListConcatenateBackRequest<L, V> {
    pub fn new(cache_name: impl Into<String>, list_name: L, values: Vec<V>) -> Self {
        Self {
            cache_name: cache_name.into(),
            list_name,
            values,
            collection_ttl: None,
            truncate_front_to_size: None,
        }
    }

    /// Set the time-to-live for the collection.
    pub fn ttl(mut self, collection_ttl: CollectionTtl) -> Self {
        self.collection_ttl = Some(collection_ttl);
        self
    }

    /// If the list exceeds this length, remove excess from the front of the list.
    pub fn truncate_back_to_size(mut self, truncate_front_to_size: u32) -> Self {
        self.truncate_front_to_size = Some(truncate_front_to_size);
        self
    }
}

impl<L: IntoBytes, V: IntoBytes> MomentoRequest for ListConcatenateBackRequest<L, V> {
    type Response = ListConcatenateBack;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ListConcatenateBack> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let values = self.values;
        let list_name = self.list_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::ListConcatenateBackRequest {
                list_name,
                values: values.into_iter().map(|v| v.into_bytes()).collect(),
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
                truncate_front_to_size: self.truncate_front_to_size.unwrap_or(0),
            },
        )?;

        let _ = cache_client
            .data_client
            .clone()
            .list_concatenate_back(request)
            .await?;
        Ok(ListConcatenateBack {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ListConcatenateBack {}
