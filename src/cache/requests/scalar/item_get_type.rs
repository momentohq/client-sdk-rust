use std::convert::TryFrom;

use momento_protos::cache_client::item_get_type_response::{self};

use crate::{
    cache::MomentoRequest,
    utils::{prep_request_with_timeout, return_unknown_error},
    CacheClient, IntoBytes, MomentoError, MomentoErrorCode, MomentoResult,
};

/// Return the type of the key in the cache.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `key` - the key for which type is requested
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use std::convert::TryInto;
/// use momento::cache::{ItemGetType, ItemType};
/// # cache_client.set(&cache_name, "key1", "value").await?;
///
/// let request = momento::cache::ItemGetTypeRequest::new(&cache_name, "key1");
///
/// let item: ItemType = match(cache_client.send_request(request).await?) {
///     ItemGetType::Hit { key_type } => key_type.try_into().expect("Expected an item type!"),
///     ItemGetType::Miss => return Err(anyhow::Error::msg("cache miss"))
/// };
/// # assert_eq!(item, ItemType::Scalar);
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ItemGetTypeRequest<K: IntoBytes> {
    cache_name: String,
    key: K,
}

impl<K: IntoBytes> ItemGetTypeRequest<K> {
    pub fn new(cache_name: impl Into<String>, key: K) -> Self {
        Self {
            cache_name: cache_name.into(),
            key,
        }
    }
}

impl<K: IntoBytes> MomentoRequest for ItemGetTypeRequest<K> {
    type Response = ItemGetType;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<ItemGetType> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::ItemGetTypeRequest {
                cache_key: self.key.into_bytes(),
            },
        )?;

        let response = cache_client
            .data_client
            .clone()
            .item_get_type(request)
            .await?
            .into_inner();

        match response.result {
            Some(item_get_type_response::Result::Missing(_)) => Ok(ItemGetType::Miss),
            Some(item_get_type_response::Result::Found(found)) => Ok(ItemGetType::Hit {
                key_type: match found.item_type() {
                    momento_protos::cache_client::item_get_type_response::ItemType::Scalar => {
                        ItemType::Scalar
                    }
                    momento_protos::cache_client::item_get_type_response::ItemType::Dictionary => {
                        ItemType::Dictionary
                    }
                    momento_protos::cache_client::item_get_type_response::ItemType::List => {
                        ItemType::List
                    }
                    momento_protos::cache_client::item_get_type_response::ItemType::Set => {
                        ItemType::Set
                    }
                    momento_protos::cache_client::item_get_type_response::ItemType::SortedSet => {
                        ItemType::SortedSet
                    }
                },
            }),
            _ => Err(return_unknown_error(
                "ItemGetType",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ItemType {
    Scalar = 0,
    Dictionary = 1,
    List = 2,
    Set = 3,
    SortedSet = 4,
}

/// Response for a get item type operation.
///
/// If you'd like to handle misses you can simply match and handle your response:
/// ```
/// # use momento::MomentoResult;
/// # use momento::cache::{ItemGetType, ItemType};
/// use std::convert::TryInto;
/// # let response = ItemGetType::Hit { key_type: ItemType::Scalar };
/// let item: ItemType = match response {
///     ItemGetType::Hit { key_type } => key_type.try_into().expect("Expected an item type!"),
///     ItemGetType::Miss => return // probably you'll do something else here
/// };
/// ```
///
/// You can cast your result directly into a Result<ItemType, MomentoError> suitable for
/// ?-propagation if you know you are expecting a GetItemType::Hit.
///
/// Of course, a Miss in this case will be turned into an Error. If that's what you want, then
/// this is what you're after:
/// ```
/// # use momento::MomentoResult;
/// # use momento::cache::{ItemGetType, ItemType};
/// use std::convert::TryInto;
/// # let response = ItemGetType::Hit { key_type: ItemType::Scalar };
/// let itemType: MomentoResult<ItemType> = response.try_into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum ItemGetType {
    Hit { key_type: ItemType },
    Miss,
}

impl TryFrom<ItemGetType> for ItemType {
    type Error = MomentoError;

    fn try_from(value: ItemGetType) -> Result<Self, Self::Error> {
        match value {
            ItemGetType::Hit { key_type } => Ok(key_type),
            ItemGetType::Miss => Err(MomentoError {
                message: "item get type response was a miss".into(),
                error_code: MomentoErrorCode::Miss,
                inner_error: None,
                details: None,
            }),
        }
    }
}
