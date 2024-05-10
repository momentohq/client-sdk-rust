use crate::{
    cache::{CollectionTtl, MomentoRequest},
    utils::prep_request_with_timeout,
    CacheClient, IntoBytes, IntoBytesIterable, MomentoResult,
};

/// Adds multiple elements to the front of the given list. Creates the list if it does not already exist.
///
/// Prepends the supplied list to the front of a list. For example, if you have a list [1, 2, 3]
/// and listConcatenateFront [4, 5, 6], you will create [4, 5, 6, 1, 2, 3].
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `list_name` - name of the list
/// * `values` - list of values to add to the front of the list
///
/// # Optional Arguments
///
/// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
/// * `truncate_back_to_size` - If the list exceeds this length, remove excess from the back of the list.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{ListConcatenateFrontResponse, ListConcatenateFrontRequest, CollectionTtl};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let list_name = "list-name";
/// let concat_front_request = ListConcatenateFrontRequest::new(cache_name, list_name, vec!["value1", "value2"])
///     .ttl(CollectionTtl::default())
///     .truncate_back_to_size(10);
///
/// match cache_client.send_request(concat_front_request).await {
///     Ok(_) => println!("Elements added to list"),
///     Err(e) => eprintln!("Error adding elements to list: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListConcatenateFrontRequest<L: IntoBytes, V: IntoBytesIterable> {
    cache_name: String,
    list_name: L,
    values: V,
    collection_ttl: Option<CollectionTtl>,
    truncate_back_to_size: Option<u32>,
}

impl<L: IntoBytes, V: IntoBytesIterable> ListConcatenateFrontRequest<L, V> {
    pub fn new(cache_name: impl Into<String>, list_name: L, values: V) -> Self {
        Self {
            cache_name: cache_name.into(),
            list_name,
            values,
            collection_ttl: None,
            truncate_back_to_size: None,
        }
    }

    /// Set the time-to-live for the collection.
    pub fn ttl(mut self, collection_ttl: CollectionTtl) -> Self {
        self.collection_ttl = Some(collection_ttl);
        self
    }

    /// If the list exceeds this length, remove excess from the back of the list.
    pub fn truncate_back_to_size(mut self, truncate_back_to_size: u32) -> Self {
        self.truncate_back_to_size = Some(truncate_back_to_size);
        self
    }
}

impl<L: IntoBytes, V: IntoBytesIterable> MomentoRequest for ListConcatenateFrontRequest<L, V> {
    type Response = ListConcatenateFrontResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ListConcatenateFrontResponse> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let values = self.values;
        let list_name = self.list_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::ListConcatenateFrontRequest {
                list_name,
                values: values.into_bytes(),
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
                truncate_back_to_size: self.truncate_back_to_size.unwrap_or(0),
            },
        )?;

        let _ = cache_client
            .data_client
            .clone()
            .list_concatenate_front(request)
            .await?;
        Ok(ListConcatenateFrontResponse {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ListConcatenateFrontResponse {}
