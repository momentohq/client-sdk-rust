use crate::cache::messages::MomentoRequest;
use crate::utils;
use crate::utils::fmt::AsDebuggableValue;
use crate::CacheClient;
use crate::{IntoBytes, MomentoError, MomentoResult};
use derive_more::Display;
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
/// use momento::cache::{GetResponse, GetRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// # cache_client.set(&cache_name, "key", "value").await?;
///
/// let get_request = GetRequest::new(
///     cache_name,
///     "key"
/// );
///
/// let item: String = match(cache_client.send_request(get_request).await?) {
///   GetResponse::Hit { value } => value.try_into().expect("I stored a string!"),
///   GetResponse::Miss => return Err(anyhow::Error::msg("cache miss"))
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
    /// Constructs a new GetRequest.
    pub fn new(cache_name: impl Into<String>, key: K) -> Self {
        Self {
            cache_name: cache_name.into(),
            key,
        }
    }
}

impl<K: IntoBytes> MomentoRequest for GetRequest<K> {
    type Response = GetResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<GetResponse> {
        let request = utils::prep_request_with_timeout(
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
            ECacheResult::Hit => Ok(GetResponse::Hit {
                value: Value {
                    raw_item: response.cache_body,
                },
            }),
            ECacheResult::Miss => Ok(GetResponse::Miss),
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
/// # use momento::cache::{GetResponse, messages::data::scalar::get::Value};
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Hit { value: Value::default() };
/// use std::convert::TryInto;
/// let item: String = match get_response {
///     GetResponse::Hit { value } => value.try_into().expect("I stored a string!"),
///     GetResponse::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// Or, if you're storing raw bytes you can get at them simply:
/// ```
/// # use momento::cache::{GetResponse, messages::data::scalar::get::Value};
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Hit { value: Value::default() };
/// let item: Vec<u8> = match get_response {
///     GetResponse::Hit { value } => value.into(),
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
/// # use momento::cache::{GetResponse, messages::data::scalar::get::Value};
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Hit { value: Value::default() };
/// use std::convert::TryInto;
/// let item: MomentoResult<String> = get_response.try_into();
/// ```
///
/// You can also go straight into a `Vec<u8>` if you prefer:
/// ```
/// # use momento::cache::{GetResponse, messages::data::scalar::get::Value};
/// # use momento::MomentoResult;
/// # let get_response = GetResponse::Hit { value: Value::default() };
/// use std::convert::TryInto;
/// let item: MomentoResult<Vec<u8>> = get_response.try_into();
/// ```
#[derive(Debug, Display, PartialEq, Eq)]
pub enum GetResponse {
    /// The item was found in the cache.
    Hit {
        /// The value of the item.
        value: Value,
    },
    /// The item was not found in the cache.
    Miss,
}

impl<I: IntoBytes> From<I> for GetResponse {
    fn from(value: I) -> Self {
        GetResponse::Hit {
            value: Value::new(value.into_bytes()),
        }
    }
}

/// Represents a value retrieved from the cache.
#[derive(PartialEq, Eq, Default)]
pub struct Value {
    /// The raw bytes of the item.
    pub(crate) raw_item: Vec<u8>,
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw_item = &self.raw_item;
        let debug_value = raw_item.as_debuggable_value();
        f.debug_struct("Value")
            .field("raw_item", &debug_value)
            .finish()
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Value {
    /// Constructs a new Value.
    pub fn new(raw_item: Vec<u8>) -> Self {
        Self { raw_item }
    }
}

impl TryFrom<Value> for String {
    type Error = MomentoError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        utils::parse_string(value.raw_item)
    }
}

impl From<Value> for Vec<u8> {
    fn from(value: Value) -> Self {
        value.raw_item
    }
}

impl TryFrom<GetResponse> for String {
    type Error = MomentoError;

    fn try_from(value: GetResponse) -> Result<Self, Self::Error> {
        match value {
            GetResponse::Hit { value } => value.try_into(),
            GetResponse::Miss => Err(MomentoError::miss("Get")),
        }
    }
}

impl TryFrom<GetResponse> for Vec<u8> {
    type Error = MomentoError;

    fn try_from(value: GetResponse) -> Result<Self, Self::Error> {
        match value {
            GetResponse::Hit { value } => Ok(value.into()),
            GetResponse::Miss => Err(MomentoError::miss("Get")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_response_display() -> MomentoResult<()> {
        let hit = GetResponse::Hit {
            value: Value::new("hello".as_bytes().to_vec()),
        };
        assert_eq!(format!("{}", hit), r#"Value { raw_item: "hello" }"#);
        assert_eq!(
            format!("{:?}", hit),
            r#"Hit { value: Value { raw_item: "hello" } }"#
        );
        assert_eq!(
            format!("{:#?}", hit),
            str::trim(
                r#"
Hit {
    value: Value {
        raw_item: "hello",
    },
}"#
            )
        );

        let hit_with_binary_value = GetResponse::Hit {
            value: Value::new(vec![0, 150, 146, 159]),
        };
        assert_eq!(
            format!("{}", hit_with_binary_value),
            r#"Value { raw_item: [0, 150, 146, 159] }"#
        );
        assert_eq!(
            format!("{:?}", hit_with_binary_value),
            r#"Hit { value: Value { raw_item: [0, 150, 146, 159] } }"#
        );
        assert_eq!(
            format!("{:#?}", hit_with_binary_value),
            str::trim(
                r#"
Hit {
    value: Value {
        raw_item: [
            0,
            150,
            146,
            159,
        ],
    },
}"#
            )
        );

        let miss = GetResponse::Miss;
        assert_eq!(format!("{}", miss), "Miss");

        Ok(())
    }
}
