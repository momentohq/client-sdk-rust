use std::convert::{TryFrom, TryInto};

use momento_protos::cache_client::set_fetch_response;

use crate::{
    cache::MomentoRequest,
    utils::{parse_string, prep_request_with_timeout},
    CacheClient, IntoBytes, MomentoError, MomentoErrorCode, MomentoResult,
};

/// Fetch the elements in the given set.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the set.
/// * `set_name` - The name of the set to fetch.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento::MomentoResult;
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::SetFetchRequest;
/// use std::convert::TryInto;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let set_name = "set";
///
/// # let add_elements_response = cache_client.set_add_elements(&cache_name, set_name, vec!["value1", "value2"]).await?;
///
/// let request = SetFetchRequest::new(cache_name, set_name);
///
/// let fetched_elements: Vec<String> = cache_client.send_request(request).await?.try_into().expect("Expected a set!");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SetFetchRequest<L: IntoBytes> {
    cache_name: String,
    set_name: L,
}

impl<L: IntoBytes> SetFetchRequest<L> {
    pub fn new(cache_name: impl Into<String>, set_name: L) -> Self {
        Self {
            cache_name: cache_name.into(),
            set_name,
        }
    }
}

impl<L: IntoBytes> MomentoRequest for SetFetchRequest<L> {
    type Response = SetFetch;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SetFetch> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::SetFetchRequest {
                set_name: self.set_name.into_bytes(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .set_fetch(request)
            .await?
            .into_inner();

        match response.set {
            Some(set_fetch_response::Set::Missing(_)) => Ok(SetFetch::Miss),
            Some(set_fetch_response::Set::Found(found)) => Ok(SetFetch::Hit {
                values: SetFetchValue {
                    raw_item: found.elements,
                },
            }),
            _ => Err(MomentoError::unknown_error(
                "SetFetch",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// TODO
#[derive(Debug, PartialEq, Eq)]
pub enum SetFetch {
    Hit { values: SetFetchValue },
    Miss,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SetFetchValue {
    pub(crate) raw_item: Vec<Vec<u8>>,
}

impl SetFetchValue {
    pub fn new(raw_item: Vec<Vec<u8>>) -> Self {
        Self { raw_item }
    }
}

impl TryFrom<SetFetchValue> for Vec<Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: SetFetchValue) -> Result<Self, Self::Error> {
        Ok(value.raw_item)
    }
}

impl TryFrom<SetFetchValue> for Vec<String> {
    type Error = MomentoError;

    fn try_from(value: SetFetchValue) -> Result<Self, Self::Error> {
        Ok(value
            .raw_item
            .into_iter()
            .map(|v| parse_string(v).expect("expected a valid UTF-8 string"))
            .collect())
    }
}

impl TryFrom<SetFetch> for Vec<Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: SetFetch) -> Result<Self, Self::Error> {
        match value {
            SetFetch::Hit { values } => Ok(values.try_into()?),
            // TODO: user MomentoError::miss helper
            SetFetch::Miss => Err(MomentoError {
                message: "set response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<SetFetch> for Vec<String> {
    type Error = MomentoError;

    fn try_from(value: SetFetch) -> Result<Self, Self::Error> {
        match value {
            SetFetch::Hit { values } => Ok(values.try_into()?),
            // TODO: user MomentoError::miss helper
            SetFetch::Miss => Err(MomentoError {
                message: "set response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl From<Vec<String>> for SetFetch {
    fn from(values: Vec<String>) -> Self {
        SetFetch::Hit {
            values: SetFetchValue::new(values.into_iter().map(|v| v.as_bytes().to_vec()).collect()),
        }
    }
}
