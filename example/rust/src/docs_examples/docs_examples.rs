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
use momento::cache::{
    CreateCacheResponse, DecreaseTtlResponse, DictionaryFetchResponse, DictionaryGetFieldResponse, DictionaryGetFieldsResponse, GetResponse, IncreaseTtlResponse, ItemType, ScoreBound, SetIfAbsentOrEqualResponse, SetIfAbsentResponse, SetIfEqualResponse, SetIfNotEqualResponse, SetIfPresentAndNotEqualResponse, SetIfPresentResponse, SortedSetAggregateFunction, SortedSetFetchResponse, SortedSetLengthByScoreRequest, SortedSetOrder, SortedSetUnionStoreRequest, UpdateTtlResponse
};
use momento::topics::TopicClient;
use momento::auth::{AuthClient, CacheSelector, DisposableTokenScopes, ExpiresIn, GenerateDisposableTokenRequest};
use momento::{CacheClient, CredentialProvider, MomentoResult};
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Duration;
use uuid::Uuid;

const FAKE_V1_API_KEY: &str = "eyJhcGlfa2V5IjogImV5SjBlWEFpT2lKS1YxUWlMQ0poYkdjaU9pSklVekkxTmlKOS5leUpwYzNNaU9pSlBibXhwYm1VZ1NsZFVJRUoxYVd4a1pYSWlMQ0pwWVhRaU9qRTJOemd6TURVNE1USXNJbVY0Y0NJNk5EZzJOVFV4TlRReE1pd2lZWFZrSWpvaUlpd2ljM1ZpSWpvaWFuSnZZMnRsZEVCbGVHRnRjR3hsTG1OdmJTSjkuOEl5OHE4NExzci1EM1lDb19IUDRkLXhqSGRUOFVDSXV2QVljeGhGTXl6OCIsICJlbmRwb2ludCI6ICJ0ZXN0Lm1vbWVudG9ocS5jb20ifQo=";
const FAKE_V2_API_KEY: &str = "eyJhbGciOiJIUzUxMiIsInR5cCI6IkpXVCJ9.eyJ0IjoiZyIsImp0aSI6InNvbWUtaWQifQ.GMr9nA6HE0ttB6llXct_2Sg5-fOKGFbJCdACZFgNbN1fhT6OPg_hVc8ThGzBrWC_RlsBpLA1nzqK3SOJDXYxAw";

fn retrieve_api_key_v1_from_your_secrets_manager() -> String {
    FAKE_V1_API_KEY.to_string()
}

fn generate_disposable_token() -> String {
    FAKE_V1_API_KEY.to_string()
}

fn retrieve_api_key_v2_from_your_secrets_manager() -> String {
    FAKE_V2_API_KEY.to_string()
}

#[allow(deprecated)]
#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_CredentialProviderFromString() -> MomentoResult<()> {
    let api_key = retrieve_api_key_v1_from_your_secrets_manager();
    let credential_provider = CredentialProvider::from_string(api_key)?;
    Ok(())
}

#[allow(deprecated)]
#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_CredentialProviderFromEnvVar() -> MomentoResult<()> {
    let credential_provider = CredentialProvider::from_env_var("V1_API_KEY".to_string())?;
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_CredentialProviderFromApiKeyV2() -> MomentoResult<()> {
    let api_key = retrieve_api_key_v2_from_your_secrets_manager();
    let endpoint = "cell-4-us-west-2-1.prod.a.momentohq.com".to_string();
    let credential_provider = CredentialProvider::from_api_key_v2(api_key, endpoint)?;
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_CredentialProviderFromEnvVarV2() -> MomentoResult<()> {
    let credential_provider = CredentialProvider::from_env_var_v2("MOMENTO_API_KEY", "MOMENTO_ENDPOINT")?;
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_CredentialProviderFromEnvVarV2Default() -> MomentoResult<()> {
    let credential_provider = CredentialProvider::from_default_env_var_v2()?;
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_CredentialProviderFromDisposableToken() -> MomentoResult<()> {
    let auth_token = generate_disposable_token();
    let credential_provider = CredentialProvider::from_disposable_token(auth_token)?;
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_ConfigurationLaptop() -> MomentoResult<()> {
    let config = momento::cache::configurations::Laptop::latest();
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_ConfigurationInRegionDefaultLatest() -> MomentoResult<()> {
    let config = momento::cache::configurations::InRegion::latest();
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_ConfigurationInRegionLowLatency() -> MomentoResult<()> {
    let config = momento::cache::configurations::LowLatency::latest();
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_ConfigurationLambdaLatest() -> MomentoResult<()> {
    let config = momento::cache::configurations::Lambda::latest();
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_InstantiateCacheClient() -> MomentoResult<()> {
    let cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(60))
        .configuration(momento::cache::configurations::Laptop::latest())
        .credential_provider(CredentialProvider::from_default_env_var_v2()?)
        .build()?;
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_CreateCache(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    match cache_client.create_cache(cache_name).await? {
        CreateCacheResponse::Created => println!("Cache {cache_name} created"),
        CreateCacheResponse::AlreadyExists => println!("Cache {cache_name} already exists"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListCaches(cache_client: &CacheClient) -> MomentoResult<()> {
    let response = cache_client.list_caches().await?;
    println!("Caches: {:#?}", response.caches);
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_FlushCache(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client.flush_cache(cache_name.to_string()).await?;
    println!("Cache {cache_name} flushed");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DeleteCache(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client.delete_cache(cache_name).await?;
    println!("Cache {cache_name} deleted");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_Set(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client.set(cache_name, "key", "value").await?;
    println!("Value stored");
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_Get(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let response = cache_client.get(cache_name, "key").await?;
    let item: String = response.try_into().expect("I stored a string!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_Delete(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client.delete(cache_name, "key").await?;
    println!("Value deleted");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_Increment(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let response = cache_client.increment(cache_name, "key", 1).await?;
    println!("Value incremented to {}", response.value);
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_ItemGetType(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let item_type: ItemType = cache_client
        .item_get_type(cache_name, "key")
        .await?
        .try_into()
        .expect("Expected an item type!");
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_ItemGetTtl(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let remaining_ttl: Duration = cache_client
        .item_get_ttl(cache_name, "key")
        .await?
        .try_into()
        .expect("Expected an item ttl!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_UpdateTtl(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
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
pub async fn example_API_IncreaseTtl(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
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
pub async fn example_API_DecreaseTtl(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
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
pub async fn example_API_SetIfAbsent(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
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
pub async fn example_API_SetIfPresent(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
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
pub async fn example_API_SetIfEqual(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
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
pub async fn example_API_SetIfNotEqual(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
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
pub async fn example_API_SetIfPresentAndNotEqual(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
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
pub async fn example_API_SetIfAbsentOrEqual(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
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
pub async fn example_API_ListConcatenateBack(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client
        .list_concatenate_back(cache_name, "list_name", vec!["value1", "value2"])
        .await?;
    println!("Elements appended to list");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListConcatenateFront(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client
        .list_concatenate_front(cache_name, "list_name", vec!["value1", "value2"])
        .await?;
    println!("Elements prepended to list");
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_ListLength(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let length: u32 = cache_client
        .list_length(cache_name, "list_name")
        .await?
        .try_into()
        .expect("Expected a list length!");
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_ListFetch(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let fetched_values: Vec<String> = cache_client
        .list_fetch(cache_name, "list_name")
        .await?
        .try_into()
        .expect("Expected a list fetch!");
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_ListPopBack(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let popped_value: String = cache_client
        .list_pop_back(cache_name, "list_name")
        .await?
        .try_into()
        .expect("Expected a popped list value!");
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_ListPopFront(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let popped_value: String = cache_client
        .list_pop_front(cache_name, "list_name")
        .await?
        .try_into()
        .expect("Expected a popped list value!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_ListRemoveValue(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client
        .list_remove_value(cache_name, "list_name", "value1")
        .await?;
    println!("Value removed from list");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryIncrement(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let response = cache_client
        .dictionary_increment(cache_name, "dictionary_name", "field", 1)
        .await?;
    println!("Incremented field in dictionary to {}", response.value);
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionarySetField(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client
        .dictionary_set_field(cache_name.to_string(), "dictionary_name", "field", "value")
        .await?;
    println!("Set field in dictionary");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionarySetFields(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
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
pub async fn example_API_DictionaryFetch(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let response = cache_client
        .dictionary_fetch(cache_name, "dictionary_name")
        .await?;

    match response {
        DictionaryFetchResponse::Hit { value } => {
            let dictionary: HashMap<String, String> =
                value.try_into().expect("I stored a dictionary!");
            println!("Fetched dictionary: {dictionary:?}");
        }
        DictionaryFetchResponse::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_DictionaryLength(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let length: u32 = cache_client
        .dictionary_length(cache_name, "dictionary_name")
        .await?
        .try_into()
        .expect("Expected a dictionary length!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryGetField(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let response = cache_client
        .dictionary_get_field(cache_name, "dictionary_name", "field")
        .await?;

    match response {
        DictionaryGetFieldResponse::Hit { value } => {
            let value: String = value.try_into().expect("I stored a string!");
            println!("Fetched value: {value}");
        }
        DictionaryGetFieldResponse::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryGetFields(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let response = cache_client
        .dictionary_get_fields(cache_name, "dictionary_name", vec!["field1", "field2"])
        .await?;

    match response {
        DictionaryGetFieldsResponse::Hit { .. } => {
            let dictionary: HashMap<String, String> = response
                .try_into()
                .expect("I stored a dictionary of strings!");
            println!("Fetched dictionary: {dictionary:?}");
        }
        DictionaryGetFieldsResponse::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryRemoveField(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client
        .dictionary_remove_field(cache_name, "dictionary_name", "field")
        .await?;
    println!("Field removed from dictionary");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_DictionaryRemoveFields(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client
        .dictionary_remove_fields(cache_name, "dictionary_name", vec!["field1", "field2"])
        .await?;
    println!("Fields removed from dictionary");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetAddElements(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client
        .set_add_elements(cache_name, "set_name", vec!["value1", "value2"])
        .await?;
    println!("Elements added to set");
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_SetFetch(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let fetched_elements: Vec<String> = cache_client
        .set_fetch(cache_name, "set_name")
        .await?
        .try_into()
        .expect("Expected a set!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SetRemoveElements(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client
        .set_remove_elements(cache_name, "set_name", vec!["element1", "element2"])
        .await?;
    println!("Elements removed from set");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetPutElement(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client
        .sorted_set_put_element(cache_name, "sorted_set_name", "value", 1.0)
        .await?;
    println!("Element added to sorted set");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetPutElements(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
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
pub async fn example_API_SortedSetFetchByRank(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
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
                println!("Fetched elements: {vec:?}");
            }
            Err(error) => {
                eprintln!("Error converting values into strings: {error}");
            }
        },
        SortedSetFetchResponse::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetFetchByScore(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let response = cache_client
        .sorted_set_fetch_by_score(cache_name, "sorted_set_name", SortedSetOrder::Ascending)
        .await?;

    match response {
        SortedSetFetchResponse::Hit { value } => match value.into_strings() {
            Ok(vec) => {
                println!("Fetched elements: {vec:?}");
            }
            Err(error) => {
                eprintln!("Error converting values into strings: {error}");
            }
        },
        SortedSetFetchResponse::Miss => println!("Cache miss"),
    }
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_SortedSetGetRank(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let rank: u64 = cache_client
        .sorted_set_get_rank(cache_name, "sorted_set_name", "value1")
        .await?
        .try_into()
        .expect("Expected a rank!");
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_SortedSetGetScore(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let score: f64 = cache_client
        .sorted_set_get_score(cache_name, "sorted_set_name", "value1")
        .await?
        .try_into()
        .expect("Expected a score!");
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_SortedSetLength(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let length: u32 = cache_client
        .sorted_set_length(cache_name, "sorted_set_name")
        .await?
        .try_into()
        .expect("Expected a list length!");
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_SortedSetLengthByScore(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let request = SortedSetLengthByScoreRequest::new(cache_name, "sorted_set_name")
        .min_score(Some(ScoreBound::Inclusive(0.0)))
        .max_score(Some(ScoreBound::Inclusive(100.0)));
    let length: u32 = cache_client
        .send_request(request)
        .await?
        .try_into()
        .expect("Expected a list length!");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_SortedSetRemoveElements(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    cache_client
        .sorted_set_remove_elements(cache_name, "sorted_set_name", vec!["value1", "value2"])
        .await?;
    println!("Elements removed from sorted set");
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_SortedSetUnionStore(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let sources = vec![
        ("one_set_name", 1.0),
        ("another_set_name", 2.0),
    ];
    let request = SortedSetUnionStoreRequest::new(cache_name, "destination_sorted_set_name", sources)
        .aggregate(SortedSetAggregateFunction::Min);
    let destination_length: u32 = cache_client
        .send_request(request)
        .await?
        .into();
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_KeyExists(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let result = cache_client.key_exists(cache_name, "key").await?;
    if result.exists {
        println!("Key exists!");
    } else {
        println!("Key does not exist!");
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_KeysExist(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    // Receive results as a HashMap
    let result_map: HashMap<String, bool> = cache_client
        .keys_exist(cache_name, vec!["key", "key1", "key2"])
        .await?
        .into();
    println!("Do these keys exist? {result_map:#?}");

    // Or receive results as a Vec
    let result_list: Vec<bool> = cache_client
        .keys_exist(cache_name, vec!["key", "key1", "key2"])
        .await?
        .into();
    println!("Do these keys exist? {result_list:#?}");
    Ok(())
}

#[allow(unused)]
#[allow(non_snake_case)]
pub async fn example_API_InstantiateTopicClient() -> MomentoResult<()> {
    let topic_client = TopicClient::builder()
        .configuration(momento::topics::configurations::Laptop::latest())
        .credential_provider(CredentialProvider::from_default_env_var_v2()?)
        .build()?;
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_TopicPublish(topic_client: &TopicClient, cache_name: &String, topic_name: &String) -> MomentoResult<()> {
    topic_client
        .publish(cache_name, topic_name, "Hello, Momento!")
        .await?;
    println!("Published message");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_TopicSubscribe(topic_client: &TopicClient, cache_name: &String, topic_name: &String) -> MomentoResult<()> {
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

#[allow(unused)]
pub async fn example_responsetypes_get_with_pattern_match(cache_client: &CacheClient, cache_name: &String) -> Result<(), anyhow::Error> {
    let item: String = match cache_client.get(cache_name, "key").await? {
        GetResponse::Hit { value } => value.try_into()?,
        GetResponse::Miss => return Err(anyhow::Error::msg("cache miss"))
    };
    Ok(())
}

#[allow(unused)]
pub async fn example_responsetypes_get_with_try_into(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let item: String = cache_client.get(cache_name, "key").await?.try_into()?;
    Ok(())
}

#[allow(unused)]
pub async fn example_responsetypes_dictionary_with_try_into(cache_client: &CacheClient, cache_name: &String) -> MomentoResult<()> {
    let item: HashMap<String, String> = cache_client
        .dictionary_fetch(cache_name, "dictionary_key")
        .await?
        .try_into()?;
    Ok(())
}

#[allow(unused)]
#[allow(deprecated)]
#[allow(non_snake_case)]
pub async fn example_API_InstantiateAuthClient() -> MomentoResult<()> {
    let auth_client = AuthClient::builder()
        .credential_provider(CredentialProvider::from_env_var("V1_API_KEY")?)
        .build()?;
    Ok(())
}

#[allow(non_snake_case)]
pub async fn example_API_GenerateDisposableToken(auth_client: &AuthClient) -> MomentoResult<()> {
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
pub async fn main() -> MomentoResult<()> {
    example_API_CredentialProviderFromString().await?;
    example_API_CredentialProviderFromEnvVar().await?;
    example_API_CredentialProviderFromApiKeyV2().await?;
    example_API_CredentialProviderFromEnvVarV2().await?;
    example_API_CredentialProviderFromEnvVarV2Default().await?;
    example_API_CredentialProviderFromDisposableToken().await?;
    example_API_ConfigurationLaptop().await?;
    example_API_ConfigurationInRegionDefaultLatest().await?;
    example_API_ConfigurationInRegionLowLatency().await?;
    example_API_ConfigurationLambdaLatest().await?;

    example_API_InstantiateCacheClient().await?;

    let cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(60))
        .configuration(momento::cache::configurations::Laptop::latest())
        .credential_provider(CredentialProvider::from_default_env_var_v2()?)
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

    // These return errors on miss, so we ignore their results.
    let _ = example_responsetypes_get_with_pattern_match(&cache_client, &cache_name).await;
    let _ = example_responsetypes_get_with_try_into(&cache_client, &cache_name).await;
    let _ = example_responsetypes_dictionary_with_try_into(&cache_client, &cache_name).await;

    example_API_InstantiateTopicClient().await?;

    let topic_client = TopicClient::builder()
        .configuration(momento::topics::configurations::Laptop::latest())
        .credential_provider(CredentialProvider::from_default_env_var_v2()?)
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

    example_API_InstantiateAuthClient().await?;

    #[allow(deprecated)]
    let auth_client = AuthClient::builder()
        .credential_provider(CredentialProvider::from_env_var("V1_API_KEY")?)
        .build()?;

    example_API_GenerateDisposableToken(&auth_client).await?;

    Ok(())
}
