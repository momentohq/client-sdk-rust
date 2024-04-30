use std::convert::{TryFrom, TryInto};

use momento_protos::cache_client::list_pop_back_response;

use crate::{
    cache::MomentoRequest,
    utils::{parse_string, prep_request_with_timeout},
    CacheClient, IntoBytes, MomentoError, MomentoErrorCode, MomentoResult,
};

/// Remove and return the last element from a list item.
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
/// use momento::cache::{ListPopBack, ListPopBackRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let list_name = "list-name";
/// # cache_client.list_concatenate_front(&cache_name, list_name, vec!["value1", "value2"]).await;
///
/// let pop_back_request = ListPopBackRequest::new(cache_name, list_name);
/// let popped_value: String = cache_client.send_request(pop_back_request).await?.try_into().expect("Expected a popped list value!");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListPopBackRequest<L: IntoBytes> {
    cache_name: String,
    list_name: L,
}

impl<L: IntoBytes> ListPopBackRequest<L> {
    pub fn new(cache_name: impl Into<String>, list_name: L) -> Self {
        Self {
            cache_name: cache_name.into(),
            list_name,
        }
    }
}

impl<L: IntoBytes> MomentoRequest for ListPopBackRequest<L> {
    type Response = ListPopBack;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ListPopBack> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::ListPopBackRequest {
                list_name: self.list_name.into_bytes(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .list_pop_back(request)
            .await?
            .into_inner();

        match response.list {
            Some(list_pop_back_response::List::Missing(_)) => Ok(ListPopBack::Miss),
            Some(list_pop_back_response::List::Found(found)) => Ok(ListPopBack::Hit {
                value: ListPopBackValue::new(found.back),
            }),
            _ => unreachable!(),
        }
    }
}

/// Response for a list pop back operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// # use momento::cache::ListPopBackValue;
/// use momento::cache::ListPopBack;
/// use std::convert::TryInto;
/// # let response = ListPopBack::Hit { value: ListPopBackValue::new("hi".into()) };
/// let popped_value: String = match response {
///     ListPopBack::Hit { value } => value.try_into().expect("Expected a popped list value!"),
///     ListPopBack::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<u32, MomentoError> suitable for
/// ?-propagation if you know you are expecting a ListPopBack::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// # use momento::cache::ListPopBackValue;
/// use momento::cache::ListPopBack;
/// use std::convert::TryInto;
/// # let response = ListPopBack::Hit { value: ListPopBackValue::new("hi".into()) };
/// let popped_value: MomentoResult<String> = response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum ListPopBack {
    Hit { value: ListPopBackValue },
    Miss,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ListPopBackValue {
    pub(crate) raw_item: Vec<u8>,
}

impl ListPopBackValue {
    pub fn new(raw_item: Vec<u8>) -> Self {
        Self { raw_item }
    }
}

impl TryFrom<ListPopBackValue> for Vec<u8> {
    type Error = MomentoError;

    fn try_from(value: ListPopBackValue) -> Result<Self, Self::Error> {
        Ok(value.raw_item)
    }
}

impl TryFrom<ListPopBackValue> for String {
    type Error = MomentoError;

    fn try_from(value: ListPopBackValue) -> Result<Self, Self::Error> {
        Ok(parse_string(value.raw_item).expect("expected a valid UTF-8 string"))
    }
}

impl TryFrom<ListPopBack> for Vec<u8> {
    type Error = MomentoError;

    fn try_from(value: ListPopBack) -> Result<Self, Self::Error> {
        match value {
            ListPopBack::Hit { value } => Ok(value.try_into()?),
            ListPopBack::Miss => Err(MomentoError {
                message: "list length response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<ListPopBack> for String {
    type Error = MomentoError;

    fn try_from(value: ListPopBack) -> Result<Self, Self::Error> {
        match value {
            ListPopBack::Hit { value } => Ok(value.try_into()?),
            ListPopBack::Miss => Err(MomentoError {
                message: "list length response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}
