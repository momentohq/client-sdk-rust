use std::convert::TryFrom;

use momento_protos::cache_client::sorted_set_length_response;

use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoError,
    MomentoErrorCode, MomentoResult,
};

/// Get the number of entries in a sorted set collection.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `sorted_set_name` - name of the sorted set
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::cache::{SortedSetLength, SortedSetLengthRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let sorted_set_name = "sorted_set";
///
/// # cache_client.sorted_set_put_elements(&cache_name, sorted_set_name.to_string(), vec![("value1", 1.0), ("value2", 2.0)]).await;
///
/// let length_request = SortedSetLengthRequest::new(cache_name, sorted_set_name);
/// let length: u32 = cache_client.send_request(length_request).await?.try_into().expect("Expected a list length!");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SortedSetLengthRequest<L: IntoBytes> {
    cache_name: String,
    sorted_set_name: L,
}

impl<L: IntoBytes> SortedSetLengthRequest<L> {
    pub fn new(cache_name: impl Into<String>, sorted_set_name: L) -> Self {
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
        }
    }
}

impl<L: IntoBytes> MomentoRequest for SortedSetLengthRequest<L> {
    type Response = SortedSetLength;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetLength> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::SortedSetLengthRequest {
                set_name: self.sorted_set_name.into_bytes(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .sorted_set_length(request)
            .await?
            .into_inner();

        match response.sorted_set {
            Some(sorted_set_length_response::SortedSet::Missing(_)) => Ok(SortedSetLength::Miss),
            Some(sorted_set_length_response::SortedSet::Found(found)) => Ok(SortedSetLength::Hit {
                length: found.length,
            }),
            _ => Err(MomentoError::unknown_error(
                "SortedSetLength",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response for a sorted set length operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SortedSetLength;
/// use std::convert::TryInto;
/// # let response = SortedSetLength::Hit { length: 5 };
/// let length: u32 = match response {
///     SortedSetLength::Hit { length } => length.try_into().expect("Expected a list length!"),
///     SortedSetLength::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<u32, MomentoError> suitable for
/// ?-propagation if you know you are expecting a SortedSetLength::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SortedSetLength;
/// use std::convert::TryInto;
/// # let response = SortedSetLength::Hit { length: 5 };
/// let length: MomentoResult<u32> = response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum SortedSetLength {
    Hit { length: u32 },
    Miss,
}

impl TryFrom<SortedSetLength> for u32 {
    type Error = MomentoError;

    fn try_from(value: SortedSetLength) -> Result<Self, Self::Error> {
        match value {
            SortedSetLength::Hit { length } => Ok(length),
            SortedSetLength::Miss => Err(MomentoError {
                message: "list length response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}
