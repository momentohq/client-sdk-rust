use std::convert::TryFrom;

use momento_protos::cache_client::{sorted_set_get_rank_response::Rank, ECacheResult};

use crate::{
    cache::{MomentoRequest, SortedSetOrder},
    utils::prep_request_with_timeout,
    CacheClient, IntoBytes, MomentoError, MomentoResult,
};

/// Get the rank (position) of a specific element in a sorted set.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `sorted_set_name` - name of the sorted set
/// * `value` - the sorted set value to get the rank of
///
/// # Optional Arguments
///
/// * `order` - The order to sort the elements by. [SortedSetOrder::Ascending] or [SortedSetOrder::Descending].
///   Defaults to Ascending.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::cache::{SortedSetOrder, SortedSetGetRankResponse, SortedSetGetRankRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let sorted_set_name = "sorted_set";
///
/// # cache_client.sorted_set_put_elements(&cache_name, sorted_set_name.to_string(), vec![("value1", 1.0), ("value2", 2.0)]).await;
///
/// let get_rank_request = SortedSetGetRankRequest::new(cache_name, sorted_set_name, "value1")
///     .order(SortedSetOrder::Ascending);
/// let rank: u64 = cache_client.send_request(get_rank_request).await?.try_into().expect("Expected a rank!");
/// # assert_eq!(rank, 0);
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SortedSetGetRankRequest<L: IntoBytes, V: IntoBytes> {
    cache_name: String,
    sorted_set_name: L,
    value: V,
    order: SortedSetOrder,
}

impl<L: IntoBytes, V: IntoBytes> SortedSetGetRankRequest<L, V> {
    /// Constructs a new SortedSetGetRankRequest.
    pub fn new(cache_name: impl Into<String>, sorted_set_name: L, value: V) -> Self {
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            value,
            order: SortedSetOrder::Ascending,
        }
    }

    /// Set the rank order of the request.
    pub fn order(mut self, order: impl Into<Option<SortedSetOrder>>) -> Self {
        self.order = order.into().unwrap_or(SortedSetOrder::Ascending);
        self
    }
}

impl<L: IntoBytes, V: IntoBytes> MomentoRequest for SortedSetGetRankRequest<L, V> {
    type Response = SortedSetGetRankResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetGetRankResponse> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.deadline_millis(),
            momento_protos::cache_client::SortedSetGetRankRequest {
                set_name: self.sorted_set_name.into_bytes(),
                value: self.value.into_bytes(),
                order: self.order as i32,
            },
        )?;

        let response = cache_client
            .next_data_client()
            .sorted_set_get_rank(request)
            .await?
            .into_inner();

        match response.rank {
            Some(Rank::ElementRank(found)) => {
                if found.result == ECacheResult::Hit as i32 {
                    Ok(SortedSetGetRankResponse::Hit { rank: found.rank })
                } else {
                    Ok(SortedSetGetRankResponse::Miss)
                }
            }
            Some(Rank::Missing(_)) => Ok(SortedSetGetRankResponse::Miss),
            _ => Err(MomentoError::unknown_error(
                "SortedSetGetRank",
                Some(format!("{response:#?}")),
            )),
        }
    }
}

/// Response for a sorted set get rank operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SortedSetGetRankResponse;
/// use std::convert::TryInto;
/// # let response = SortedSetGetRankResponse::Hit { rank: 5 };
/// let rank: u64 = match response {
///     SortedSetGetRankResponse::Hit { rank } => rank.try_into().expect("Expected a rank!"),
///     SortedSetGetRankResponse::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<u64, MomentoError> suitable for
/// ?-propagation if you know you are expecting a SortedSetGetRankResponse::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::SortedSetGetRankResponse;
/// use std::convert::TryInto;
/// # let response = SortedSetGetRankResponse::Hit { rank: 5 };
/// let rank: MomentoResult<u64> = response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum SortedSetGetRankResponse {
    /// The sorted set was found.
    Hit {
        /// The rank of the element in the sorted set.
        rank: u64,
    },
    /// The sorted set was not found.
    Miss,
}

impl TryFrom<SortedSetGetRankResponse> for u64 {
    type Error = MomentoError;

    fn try_from(value: SortedSetGetRankResponse) -> Result<Self, Self::Error> {
        match value {
            SortedSetGetRankResponse::Hit { rank } => Ok(rank),
            SortedSetGetRankResponse::Miss => Err(MomentoError::miss("SortedSetGetRank")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorted_set_get_rank_request_builder() {
        let cache_name = "my_cache";
        let sorted_set_name = "my_sorted_set";

        // Test with ascending order
        let request = SortedSetGetRankRequest::new(cache_name, sorted_set_name, "value1")
            .order(SortedSetOrder::Ascending);

        assert_eq!(request.cache_name, cache_name);
        assert_eq!(
            request.sorted_set_name.into_bytes(),
            sorted_set_name.as_bytes()
        );
        assert_eq!(request.value, "value1");
        assert_eq!(request.order, SortedSetOrder::Ascending);

        // Test with descending order
        let request = SortedSetGetRankRequest::new(cache_name, sorted_set_name, "value1")
            .order(SortedSetOrder::Descending);

        assert_eq!(request.cache_name, cache_name);
        assert_eq!(
            request.sorted_set_name.into_bytes(),
            sorted_set_name.as_bytes()
        );
        assert_eq!(request.value, "value1");
        assert_eq!(request.order, SortedSetOrder::Descending);

        // Test with default order
        let request = SortedSetGetRankRequest::new(cache_name, sorted_set_name, "value1");

        assert_eq!(request.cache_name, cache_name);
        assert_eq!(
            request.sorted_set_name.into_bytes(),
            sorted_set_name.as_bytes()
        );
        assert_eq!(request.value, "value1");
        assert_eq!(request.order, SortedSetOrder::Ascending);
    }
}
