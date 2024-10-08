use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes,
    IntoBytesIterable, MomentoError,
};
use momento_protos::cache_client::{
    dictionary_delete_request as DictionaryFieldSelector,
    DictionaryDeleteRequest as DictionaryRemoveFieldsRequestProto,
};

/// Remove multiple fields from a dictionary.
///
/// # Arguments
/// * `cache_name` - The name of the cache containing the dictionary.
/// * `dictionary_name` - The name of the dictionary to remove fields from.
/// * `fields` - The fields to remove.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{DictionaryRemoveFieldsResponse, DictionaryRemoveFieldsRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let dictionary_name = "dictionary";
/// let fields = vec!["field1", "field2"];
///
/// let remove_fields_request = DictionaryRemoveFieldsRequest::new(
///   cache_name,
///   dictionary_name,
///   fields
/// );
///
/// match cache_client.send_request(remove_fields_request).await {
///   Ok(DictionaryRemoveFieldsResponse {}) => println!("Fields removed from dictionary"),
///   Err(e) => eprintln!("Error removing fields from dictionary: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct DictionaryRemoveFieldsRequest<D: IntoBytes, F: IntoBytesIterable> {
    cache_name: String,
    dictionary_name: D,
    fields: F,
}

impl<D: IntoBytes, F: IntoBytesIterable> DictionaryRemoveFieldsRequest<D, F> {
    /// Constructs a new DictionaryRemoveFieldsRequest.
    pub fn new(cache_name: impl Into<String>, dictionary_name: D, fields: F) -> Self {
        Self {
            cache_name: cache_name.into(),
            dictionary_name,
            fields,
        }
    }
}

impl<D: IntoBytes, F: IntoBytesIterable> MomentoRequest for DictionaryRemoveFieldsRequest<D, F> {
    type Response = DictionaryRemoveFieldsResponse;

    async fn send(self, cache_client: &CacheClient) -> Result<Self::Response, MomentoError> {
        let fields_to_delete = DictionaryFieldSelector::Some {
            fields: self.fields.into_bytes(),
        };
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            DictionaryRemoveFieldsRequestProto {
                dictionary_name: self.dictionary_name.into_bytes(),
                delete: Some(DictionaryFieldSelector::Delete::Some(fields_to_delete)),
            },
        )?;

        cache_client
.next_data_client()
            .dictionary_delete(request)
            .await?
            .into_inner();

        Ok(DictionaryRemoveFieldsResponse {})
    }
}

/// The response type for a successful dictionary remove fields request.
#[derive(Debug, PartialEq, Eq)]
pub struct DictionaryRemoveFieldsResponse {}
