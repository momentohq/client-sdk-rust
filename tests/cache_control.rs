use uuid::Uuid;

use momento_test_util::CACHE_TEST_STATE;

#[tokio::test]
async fn delete_nonexistent_cache_returns_not_found() {
    let client = CACHE_TEST_STATE.client.clone();
    let cache_name = "fake-cache-".to_string() + &Uuid::new_v4().to_string();
    let result = client.delete_cache(cache_name).await.unwrap_err();
    let _err_msg = "not found".to_string();
    assert!(matches!(result.to_string(), _err_message))
}

#[tokio::test]
async fn create_existing_cache_returns_already_exists() {
    let client = CACHE_TEST_STATE.client.clone();
    let cache_name = CACHE_TEST_STATE.cache_name.clone();
    let result = client.create_cache(cache_name).await.unwrap_err();
    let _err_msg = "already exists".to_string();
    assert!(matches!(result.to_string(), _err_message))
}

#[tokio::test]
async fn lists_existing_test_cache() {
    let client = CACHE_TEST_STATE.client.clone();
    let cache_name = CACHE_TEST_STATE.cache_name.clone();
    let result = client.list_caches().await.unwrap();
    let cache_names: Vec<String> = result
        .caches
        .iter()
        .map(|cache_info| cache_info.name.clone())
        .collect();
    assert!(cache_names.contains(&cache_name));
}
