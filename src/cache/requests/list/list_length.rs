use std::convert::TryFrom;

use momento_protos::cache_client::list_length_response;

use crate::{cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoError, MomentoErrorCode, MomentoResult};

/// TODO
pub struct ListLengthRequest<K: IntoBytes> {
  cache_name: String,
  list_name: K,
}

impl<K: IntoBytes> ListLengthRequest<K> {
  pub fn new(cache_name: impl Into<String>, list_name: K) -> Self {
      Self {
          cache_name: cache_name.into(),
          list_name,
      }
  }
}

impl<K: IntoBytes> MomentoRequest for ListLengthRequest<K> {
  type Response = ListLength;

  async fn send(self, cache_client: &CacheClient) -> MomentoResult<ListLength> {
      let request = prep_request_with_timeout(
          &self.cache_name,
          cache_client.configuration.deadline_millis(),
          momento_protos::cache_client::ListLengthRequest {
              list_name: self.list_name.into_bytes(),
          },
      )?;

      let response = cache_client
          .data_client
          .clone()
          .list_length(request)
          .await?
          .into_inner();

      match response.list {
          Some(list_length_response::List::Missing(_)) => Ok(ListLength::Miss),
          Some(list_length_response::List::Found(found)) => Ok(ListLength::Hit {
              length: found.length,
          }),
          _ => unreachable!(),
      }
  }
}

/// TODO
#[derive(Debug, PartialEq, Eq)]
pub enum ListLength {
    Hit { length: u32 },
    Miss,
}

impl TryFrom<ListLength> for u32 {
  type Error = MomentoError;

  fn try_from(value: ListLength) -> Result<Self, Self::Error> {
      match value {
        ListLength::Hit { length } => Ok(length),
        ListLength::Miss => Err(MomentoError {
              message: "list length response was a miss".into(),
              error_code: MomentoErrorCode::Miss,
              inner_error: None,
              details: None,
          }),
      }
  }
}