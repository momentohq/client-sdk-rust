// This is necessary because our docs currently only support finding an example function when the
// signature for the example function appears entirely on one line. rustfmt really wants to add
// line breaks to long function signatures, so we have to disable it.
#![cfg_attr(rustfmt, rustfmt_skip)]

/***************************************************************************************************
 * NOTE
 ***************************************************************************************************
 *
 * This file is consumed by the dev docs site to show Rust example snippets.
 *
 * The dev docs use some regexes to extract code from these files. Therefore, it's important that:
 * - All functions in this file have their signature on a single line (no newlines to format arguments etc.)
 * - All function bodies end with `Ok(())` on a line by itself.
 *
 **************************************************************************************************/



use futures::StreamExt;
use momento::cache::configurations::Laptop;
use momento::cache::{
    CreateCacheResponse, DecreaseTtlResponse, DictionaryFetchResponse, DictionaryGetFieldResponse, DictionaryGetFieldsResponse, GetResponse, IncreaseTtlResponse, ItemType, ScoreBound, SetIfAbsentOrEqualResponse, SetIfAbsentResponse, SetIfEqualResponse, SetIfNotEqualResponse, SetIfPresentAndNotEqualResponse, SetIfPresentResponse, SortedSetAggregateFunction, SortedSetFetchResponse, SortedSetLengthByScoreRequest, SortedSetOrder, SortedSetUnionStoreRequest, UpdateTtlResponse
};
use momento::topics::TopicClient;
use momento::auth::{AuthClient, CacheSelector, DisposableTokenScopes, ExpiresIn, GenerateDisposableTokenRequest};
use momento::{CacheClient, CredentialProvider, MomentoError};
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Duration;
use uuid::Uuid;

#[allow(non_snake_case)]
pub fn example_API_CredentialProviderFromString() {
    let _credential_provider = CredentialProvider::from_string("my-api-key".to_string());
}

#[allow(non_snake_case)]
pub fn example_API_CredentialProviderFromEnvVar() {
    let _credential_provider = CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string());
}

#[allow(non_snake_case)]
pub fn example_API_ConfigurationLaptop() {
    let _config = momento::cache::configurations::Laptop::latest();
}

#[allow(non_snake_case)]
pub fn example_API_ConfigurationInRegionDefaultLatest() {
    let _config = momento::cache::configurations::InRegion::latest();
}

#[allow(non_snake_case)]
pub fn example_API_ConfigurationInRegionLowLatency() {
    let _config = momento::cache::configurations::LowLatency::latest();
}

#[allow(non_snake_case)]
pub fn example_API_ConfigurationLambdaLatest() {
    let _config = momento::cache::configurations::Lambda::latest();
}

#[allow(non_snake_case)]
pub fn example_API_InstantiateCacheClient() -> Result<(), MomentoError> {
    let _cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(60))
        .configuration(Laptop::latest())
        .credential_provider(CredentialProvider::from_env_var(
            "MOMENTO_API_KEY".to_string(),
        )?)
        .build()?;
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_CreateCache(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    match cache_client.create_cache(cache_name).await? {
        CreateCacheResponse::Created => println!("Cache {} created", cache_name),
        CreateCacheResponse::AlreadyExists => println!("Cache {} already exists", cache_name),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListCaches(cache_client: &CacheClient) -> Result<(), MomentoError> {
    let response = cache_client.list_caches().await?;
    println!("Caches: {:#?}", response.caches);
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_FlushCache(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client.flush_cache(cache_name.to_string()).await?;
    println!("Cache {} flushed", cache_name);
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DeleteCache(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client.delete_cache(cache_name).await?;
    println!("Cache {} deleted", cache_name);
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_Set(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client.set(cache_name, "key", "value").await?;
    println!("Value stored");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_Get(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let response = cache_client.get(cache_name, "key").await?;
    let _item: String = response.try_into().expect("I stored a string!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_Delete(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client.delete(cache_name, "key").await?;
    println!("Value deleted");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_Increment(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let response = cache_client.increment(cache_name, "key", 1).await?;
    println!("Value incremented to {}", response.value);
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ItemGetType(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _item_type: ItemType = cache_client
        .item_get_type(cache_name, "key")
        .await?
        .try_into()
        .expect("Expected an item type!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ItemGetTtl(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _remaining_ttl: Duration = cache_client
        .item_get_ttl(cache_name, "key")
        .await?
        .try_into()
        .expect("Expected an item ttl!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_UpdateTtl(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    match cache_client
        .update_ttl(cache_name, "key", Duration::from_secs(10))
        .await?
    {
        UpdateTtlResponse::Set => println!("TTL updated"),
        UpdateTtlResponse::Miss => println!("Cache miss, unable to find key to update TTL"),
    };
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_IncreaseTtl(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    match cache_client
        .increase_ttl(cache_name, "key", Duration::from_secs(5))
        .await?
    {
        IncreaseTtlResponse::Set => println!("TTL updated"),
        IncreaseTtlResponse::NotSet => println!("unable to increase TTL"),
        IncreaseTtlResponse::Miss => println!("Cache miss, unable to find key to increase TTL"),
    };
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DecreaseTtl(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    match cache_client
        .decrease_ttl(cache_name, "key", Duration::from_secs(3))
        .await?
    {
        DecreaseTtlResponse::Set => println!("TTL updated"),
        DecreaseTtlResponse::NotSet => println!("unable to decrease TTL"),
        DecreaseTtlResponse::Miss => println!("Cache miss, unable to find key to decrease TTL"),
    };
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetIfAbsent(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    match cache_client
        .set_if_absent(cache_name, "key", "value")
        .await?
    {
        SetIfAbsentResponse::Stored => println!("Value stored"),
        SetIfAbsentResponse::NotStored => println!("Value not stored"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetIfPresent(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    match cache_client
        .set_if_present(cache_name, "key", "value")
        .await?
    {
        SetIfPresentResponse::Stored => println!("Value stored"),
        SetIfPresentResponse::NotStored => println!("Value not stored"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetIfEqual(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    match cache_client
        .set_if_equal(cache_name, "key", "new-value", "cached-value")
        .await?
    {
        SetIfEqualResponse::Stored => println!("Value stored"),
        SetIfEqualResponse::NotStored => println!("Value not stored"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetIfNotEqual(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    match cache_client
        .set_if_not_equal(cache_name, "key", "new-value", "cached-value")
        .await?
    {
        SetIfNotEqualResponse::Stored => println!("Value stored"),
        SetIfNotEqualResponse::NotStored => println!("Value not stored"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetIfPresentAndNotEqual(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    match cache_client
        .set_if_present_and_not_equal(cache_name, "key", "new-value", "cached-value")
        .await?
    {
        SetIfPresentAndNotEqualResponse::Stored => println!("Value stored"),
        SetIfPresentAndNotEqualResponse::NotStored => println!("Value not stored"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetIfAbsentOrEqual(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    match cache_client
        .set_if_absent_or_equal(cache_name, "key", "new-value", "cached-value")
        .await?
    {
        SetIfAbsentOrEqualResponse::Stored => println!("Value stored"),
        SetIfAbsentOrEqualResponse::NotStored => println!("Value not stored"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListConcatenateBack(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client
        .list_concatenate_back(cache_name, "list_name", vec!["value1", "value2"])
        .await?;
    println!("Elements appended to list");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListConcatenateFront(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client
        .list_concatenate_front(cache_name, "list_name", vec!["value1", "value2"])
        .await?;
    println!("Elements prepended to list");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListLength(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _length: u32 = cache_client
        .list_length(cache_name, "list_name")
        .await?
        .try_into()
        .expect("Expected a list length!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListFetch(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _fetched_values: Vec<String> = cache_client
        .list_fetch(cache_name, "list_name")
        .await?
        .try_into()
        .expect("Expected a list fetch!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListPopBack(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _popped_value: String = cache_client
        .list_pop_back(cache_name, "list_name")
        .await?
        .try_into()
        .expect("Expected a popped list value!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListPopFront(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _popped_value: String = cache_client
        .list_pop_front(cache_name, "list_name")
        .await?
        .try_into()
        .expect("Expected a popped list value!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListRemoveValue(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client
        .list_remove_value(cache_name, "list_name", "value1")
        .await?;
    println!("Value removed from list");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryIncrement(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let response = cache_client
        .dictionary_increment(cache_name, "dictionary_name", "field", 1)
        .await?;
    println!("Incremented field in dictionary to {}", response.value);
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionarySetField(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client
        .dictionary_set_field(cache_name.to_string(), "dictionary_name", "field", "value")
        .await?;
    println!("Set field in dictionary");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionarySetFields(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client
        .dictionary_set_fields(
            cache_name.to_string(),
            "dictionary_name",
            vec![("field1", "value1"), ("field2", "value2")],
        )
        .await?;
    println!("Set fields in dictionary");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryFetch(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let response = cache_client
        .dictionary_fetch(cache_name, "dictionary_name")
        .await?;

    match response {
        DictionaryFetchResponse::Hit { value } => {
            let dictionary: HashMap<String, String> =
                value.try_into().expect("I stored a dictionary!");
            println!("Fetched dictionary: {:?}", dictionary);
        }
        DictionaryFetchResponse::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryLength(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _length: u32 = cache_client
        .dictionary_length(cache_name, "dictionary_name")
        .await?
        .try_into()
        .expect("Expected a dictionary length!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryGetField(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let response = cache_client
        .dictionary_get_field(cache_name, "dictionary_name", "field")
        .await?;

    match response {
        DictionaryGetFieldResponse::Hit { value } => {
            let value: String = value.try_into().expect("I stored a string!");
            println!("Fetched value: {}", value);
        }
        DictionaryGetFieldResponse::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryGetFields(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let response = cache_client
        .dictionary_get_fields(cache_name, "dictionary_name", vec!["field1", "field2"])
        .await?;

    match response {
        DictionaryGetFieldsResponse::Hit { .. } => {
            let dictionary: HashMap<String, String> = response
                .try_into()
                .expect("I stored a dictionary of strings!");
            println!("Fetched dictionary: {:?}", dictionary);
        }
        DictionaryGetFieldsResponse::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryRemoveField(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client
        .dictionary_remove_field(cache_name, "dictionary_name", "field")
        .await?;
    println!("Field removed from dictionary");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryRemoveFields(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client
        .dictionary_remove_fields(cache_name, "dictionary_name", vec!["field1", "field2"])
        .await?;
    println!("Fields removed from dictionary");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetAddElements(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client
        .set_add_elements(cache_name, "set_name", vec!["value1", "value2"])
        .await?;
    println!("Elements added to set");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetFetch(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _fetched_elements: Vec<String> = cache_client
        .set_fetch(cache_name, "set_name")
        .await?
        .try_into()
        .expect("Expected a set!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetRemoveElements(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client
        .set_remove_elements(cache_name, "set_name", vec!["element1", "element2"])
        .await?;
    println!("Elements removed from set");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetPutElement(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client
        .sorted_set_put_element(cache_name, "sorted_set_name", "value", 1.0)
        .await?;
    println!("Element added to sorted set");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetPutElements(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client
        .sorted_set_put_elements(
            cache_name,
            "sorted_set_name",
            vec![("value1", 1.0), ("value2", 2.0)],
        )
        .await?;
    println!("Elements added to sorted set");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetFetchByRank(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let response = cache_client
        .sorted_set_fetch_by_rank(
            cache_name,
            "sorted_set_name",
            SortedSetOrder::Ascending,
            None,
            None,
        )
        .await?;

    match response {
        SortedSetFetchResponse::Hit { value } => match value.into_strings() {
            Ok(vec) => {
                println!("Fetched elements: {:?}", vec);
            }
            Err(error) => {
                eprintln!("Error converting values into strings: {}", error);
            }
        },
        SortedSetFetchResponse::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetFetchByScore(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let response = cache_client
        .sorted_set_fetch_by_score(cache_name, "sorted_set_name", SortedSetOrder::Ascending)
        .await?;

    match response {
        SortedSetFetchResponse::Hit { value } => match value.into_strings() {
            Ok(vec) => {
                println!("Fetched elements: {:?}", vec);
            }
            Err(error) => {
                eprintln!("Error converting values into strings: {}", error);
            }
        },
        SortedSetFetchResponse::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetGetRank(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _rank: u64 = cache_client
        .sorted_set_get_rank(cache_name, "sorted_set_name", "value1")
        .await?
        .try_into()
        .expect("Expected a rank!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetGetScore(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _score: f64 = cache_client
        .sorted_set_get_score(cache_name, "sorted_set_name", "value1")
        .await?
        .try_into()
        .expect("Expected a score!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetLength(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _length: u32 = cache_client
        .sorted_set_length(cache_name, "sorted_set_name")
        .await?
        .try_into()
        .expect("Expected a list length!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetLengthByScore(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let request = SortedSetLengthByScoreRequest::new(cache_name, "sorted_set_name")
        .min_score(Some(ScoreBound::Inclusive(0.0)))
        .max_score(Some(ScoreBound::Inclusive(100.0)));
    let _length: u32 = cache_client
        .send_request(request)
        .await?
        .try_into()
        .expect("Expected a list length!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetRemoveElements(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    cache_client
        .sorted_set_remove_elements(cache_name, "sorted_set_name", vec!["value1", "value2"])
        .await?;
    println!("Elements removed from sorted set");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetUnionStore(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let sources = vec![
        ("one_set_name", 1.0),
        ("another_set_name", 2.0),
    ];
    let request = SortedSetUnionStoreRequest::new(cache_name, "destination_sorted_set_name", sources)
        .aggregate(SortedSetAggregateFunction::Min);
    let _destination_length: u32 = cache_client
        .send_request(request)
        .await?
        .into();
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_KeyExists(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let result = cache_client.key_exists(cache_name, "key").await?;
    if result.exists {
        println!("Key exists!");
    } else {
        println!("Key does not exist!");
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_KeysExist(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
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
    let _topic_client = TopicClient::builder()
        .configuration(momento::topics::configurations::Laptop::latest())
        .credential_provider(CredentialProvider::from_env_var("MOMENTO_API_KEY")?)
        .build()?;
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_TopicPublish(topic_client: &TopicClient, cache_name: &String, topic_name: &String) -> Result<(), MomentoError> {
    topic_client
        .publish(cache_name, topic_name, "Hello, Momento!")
        .await?;
    println!("Published message");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_TopicSubscribe(topic_client: &TopicClient, cache_name: &String, topic_name: &String) -> Result<(), MomentoError> {
    // Make a subscription
    let mut subscription = topic_client
        .subscribe(cache_name, topic_name)
        .await
        .expect("subscribe rpc failed");

    // Consume the subscription
    while let Some(item) = subscription.next().await {
        println!("Received subscription item: {item:?}")
    }
    Ok(())
}

pub async fn example_responsetypes_get_with_pattern_match(cache_client: &CacheClient, cache_name: &String) -> Result<(), anyhow::Error> {
    let _item: String = match cache_client.get(cache_name, "key").await? {
        GetResponse::Hit { value } => value.try_into()?,
        GetResponse::Miss => return Err(anyhow::Error::msg("cache miss"))
    };
    Ok(())
}

pub async fn example_responsetypes_get_with_try_into(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _item: String = cache_client.get(cache_name, "key").await?.try_into()?;
    Ok(())
}

pub async fn example_responsetypes_dictionary_with_try_into(cache_client: &CacheClient, cache_name: &String) -> Result<(), MomentoError> {
    let _item: HashMap<String, String> = cache_client
        .dictionary_fetch(cache_name, "dictionary_key")
        .await?
        .try_into()?;
    Ok(())
}

#[allow(non_snake_case)]
pub fn example_API_InstantiateAuthClient() -> Result<(), MomentoError> {
    let _auth_client = AuthClient::builder()
        .credential_provider(CredentialProvider::from_env_var("MOMENTO_API_KEY")?)
        .build()?;
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_GenerateDisposableToken(auth_client: &AuthClient) -> Result<(), MomentoError> {
    // Basic example
    let expiry = ExpiresIn::minutes(30);
    let scope = DisposableTokenScopes::cache_key_read_only(CacheSelector::AllCaches, "key");
    let response = auth_client
        .generate_disposable_token(scope, expiry)
        .await?;
    let token = response.clone().auth_token();
    println!(
        "Generated disposable token ending with '{}' that expires at epoch {}", 
        &token[token.len() - 10 .. token.len() - 1], response.expires_at()
    );

    // Generate a token with optional token ID that can be used with Momento Topics
    let expiry = ExpiresIn::minutes(30);
    let scope = DisposableTokenScopes::cache_key_read_only(CacheSelector::AllCaches, "key");
    let request = GenerateDisposableTokenRequest::new(scope, expiry).token_id("my-token-id".to_string());
    let response = auth_client.send_request(request).await?;
    let token = response.clone().auth_token();
    println!(
        "Generated disposable token ending with '{}' that expires at epoch {}", 
        &token[token.len() - 10 .. token.len() - 1], response.expires_at()
    );
    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<(), MomentoError> {
    example_API_CredentialProviderFromString();
    example_API_CredentialProviderFromEnvVar();
    example_API_ConfigurationLaptop();
    example_API_ConfigurationInRegionDefaultLatest();
    example_API_ConfigurationInRegionLowLatency();
    example_API_ConfigurationLambdaLatest();

    example_API_InstantiateCacheClient()?;

    let cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(60))
        .configuration(Laptop::latest())
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

    example_API_DictionaryIncrement(&cache_client, &cache_name).await?;
    example_API_DictionarySetField(&cache_client, &cache_name).await?;
    example_API_DictionarySetFields(&cache_client, &cache_name).await?;
    example_API_DictionaryFetch(&cache_client, &cache_name).await?;
    example_API_DictionaryLength(&cache_client, &cache_name).await?;
    example_API_DictionaryGetField(&cache_client, &cache_name).await?;
    example_API_DictionaryGetFields(&cache_client, &cache_name).await?;
    example_API_DictionaryRemoveField(&cache_client, &cache_name).await?;
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
    example_API_SortedSetLengthByScore(&cache_client, &cache_name).await?;
    example_API_SortedSetRemoveElements(&cache_client, &cache_name).await?;
    example_API_SortedSetUnionStore(&cache_client, &cache_name).await?;

    example_API_InstantiateTopicClient()?;

    let topic_client = TopicClient::builder()
        .configuration(momento::topics::configurations::Laptop::latest())
        .credential_provider(CredentialProvider::from_env_var("MOMENTO_API_KEY")?)
        .build()?;
    let topic_name = format!("{}-{}", "docs-examples-topic", Uuid::new_v4());

    example_API_TopicPublish(&topic_client, &cache_name, &topic_name).await?;

    // Wrap the future with a `Timeout` set to expire in 10 milliseconds.
    let _ = tokio::time::timeout(
        Duration::from_millis(10),
        example_API_TopicSubscribe(&topic_client, &cache_name, &topic_name),
    )
    .await;

    example_API_DeleteCache(&cache_client, &cache_name).await?;

    example_API_InstantiateAuthClient()?;

    let auth_client = AuthClient::builder()
        .credential_provider(CredentialProvider::from_env_var(
            "MOMENTO_API_KEY".to_string())?)
            .build()?;

    example_API_GenerateDisposableToken(&auth_client).await?;

    Ok(())
}
