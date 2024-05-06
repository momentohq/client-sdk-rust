use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoError,
};
use momento_protos::cache_client::{
    dictionary_delete_request as DictionaryFieldSelector,
    DictionaryDeleteRequest as DictionaryRemoveFieldsRequestProto,
};

/// Remove a field from a dictionary.
///
/// # Arguments
/// * `cache_name` - The name of the cache containing the dictionary.
/// * `dictionary_name` - The name of the dictionary to remove field from.
/// * `field` - The field to remove.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{DictionaryRemoveField, DictionaryRemoveFieldRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let dictionary_name = "dictionary";
/// let field = "field";
///
/// let remove_field_request = DictionaryRemoveFieldRequest::new(
///   cache_name,
///   dictionary_name,
///   field
/// );
///
/// match cache_client.send_request(remove_field_request).await {
///   Ok(DictionaryRemoveField {}) => println!("Field removed from dictionary"),
///   Err(e) => eprintln!("Error removing field from dictionary: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct DictionaryRemoveFieldRequest<D: IntoBytes, F: IntoBytes> {
    cache_name: String,
    dictionary_name: D,
    field: F,
}

impl<D: IntoBytes, F: IntoBytes> DictionaryRemoveFieldRequest<D, F> {
    pub fn new(cache_name: impl Into<String>, dictionary_name: D, field: F) -> Self {
        Self {
            cache_name: cache_name.into(),
            dictionary_name,
            field,
        }
    }
}

impl<D: IntoBytes, F: IntoBytes> MomentoRequest for DictionaryRemoveFieldRequest<D, F> {
    type Response = DictionaryRemoveField;

    async fn send(self, cache_client: &CacheClient) -> Result<Self::Response, MomentoError> {
        let fields_to_delete = DictionaryFieldSelector::Some {
            fields: vec![self.field.into_bytes()],
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
            .data_client
            .clone()
            .dictionary_delete(request)
            .await?
            .into_inner();

        Ok(DictionaryRemoveField {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DictionaryRemoveField {}
