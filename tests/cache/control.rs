use momento::cache::{CreateCacheResponse, FlushCache, GetResponse, GetValue, SetResponse};
use momento::MomentoErrorCode;
use momento::MomentoResult;
use momento_test_util::CACHE_TEST_STATE;
use momento_test_util::{unique_cache_name, TestScalar};

mod create_delete_list_cache {
    use super::*;

    #[tokio::test]
    async fn delete_nonexistent_cache_returns_not_found() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let result = client.delete_cache(cache_name).await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn create_existing_cache_returns_already_exists() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let result = client.create_cache(cache_name).await?;
        assert_eq!(result, CreateCacheResponse::AlreadyExists {});
        Ok(())
    }
}

mod flush_cache {
    use momento::cache::DeleteCache;

    use super::*;

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
        assert!(
            cache_names.contains(cache_name),
            "Expected {} to be in list of caches: {:#?}",
            cache_name,
            cache_names
        );
        Ok(())
    }

    #[tokio::test]
    async fn flush_nonexistent_cache_returns_not_found() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let result = client.flush_cache(cache_name).await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    // Note: the flush_cache test requires creating its own cache as to not interfere with the other integration
    // tests that all share the same cache. Flushing the cache when other tests are running concurrently creates
    // a race condition and nondeterministic behavior.
    #[tokio::test]
    async fn flush_existing_cache_returns_success() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &unique_cache_name();

        // Create isolated cache for this test
        let create_result = client.create_cache(cache_name).await?;
        assert_eq!(create_result, CreateCacheResponse::Created {});

        // Insert some elements
        let item1 = TestScalar::new();
        let set_result1 = client.set(cache_name, item1.key(), item1.value()).await?;
        assert_eq!(set_result1, SetResponse {});

        let item2 = TestScalar::new();
        let set_result2 = client.set(cache_name, item2.key(), item2.value()).await?;
        assert_eq!(set_result2, SetResponse {});

        // Verify that the elements are in the cache
        let get_result1 = client.get(cache_name, item1.key()).await?;
        assert_eq!(
            get_result1,
            GetResponse::Hit {
                value: GetValue::new(item1.value().into())
            }
        );
        let get_result2 = client.get(cache_name, item2.key()).await?;
        assert_eq!(
            get_result2,
            GetResponse::Hit {
                value: GetValue::new(item2.value().into())
            }
        );

        // Flush the cache
        let result = client.flush_cache(cache_name).await?;
        assert_eq!(result, FlushCache {});

        // Verify that the elements were flushed from the cache
        let get_result3 = client.get(cache_name, item1.key()).await?;
        assert_eq!(get_result3, GetResponse::Miss {});
        let get_result4 = client.get(cache_name, item2.key()).await?;
        assert_eq!(get_result4, GetResponse::Miss {});

        // Delete the cache
        let delete_result = client.delete_cache(cache_name).await?;
        assert_eq!(delete_result, DeleteCache {});

        Ok(())
    }
}
