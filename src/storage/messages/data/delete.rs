use crate::storage::messages::momento_storage_request::MomentoStorageRequest;
use crate::storage::PreviewStorageClient;
use crate::utils::prep_storage_request_with_timeout;
use crate::MomentoResult;

/// Deletes an item in a Momento Store
///
/// # Arguments
///
/// * `store_name` - name of the store
/// * `key` - key of the item to delete
///
/// # Examples
/// Assumes that a PreviewStorageClient named `storage_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_storage_client;
/// # tokio_test::block_on(async {
/// # let (storage_client, store_name) = create_doctest_storage_client();
/// use momento::storage::{DeleteResponse, DeleteRequest};
/// use momento::MomentoErrorCode;
///
/// let delete_request = DeleteRequest::new(
///     &store_name,
///     "key"
/// );
///
/// match storage_client.send_request(delete_request).await {
///     Ok(_) => println!("DeleteResponse successful"),
///     Err(e) => if let MomentoErrorCode::StoreNotFoundError = e.error_code {
///         println!("Store not found: {}", &store_name);
///     } else {
///         eprintln!("Error deleting value in store {}: {}", &store_name, e);
///     }
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct DeleteRequest {
    store_name: String,
    key: String,
}

impl DeleteRequest {
    /// Constructs a new DeleteRequest.
    pub fn new(store_name: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            store_name: store_name.into(),
            key: key.into(),
        }
    }
}

impl MomentoStorageRequest for DeleteRequest {
    type Response = DeleteResponse;

    async fn send(self, storage_client: &PreviewStorageClient) -> MomentoResult<DeleteResponse> {
        let request = prep_storage_request_with_timeout(
            &self.store_name,
            storage_client.configuration.deadline_millis(),
            momento_protos::store::StoreDeleteRequest { key: self.key },
        )?;

        let _ = storage_client
            .storage_client
            .clone()
            .delete(request)
            .await?;
        Ok(DeleteResponse {})
    }
}

/// The response type for a successful delete request
#[derive(Debug, PartialEq, Eq)]
pub struct DeleteResponse {}
