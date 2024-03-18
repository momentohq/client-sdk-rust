use crate::cache_client::CacheClient;
use crate::requests::cache::MomentoRequest;
use crate::simple_cache_client::prep_request;
use crate::utils::parse_string;
use crate::{IntoBytes, MomentoError, MomentoResult};
use momento_protos::cache_client::ECacheResult;
use std::convert::{TryFrom, TryInto};

/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_client();
/// use std::convert::TryInto;
/// use momento::requests::cache::basic::get::Get;
///
/// let get_response = cache_client.get(&cache_name, "key").await?;
///
/// let item: String = match get_response {
///     Get::Hit { value } => value.try_into().expect("I stored a string!"),
///     Get::Miss => return Err(anyhow::Error::msg("cache miss")) // probably you'll do something else here
/// };
///
/// # Ok(())
/// # })
/// #
/// }
/// ```
pub struct GetRequest<K: IntoBytes> {
    cache_name: String,
    key: K,
}

impl<K: IntoBytes> GetRequest<K> {
    pub fn new(cache_name: String, key: K) -> Self {
        Self { cache_name, key }
    }
}

impl<K: IntoBytes> MomentoRequest for GetRequest<K> {
    type Response = Get;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<Get> {
        let request = prep_request(
            &self.cache_name,
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
            _ => unreachable!(),
        }
    }
}

/// Response for a cache get operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::response::Get;
/// # use momento::MomentoResult;
/// # let get_response = Get::Hit { value: momento::response::GetValue::new(vec![]) };
/// use std::convert::TryInto;
/// let item: String = match get_response {
///     Get::Hit { value } => value.try_into().expect("I stored a string!"),
///     Get::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// Or, if you're storing raw bytes you can get at them simply:
/// ```
/// # use momento::response::Get;
/// # use momento::MomentoResult;
/// # let get_response = Get::Hit { value: momento::response::GetValue::new(vec![]) };
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
/// # use momento::response::Get;
/// # use momento::MomentoResult;
/// # let get_response = Get::Hit { value: momento::response::GetValue::new(vec![]) };
/// use std::convert::TryInto;
/// let item: MomentoResult<String> = get_response.try_into();
/// ```
///
/// You can also go straight into a Vec<u8> if you prefer:
/// ```
/// # use momento::response::Get;
/// # use momento::MomentoResult;
/// # let get_response = Get::Hit { value: momento::response::GetValue::new(vec![]) };
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
            Get::Miss => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("get response was a miss"),
            }),
        }
    }
}

impl TryFrom<Get> for Vec<u8> {
    type Error = MomentoError;

    fn try_from(value: Get) -> Result<Self, Self::Error> {
        match value {
            Get::Hit { value } => Ok(value.into()),
            Get::Miss => Err(MomentoError::Miss {
                description: std::borrow::Cow::Borrowed("get response was a miss"),
            }),
        }
    }
}
