use crate::cache::messages::MomentoRequest;
use crate::utils::parse_string;
use crate::utils::prep_request_with_timeout;
use crate::CacheClient;
use crate::{IntoBytes, MomentoError, MomentoResult};
use momento_protos::cache_client::ECacheResult;
use std::convert::{TryFrom, TryInto};

/// Request to get an item from a cache
///
/// # Arguments
///
/// * `cache_name` - name of cache
/// * `key` - key of entry within the cache.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::cache::{Get, GetRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// # cache_client.set(&cache_name, "key", "value").await?;
///
/// let get_request = GetRequest::new(
///     cache_name,
///     "key"
/// );
///
/// let item: String = match(cache_client.send_request(get_request).await?) {
///   Get::Hit { value } => value.try_into().expect("I stored a string!"),
///   Get::Miss => return Err(anyhow::Error::msg("cache miss"))
/// };
/// # assert_eq!(item, "value");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct GetRequest<K: IntoBytes> {
    cache_name: String,
    key: K,
}

impl<K: IntoBytes> GetRequest<K> {
    pub fn new(cache_name: impl Into<String>, key: K) -> Self {
        Self {
            cache_name: cache_name.into(),
            key,
        }
    }
}

impl<K: IntoBytes> MomentoRequest for GetRequest<K> {
    type Response = Get;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<Get> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::GetRequest {
                cache_key: self.key.into_bytes(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .get(request)
            .await?
            .into_inner();
        match response.result() {
            ECacheResult::Hit => Ok(Get::Hit {
                value: GetValue {
                    raw_item: response.cache_body,
                },
            }),
            ECacheResult::Miss => Ok(Get::Miss),
            _ => Err(MomentoError::unknown_error(
                "Get",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response for a cache get operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::cache::{Get, GetValue};
/// # use momento::MomentoResult;
/// # let get_response = Get::Hit { value: GetValue::new(vec![]) };
/// use std::convert::TryInto;
/// let item: String = match get_response {
///     Get::Hit { value } => value.try_into().expect("I stored a string!"),
///     Get::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// Or, if you're storing raw bytes you can get at them simply:
/// ```
/// # use momento::cache::{Get, GetValue};
/// # use momento::MomentoResult;
/// # let get_response = Get::Hit { value: GetValue::new(vec![]) };
/// let item: Vec<u8> = match get_response {
///     Get::Hit { value } => value.into(),
///     Get::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<String, MomentoError> suitable for
/// ?-propagation if you know you are expecting a String item.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::cache::{Get, GetValue};
/// # use momento::MomentoResult;
/// # let get_response = Get::Hit { value: GetValue::new(vec![]) };
/// use std::convert::TryInto;
/// let item: MomentoResult<String> = get_response.try_into();
/// ```
///
/// You can also go straight into a `Vec<u8>` if you prefer:
/// ```
/// # use momento::cache::{Get, GetValue};
/// # use momento::MomentoResult;
/// # let get_response = Get::Hit { value: GetValue::new(vec![]) };
/// use std::convert::TryInto;
/// let item: MomentoResult<Vec<u8>> = get_response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum Get {
    Hit { value: GetValue },
    Miss,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GetValue {
    pub(crate) raw_item: Vec<u8>,
}
impl GetValue {
    pub fn new(raw_item: Vec<u8>) -> Self {
        Self { raw_item }
    }
}

impl TryFrom<GetValue> for String {
    type Error = MomentoError;

    fn try_from(value: GetValue) -> Result<Self, Self::Error> {
        parse_string(value.raw_item)
    }
}

impl From<GetValue> for Vec<u8> {
    fn from(value: GetValue) -> Self {
        value.raw_item
    }
}

impl TryFrom<Get> for String {
    type Error = MomentoError;

    fn try_from(value: Get) -> Result<Self, Self::Error> {
        match value {
            Get::Hit { value } => value.try_into(),
            Get::Miss => Err(MomentoError::miss("Get")),
        }
    }
}

impl TryFrom<Get> for Vec<u8> {
    type Error = MomentoError;

    fn try_from(value: Get) -> Result<Self, Self::Error> {
        match value {
            Get::Hit { value } => Ok(value.into()),
            Get::Miss => Err(MomentoError::miss("Get")),
        }
    }
}
