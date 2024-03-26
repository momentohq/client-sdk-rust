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

    // Find test cache and check limits match expected for alpha cell
    let cache_info = result
        .caches
        .iter()
        .find(|cache_info| cache_info.name == cache_name)
        .unwrap();

    let expected_throughput_limit = 10240;
    let expected_item_size_limit = 4883;
    let expected_throttling_limit = 1000;
    let expected_max_ttl = 86400;
    let expected_publish_rate = 100;
    let expected_subscription_count = 100;
    let expected_publish_message_size = 100;

    assert_eq!(
        cache_info.cache_limits.max_throughput_kbps,
        expected_throughput_limit
    );
    assert_eq!(
        cache_info.cache_limits.max_item_size_kb,
        expected_item_size_limit
    );
    assert_eq!(
        cache_info.cache_limits.max_traffic_rate,
        expected_throttling_limit
    );
    assert_eq!(cache_info.cache_limits.max_ttl_seconds, expected_max_ttl);
    assert_eq!(
        cache_info.topic_limits.max_publish_rate,
        expected_publish_rate
    );
    assert_eq!(
        cache_info.topic_limits.max_subscription_count,
        expected_subscription_count
    );
    assert_eq!(
        cache_info.topic_limits.max_publish_message_size_kb,
        expected_publish_message_size
    );
}
