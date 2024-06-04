use momento_protos::control_client;
use tonic::Request;

use crate::storage::messages::momento_store_request::MomentoStorageRequest;
use crate::storage::PreviewStorageClient;
use crate::MomentoResult;

/// Request to list all stores in your account.
///
/// # Example
/// Assumes that a PreviewStorageClient named `storage_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_storage_client;
/// # tokio_test::block_on(async {
/// use momento::storage::{ListStoresResponse, ListStoresRequest};
/// # let (storage_client, store_name) = create_doctest_storage_client();
///
/// let list_stores_request = ListStoresRequest {};
///
/// match storage_client.send_request(list_stores_request).await {
///     Ok(response) => println!("Stores: {:#?}", response.stores),
///     Err(e) => eprintln!("Error listing stores: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListStoresRequest {}

impl MomentoStorageRequest for ListStoresRequest {
    type Response = ListStoresResponse;

    async fn send(
        self,
        storage_client: &PreviewStorageClient,
    ) -> MomentoResult<ListStoresResponse> {
        let request = Request::new(control_client::ListStoresRequest {
            next_token: "".to_string(),
        });

        let response = storage_client
            .control_client
            .clone()
            .list_stores(request)
            .await?
            .into_inner();

        Ok(ListStoresResponse::from_response(response))
    }
}

/// Information about a store.
#[derive(Debug, PartialEq, Eq)]
pub struct StoreInfo {
    /// The name of the store.
    pub name: String,
}

/// The response type for a successful list stores request.
#[derive(Debug, PartialEq, Eq)]
pub struct ListStoresResponse {
    /// The stores in your account.
    pub stores: Vec<StoreInfo>,
}

impl ListStoresResponse {
    /// Convert a ListStoresResponse from the server into a ListStoresResponse.
    pub fn from_response(response: control_client::ListStoresResponse) -> ListStoresResponse {
        let mut stores = Vec::new();
        for store in response.store {
            stores.push(StoreInfo {
                name: store.store_name,
            });
        }
        ListStoresResponse { stores }
    }
}

impl From<ListStoresResponse> for Vec<StoreInfo> {
    fn from(response: ListStoresResponse) -> Self {
        response.stores
    }
}
