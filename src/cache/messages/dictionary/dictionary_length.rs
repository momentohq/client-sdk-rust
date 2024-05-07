use std::convert::TryFrom;

use momento_protos::cache_client::{
    dictionary_length_response, DictionaryLengthRequest as DictionaryLengthRequestProto,
};

use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoError,
    MomentoResult,
};

/// Gets the number of elements in the given dictionary.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `dictionary_name` - name of the dictionary
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::convert::TryInto;
/// use momento::cache::{DictionaryLength, DictionaryLengthRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let dictionary_name = "dictionary-name";
/// # cache_client.dictionary_set_fields(&cache_name, dictionary_name, vec![("field1", "value1"), ("field2", "value2")]).await;
///
/// let length_request = DictionaryLengthRequest::new(cache_name, dictionary_name);
/// let length: u32 = cache_client.send_request(length_request).await?.try_into().expect("Expected a dictionary length!");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct DictionaryLengthRequest<D: IntoBytes> {
    cache_name: String,
    dictionary_name: D,
}

impl<D: IntoBytes> DictionaryLengthRequest<D> {
    pub fn new(cache_name: impl Into<String>, dictionary_name: D) -> Self {
        Self {
            cache_name: cache_name.into(),
            dictionary_name,
        }
    }
}

impl<D: IntoBytes> MomentoRequest for DictionaryLengthRequest<D> {
    type Response = DictionaryLength;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<DictionaryLength> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            DictionaryLengthRequestProto {
                dictionary_name: self.dictionary_name.into_bytes(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .dictionary_length(request)
            .await?
            .into_inner();

        match response.dictionary {
            Some(dictionary_length_response::Dictionary::Missing(_)) => Ok(DictionaryLength::Miss),
            Some(dictionary_length_response::Dictionary::Found(found)) => {
                Ok(DictionaryLength::Hit {
                    length: found.length,
                })
            }
            _ => Err(MomentoError::unknown_error(
                "DictionaryLength",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response for a dictionary length operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::DictionaryLength;
/// use std::convert::TryInto;
/// # let response = DictionaryLength::Hit { length: 5 };
/// let length: u32 = match response {
///     DictionaryLength::Hit { length } => length.try_into().expect("Expected a dictionary length!"),
///     DictionaryLength::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<u32, MomentoError> suitable for
/// ?-propagation if you know you are expecting a DictionaryLength::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// use momento::cache::DictionaryLength;
/// use std::convert::TryInto;
/// # let response = DictionaryLength::Hit { length: 5 };
/// let length: MomentoResult<u32> = response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryLength {
    Hit { length: u32 },
    Miss,
}

impl TryFrom<DictionaryLength> for u32 {
    type Error = MomentoError;

    fn try_from(value: DictionaryLength) -> Result<Self, Self::Error> {
        match value {
            DictionaryLength::Hit { length } => Ok(length),
            DictionaryLength::Miss => Err(MomentoError::miss("DictionaryLength")),
        }
    }
}
