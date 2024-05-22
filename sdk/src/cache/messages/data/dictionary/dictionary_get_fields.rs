use super::dictionary_get_field::{DictionaryGetFieldResponse, Value};
use crate::cache::messages::MomentoRequest;
use crate::utils::{parse_string, prep_request_with_timeout};
use crate::{
    CacheClient, IntoBytes, IntoBytesIterable, MomentoError, MomentoErrorCode, MomentoResult,
};
use momento_protos::cache_client::{
    dictionary_get_response::Dictionary as DictionaryProto,
    DictionaryGetRequest as DictionaryGetRequestProto, ECacheResult,
};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

/// Request to get multiple fields from a dictionary.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the dictionary.
/// * `dictionary_name` - The name of the dictionary to get fields from.
/// * `fields` - The fields to get.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use std::collections::HashMap;
/// # use std::convert::TryInto;
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{DictionaryGetFieldsResponse, DictionaryGetFieldsRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let dictionary_name = "dictionary";
/// let fields = vec!["field1", "field2"];
///
/// let set_response = cache_client.dictionary_set_fields(
///   cache_name.to_string(),
///   dictionary_name,
///   vec![("field1", "value1"), ("field2", "value2")]
/// ).await?;
///
/// let get_fields_request = DictionaryGetFieldsRequest::new(
///    cache_name,
///    dictionary_name,
///    fields
/// );
///
/// let get_fields_response = cache_client.send_request(get_fields_request).await?;
///
/// let returned_dictionary: HashMap<String, String> = get_fields_response.try_into()
///   .expect("dictionary should be returned");
/// println!("{:?}", returned_dictionary);
/// # Ok(())
/// # })
/// # }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct DictionaryGetFieldsRequest<D: IntoBytes, F: IntoBytesIterable + Clone> {
    cache_name: String,
    dictionary_name: D,
    fields: F,
}

impl<D: IntoBytes, F: IntoBytesIterable + Clone> DictionaryGetFieldsRequest<D, F> {
    /// Constructs a new DictionaryGetFieldsRequest.
    pub fn new(cache_name: impl Into<String>, dictionary_name: D, fields: F) -> Self {
        Self {
            cache_name: cache_name.into(),
            dictionary_name,
            fields,
        }
    }
}

impl<D: IntoBytes, F: IntoBytesIterable + Clone> MomentoRequest
    for DictionaryGetFieldsRequest<D, F>
{
    type Response = DictionaryGetFieldsResponse<F>;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<Self::Response> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            DictionaryGetRequestProto {
                dictionary_name: self.dictionary_name.into_bytes(),
                fields: self.fields.clone().into_bytes(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .dictionary_get(request)
            .await?
            .into_inner();

        match response.dictionary {
            Some(DictionaryProto::Missing(_)) => Ok(DictionaryGetFieldsResponse::Miss),
            Some(DictionaryProto::Found(elements)) => {
                let responses: Result<Vec<DictionaryGetFieldResponse>, MomentoError> = elements
                    .items
                    .into_iter()
                    .map(|value| match value.result() {
                        ECacheResult::Hit => Ok(DictionaryGetFieldResponse::Hit {
                            value: Value::new(value.cache_body),
                        }),
                        ECacheResult::Miss => Ok(DictionaryGetFieldResponse::Miss),
                        _ => Err(MomentoError::unknown_error(
                            "DictionaryGetFields",
                            Some(format!("{:#?}", value)),
                        )),
                    })
                    .collect();

                match responses {
                    Ok(responses) => Ok(DictionaryGetFieldsResponse::Hit {
                        fields: self.fields,
                        responses,
                    }),
                    Err(e) => Err(e),
                }
            }
            _ => Err(MomentoError::unknown_error(
                "DictionaryGetFields",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response object for a [DictionaryGetFieldsRequest].
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// fn main() -> anyhow::Result<()> {
/// # use std::collections::HashMap;
/// # use momento::cache::DictionaryGetFieldsResponse;
/// # use momento::MomentoResult;
/// # let fetch_response: DictionaryGetFieldsResponse<Vec<String>> = DictionaryGetFieldsResponse::default();
/// use std::convert::TryInto;
/// let item: HashMap<String, String> = match fetch_response {
///   DictionaryGetFieldsResponse::Hit { .. } => fetch_response.try_into().expect("I stored strings!"),
///   DictionaryGetFieldsResponse::Miss => panic!("I expected a hit!"),
/// };
/// # Ok(())
/// }
/// ```
///
/// Or if you're storing raw bytes you can get at them simply:
/// ```
/// # use std::collections::HashMap;
/// # use momento::cache::DictionaryGetFieldsResponse;
/// # use momento::MomentoResult;
/// # let fetch_response: DictionaryGetFieldsResponse<Vec<String>> = DictionaryGetFieldsResponse::default();
/// use std::convert::TryInto;
/// let item: HashMap<Vec<u8>, Vec<u8>> = match fetch_response {
///  DictionaryGetFieldsResponse::Hit { .. } => fetch_response.try_into().expect("I stored raw bytes!"),
/// DictionaryGetFieldsResponse::Miss => panic!("I expected a hit!"),
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
/// # use momento::cache::DictionaryGetFieldsResponse;
/// # use momento::MomentoResult;
/// # let fetch_response: DictionaryGetFieldsResponse<Vec<String>> = DictionaryGetFieldsResponse::default();
/// use std::convert::TryInto;
/// let item: MomentoResult<HashMap<String, String>> = fetch_response.try_into();
/// ```
///
/// You can also go straight into a `HashMap<Vec<u8>, Vec<u8>>` if you prefer:
/// ```
/// # use std::collections::HashMap;
/// # use momento::cache::DictionaryGetFieldsResponse;
/// # use momento::MomentoResult;
/// # let fetch_response: DictionaryGetFieldsResponse<Vec<String>> = DictionaryGetFieldsResponse::default();
/// use std::convert::TryInto;
/// let item: MomentoResult<HashMap<Vec<u8>, Vec<u8>>> = fetch_response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryGetFieldsResponse<F: IntoBytesIterable> {
    /// The dictionary was found.
    Hit {
        /// The fields that were requested.
        fields: F,
        /// The responses for each field.
        responses: Vec<DictionaryGetFieldResponse>,
    },
    /// The dictionary was not found.
    Miss,
}

impl<F: IntoBytes> Default for DictionaryGetFieldsResponse<Vec<F>> {
    fn default() -> Self {
        DictionaryGetFieldsResponse::Hit {
            fields: vec![],
            responses: vec![],
        }
    }
}

impl<F: IntoBytesIterable> TryFrom<DictionaryGetFieldsResponse<F>> for HashMap<String, String> {
    type Error = MomentoError;

    fn try_from(value: DictionaryGetFieldsResponse<F>) -> Result<Self, Self::Error> {
        match value {
            DictionaryGetFieldsResponse::Hit {
                fields, responses, ..
            } => {
                let mut result = HashMap::new();
                for (field, response) in fields.into_bytes().into_iter().zip(responses.into_iter())
                {
                    match response {
                        DictionaryGetFieldResponse::Hit { value } => {
                            let key: String = parse_string(field.into_bytes())?;
                            let value: String = value.try_into()?;
                            result.insert(key, value);
                        }
                        DictionaryGetFieldResponse::Miss => (),
                    }
                }
                Ok(result)
            }
            // In other SDKs we do not convert a `Miss` into an empty HashMap
            DictionaryGetFieldsResponse::Miss => Err(MomentoError::miss("DictionaryGetFields")),
        }
    }
}

impl<F: IntoBytesIterable> TryFrom<DictionaryGetFieldsResponse<F>> for HashMap<Vec<u8>, Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: DictionaryGetFieldsResponse<F>) -> Result<Self, Self::Error> {
        match value {
            DictionaryGetFieldsResponse::Hit {
                fields, responses, ..
            } => {
                let mut result = HashMap::new();
                for (field, response) in fields.into_bytes().into_iter().zip(responses.into_iter())
                {
                    match response {
                        DictionaryGetFieldResponse::Hit { value } => {
                            result.insert(field.into_bytes(), value.into());
                        }
                        DictionaryGetFieldResponse::Miss => (),
                    }
                }
                Ok(result)
            }
            // In other SDKs we do not convert a `Miss` into an empty HashMap
            DictionaryGetFieldsResponse::Miss => Err(MomentoError {
                message: "dictionary get fields response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}
