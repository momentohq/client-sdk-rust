use momento::{requests::MomentoErrorCode, MomentoResult};
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

    // Key should not exist yet
    let result = client.key_exists(cache_name, "key").await?;
    assert!(
        !result.exists,
        "Expected key 'key' to not exist in cache {}, but it does",
        cache_name
    );

    // Key should exist after setting a key
    client.set(cache_name, "key", "value").await?;
    let result = client.key_exists(cache_name, "key").await?;
    assert!(
        result.exists,
        "Expected key 'key' to exist in cache {}, but it does not",
        cache_name
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
    let result = client.keys_exist(cache_name, empty_vector).await?;
    assert!(
        result.exists.is_empty(),
        "Expected empty list of bools given no keys, but received {:#?}",
        result.exists
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
    assert_eq!(result.exists.len(), 4);
    assert_eq!(result.exists, [true, false, true, false]);

    // these dictionary entries should be true
    assert!(result.exists_dictionary[&key1]);
    assert!(result.exists_dictionary[&key3]);

    // these dictionary entries should be false
    assert!(!result.exists_dictionary[&key2]);
    assert!(!result.exists_dictionary[&key4]);

    Ok(())
}
