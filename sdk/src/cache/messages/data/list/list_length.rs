use std::convert::TryFrom;

use momento_protos::cache_client::list_length_response;

use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoError,
    MomentoResult,
};

/// Gets the number of elements in the given list.
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
/// use momento::cache::{ListLengthResponse, ListLengthRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let list_name = "list-name";
/// # cache_client.list_concatenate_front(&cache_name, list_name, vec!["value1", "value2"]).await;
///
/// let length_request = ListLengthRequest::new(cache_name, list_name);
/// let length: u32 = cache_client.send_request(length_request).await?.try_into().expect("Expected a list length!");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListLengthRequest<L: IntoBytes> {
    cache_name: String,
    list_name: L,
}

impl<L: IntoBytes> ListLengthRequest<L> {
    /// Constructs a new ListLengthRequest.
    pub fn new(cache_name: impl Into<String>, list_name: L) -> Self {
        Self {
            cache_name: cache_name.into(),
            list_name,
        }
    }
}

impl<L: IntoBytes> MomentoRequest for ListLengthRequest<L> {
    type Response = ListLengthResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ListLengthResponse> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.deadline_millis(),
            momento_protos::cache_client::ListLengthRequest {
                list_name: self.list_name.into_bytes(),
            },
        )?;

        let response = cache_client
            .next_data_client()
            .list_length(request)
            .await?
            .into_inner();

        match response.list {
            Some(list_length_response::List::Missing(_)) => Ok(ListLengthResponse::Miss),
            Some(list_length_response::List::Found(found)) => Ok(ListLengthResponse::Hit {
                length: found.length,
            }),
            _ => Err(MomentoError::unknown_error(
                "ListLength",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response for a list length operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::ListLengthResponse;
/// use std::convert::TryInto;
/// # let response = ListLengthResponse::Hit { length: 5 };
/// let length: u32 = match response {
///     ListLengthResponse::Hit { length } => length.try_into().expect("Expected a list length!"),
///     ListLengthResponse::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<u32, MomentoError> suitable for
/// ?-propagation if you know you are expecting a ListLengthResponse::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::ListLengthResponse;
/// use std::convert::TryInto;
/// # let response = ListLengthResponse::Hit { length: 5 };
/// let length: MomentoResult<u32> = response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum ListLengthResponse {
    /// The list was found.
    Hit {
        /// The length of the list.
        length: u32,
    },
    /// The list was not found.
    Miss,
}

impl TryFrom<ListLengthResponse> for u32 {
    type Error = MomentoError;

    fn try_from(value: ListLengthResponse) -> Result<Self, Self::Error> {
        match value {
            ListLengthResponse::Hit { length } => Ok(length),
            ListLengthResponse::Miss => Err(MomentoError::miss("ListLength")),
        }
    }
}
