use std::convert::{TryFrom, TryInto};

use momento_protos::{
    cache_client::{
        list_fetch_request::{EndIndex, StartIndex},
        list_fetch_response,
    },
    common::Unbounded,
};

use crate::{
    cache::MomentoRequest,
    utils::{parse_string, prep_request_with_timeout},
    CacheClient, IntoBytes, IntoBytesIterable, MomentoError, MomentoResult,
};

/// Gets a list item from a cache with optional slices.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `list_name` - name of the list
///
/// # Optional Arguments
///
/// * `start_index` - The starting inclusive element of the list to fetch. Default is 0.
/// * `end_index` - The ending exclusive element of the list to fetch. Default is up to and including end of list.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::cache::{ListFetchResponse, ListFetchRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let list_name = "list-name";
/// # cache_client.list_concatenate_front(&cache_name, list_name, vec!["value1", "value2"]).await;
///
/// let fetch_request = ListFetchRequest::new(cache_name, list_name).start_index(1).end_index(3);
/// let fetched_values: Vec<String> = cache_client.send_request(fetch_request).await?.try_into().expect("Expected a list fetch!");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListFetchRequest<L: IntoBytes> {
    cache_name: String,
    list_name: L,
    start_index: Option<i32>,
    end_index: Option<i32>,
}

impl<L: IntoBytes> ListFetchRequest<L> {
    /// Constructs a new ListFetchRequest.
    pub fn new(cache_name: impl Into<String>, list_name: L) -> Self {
        Self {
            cache_name: cache_name.into(),
            list_name,
            start_index: None,
            end_index: None,
        }
    }

    /// Set the starting inclusive element of the list to fetch.
    pub fn start_index(mut self, start_index: impl Into<Option<i32>>) -> Self {
        self.start_index = start_index.into();
        self
    }

    /// Set the ending exclusive element of the list to fetch.
    pub fn end_index(mut self, end_index: impl Into<Option<i32>>) -> Self {
        self.end_index = end_index.into();
        self
    }
}

impl<L: IntoBytes> MomentoRequest for ListFetchRequest<L> {
    type Response = ListFetchResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ListFetchResponse> {
        let start_index = match self.start_index {
            Some(start) => Some(StartIndex::InclusiveStart(start)),
            None => Some(StartIndex::UnboundedStart(Unbounded {})),
        };
        let end_index = match self.end_index {
            Some(end) => Some(EndIndex::ExclusiveEnd(end)),
            None => Some(EndIndex::UnboundedEnd(Unbounded {})),
        };
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::ListFetchRequest {
                list_name: self.list_name.into_bytes(),
                start_index,
                end_index,
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .list_fetch(request)
            .await?
            .into_inner();

        match response.list {
            Some(list_fetch_response::List::Missing(_)) => Ok(ListFetchResponse::Miss),
            Some(list_fetch_response::List::Found(found)) => Ok(ListFetchResponse::Hit {
                values: Value {
                    raw_item: found.values,
                },
            }),
            _ => Err(MomentoError::unknown_error(
                "ListFetch",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response for a list fetch operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::ListFetchResponse;
/// use std::convert::TryInto;
/// # let response: ListFetchResponse = vec!["abc", "123"].into();
/// let fetched_values: Vec<String> = match response {
///     ListFetchResponse::Hit { values } => values.try_into().expect("Expected to fetch a list of strings!"),
///     ListFetchResponse::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a `Result<Vec<String>, MomentoError>` suitable for
/// ?-propagation if you know you are expecting a ListFetchResponse::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::ListFetchResponse;
/// use std::convert::TryInto;
/// # let response: ListFetchResponse = vec!["abc", "123"].into();
/// let fetched_values: Vec<String> = response.try_into().expect("Expected to fetch a list of strings!");
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum ListFetchResponse {
    /// The list was found.
    Hit {
        /// The list values.
        values: Value,
    },
    /// The list was not found.
    Miss,
}

impl<I: IntoBytesIterable> From<I> for ListFetchResponse {
    fn from(values: I) -> Self {
        ListFetchResponse::Hit {
            values: Value::new(values.into_bytes()),
        }
    }
}

/// Represents the values of a list fetch operation.
#[derive(Debug, PartialEq, Eq)]
pub struct Value {
    /// The raw list values.
    pub(crate) raw_item: Vec<Vec<u8>>,
}

impl Value {
    /// Constructs a new Value.
    pub fn new(raw_item: Vec<Vec<u8>>) -> Self {
        Self { raw_item }
    }
}

impl TryFrom<Value> for Vec<Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(value.raw_item)
    }
}

impl TryFrom<Value> for Vec<String> {
    type Error = MomentoError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.raw_item.into_iter().map(parse_string).collect()
    }
}

impl TryFrom<ListFetchResponse> for Vec<Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: ListFetchResponse) -> Result<Self, Self::Error> {
        match value {
            ListFetchResponse::Hit { values } => Ok(values.try_into()?),
            ListFetchResponse::Miss => Err(MomentoError::miss("ListFetch")),
        }
    }
}

impl TryFrom<ListFetchResponse> for Vec<String> {
    type Error = MomentoError;

    fn try_from(value: ListFetchResponse) -> Result<Self, Self::Error> {
        match value {
            ListFetchResponse::Hit { values } => Ok(values.try_into()?),
            ListFetchResponse::Miss => Err(MomentoError::miss("ListFetch")),
        }
    }
}
