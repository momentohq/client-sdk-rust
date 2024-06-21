use momento_protos::control_client;
use tonic::Request;

use crate::status_to_error;
use crate::storage::messages::momento_store_request::MomentoStorageRequest;
use crate::storage::PreviewStorageClient;
use crate::{utils, MomentoResult};

/// Request to create a store.
///
/// # Arguments
///
/// * `store_name` - The name of the store to create.
///
/// # Example
/// Assumes that a PreviewStorageClient named `storage_client` has been created and is available.
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_storage_client;
/// # tokio_test::block_on(async {
/// use momento::storage::{CreateStoreResponse, CreateStoreRequest};
/// # let (storage_client, store_name) = create_doctest_storage_client();
///
/// let create_store_request = CreateStoreRequest::new(&store_name);
///
/// match storage_client.send_request(create_store_request).await? {
///     CreateStoreResponse::Created => println!("Store {} created", &store_name),
///     CreateStoreResponse::AlreadyExists => println!("Store {} already exists", &store_name),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct CreateStoreRequest {
    /// The name of the store to create.
    pub store_name: String,
}

impl CreateStoreRequest {
    /// Constructs a new CreateStoreRequest.
    pub fn new(store_name: impl Into<String>) -> Self {
        CreateStoreRequest {
            store_name: store_name.into(),
        }
    }
}

impl MomentoStorageRequest for CreateStoreRequest {
    type Response = CreateStoreResponse;

    async fn send(
        self,
        storage_client: &PreviewStorageClient,
    ) -> MomentoResult<CreateStoreResponse> {
        utils::is_store_name_valid(&self.store_name)?;
        let request = Request::new(control_client::CreateStoreRequest {
            store_name: self.store_name,
        });

        let result = storage_client
            .control_client
            .clone()
            .create_store(request)
            .await;
        match result {
            Ok(_) => Ok(CreateStoreResponse::Created {}),
            Err(e) => {
                if e.code() == tonic::Code::AlreadyExists {
                    return Ok(CreateStoreResponse::AlreadyExists {});
                }
                Err(status_to_error(e))
            }
        }
    }
}

/// The response type for a successful create store request
#[derive(Debug, PartialEq, Eq)]
pub enum CreateStoreResponse {
    /// The store was created.
    Created,
    /// The store already exists.
    AlreadyExists,
}
