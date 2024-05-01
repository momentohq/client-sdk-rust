use momento::cache::{CreateCache, FlushCache, Get, GetValue, Set};
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
        assert_eq!(result, CreateCache::AlreadyExists {});
        Ok(())
    }
}

mod flush_cache {
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

    // This test uses the ignore macro so that we can run it separately from the other
    // test targets using `cargo test -- --ignored`. This is because all the tests share
    // the same cache and flushing it will affect the other tests in nondeterministic ways.
    #[tokio::test]
    #[ignore]
    async fn flush_existing_cache_returns_success() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;

        // Insert some elements
        let item1 = TestScalar::new();
        let set_result1 = client.set(cache_name, item1.key(), item1.value()).await?;
        assert_eq!(set_result1, Set {});

        let item2 = TestScalar::new();
        let set_result2 = client.set(cache_name, item2.key(), item2.value()).await?;
        assert_eq!(set_result2, Set {});

        // Verify that the elements are in the cache
        let get_result1 = client.get(cache_name, item1.key()).await?;
        assert_eq!(
            get_result1,
            Get::Hit {
                value: GetValue::new(item1.value().into())
            }
        );
        let get_result2 = client.get(cache_name, item2.key()).await?;
        assert_eq!(
            get_result2,
            Get::Hit {
                value: GetValue::new(item2.value().into())
            }
        );

        // Flush the cache
        let result = client.flush_cache(cache_name).await?;
        assert_eq!(result, FlushCache {});

        // Verify that the elements were flushed from the cache
        let get_result3 = client.get(cache_name, item1.key()).await?;
        assert_eq!(get_result3, Get::Miss {});
        let get_result4 = client.get(cache_name, item2.key()).await?;
        assert_eq!(get_result4, Get::Miss {});

        Ok(())
    }
}
