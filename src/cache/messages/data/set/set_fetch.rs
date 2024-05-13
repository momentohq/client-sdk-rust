use std::convert::{TryFrom, TryInto};

use momento_protos::cache_client::set_fetch_response;

use crate::{
    cache::MomentoRequest,
    utils::{parse_string, prep_request_with_timeout},
    CacheClient, IntoBytes, IntoBytesIterable, MomentoError, MomentoResult,
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
pub struct SetFetchRequest<S: IntoBytes> {
    cache_name: String,
    set_name: S,
}

impl<S: IntoBytes> SetFetchRequest<S> {
    pub fn new(cache_name: impl Into<String>, set_name: S) -> Self {
        Self {
            cache_name: cache_name.into(),
            set_name,
        }
    }
}

impl<S: IntoBytes> MomentoRequest for SetFetchRequest<S> {
    type Response = SetFetchResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SetFetchResponse> {
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
            Some(set_fetch_response::Set::Missing(_)) => Ok(SetFetchResponse::Miss),
            Some(set_fetch_response::Set::Found(found)) => Ok(SetFetchResponse::Hit {
                values: Value {
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

/// Response for a set fetch operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SetFetchResponse;
/// use std::convert::TryInto;
/// # let response = SetFetchResponse::from(vec!["abc", "123"]);
/// let fetched_values: Vec<String> = match response {
///     SetFetchResponse::Hit { values } => values.try_into().expect("Expected to fetch a set!"),
///     SetFetchResponse::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a `Result<Vec<String>, MomentoError>` suitable for
/// ?-propagation if you know you are expecting a ListFetch::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SetFetchResponse;
/// use std::convert::TryInto;
/// # let response = SetFetchResponse::from(vec!["abc", "123"]);
/// let fetched_values: Vec<String> = response.try_into().expect("Expected to fetch a set!");
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum SetFetchResponse {
    Hit { values: Value },
    Miss,
}

impl<I: IntoBytesIterable> From<I> for SetFetchResponse {
    fn from(values: I) -> Self {
        SetFetchResponse::Hit {
            values: Value::new(values.into_bytes()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Value {
    pub(crate) raw_item: Vec<Vec<u8>>,
}

impl Value {
    pub fn new(raw_item: Vec<Vec<u8>>) -> Self {
        Self { raw_item }
    }
}

impl From<Value> for Vec<Vec<u8>> {
    fn from(value: Value) -> Self {
        value.raw_item
    }
}

impl TryFrom<Value> for Vec<String> {
    type Error = MomentoError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(value
            .raw_item
            .into_iter()
            .map(parse_string)
            .collect())
    }
}

impl TryFrom<SetFetchResponse> for Vec<Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: SetFetchResponse) -> Result<Self, Self::Error> {
        match value {
            SetFetchResponse::Hit { values } => Ok(values.into()),
            SetFetchResponse::Miss => Err(MomentoError::miss("SetFetch")),
        }
    }
}

impl TryFrom<SetFetchResponse> for Vec<String> {
    type Error = MomentoError;

    fn try_from(value: SetFetchResponse) -> Result<Self, Self::Error> {
        match value {
            SetFetchResponse::Hit { values } => Ok(values.try_into()?),
            SetFetchResponse::Miss => Err(MomentoError::miss("SetFetch")),
        }
    }
}
