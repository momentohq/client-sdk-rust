use std::convert::{TryFrom, TryInto};
use crate::storage::messages::momento_store_request::MomentoStorageRequest;
use crate::storage::messages::store_value::StoreValue;
use crate::storage::PreviewStorageClient;
use crate::{MomentoErrorCode, utils};
use crate::{MomentoError, MomentoResult};

/// Request to get an item from a store
///
/// # Arguments
///
/// * `store_name` - name of the store
/// * `key` - key of entry within the store.
///
/// # Examples
/// Assumes that a PreviewStorageClient named `storage_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use std::convert::TryInto;
/// use momento_test_util::create_doctest_storage_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::storage::{GetResponse, GetRequest};
/// # let (storage_client, store_name) = create_doctest_storage_client();
/// # storage_client.put(&store_name, "key", "value").await?;
///
/// let get_request = GetRequest::new(
///     store_name,
///     "key"
/// );
///
/// let item: String = storage_client.send_request(get_request).await?.try_into().expect("I stored a string!");
/// # assert_eq!(item, "value");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct GetRequest {
    store_name: String,
    key: String,
}

impl GetRequest {
    /// Constructs a new GetRequest.
    pub fn new(store_name: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            store_name: store_name.into(),
            key: key.into(),
        }
    }
}

impl MomentoStorageRequest for GetRequest {
    type Response = GetResponse;

    async fn send(self, storage_client: &PreviewStorageClient) -> MomentoResult<GetResponse> {
        let request = utils::prep_storage_request_with_timeout(
            &self.store_name,
            storage_client.configuration.deadline_millis(),
            momento_protos::store::StoreGetRequest { key: self.key },
        )?;

        let response = storage_client
            .storage_client
            .clone()
            .get(request)
            .await;

        match response {
            Ok(success) => match success.into_inner().value {
                None => Err(MomentoError::unknown_error("StoreGet", None)),
                Some(store_value) => match store_value.value {
                    None => Err(MomentoError::unknown_error("StoreGet", None)),
                    Some(value) => Ok(GetResponse {
                        value: Some(value.into()),
                    }),
                },
            },
            Err(status) => {
                let e: MomentoError = status.into();
                match e.error_code {
                    MomentoErrorCode::ItemNotFoundError => Ok(GetResponse { value: None }),
                    _ => Err(e),
                }
            }
        }
    }
}

/// Response for a store get operation.
///
/// You can cast your result directly into a Result<String, MomentoError> suitable for
/// ?-propagation if you know you are expecting a String item.
///
/// ```
/// # use momento::storage::GetResponse;
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Success { value: "value".into() };
/// use std::convert::TryInto;
/// let item: MomentoResult<String> = get_response.try_into();
/// ```
///
/// You can also go convert into a `Vec<u8>`, `i64`, or `f64` depending on the type you stored:
/// ```
/// # use momento::storage::GetResponse;
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Success { value: vec![1, 2, 3, 4, 5].into() };
/// use std::convert::TryInto;
/// let item: MomentoResult<Vec<u8>> = get_response.try_into();
/// ```
///
/// ```
/// # use momento::storage::GetResponse;
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Success { value: 1.into() };
/// use std::convert::TryInto;
/// let item: MomentoResult<i64> = get_response.try_into();
/// ```
///
/// ```
/// # use momento::storage::GetResponse;
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Success { value: 1.0.into() };
/// use std::convert::TryInto;
/// let item: MomentoResult<f64> = get_response.try_into();
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct GetResponse {
    /// The value of the item.
    pub value: Option<StoreValue>,
}

fn not_found_error() -> MomentoError {
    MomentoError {
        message: "Storage value not found".into(),
        error_code: MomentoErrorCode::ItemNotFoundError,
        inner_error: None,
        details: None,
    }
}

impl<I: Into<StoreValue>> From<I> for GetResponse {
    fn from(value: I) -> Self {
        GetResponse {
            value: Some(value.into()),
        }
    }
}

impl TryFrom<GetResponse> for Option<Vec<u8>> {
    type Error = MomentoError;

    fn try_from(response: GetResponse) -> Result<Self, Self::Error> {
        match response.value {
            None => Ok(None),
            Some(v) => v.try_into().map(Some),
        }
    }
}

impl TryFrom<GetResponse> for Vec<u8> {
    type Error = MomentoError;

    fn try_from(response: GetResponse) -> Result<Self, Self::Error> {
        match response.value {
            None => Err(not_found_error()),
            Some(v) => v.try_into(),
        }
    }
}

impl TryFrom<GetResponse> for Option<String> {
    type Error = MomentoError;

    fn try_from(response: GetResponse) -> Result<Self, Self::Error> {
        match response.value {
                None => Ok(None),
                Some(v) => v.try_into().map(Some),
        }
    }
}

impl TryFrom<GetResponse> for String {
    type Error = MomentoError;

    fn try_from(response: GetResponse) -> Result<Self, Self::Error> {
        match response.value {
            None => Err(not_found_error()),
            Some(v) => v.try_into(),
        }
    }
}

impl TryFrom<GetResponse> for Option<i64> {
    type Error = MomentoError;

    fn try_from(response: GetResponse) -> Result<Self, Self::Error> {
        match response.value {
                None => Ok(None),
                Some(v) => v.try_into().map(Some),
        }
    }
}

impl TryFrom<GetResponse> for i64 {
    type Error = MomentoError;

    fn try_from(response: GetResponse) -> Result<Self, Self::Error> {
        match response.value {
            None => Err(not_found_error()),
            Some(v) => v.try_into(),
        }
    }
}

impl TryFrom<GetResponse> for Option<f64> {
    type Error = MomentoError;

    fn try_from(response: GetResponse) -> Result<Self, Self::Error> {
        match response.value {
                None => Ok(None),
                Some(v) => v.try_into().map(Some),
        }
    }
}

impl TryFrom<GetResponse> for f64 {
    type Error = MomentoError;

    fn try_from(response: GetResponse) -> Result<Self, Self::Error> {
        match response.value {
            None => Err(not_found_error()),
            Some(v) => v.try_into(),
        }
    }
}