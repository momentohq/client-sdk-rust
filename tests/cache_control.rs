use momento::requests::cache::create_cache::CreateCache;
use momento::requests::cache::{
    basic::{
        get::{Get, GetValue},
        set::Set,
    },
    flush_cache::FlushCache,
};
use momento::MomentoResult;
use uuid::Uuid;

use momento_test_util::CACHE_TEST_STATE;

#[tokio::test]
async fn delete_nonexistent_cache_returns_not_found() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = "fake-cache-".to_string() + &Uuid::new_v4().to_string();
    let result = client.delete_cache(cache_name).await.unwrap_err();
    let _err_msg = "not found".to_string();
    assert!(matches!(result.to_string(), _err_message));
    Ok(())
}

#[tokio::test]
async fn create_existing_cache_returns_already_exists() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = &CACHE_TEST_STATE.cache_name;
    let result = client.create_cache(cache_name).await?;
    assert_eq!(result, CreateCache::AlreadyExists {});
    Ok(())
}

#[tokio::test]
async fn lists_existing_test_cache() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = &CACHE_TEST_STATE.cache_name;
    let result = client.list_caches().await?;
    let cache_names: Vec<String> = result
        .caches
        .iter()
        .map(|cache_info| cache_info.name.clone())
        .collect();
    assert!(cache_names.contains(cache_name));
    Ok(())
}

#[tokio::test]
async fn flush_nonexistent_cache_returns_not_found() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = "fake-cache-".to_string() + &Uuid::new_v4().to_string();
    let result = client.flush_cache(cache_name).await.unwrap_err();
    let _err_msg = "not found".to_string();
    assert!(matches!(result.to_string(), _err_message));
    Ok(())
}

#[tokio::test]
async fn flush_existing_cache_returns_success() -> MomentoResult<()> {
    let client = &CACHE_TEST_STATE.client;
    let cache_name = &CACHE_TEST_STATE.cache_name;

    // Insert some elements
    let set_result1 = client.set(cache_name, "key1", "value1").await?;
    assert_eq!(set_result1, Set {});
    let set_result2 = client.set(cache_name, "key2", "value2").await?;
    assert_eq!(set_result2, Set {});

    // Verify that the elements are in the cache
    let get_result1 = client.get(cache_name, "key1").await?;
    assert_eq!(
        get_result1,
        Get::Hit {
            value: GetValue::new("value1".into())
        }
    );
    let get_result2 = client.get(cache_name, "key2").await?;
    assert_eq!(
        get_result2,
        Get::Hit {
            value: GetValue::new("value2".into())
        }
    );

    // Flush the cache
    let result = client.flush_cache(cache_name).await?;
    assert_eq!(result, FlushCache {});

    // Verify that the elements were flushed from the cache
    let get_result3 = client.get(cache_name, "key1").await?;
    assert_eq!(get_result3, Get::Miss {});
    let get_result4 = client.get(cache_name, "key2").await?;
    assert_eq!(get_result4, Get::Miss {});

    Ok(())
}
