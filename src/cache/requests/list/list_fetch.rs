use std::convert::TryFrom;

use momento_protos::{
    cache_client::{
        list_fetch_request::{EndIndex, StartIndex},
        list_fetch_response,
    },
    common::Unbounded,
};

use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoError,
    MomentoErrorCode, MomentoResult,
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
/// * `end_index` - The ending exclusive element of the list to fetch. Default is end of list.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::cache::{ListFetch, ListFetchRequest};
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
pub struct ListFetchRequest<K: IntoBytes> {
    cache_name: String,
    list_name: K,
    start_index: Option<i32>,
    end_index: Option<i32>,
}

impl<K: IntoBytes> ListFetchRequest<K> {
    pub fn new(cache_name: impl Into<String>, list_name: K) -> Self {
        Self {
            cache_name: cache_name.into(),
            list_name,
            start_index: None,
            end_index: None,
        }
    }

    /// Set the starting inclusive element of the list to fetch.
    pub fn start_index(mut self, start_index: i32) -> Self {
        self.start_index = Some(start_index);
        self
    }

    /// Set the ending exclusive element of the list to fetch.
    pub fn end_index(mut self, end_index: i32) -> Self {
        self.end_index = Some(end_index);
        self
    }
}

impl<K: IntoBytes> MomentoRequest for ListFetchRequest<K> {
    type Response = ListFetch;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ListFetch> {
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
            Some(list_fetch_response::List::Missing(_)) => Ok(ListFetch::Miss),
            Some(list_fetch_response::List::Found(found)) => Ok(ListFetch::Hit {
                values: found.values,
            }),
            _ => unreachable!(),
        }
    }
}

/// Response for a list fetch operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// # use momento::cache::ListFetch;
/// use std::convert::TryInto;
/// # let values = vec!["abc", "123"].iter().map(|s| s.as_bytes().to_vec()).collect();
/// # let response = ListFetch::Hit { values: values };
/// let fetched_values: Vec<String> = match response {
///     ListFetch::Hit { values } => values.into_iter().map(|v| String::from_utf8(v).unwrap()).collect(),
///     ListFetch::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<Vec<String>, MomentoError> suitable for
/// ?-propagation if you know you are expecting a ListFetch::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// # use momento::cache::ListFetch;
/// use std::convert::TryInto;
/// # let values = vec!["abc", "123"].iter().map(|s| s.as_bytes().to_vec()).collect();
/// # let response = ListFetch::Hit { values: values };
/// let fetched_values: Vec<String> = response.try_into().expect("Expected to fetch a list of strings!");
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum ListFetch {
    Hit { values: Vec<Vec<u8>> },
    Miss,
}

impl TryFrom<ListFetch> for Vec<Vec<u8>> {
    type Error = MomentoError;

    fn try_from(value: ListFetch) -> Result<Self, Self::Error> {
        match value {
            ListFetch::Hit { values } => Ok(values),
            ListFetch::Miss => Err(MomentoError {
                message: "list fetch response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<ListFetch> for Vec<String> {
    type Error = MomentoError;

    fn try_from(value: ListFetch) -> Result<Self, Self::Error> {
        match value {
            ListFetch::Hit { values } => Ok(values
                .into_iter()
                .map(|v| String::from_utf8(v).expect("list value was not a valid utf-8 string"))
                .collect()),
            ListFetch::Miss => Err(MomentoError {
                message: "sorted set was not found".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl From<Vec<String>> for ListFetch {
    fn from(values: Vec<String>) -> Self {
        ListFetch::Hit {
            values: values.into_iter().map(|v| v.into_bytes()).collect(),
        }
    }
}
