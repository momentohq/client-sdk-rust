use crate::cache::CollectionTtl;
use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoError,
    MomentoResult,
};
use momento_protos::cache_client::list_retain_response;
use momento_protos::{
    cache_client::list_retain_request::{EndIndex, StartIndex},
    common::Unbounded,
};
use std::convert::TryFrom;

/// Retains only slice of the list defined by the provided range. Items outside of this range will be dropped from the list.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `list_name` - name of the list
///
/// # Optional Arguments
///
/// * `start_index` - The starting inclusive element of the list to retain. Default is 0.
/// * `end_index` - The ending exclusive element of the list to retain. Default is up to and including end of list.
/// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::cache::{ListRetainResponse, ListRetainRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let list_name = "list-name";
/// # cache_client.list_concatenate_front(&cache_name, list_name, vec!["value1", "value2"]).await;
///
/// let retain_request = ListRetainRequest::new(cache_name, list_name).start_index(1).end_index(3);
/// match cache_client.send_request(retain_request).await {
///     Ok(_) => println!("Retained the values within the provided range."),
///     Err(e) => eprintln!("Error retaining the list: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListRetainRequest<L: IntoBytes> {
    cache_name: String,
    list_name: L,
    start_index: Option<i32>,
    end_index: Option<i32>,
    collection_ttl: Option<CollectionTtl>,
}

impl<L: IntoBytes> ListRetainRequest<L> {
    /// Constructs a new ListRetainRequest.
    pub fn new(cache_name: impl Into<String>, list_name: L) -> Self {
        Self {
            cache_name: cache_name.into(),
            list_name,
            start_index: None,
            end_index: None,
            collection_ttl: None,
        }
    }

    /// Set the starting inclusive element of the list to retain.
    pub fn start_index(mut self, start_index: impl Into<Option<i32>>) -> Self {
        self.start_index = start_index.into();
        self
    }

    /// Set the ending exclusive element of the list to retain.
    pub fn end_index(mut self, end_index: impl Into<Option<i32>>) -> Self {
        self.end_index = end_index.into();
        self
    }

    /// Set the time-to-live for the list.
    pub fn ttl(mut self, collection_ttl: impl Into<Option<CollectionTtl>>) -> Self {
        self.collection_ttl = collection_ttl.into();
        self
    }
}

impl<L: IntoBytes> MomentoRequest for ListRetainRequest<L> {
    type Response = ListRetainResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ListRetainResponse> {
        let start_index = match self.start_index {
            Some(start) => Some(StartIndex::InclusiveStart(start)),
            None => Some(StartIndex::UnboundedStart(Unbounded {})),
        };
        let end_index = match self.end_index {
            Some(end) => Some(EndIndex::ExclusiveEnd(end)),
            None => Some(EndIndex::UnboundedEnd(Unbounded {})),
        };
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.deadline_millis(),
            momento_protos::cache_client::ListRetainRequest {
                list_name: self.list_name.into_bytes(),
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
                start_index,
                end_index,
            },
        )?;

        let response = cache_client
            .next_data_client()
            .list_retain(request)
            .await?
            .into_inner();

        match response.list {
            Some(list_retain_response::List::Missing(_)) => Ok(ListRetainResponse::Miss),
            Some(list_retain_response::List::Found(found)) => Ok(ListRetainResponse::Hit {
                length: found.list_length,
            }),
            _ => Err(MomentoError::unknown_error(
                "ListRetain",
                Some(format!("{response:#?}")),
            )),
        }
    }
}

/// Response for a list retain operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::ListRetainResponse;
/// use std::convert::TryInto;
/// # let response = ListRetainResponse::Hit { length: 5 };
/// let length: u32 = match response {
///     ListRetainResponse::Hit { length } => length.try_into().expect("Expected a list length!"),
///     ListRetainResponse::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<u32, MomentoError> suitable for
/// ?-propagation if you know you are expecting a ListRetainResponse::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::ListRetainResponse;
/// use std::convert::TryInto;
/// # let response = ListRetainResponse::Hit { length: 5 };
/// let length: MomentoResult<u32> = response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum ListRetainResponse {
    /// The list was found.
    Hit {
        /// The length of the list.
        length: u32,
    },
    /// The list was not found.
    Miss,
}

impl TryFrom<ListRetainResponse> for u32 {
    type Error = MomentoError;

    fn try_from(value: ListRetainResponse) -> Result<Self, Self::Error> {
        match value {
            ListRetainResponse::Hit { length } => Ok(length),
            ListRetainResponse::Miss => Err(MomentoError::miss("ListRetain")),
        }
    }
}
