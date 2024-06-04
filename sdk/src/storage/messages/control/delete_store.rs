use momento_protos::control_client;
use tonic::Request;

use crate::storage::messages::momento_store_request::MomentoStorageRequest;
use crate::storage::PreviewStorageClient;
use crate::{utils, MomentoResult};

/// Request to delete a store
///
/// # Arguments
///
/// * `store_name` - The name of the store to be deleted.
///
/// # Examples
/// Assumes that a PreviewStorageClient named `storage_client` has been created and is available.
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_storage_client;
/// # tokio_test::block_on(async {
/// use momento::storage::{DeleteStoreResponse, DeleteStoreRequest};
/// use momento::MomentoErrorCode;
/// # let (storage_client, store_name) = create_doctest_storage_client();
///
/// let delete_store_request = DeleteStoreRequest::new(&store_name);
///
/// match storage_client.send_request(delete_store_request).await {
///     Ok(_) => println!("Store deleted: {}", &store_name),
///     Err(e) => if let MomentoErrorCode::NotFoundError = e.error_code {
///         println!("Store not found: {}", &store_name);
///     } else {
///         eprintln!("Error deleting store {}: {}", &store_name, e);
///     }
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct DeleteStoreRequest {
    /// The name of the store to be deleted.
    pub store_name: String,
}

impl DeleteStoreRequest {
    /// Constructs a new DeleteStoreRequest.
    pub fn new(store_name: impl Into<String>) -> Self {
        DeleteStoreRequest {
            store_name: store_name.into(),
        }
    }
}

impl MomentoStorageRequest for DeleteStoreRequest {
    type Response = DeleteStoreResponse;

    async fn send(
        self,
        storage_client: &PreviewStorageClient,
    ) -> MomentoResult<DeleteStoreResponse> {
        let store_name = &self.store_name;

        utils::is_store_name_valid(store_name)?;
        let request = Request::new(control_client::DeleteStoreRequest {
            store_name: store_name.to_string(),
        });

        let _ = storage_client
            .control_client
            .clone()
            .delete_store(request)
            .await?;
        Ok(DeleteStoreResponse {})
    }
}

/// The response type for a successful delete store request
#[derive(Debug, PartialEq, Eq)]
pub struct DeleteStoreResponse {}
