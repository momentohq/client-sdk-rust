use crate::cache::messages::MomentoRequest;
use crate::utils::fmt::{AsDebuggableValue, DebuggableValue};
use crate::utils::{parse_string, prep_request_with_timeout};
use crate::{CacheClient, IntoBytes, MomentoError, MomentoResult};
use derive_more::Display;
use momento_protos::cache_client::{
    dictionary_fetch_response::Dictionary as DictionaryProto,
    DictionaryFetchRequest as DictionaryFetchRequestProto,
};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;

/// Request to fetch a dictionary from a cache.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the dictionary.
/// * `dictionary_name` - The name of the dictionary to fetch.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use std::collections::HashMap;
/// # use std::convert::TryInto;
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{DictionaryFetchRequest, DictionaryFetchResponse};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let dictionary_name = "dictionary";
///
/// let set_response = cache_client.dictionary_set_field(
///    cache_name.to_string(),
///    dictionary_name,
///    "field1",
///    "value1"
/// ).await?;
///
/// let fetch_request = DictionaryFetchRequest::new(cache_name, dictionary_name);
///
/// let fetch_response = cache_client.send_request(fetch_request).await?;
///
/// let returned_dictionary: HashMap<String, String> = fetch_response.try_into()
///    .expect("dictionary should be returned");
/// println!("{:?}", returned_dictionary);
/// # Ok(())
/// # })
/// # }
/// ```
pub struct DictionaryFetchRequest<D: IntoBytes> {
    cache_name: String,
    dictionary_name: D,
}

impl<D: IntoBytes> DictionaryFetchRequest<D> {
    /// Constructs a new DictionaryFetchRequest.
    pub fn new(cache_name: impl Into<String>, dictionary_name: D) -> Self {
        Self {
            cache_name: cache_name.into(),
            dictionary_name,
        }
    }
}

impl<D: IntoBytes> MomentoRequest for DictionaryFetchRequest<D> {
    type Response = DictionaryFetchResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<Self::Response> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            DictionaryFetchRequestProto {
                dictionary_name: self.dictionary_name.into_bytes(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .dictionary_fetch(request)
            .await?
            .into_inner();

        match response.dictionary {
            Some(DictionaryProto::Missing(_)) => Ok(DictionaryFetchResponse::Miss),
            Some(DictionaryProto::Found(elements)) => {
                let raw_item = elements
                    .items
                    .into_iter()
                    .map(|element| (element.field, element.value))
                    .collect();
                Ok(DictionaryFetchResponse::Hit {
                    value: Value::new(raw_item),
                })
            }
            _ => Err(MomentoError::unknown_error(
                "DictionaryFetch",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response to a dictionary fetch request.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// fn main() -> anyhow::Result<()> {
/// # use std::collections::HashMap;
/// # use momento::cache::{DictionaryFetchResponse, messages::data::dictionary::dictionary_fetch::Value};
/// # use momento::MomentoResult;
/// # let fetch_response = DictionaryFetchResponse::Hit { value: Value::default() };
/// use std::convert::TryInto;
/// let item: HashMap<String, String> = match fetch_response {
///    DictionaryFetchResponse::Hit { value } => value.try_into().expect("I stored strings!"),
///   DictionaryFetchResponse::Miss => panic!("I expected a hit!"),
/// };
/// # Ok(())
/// }
/// ```
///
/// Or, if you're storing raw bytes you can get at them simply:
/// ```
/// # use std::collections::HashMap;
/// # use momento::cache::{DictionaryFetchResponse, messages::data::dictionary::dictionary_fetch::Value};
/// # use momento::MomentoResult;
/// # let fetch_response = DictionaryFetchResponse::Hit { value: Value::default() };
/// use std::convert::TryInto;
/// let item: HashMap<Vec<u8>, Vec<u8>> = match fetch_response {
///   DictionaryFetchResponse::Hit { value } => value.into(),
///   DictionaryFetchResponse::Miss => panic!("I expected a hit!"),
/// };
/// ```
///
/// You can cast your result directly into a Result<HashMap<String, String>, MomentoError> suitable for
/// ?-propagation if you know you are expecting a HashMap<String, String> item.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use std::collections::HashMap;
/// # use momento::cache::{DictionaryFetchResponse, messages::data::dictionary::dictionary_fetch::Value};
/// # use momento::MomentoResult;
/// # let fetch_response = DictionaryFetchResponse::Hit { value: Value::default() };
/// use std::convert::TryInto;
/// let item: MomentoResult<HashMap<String, String>> = fetch_response.try_into();
/// ```
///
/// You can also go straight into a `HashMap<Vec<u8>, Vec<u8>>` if you prefer:
/// ```
/// # use std::collections::HashMap;
/// # use momento::cache::{DictionaryFetchResponse, messages::data::dictionary::dictionary_fetch::Value};
/// # use momento::MomentoResult;
/// # let fetch_response = DictionaryFetchResponse::Hit { value: Value::default() };
/// use std::convert::TryInto;
/// let item: MomentoResult<HashMap<Vec<u8>, Vec<u8>>> = fetch_response.try_into();
/// ```
#[derive(Debug, Display, PartialEq, Eq)]
pub enum DictionaryFetchResponse {
    /// The dictionary was found.
    Hit {
        /// The dictionary values.
        value: Value,
    },
    /// The dictionary was not found.
    Miss,
}

/// A dictionary fetched from a cache.
#[derive(PartialEq, Eq, Default)]
pub struct Value {
    /// The raw dictionary item.
    pub(crate) raw_item: HashMap<Vec<u8>, Vec<u8>>,
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let debug_map_with_strings: HashMap<DebuggableValue, DebuggableValue> = self
            .raw_item
            .iter()
            .map(|(k, v)| (k.as_debuggable_value(), v.as_debuggable_value()))
            .collect();
        f.debug_struct("Value")
            .field("raw_item", &debug_map_with_strings)
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
    pub fn new(raw_item: HashMap<Vec<u8>, Vec<u8>>) -> Self {
        Self { raw_item }
    }
}

impl TryFrom<Value> for HashMap<String, String> {
    type Error = MomentoError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let mut result = HashMap::new();
        for (key, value) in value.raw_item {
            let key = parse_string(key)?;
            let value = parse_string(value)?;
            result.insert(key, value);
        }
        Ok(result)
    }
}

impl From<Value> for HashMap<Vec<u8>, Vec<u8>> {
    fn from(value: Value) -> Self {
        value.raw_item
    }
}

impl TryFrom<DictionaryFetchResponse> for HashMap<Vec<u8>, Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: DictionaryFetchResponse) -> Result<Self, Self::Error> {
        match value {
            DictionaryFetchResponse::Hit { value } => Ok(value.into()),
            DictionaryFetchResponse::Miss => Err(MomentoError::miss("DictionaryFetch")),
        }
    }
}

impl TryFrom<DictionaryFetchResponse> for HashMap<String, String> {
    type Error = MomentoError;

    fn try_from(value: DictionaryFetchResponse) -> Result<Self, Self::Error> {
        match value {
            DictionaryFetchResponse::Hit { value } => value.try_into(),
            DictionaryFetchResponse::Miss => Err(MomentoError::miss("DictionaryFetch")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_fetch_response_display() -> MomentoResult<()> {
        let hit = DictionaryFetchResponse::Hit {
            value: Value::new(HashMap::from([(
                "taco".as_bytes().to_vec(),
                "TACO".as_bytes().to_vec(),
            )])),
        };
        assert_eq!(
            format!("{}", hit),
            r#"Value { raw_item: {"taco": "TACO"} }"#
        );
        assert_eq!(
            format!("{:?}", hit),
            r#"Hit { value: Value { raw_item: {"taco": "TACO"} } }"#
        );
        assert_eq!(
            format!("{:#?}", hit),
            str::trim(
                r#"
Hit {
    value: Value {
        raw_item: {
            "taco": "TACO",
        },
    },
}"#
            )
        );

        let hit_with_binary_value = DictionaryFetchResponse::Hit {
            value: Value::new(HashMap::from([(
                "taco".as_bytes().to_vec(),
                vec![0, 150, 146, 159],
            )])),
        };
        assert_eq!(
            format!("{}", hit_with_binary_value),
            r#"Value { raw_item: {"taco": [0, 150, 146, 159]} }"#
        );
        assert_eq!(
            format!("{:?}", hit_with_binary_value),
            r#"Hit { value: Value { raw_item: {"taco": [0, 150, 146, 159]} } }"#
        );
        assert_eq!(
            format!("{:#?}", hit_with_binary_value),
            str::trim(
                r#"
Hit {
    value: Value {
        raw_item: {
            "taco": [
                0,
                150,
                146,
                159,
            ],
        },
    },
}"#
            )
        );

        let hit_with_binary_key_and_value = DictionaryFetchResponse::Hit {
            value: Value::new(HashMap::from([(
                vec![0, 159, 146, 150],
                vec![0, 150, 146, 159],
            )])),
        };
        assert_eq!(
            format!("{}", hit_with_binary_key_and_value),
            r#"Value { raw_item: {[0, 159, 146, 150]: [0, 150, 146, 159]} }"#
        );
        assert_eq!(
            format!("{:?}", hit_with_binary_key_and_value),
            r#"Hit { value: Value { raw_item: {[0, 159, 146, 150]: [0, 150, 146, 159]} } }"#
        );
        assert_eq!(
            format!("{:#?}", hit_with_binary_key_and_value),
            str::trim(
                r#"
Hit {
    value: Value {
        raw_item: {
            [
                0,
                159,
                146,
                150,
            ]: [
                0,
                150,
                146,
                159,
            ],
        },
    },
}"#
            )
        );

        let miss = DictionaryFetchResponse::Miss;
        assert_eq!(format!("{}", miss), "Miss");
        assert_eq!(format!("{:?}", miss), "Miss");

        Ok(())
    }
}
