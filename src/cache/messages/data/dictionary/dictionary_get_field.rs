use crate::cache::messages::MomentoRequest;
use crate::utils::{parse_string, prep_request_with_timeout};
use crate::{CacheClient, IntoBytes, MomentoError, MomentoResult};
use momento_protos::cache_client::dictionary_get_response::DictionaryGetResponsePart;
use momento_protos::cache_client::{
    dictionary_get_response::Dictionary as DictionaryProto,
    DictionaryGetRequest as DictionaryGetRequestProto, ECacheResult,
};
use std::convert::{TryFrom, TryInto};

/// Request to get a field from a dictionary.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the dictionary.
/// * `dictionary_name` - The name of the dictionary to get fields from.
/// * `field` - The field to get.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use std::collections::HashMap;
/// # use std::convert::TryInto;
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{DictionaryGetFieldResponse, DictionaryGetFieldRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let dictionary_name = "dictionary";
/// let field = "field1";
///
/// let set_response = cache_client.dictionary_set_fields(
///   cache_name.to_string(),
///   dictionary_name,
///   vec![("field1", "value1"), ("field2", "value2")]
/// ).await?;
///
/// let get_field_request = DictionaryGetFieldRequest::new(
///    cache_name,
///    dictionary_name,
///    field
/// );
///
/// let get_field_response = cache_client.send_request(get_field_request).await?;
///
/// let returned_value: String = get_field_response.try_into()
///  .expect("value should be returned");
/// println!("{:?}", returned_value);
/// # Ok(())
/// # })
/// # }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct DictionaryGetFieldRequest<D: IntoBytes, F: IntoBytes> {
    cache_name: String,
    dictionary_name: D,
    field: F,
}

impl<D: IntoBytes, F: IntoBytes> DictionaryGetFieldRequest<D, F> {
    pub fn new(cache_name: impl Into<String>, dictionary_name: D, field: F) -> Self {
        Self {
            cache_name: cache_name.into(),
            dictionary_name,
            field,
        }
    }
}

impl<D: IntoBytes, F: IntoBytes> MomentoRequest for DictionaryGetFieldRequest<D, F> {
    type Response = DictionaryGetFieldResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<Self::Response> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            DictionaryGetRequestProto {
                dictionary_name: self.dictionary_name.into_bytes(),
                fields: vec![self.field.into_bytes()],
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .dictionary_get(request)
            .await?
            .into_inner();

        match response.dictionary {
            Some(DictionaryProto::Missing(_)) => Ok(DictionaryGetFieldResponse::Miss),
            Some(DictionaryProto::Found(elements)) => {
                let mut responses: Vec<DictionaryGetResponsePart> =
                    elements.items.into_iter().collect();

                match responses.pop() {
                    Some(value) => match value.result() {
                        ECacheResult::Hit => Ok(DictionaryGetFieldResponse::Hit {
                            value: DictionaryGetFieldValue::new(value.cache_body),
                        }),
                        ECacheResult::Miss => Ok(DictionaryGetFieldResponse::Miss),
                        _ => Err(MomentoError::unknown_error(
                            "DictionaryGetField",
                            Some(format!("{:#?}", value)),
                        )),
                    },
                    None => Err(MomentoError::unknown_error(
                        "DictionaryGetField",
                        Some("Expected to receive one element".to_string()),
                    )),
                }
            }
            _ => Err(MomentoError::unknown_error(
                "DictionaryGetField",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response object for a [DictionaryGetFieldRequest](crate::cache::DictionaryGetFieldRequest).
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// fn main() -> anyhow::Result<()> {
/// # use momento::cache::DictionaryGetFieldResponse;
/// # use momento::MomentoResult;
/// # let fetch_response: DictionaryGetFieldResponse = DictionaryGetFieldResponse::default();
/// use std::convert::TryInto;
/// let item: String = match fetch_response {
///    DictionaryGetFieldResponse::Hit { value } => value.try_into().expect("I stored a string!"),
///    DictionaryGetFieldResponse::Miss => panic!("I expected a hit!"),
/// };
/// # Ok(())
/// }
/// ```
///
/// Or if you're storing raw bytes you can get at them simply:
/// ```
/// # use momento::cache::DictionaryGetFieldResponse;
/// # use momento::MomentoResult;
/// # let fetch_response: DictionaryGetFieldResponse = DictionaryGetFieldResponse::default();
/// use std::convert::TryInto;
/// let item: Vec<u8> = match fetch_response {
///   DictionaryGetFieldResponse::Hit { value } => value.into(),
///   DictionaryGetFieldResponse::Miss => panic!("I expected a hit!"),
/// };
/// ```
///
/// You can cast your result directly into a `Result<String, MomentoError>`` suitable for
/// ?-propagation if you know you are expecting a String value.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::cache::DictionaryGetFieldResponse;
/// # use momento::MomentoResult;
/// # let fetch_response = DictionaryGetFieldResponse::default();
/// use std::convert::TryInto;
/// let item: MomentoResult<String> = fetch_response.try_into();
/// ```
///
/// You can also go straight into a `Vec<u8>` if you prefer:
/// ```
/// # use momento::cache::DictionaryGetFieldResponse;
/// # use momento::MomentoResult;
/// # let fetch_response: DictionaryGetFieldResponse = DictionaryGetFieldResponse::default();
/// use std::convert::TryInto;
/// let item: MomentoResult<Vec<u8>> = fetch_response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryGetFieldResponse {
    Hit { value: DictionaryGetFieldValue },
    Miss,
}

impl Default for DictionaryGetFieldResponse {
    fn default() -> Self {
        DictionaryGetFieldResponse::Hit {
            value: DictionaryGetFieldValue::new(Vec::new()),
        }
    }
}

impl TryFrom<DictionaryGetFieldResponse> for String {
    type Error = MomentoError;

    fn try_from(value: DictionaryGetFieldResponse) -> Result<Self, Self::Error> {
        match value {
            DictionaryGetFieldResponse::Hit { value } => value.try_into(),
            DictionaryGetFieldResponse::Miss => Err(MomentoError::miss("DictionaryGetField")),
        }
    }
}

impl TryFrom<DictionaryGetFieldResponse> for Vec<u8> {
    type Error = MomentoError;

    fn try_from(value: DictionaryGetFieldResponse) -> Result<Self, Self::Error> {
        match value {
            DictionaryGetFieldResponse::Hit { value } => Ok(value.into()),
            DictionaryGetFieldResponse::Miss => Err(MomentoError::miss("DictionaryGetField")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DictionaryGetFieldValue {
    pub(crate) raw_item: Vec<u8>,
}

impl DictionaryGetFieldValue {
    pub fn new(raw_item: Vec<u8>) -> Self {
        Self { raw_item }
    }
}

impl TryFrom<DictionaryGetFieldValue> for String {
    type Error = MomentoError;

    fn try_from(value: DictionaryGetFieldValue) -> Result<Self, Self::Error> {
        parse_string(value.raw_item)
    }
}

impl From<DictionaryGetFieldValue> for Vec<u8> {
    fn from(value: DictionaryGetFieldValue) -> Self {
        value.raw_item
    }
}
