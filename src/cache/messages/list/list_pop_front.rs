use std::convert::{TryFrom, TryInto};

use momento_protos::cache_client::list_pop_front_response;

use crate::{
    cache::MomentoRequest,
    utils::{parse_string, prep_request_with_timeout},
    CacheClient, IntoBytes, MomentoError, MomentoResult,
};

/// Remove and return the first element from a list item.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `list_name` - name of the list
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::cache::{ListPopFrontResponse, ListPopFrontRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let list_name = "list-name";
/// # cache_client.list_concatenate_front(&cache_name, list_name, vec!["value1", "value2"]).await;
///
/// let pop_front_request = ListPopFrontRequest::new(cache_name, list_name);
/// let popped_value: String = cache_client.send_request(pop_front_request).await?.try_into().expect("Expected a popped list value!");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListPopFrontRequest<L: IntoBytes> {
    cache_name: String,
    list_name: L,
}

impl<L: IntoBytes> ListPopFrontRequest<L> {
    pub fn new(cache_name: impl Into<String>, list_name: L) -> Self {
        Self {
            cache_name: cache_name.into(),
            list_name,
        }
    }
}

impl<L: IntoBytes> MomentoRequest for ListPopFrontRequest<L> {
    type Response = ListPopFrontResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ListPopFrontResponse> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::ListPopFrontRequest {
                list_name: self.list_name.into_bytes(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .list_pop_front(request)
            .await?
            .into_inner();

        match response.list {
            Some(list_pop_front_response::List::Missing(_)) => Ok(ListPopFrontResponse::Miss),
            Some(list_pop_front_response::List::Found(found)) => Ok(ListPopFrontResponse::Hit {
                value: ListPopFrontValue::new(found.front),
            }),
            _ => Err(MomentoError::unknown_error(
                "ListPopFront",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response for a list pop front operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// # use momento::cache::ListPopFrontValue;
/// use momento::cache::ListPopFrontResponse;
/// use std::convert::TryInto;
/// # let response = ListPopFrontResponse::Hit { value: ListPopFrontValue::new("hi".into()) };
/// let popped_value: String = match response {
///     ListPopFrontResponse::Hit { value } => value.try_into().expect("Expected a valid UTF-8 string"),
///     ListPopFrontResponse::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<u32, MomentoError> suitable for
/// ?-propagation if you know you are expecting a ListPopFrontResponse::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// # use momento::cache::ListPopFrontValue;
/// use momento::cache::ListPopFrontResponse;
/// use std::convert::TryInto;
/// # let response = ListPopFrontResponse::Hit { value: ListPopFrontValue::new("hi".into()) };
/// let popped_value: MomentoResult<String> = response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum ListPopFrontResponse {
    Hit { value: ListPopFrontValue },
    Miss,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ListPopFrontValue {
    pub(crate) raw_item: Vec<u8>,
}

impl ListPopFrontValue {
    pub fn new(raw_item: Vec<u8>) -> Self {
        Self { raw_item }
    }
}

impl TryFrom<ListPopFrontValue> for Vec<u8> {
    type Error = MomentoError;

    fn try_from(value: ListPopFrontValue) -> Result<Self, Self::Error> {
        Ok(value.raw_item)
    }
}

impl TryFrom<ListPopFrontValue> for String {
    type Error = MomentoError;

    fn try_from(value: ListPopFrontValue) -> Result<Self, Self::Error> {
        Ok(parse_string(value.raw_item).expect("expected a valid UTF-8 string"))
    }
}

impl TryFrom<ListPopFrontResponse> for Vec<u8> {
    type Error = MomentoError;

    fn try_from(value: ListPopFrontResponse) -> Result<Self, Self::Error> {
        match value {
            ListPopFrontResponse::Hit { value } => Ok(value.try_into()?),
            ListPopFrontResponse::Miss => Err(MomentoError::miss("ListPopFront")),
        }
    }
}

impl TryFrom<ListPopFrontResponse> for String {
    type Error = MomentoError;

    fn try_from(value: ListPopFrontResponse) -> Result<Self, Self::Error> {
        match value {
            ListPopFrontResponse::Hit { value } => Ok(value.try_into()?),
            ListPopFrontResponse::Miss => Err(MomentoError::miss("ListPopFront")),
        }
    }
}
