use std::convert::{TryFrom, TryInto};

use derive_more::Display;

use crate::storage::messages::momento_store_request::MomentoStorageRequest;
use crate::storage::messages::store_value::StoreValue;
use crate::storage::PreviewStorageClient;
use crate::utils;
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
/// # use momento_test_util::create_doctest_storage_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::storage::{GetResponse, GetRequest};
/// # let (storage_client, store_name) = create_doctest_storage_client();
/// # storage_client.set(&store_name, "key", "value").await?;
///
/// let get_request = GetRequest::new(
///     store_name,
///     "key"
/// );
///
/// let item: String = match storage_client.send_request(get_request).await? {
///   GetResponse::Hit { value } => value.try_into().expect("I stored a string!"),
///   GetResponse::Miss => return Err(anyhow::Error::msg("store miss"))
/// };
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
            .await?
            .into_inner();
        match response.value {
            None => Ok(GetResponse::Miss),
            Some(store_value) => match store_value.value {
                None => Ok(GetResponse::Miss),
                Some(value) => Ok(GetResponse::Hit {
                    value: value.into(),
                }),
            },
        }
    }
}

/// Response for a store get operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::storage::GetResponse;
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Hit { value: "value".into() };
/// use std::convert::TryInto;
/// let item: String = match get_response {
///     GetResponse::Hit { value } => value.try_into().expect("I stored a string!"),
///     GetResponse::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<String, MomentoError> suitable for
/// ?-propagation if you know you are expecting a String item.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::storage::GetResponse;
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Hit { value: "value".into() };
/// use std::convert::TryInto;
/// let item: MomentoResult<String> = get_response.try_into();
/// ```
///
/// You can also go convert into a `Vec<u8>`, `i64`, or `f64` depending on the type you stored:
/// ```
/// # use momento::storage::GetResponse;
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Hit { value: vec![1, 2, 3, 4, 5].into() };
/// use std::convert::TryInto;
/// let item: MomentoResult<Vec<u8>> = get_response.try_into();
/// ```
///
/// ```
/// # use momento::storage::GetResponse;
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Hit { value: 1.into() };
/// use std::convert::TryInto;
/// let item: MomentoResult<i64> = get_response.try_into();
/// ```
///
/// ```
/// # use momento::storage::GetResponse;
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Hit { value: 1.0.into() };
/// use std::convert::TryInto;
/// let item: MomentoResult<f64> = get_response.try_into();
/// ```
#[derive(Debug, Display, PartialEq)]
pub enum GetResponse {
    /// The item was found in the store.
    Hit {
        /// The value of the item.
        value: StoreValue,
    },
    /// The item was not found in the store.
    Miss,
}

impl<I: Into<StoreValue>> From<I> for GetResponse {
    fn from(value: I) -> Self {
        GetResponse::Hit {
            value: value.into(),
        }
    }
}

impl TryFrom<GetResponse> for Vec<u8> {
    type Error = MomentoError;

    fn try_from(value: GetResponse) -> Result<Self, Self::Error> {
        match value {
            GetResponse::Hit { value } => value.try_into(),
            GetResponse::Miss => Err(MomentoError::miss("StoreGet")),
        }
    }
}

impl TryFrom<GetResponse> for String {
    type Error = MomentoError;

    fn try_from(value: GetResponse) -> Result<Self, Self::Error> {
        match value {
            GetResponse::Hit { value } => value.try_into(),
            GetResponse::Miss => Err(MomentoError::miss("StoreGet")),
        }
    }
}

impl TryFrom<GetResponse> for i64 {
    type Error = MomentoError;

    fn try_from(value: GetResponse) -> Result<Self, Self::Error> {
        match value {
            GetResponse::Hit { value } => value.try_into(),
            GetResponse::Miss => Err(MomentoError::miss("StoreGet")),
        }
    }
}

impl TryFrom<GetResponse> for f64 {
    type Error = MomentoError;

    fn try_from(value: GetResponse) -> Result<Self, Self::Error> {
        match value {
            GetResponse::Hit { value } => value.try_into(),
            GetResponse::Miss => Err(MomentoError::miss("StoreGet")),
        }
    }
}
