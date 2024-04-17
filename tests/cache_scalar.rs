use std::collections::HashMap;

use momento::{
    cache::{Delete, ItemGetType, ItemType},
    MomentoErrorCode, MomentoResult,
};
use momento_test_util::CACHE_TEST_STATE;
use uuid::Uuid;

#[tokio::test]
async fn key_exists_invalid_cache_name() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let result = client.key_exists("   ", "key").await.unwrap_err();
    assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
    Ok(())
}

#[tokio::test]
async fn key_exists_nonexistent_cache() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = "fake-cache-".to_string() + &Uuid::new_v4().to_string();
    let result = client.key_exists(cache_name, "key").await.unwrap_err();
    assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
    Ok(())
}

#[tokio::test]
async fn key_exists_happy_path() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = &CACHE_TEST_STATE.cache_name;
    let key = &Uuid::new_v4().to_string();

    // Key should not exist yet
    let result = client.key_exists(cache_name, &**key).await?;
    assert!(
        !result.exists,
        "Expected key {} to not exist in cache {}, but it does",
        &**key, cache_name
    );

    // Key should exist after setting a key
    client.set(cache_name, &**key, "value").await?;
    let result = client.key_exists(cache_name, &**key).await?;
    assert!(
        result.exists,
        "Expected key {} to exist in cache {}, but it does not",
        &**key, cache_name
    );

    Ok(())
}

#[tokio::test]
async fn keys_exist_invalid_cache_name() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let result = client.keys_exist("   ", vec!["key"]).await.unwrap_err();
    assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
    Ok(())
}

#[tokio::test]
async fn keys_exist_nonexistent_cache() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = "fake-cache-".to_string() + &Uuid::new_v4().to_string();
    let result = client
        .keys_exist(cache_name, vec!["key"])
        .await
        .unwrap_err();
    assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
    Ok(())
}

#[tokio::test]
async fn keys_exist_happy_path() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = &CACHE_TEST_STATE.cache_name;

    // Should return empty list if given empty key list
    let empty_vector: Vec<String> = vec![];
    let keys_received: Vec<bool> = client.keys_exist(cache_name, empty_vector).await?.into();
    assert!(
        keys_received.is_empty(),
        "Expected empty list of bools given no keys, but received {:#?}",
        keys_received
    );

    // Key should return true only for keys that exist in the cache
    let key1 = "keys-exist-".to_string() + &Uuid::new_v4().to_string();
    let key2 = "keys-exist-".to_string() + &Uuid::new_v4().to_string();
    let key3 = "keys-exist-".to_string() + &Uuid::new_v4().to_string();
    let key4 = "keys-exist-".to_string() + &Uuid::new_v4().to_string();
    client.set(cache_name, &*key1, &*key1).await?;
    client.set(cache_name, &*key3, &*key3).await?;

    let result = client
        .keys_exist(cache_name, vec![&*key1, &*key2, &*key3, &*key4])
        .await?;

    let keys_list: Vec<bool> = result.clone().into();
    assert_eq!(keys_list.len(), 4);
    assert_eq!(keys_list, [true, false, true, false]);

    let keys_dict: HashMap<String, bool> = result.into();

    // these dictionary entries should be true
    assert!(keys_dict[&key1]);
    assert!(keys_dict[&key3]);

    // these dictionary entries should be false
    assert!(!keys_dict[&key2]);
    assert!(!keys_dict[&key4]);

    Ok(())
}

#[tokio::test]
async fn increment_invalid_cache_name() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let result = client.increment("   ", "key", 1).await.unwrap_err();
    assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
    Ok(())
}

#[tokio::test]
async fn increment_nonexistent_cache() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = "fake-cache-".to_string() + &Uuid::new_v4().to_string();
    let result = client.increment(cache_name, "key", 1).await.unwrap_err();
    assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
    Ok(())
}

#[tokio::test]
async fn increment_happy_path() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = &CACHE_TEST_STATE.cache_name;
    let key = &Uuid::new_v4().to_string();

    // Incrementing a key that doesn't exist should create it
    let result = client.increment(cache_name, &**key, 1).await?;
    assert_eq!(result.value, 1);

    // Incrementing an existing key should increment it
    let result = client.increment(cache_name, &**key, 1).await?;
    assert_eq!(result.value, 2);

    // Incrementing by a negative number should decrement the value
    let result = client.increment(cache_name, &**key, -2).await?;
    assert_eq!(result.value, 0);

    Ok(())
}

#[tokio::test]
async fn item_get_type_invalid_cache_name() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let result = client.item_get_type("   ", "key").await.unwrap_err();
    assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
    Ok(())
}

#[tokio::test]
async fn item_get_type_nonexistent_cache() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = "fake-cache-".to_string() + &Uuid::new_v4().to_string();
    let result = client.item_get_type(cache_name, "key").await.unwrap_err();
    assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
    Ok(())
}

#[tokio::test]
async fn item_get_type_happy_path() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = &CACHE_TEST_STATE.cache_name;
    let key = &Uuid::new_v4().to_string();

    // Expect miss when key is not set
    let result = client.item_get_type(cache_name, &**key).await?;
    assert_eq!(result, ItemGetType::Miss {});
    client.delete(cache_name, &**key).await?;

    // Expect Scalar after using set
    client.set(cache_name, &**key, "value").await?;
    let result = client.item_get_type(cache_name, &**key).await?;
    match result {
        ItemGetType::Hit { key_type } => assert_eq!(
            key_type,
            ItemType::Scalar,
            "Expected Scalar, got {:?}",
            key_type
        ),
        _ => panic!("Expected Hit, got {:?}", result),
    }
    client.delete(cache_name, &**key).await?;

    // Expect Set after using setAddElements
    client
        .set_add_elements(cache_name, &**key, vec!["value1", "value2"])
        .await?;
    let result = client.item_get_type(cache_name, &**key).await?;
    match result {
        ItemGetType::Hit { key_type } => {
            assert_eq!(key_type, ItemType::Set, "Expected Set, got {:?}", key_type)
        }
        _ => panic!("Expected Hit, got {:?}", result),
    }
    client.delete(cache_name, &**key).await?;

    // Expect SortedSet after using sortedSetPutElements
    client
        .sorted_set_put_elements(cache_name, &**key, vec![("value1", 1.0), ("value2", 2.0)])
        .await?;
    let result = client.item_get_type(cache_name, &**key).await?;
    match result {
        ItemGetType::Hit { key_type } => assert_eq!(
            key_type,
            ItemType::SortedSet,
            "Expected SortedSet, got {:?}",
            key_type
        ),
        _ => panic!("Expected Hit, got {:?}", result),
    }
    client.delete(cache_name, &**key).await?;

    Ok(())
}

#[tokio::test]
async fn delete_invalid_cache_name() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let result = client.delete("   ", "key").await.unwrap_err();
    assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
    Ok(())
}

#[tokio::test]
async fn delete_nonexistent_cache() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = "fake-cache-".to_string() + &Uuid::new_v4().to_string();
    let result = client.delete(cache_name, "key").await.unwrap_err();
    assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
    Ok(())
}

#[tokio::test]
async fn delete_happy_path() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = &CACHE_TEST_STATE.cache_name;
    let key = &Uuid::new_v4().to_string();

    // Deleting a key that doesn't exist should not error
    let result = client.delete(cache_name, &**key).await?;
    assert_eq!(
        result,
        Delete {},
        "Expected successful Delete of nonexistent key, got {:?}",
        result
    );

    // Deleting a key that exists should delete it
    client.set(cache_name, &**key, "value").await?;
    let result = client.delete(cache_name, &**key).await?;
    assert_eq!(
        result,
        Delete {},
        "Expected successful Delete of existing key, got {:?}",
        result
    );

    // Key should not exist after deletion
    let result = client.key_exists(cache_name, &**key).await?;
    assert!(
        !result.exists,
        "Expected key 'key' to not exist in cache {} after deletion, but it does",
        cache_name
    );

    Ok(())
}
