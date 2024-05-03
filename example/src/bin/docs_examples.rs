use futures::StreamExt;
use momento::cache::configurations::laptop;
use momento::cache::{
    CreateCache, DecreaseTtl, DictionaryFetch, DictionaryGetFields, IncreaseTtl, ItemType,
    SetIfAbsent, SetIfAbsentOrEqual, SetIfEqual, SetIfNotEqual, SetIfPresent,
    SetIfPresentAndNotEqual, SortedSetFetch, SortedSetOrder, UpdateTtl,
};
use momento::topics::TopicClient;
use momento::{CacheClient, CredentialProvider, MomentoError, MomentoErrorCode};
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Duration;
use uuid::Uuid;

#[allow(non_snake_case)]
pub fn example_API_CredentialProviderFromString() {
    let _credential_provider = CredentialProvider::from_string("my-api-key".to_string());
}

#[allow(non_snake_case)]
pub fn example_API_ConfigurationLaptop() {
    let _config = momento::cache::configurations::laptop::latest();
}

#[allow(non_snake_case)]
pub fn example_API_ConfigurationInRegionDefaultLatest() {
    let _config = momento::cache::configurations::in_region::latest();
}

#[allow(non_snake_case)]
pub fn example_API_ConfigurationInRegionLowLatency() {
    let _config = momento::cache::configurations::low_latency::latest();
}

#[allow(non_snake_case)]
pub fn example_API_ConfigurationLambdaLatest() {
    let _config = momento::cache::configurations::lambda::latest();
}

#[allow(non_snake_case)]
pub fn example_API_InstantiateCacheClient() -> Result<(), MomentoError> {
    let _cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(60))
        .configuration(laptop::latest())
        .credential_provider(CredentialProvider::from_env_var(
            "MOMENTO_API_KEY".to_string(),
        )?)
        .build()?;
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_CreateCache(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client.create_cache(cache_name).await? {
        CreateCache::Created => println!("Cache {} created", cache_name),
        CreateCache::AlreadyExists => println!("Cache {} already exists", cache_name),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListCaches(cache_client: &CacheClient) -> Result<(), MomentoError> {
    match cache_client.list_caches().await {
        Ok(response) => println!("Caches: {:#?}", response.caches),
        Err(e) => eprintln!("Error listing caches: {}", e),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_FlushCache(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client.flush_cache(cache_name.to_string()).await {
        Ok(_) => println!("Flushed cache: {}", cache_name),
        Err(e) => {
            if let MomentoErrorCode::NotFoundError = e.error_code {
                println!("Cache not found: {}", cache_name);
            } else {
                eprintln!("Error flushing cache: {}", e);
            }
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DeleteCache(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client.delete_cache(cache_name).await {
        Ok(_) => println!("Cache deleted: {}", cache_name),
        Err(e) => {
            if let MomentoErrorCode::NotFoundError = e.error_code {
                println!("Cache not found: {}", cache_name);
            } else {
                eprintln!("Error deleting cache {}: {}", cache_name, e);
            }
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_Set(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client.set(cache_name, "key", "value").await {
        Ok(_) => println!("Set successful"),
        Err(e) => {
            if let MomentoErrorCode::NotFoundError = e.error_code {
                println!("Cache not found: {}", cache_name);
            } else {
                eprintln!("Error setting value in cache {}: {}", cache_name, e);
            }
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_Get(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let response = cache_client.get(cache_name, "key").await?;
    let _item: String = response.try_into().expect("I stored a string!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_Delete(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client.delete(cache_name, "key").await {
        Ok(_) => println!("Delete successful"),
        Err(e) => {
            if let MomentoErrorCode::NotFoundError = e.error_code {
                println!("Cache not found: {}", cache_name);
            } else {
                eprintln!("Error deleting value in cache {}: {}", cache_name, e);
            }
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_Increment(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client.increment(cache_name, "key", 1).await {
        Ok(r) => println!("Incremented value: {}", r.value),
        Err(e) => {
            if let MomentoErrorCode::NotFoundError = e.error_code {
                println!("Cache not found: {}", cache_name);
            } else {
                eprintln!("Error incrementing value in cache {}: {}", cache_name, e);
            }
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ItemGetType(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let response = cache_client.item_get_type(cache_name, "key").await?;
    let _item: ItemType = response.try_into().expect("Expected an item type!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ItemGetTtl(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let _remaining_ttl: Duration = cache_client
        .item_get_ttl(cache_name, "key")
        .await?
        .try_into()
        .expect("Expected an item ttl!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_UpdateTtl(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .update_ttl(cache_name, "key", Duration::from_secs(10))
        .await?
    {
        UpdateTtl::Set => println!("TTL updated"),
        UpdateTtl::Miss => println!("Cache miss, unable to find key to update TTL"),
    };
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_IncreaseTtl(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .increase_ttl(cache_name, "key", Duration::from_secs(5))
        .await?
    {
        IncreaseTtl::Set => println!("TTL updated"),
        IncreaseTtl::NotSet => println!("unable to increase TTL"),
        IncreaseTtl::Miss => println!("Cache miss, unable to find key to increase TTL"),
    };
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DecreaseTtl(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .decrease_ttl(cache_name, "key", Duration::from_secs(3))
        .await?
    {
        DecreaseTtl::Set => println!("TTL updated"),
        DecreaseTtl::NotSet => println!("unable to decrease TTL"),
        DecreaseTtl::Miss => println!("Cache miss, unable to find key to decrease TTL"),
    };
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetIfAbsent(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client.set_if_absent(cache_name, "key", "value").await {
        Ok(response) => match response {
            SetIfAbsent::Stored => println!("Value stored"),
            SetIfAbsent::NotStored => println!("Value not stored"),
        },
        Err(e) => {
            if let MomentoErrorCode::NotFoundError = e.error_code {
                println!("Cache not found: {}", cache_name);
            } else {
                eprintln!("Error setting value in cache {}: {}", cache_name, e);
            }
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetIfPresent(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .set_if_present(cache_name, "key", "value")
        .await
    {
        Ok(response) => match response {
            SetIfPresent::Stored => println!("Value stored"),
            SetIfPresent::NotStored => println!("Value not stored"),
        },
        Err(e) => {
            if let MomentoErrorCode::NotFoundError = e.error_code {
                println!("Cache not found: {}", cache_name);
            } else {
                eprintln!("Error setting value in cache {}: {}", cache_name, e);
            }
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetIfEqual(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .set_if_equal(cache_name, "key", "new-value", "cached-value")
        .await
    {
        Ok(response) => match response {
            SetIfEqual::Stored => println!("Value stored"),
            SetIfEqual::NotStored => println!("Value not stored"),
        },
        Err(e) => {
            if let MomentoErrorCode::NotFoundError = e.error_code {
                println!("Cache not found: {}", cache_name);
            } else {
                eprintln!("Error setting value in cache {}: {}", cache_name, e);
            }
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetIfNotEqual(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .set_if_not_equal(cache_name, "key", "new-value", "cached-value")
        .await
    {
        Ok(response) => match response {
            SetIfNotEqual::Stored => println!("Value stored"),
            SetIfNotEqual::NotStored => println!("Value not stored"),
        },
        Err(e) => {
            if let MomentoErrorCode::NotFoundError = e.error_code {
                println!("Cache not found: {}", cache_name);
            } else {
                eprintln!("Error setting value in cache {}: {}", cache_name, e);
            }
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetIfPresentAndNotEqual(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .set_if_present_and_not_equal(cache_name, "key", "new-value", "cached-value")
        .await
    {
        Ok(response) => match response {
            SetIfPresentAndNotEqual::Stored => println!("Value stored"),
            SetIfPresentAndNotEqual::NotStored => println!("Value not stored"),
        },
        Err(e) => {
            if let MomentoErrorCode::NotFoundError = e.error_code {
                println!("Cache not found: {}", cache_name);
            } else {
                eprintln!("Error setting value in cache {}: {}", cache_name, e);
            }
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetIfAbsentOrEqual(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .set_if_absent_or_equal(cache_name, "key", "new-value", "cached-value")
        .await
    {
        Ok(response) => match response {
            SetIfAbsentOrEqual::Stored => println!("Value stored"),
            SetIfAbsentOrEqual::NotStored => println!("Value not stored"),
        },
        Err(e) => {
            if let MomentoErrorCode::NotFoundError = e.error_code {
                println!("Cache not found: {}", cache_name);
            } else {
                eprintln!("Error setting value in cache {}: {}", cache_name, e);
            }
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListConcatenateBack(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .list_concatenate_back(cache_name, "list_name", vec!["value1", "value2"])
        .await
    {
        Ok(_) => println!("Elements added to list"),
        Err(e) => eprintln!("Error adding elements to list: {}", e),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListConcatenateFront(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .list_concatenate_front(cache_name, "list_name", vec!["value1", "value2"])
        .await
    {
        Ok(_) => println!("Elements added to list"),
        Err(e) => eprintln!("Error adding elements to list: {}", e),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListLength(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let _length: u32 = cache_client
        .list_length(cache_name, "list_name")
        .await?
        .try_into()
        .expect("Expected a list length!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListFetch(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let _fetched_values: Vec<String> = cache_client
        .list_fetch(cache_name, "list_name")
        .await?
        .try_into()
        .expect("Expected a list fetch!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListPopBack(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let _popped_value: String = cache_client
        .list_pop_back(cache_name, "list_name")
        .await?
        .try_into()
        .expect("Expected a popped list value!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListPopFront(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let _popped_value: String = cache_client
        .list_pop_front(cache_name, "list_name")
        .await?
        .try_into()
        .expect("Expected a popped list value!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListRemoveValue(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .list_remove_value(cache_name, "list_name", "value1")
        .await
    {
        Ok(_) => println!("Successfully removed value"),
        Err(e) => eprintln!("Error removing value: {:?}", e),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionarySetFields(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .dictionary_set_field(cache_name.to_string(), "dictionary_name", "field", "value")
        .await
    {
        Ok(_) => println!("Field set in dictionary"),
        Err(e) => eprintln!("Error setting field in dictionary: {}", e),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryFetch(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .dictionary_fetch(cache_name, "dictionary_name")
        .await?
    {
        DictionaryFetch::Hit { value } => {
            let dictionary: HashMap<String, String> =
                value.try_into().expect("I stored a dictionary!");
            println!("Fetched dictionary: {:?}", dictionary);
        }
        DictionaryFetch::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryLength(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let _length: u32 = cache_client
        .dictionary_length(cache_name, "dictionary_name")
        .await?
        .try_into()
        .expect("Expected a dictionary length!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryGetFields(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let response = cache_client
        .dictionary_get_fields(cache_name, "dictionary_name", vec!["field1", "field2"])
        .await?;

    match response {
        DictionaryGetFields::Hit { .. } => {
            let dictionary: HashMap<String, String> = response
                .try_into()
                .expect("I stored a dictionary of strings!");
            println!("Fetched dictionary: {:?}", dictionary);
        }
        DictionaryGetFields::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryRemoveFields(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .dictionary_remove_fields(cache_name, "dictionary_name", vec!["field1", "field2"])
        .await
    {
        Ok(_) => println!("Fields removed successfully"),
        Err(e) => println!("Error removing fields: {}", e),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetAddElements(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .set_add_elements(cache_name, "set_name", vec!["value1", "value2"])
        .await
    {
        Ok(_) => println!("Elements added to set"),
        Err(e) => eprintln!("Error adding elements to set: {}", e),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetFetch(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let _fetched_elements: Vec<String> = cache_client
        .set_fetch(cache_name, "set_name")
        .await?
        .try_into()
        .expect("Expected a set!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetRemoveElements(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .set_remove_elements(cache_name, "set_name", vec!["element1", "element2"])
        .await
    {
        Ok(_) => println!("Elements removed from set"),
        Err(e) => eprintln!("Error removing elements from set: {}", e),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetPutElement(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .sorted_set_put_element(cache_name, "sorted_set_name", "value", 1.0)
        .await
    {
        Ok(_) => println!("Element added to sorted set"),
        Err(e) => eprintln!("Error adding element to sorted set: {}", e),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetPutElements(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .sorted_set_put_elements(
            cache_name,
            "sorted_set_name",
            vec![("value1", 1.0), ("value2", 2.0)],
        )
        .await
    {
        Ok(_) => println!("Elements added to sorted set"),
        Err(e) => eprintln!("Error adding elements to sorted set: {}", e),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetFetchByRank(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let fetch_response = cache_client
        .sorted_set_fetch_by_rank(
            cache_name,
            "sorted_set_name",
            SortedSetOrder::Ascending,
            None,
            None,
        )
        .await?;

    match fetch_response {
        SortedSetFetch::Hit { value } => match value.into_strings() {
            Ok(vec) => {
                println!("Fetched elements: {:?}", vec);
            }
            Err(error) => {
                eprintln!("Error: {}", error);
            }
        },
        SortedSetFetch::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetFetchByScore(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let fetch_response = cache_client
        .sorted_set_fetch_by_score(cache_name, "sorted_set_name", SortedSetOrder::Ascending)
        .await?;

    match fetch_response {
        SortedSetFetch::Hit { value } => match value.into_strings() {
            Ok(vec) => {
                println!("Fetched elements: {:?}", vec);
            }
            Err(error) => {
                eprintln!("Error: {}", error);
            }
        },
        SortedSetFetch::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetGetRank(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let _rank: u64 = cache_client
        .sorted_set_get_rank(cache_name, "sorted_set_name", "value1")
        .await?
        .try_into()
        .expect("Expected a rank!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetGetScore(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let _score: f64 = cache_client
        .sorted_set_get_score(cache_name, "sorted_set_name", "value1")
        .await?
        .try_into()
        .expect("Expected a score!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetLength(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let _length: u32 = cache_client
        .sorted_set_length(cache_name, "sorted_set_name")
        .await?
        .try_into()
        .expect("Expected a list length!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetRemoveElements(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    match cache_client
        .sorted_set_remove_elements(cache_name, "sorted_set_name", vec!["value1", "value2"])
        .await
    {
        Ok(_) => println!("Elements removed from sorted set"),
        Err(e) => eprintln!("Error removing elements from sorted set: {}", e),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_KeyExists(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    let result = cache_client.key_exists(cache_name, "key").await?;
    if result.exists {
        println!("Key exists!");
    } else {
        println!("Key does not exist!");
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_KeysExist(
    cache_client: &CacheClient,
    cache_name: &String,
) -> Result<(), MomentoError> {
    // Receive results as a HashMap
    let result_map: HashMap<String, bool> = cache_client
        .keys_exist(cache_name, vec!["key", "key1", "key2"])
        .await?
        .into();
    println!("Do these keys exist? {:#?}", result_map);

    // Or receive results as a Vec
    let result_list: Vec<bool> = cache_client
        .keys_exist(cache_name, vec!["key", "key1", "key2"])
        .await?
        .into();
    println!("Do these keys exist? {:#?}", result_list);
    Ok(())
}

#[allow(non_snake_case)]
pub fn example_API_InstantiateTopicClient() -> Result<(), MomentoError> {
    let _topic_client = TopicClient::connect(
        CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())?,
        None,
    )
    .expect("could not connect");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_TopicPublish(
    topic_client: &TopicClient,
    cache_name: &String,
    topic_name: &String,
) -> Result<(), MomentoError> {
    match topic_client
        .publish(
            cache_name.to_string(),
            topic_name.to_string(),
            "Hello, Momento!",
        )
        .await
    {
        Ok(_) => println!("Published message!",),
        Err(e) => eprintln!("Error publishing message: {}", e),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_TopicSubscribe(
    topic_client: &TopicClient,
    cache_name: &String,
    topic_name: &String,
) -> Result<(), MomentoError> {
    // Make a subscription
    let mut subscription = topic_client
        .subscribe(cache_name.to_string(), topic_name.to_string(), None)
        .await
        .expect("subscribe rpc failed");

    // Consume the subscription
    while let Some(item) = subscription.next().await {
        println!("Received subscription item: {item:?}")
    }
    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<(), MomentoError> {
    example_API_CredentialProviderFromString();
    example_API_ConfigurationLaptop();
    example_API_ConfigurationInRegionDefaultLatest();
    example_API_ConfigurationInRegionLowLatency();
    example_API_ConfigurationLambdaLatest();

    example_API_InstantiateCacheClient()?;

    let cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(60))
        .configuration(laptop::latest())
        .credential_provider(CredentialProvider::from_env_var(
            "MOMENTO_API_KEY".to_string(),
        )?)
        .build()?;
    let cache_name = format!("{}-{}", "docs-examples", Uuid::new_v4());

    example_API_CreateCache(&cache_client, &cache_name).await?;
    example_API_ListCaches(&cache_client).await?;
    example_API_FlushCache(&cache_client, &cache_name).await?;

    example_API_Set(&cache_client, &cache_name).await?;
    example_API_Get(&cache_client, &cache_name).await?;
    example_API_Delete(&cache_client, &cache_name).await?;
    example_API_Increment(&cache_client, &cache_name).await?;
    example_API_ItemGetType(&cache_client, &cache_name).await?;
    example_API_ItemGetTtl(&cache_client, &cache_name).await?;
    example_API_UpdateTtl(&cache_client, &cache_name).await?;
    example_API_IncreaseTtl(&cache_client, &cache_name).await?;
    example_API_DecreaseTtl(&cache_client, &cache_name).await?;
    example_API_KeyExists(&cache_client, &cache_name).await?;
    example_API_KeysExist(&cache_client, &cache_name).await?;

    example_API_SetIfAbsent(&cache_client, &cache_name).await?;
    example_API_SetIfPresent(&cache_client, &cache_name).await?;
    example_API_SetIfEqual(&cache_client, &cache_name).await?;
    example_API_SetIfNotEqual(&cache_client, &cache_name).await?;
    example_API_SetIfPresentAndNotEqual(&cache_client, &cache_name).await?;
    example_API_SetIfAbsentOrEqual(&cache_client, &cache_name).await?;

    example_API_ListConcatenateBack(&cache_client, &cache_name).await?;
    example_API_ListConcatenateFront(&cache_client, &cache_name).await?;
    example_API_ListLength(&cache_client, &cache_name).await?;
    example_API_ListFetch(&cache_client, &cache_name).await?;
    example_API_ListPopBack(&cache_client, &cache_name).await?;
    example_API_ListPopFront(&cache_client, &cache_name).await?;
    example_API_ListRemoveValue(&cache_client, &cache_name).await?;

    example_API_DictionarySetFields(&cache_client, &cache_name).await?;
    example_API_DictionaryFetch(&cache_client, &cache_name).await?;
    example_API_DictionaryLength(&cache_client, &cache_name).await?;
    example_API_DictionaryGetFields(&cache_client, &cache_name).await?;
    example_API_DictionaryRemoveFields(&cache_client, &cache_name).await?;

    example_API_SetAddElements(&cache_client, &cache_name).await?;
    example_API_SetFetch(&cache_client, &cache_name).await?;
    example_API_SetRemoveElements(&cache_client, &cache_name).await?;

    example_API_SortedSetPutElement(&cache_client, &cache_name).await?;
    example_API_SortedSetPutElements(&cache_client, &cache_name).await?;
    example_API_SortedSetFetchByRank(&cache_client, &cache_name).await?;
    example_API_SortedSetFetchByScore(&cache_client, &cache_name).await?;
    example_API_SortedSetGetRank(&cache_client, &cache_name).await?;
    example_API_SortedSetGetScore(&cache_client, &cache_name).await?;
    example_API_SortedSetLength(&cache_client, &cache_name).await?;
    example_API_SortedSetRemoveElements(&cache_client, &cache_name).await?;

    example_API_InstantiateTopicClient()?;

    let topic_client = TopicClient::connect(
        CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())?,
        None,
    )
    .expect("TopicClient could not connect");
    let topic_name = format!("{}-{}", "docs-examples-topic", Uuid::new_v4());

    example_API_TopicPublish(&topic_client, &cache_name, &topic_name).await?;

    // Wrap the future with a `Timeout` set to expire in 10 milliseconds.
    let _ = tokio::time::timeout(
        Duration::from_millis(10),
        example_API_TopicSubscribe(&topic_client, &cache_name, &topic_name),
    )
    .await;

    example_API_DeleteCache(&cache_client, &cache_name).await?;
    Ok(())
}
