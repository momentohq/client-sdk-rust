use crate::storage::messages::momento_store_request::MomentoStorageRequest;
use crate::storage::messages::store_value::StoreValue;
use crate::storage::PreviewStorageClient;
use crate::utils::prep_storage_request_with_timeout;
use crate::MomentoResult;

/// Request to put a value in a store.
///
/// # Arguments
///
/// * `store_name` - The name of the store to add a value to.
/// * `key` - key of the item whose value we are putting
/// * `value` - data to store
///
///
/// # Example
/// Assumes that a PreviewStorageClient named `storage_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_storage_client;
/// # tokio_test::block_on(async {
/// use std::time::Duration;
/// use momento::storage::{PutResponse, PutRequest};
/// use momento::MomentoErrorCode;
/// # let (storage_client, store_name) = create_doctest_storage_client();
///
/// let put_request = PutRequest::new(
///     &store_name,
///     "key",
///     "value1"
/// );
///
/// match storage_client.send_request(put_request).await {
///     Ok(_) => println!("PutResponse successful"),
///     Err(e) => if let MomentoErrorCode::NotFoundError = e.error_code {
///         println!("Store not found: {}", &store_name);
///     } else {
///         eprintln!("Error putting value in store {}: {}", &store_name, e);
///     }
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct PutRequest {
    store_name: String,
    key: String,
    value: StoreValue,
}

impl PutRequest {
    /// Construct a new PutRequest.
    pub fn new(
        store_name: impl Into<String>,
        key: impl Into<String>,
        value: impl Into<StoreValue>,
    ) -> Self {
        Self {
            store_name: store_name.into(),
            key: key.into(),
            value: value.into(),
        }
    }
}

impl MomentoStorageRequest for PutRequest {
    type Response = PutResponse;

    async fn send(self, storage_client: &PreviewStorageClient) -> MomentoResult<PutResponse> {
        let request = prep_storage_request_with_timeout(
            &self.store_name,
            storage_client.configuration.deadline_millis(),
            momento_protos::store::StorePutRequest {
                key: self.key,
                value: Some(self.value.into()),
            },
        )?;

        storage_client.storage_client.clone().put(request).await?;
        Ok(PutResponse {})
    }
}

/// The response type for a successful put request.
#[derive(Debug, PartialEq, Eq)]
pub struct PutResponse {}
