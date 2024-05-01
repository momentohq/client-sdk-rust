use super::dictionary_get_field::{DictionaryGetField, DictionaryGetFieldValue};
use crate::cache::requests::MomentoRequest;
use crate::utils::{parse_string, prep_request_with_timeout};
use crate::{CacheClient, IntoBytes, MomentoError, MomentoErrorCode, MomentoResult};
use momento_protos::cache_client::{
    dictionary_get_response::Dictionary as DictionaryProto,
    DictionaryGetRequest as DictionaryGetRequestProto, ECacheResult,
};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

pub struct DictionaryGetFieldsRequest<D: IntoBytes, F: IntoBytes + Clone> {
    cache_name: String,
    dictionary_name: D,
    fields: Vec<F>,
}

impl<D: IntoBytes, F: IntoBytes + Clone> DictionaryGetFieldsRequest<D, F> {
    pub fn new(cache_name: impl Into<String>, dictionary_name: D, fields: Vec<F>) -> Self {
        Self {
            cache_name: cache_name.into(),
            dictionary_name,
            fields,
        }
    }
}

impl<D: IntoBytes, F: IntoBytes + Clone> MomentoRequest for DictionaryGetFieldsRequest<D, F> {
    type Response = DictionaryGetFields<F>;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<Self::Response> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            DictionaryGetRequestProto {
                dictionary_name: self.dictionary_name.into_bytes(),
                fields: self
                    .fields
                    .clone()
                    .into_iter()
                    .map(|field| field.into_bytes())
                    .collect(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .dictionary_get(request)
            .await?
            .into_inner();

        match response.dictionary {
            Some(DictionaryProto::Missing(_)) => Ok(DictionaryGetFields::Miss),
            Some(DictionaryProto::Found(elements)) => {
                let responses: Result<Vec<DictionaryGetField>, MomentoError> = elements
                    .items
                    .into_iter()
                    .map(|value| match value.result() {
                        ECacheResult::Hit => Ok(DictionaryGetField::Hit {
                            value: DictionaryGetFieldValue::new(value.cache_body),
                        }),
                        ECacheResult::Miss => Ok(DictionaryGetField::Miss),
                        _ => Err(MomentoError::unknown_error(
                            "DictionaryGetFields",
                            Some(format!("{:#?}", value)),
                        )),
                    })
                    .collect();

                match responses {
                    Ok(responses) => Ok(DictionaryGetFields::Hit {
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

/// Response to a dictionary fetch request.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// fn main() -> anyhow::Result<()> {
/// # use std::collections::HashMap;
/// # use momento::cache::{DictionaryFetch, DictionaryFetchValue};
/// # use momento::MomentoResult;
/// # let fetch_response = DictionaryFetch::Hit { value: DictionaryFetchValue::default() };
/// use std::convert::TryInto;
/// let item: HashMap<String, String> = match fetch_response {
///    DictionaryFetch::Hit { value } => value.try_into().expect("I stored strings!"),
///   DictionaryFetch::Miss => panic!("I expected a hit!"),
/// };
/// # Ok(())
/// }
/// ```
///
/// Or, if you're storing raw bytes you can get at them simply:
/// ```
/// # use std::collections::HashMap;
/// # use momento::cache::{DictionaryFetch, DictionaryFetchValue};
/// # use momento::MomentoResult;
/// # let fetch_response = DictionaryFetch::Hit { value: DictionaryFetchValue::default() };
/// use std::convert::TryInto;
/// let item: HashMap<Vec<u8>, Vec<u8>> = match fetch_response {
///   DictionaryFetch::Hit { value } => value.into(),
///   DictionaryFetch::Miss => panic!("I expected a hit!"),
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
/// # use momento::cache::{DictionaryFetch, DictionaryFetchValue};
/// # use momento::MomentoResult;
/// # let fetch_response = DictionaryFetch::Hit { value: DictionaryFetchValue::default() };
/// use std::convert::TryInto;
/// let item: MomentoResult<HashMap<String, String>> = fetch_response.try_into();
/// ```
///
/// You can also go straight into a `HashMap<Vec<u8>, Vec<u8>>` if you prefer:
/// ```
/// # use std::collections::HashMap;
/// # use momento::cache::{DictionaryFetch, DictionaryFetchValue};
/// # use momento::MomentoResult;
/// # let fetch_response = DictionaryFetch::Hit { value: DictionaryFetchValue::default() };
/// use std::convert::TryInto;
/// let item: MomentoResult<HashMap<Vec<u8>, Vec<u8>>> = fetch_response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryGetFields<F: IntoBytes> {
    Hit {
        fields: Vec<F>,
        responses: Vec<DictionaryGetField>,
    },
    Miss,
}

impl<F: IntoBytes> TryFrom<DictionaryGetFields<F>> for HashMap<String, String> {
    type Error = MomentoError;

    fn try_from(value: DictionaryGetFields<F>) -> Result<Self, Self::Error> {
        match value {
            DictionaryGetFields::Hit {
                fields, responses, ..
            } => {
                let mut result = HashMap::new();
                for (field, response) in fields.into_iter().zip(responses.into_iter()) {
                    match response {
                        DictionaryGetField::Hit { value } => {
                            let key: String = parse_string(field.into_bytes())?;
                            let value: String = value.try_into()?;
                            result.insert(key, value);
                        }
                        DictionaryGetField::Miss => (),
                    }
                }
                Ok(result)
            }
            // In other SDKs we do not convert a `Miss` into an empty HashMap
            DictionaryGetFields::Miss => Err(MomentoError {
                message: "dictionary get fields response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl<F: IntoBytes> TryFrom<DictionaryGetFields<F>> for HashMap<Vec<u8>, Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: DictionaryGetFields<F>) -> Result<Self, Self::Error> {
        match value {
            DictionaryGetFields::Hit {
                fields, responses, ..
            } => {
                let mut result = HashMap::new();
                for (field, response) in fields.into_iter().zip(responses.into_iter()) {
                    match response {
                        DictionaryGetField::Hit { value } => {
                            result.insert(field.into_bytes(), value.into());
                        }
                        DictionaryGetField::Miss => (),
                    }
                }
                Ok(result)
            }
            // In other SDKs we do not convert a `Miss` into an empty HashMap
            DictionaryGetFields::Miss => Err(MomentoError {
                message: "dictionary get fields response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}
