use std::collections::HashMap;

use momento::{MomentoErrorCode, MomentoResult};
use momento_test_util::{unique_cache_name, TestScalar, CACHE_TEST_STATE};

mod key_exists {
    use momento_test_util::{unique_cache_name, TestScalar};

    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client.key_exists("   ", "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let result = client.key_exists(cache_name, "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item = TestScalar::new();

        // Key should not exist yet
        let result = client.key_exists(cache_name, item.key()).await?;
        assert!(
            !result.exists,
            "Expected key {} to not exist in cache {}, but it does",
            item.key(),
            cache_name
        );

        // Key should exist after setting a key
        client.set(cache_name, item.key(), item.value()).await?;
        let result = client.key_exists(cache_name, item.key()).await?;
        assert!(
            result.exists,
            "Expected key {} to exist in cache {}, but it does not",
            item.key(),
            cache_name
        );

        Ok(())
    }
}

mod keys_exists {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client.keys_exist("   ", vec!["key"]).await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let result = client
            .keys_exist(cache_name, vec!["key"])
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();

        // Should return empty list if given empty key list
        let empty_vector: Vec<String> = vec![];
        let keys_received: Vec<bool> = client.keys_exist(cache_name, empty_vector).await?.into();
        assert!(
            keys_received.is_empty(),
            "Expected empty list of bools given no keys, but received {:#?}",
            keys_received
        );

        // Key should return true only for keys that exist in the cache
        let items = (0..4)
            .map(|_| TestScalar::new())
            .collect::<Vec<TestScalar>>();

        client
            .set(cache_name, items[0].key(), items[0].value())
            .await?;
        client
            .set(cache_name, items[2].key(), items[2].value())
            .await?;

        let result = client
            .keys_exist(
                cache_name,
                items
                    .iter()
                    .map(|item| item.key().to_string())
                    .collect::<Vec<String>>(),
            )
            .await?;

        let keys_list: Vec<bool> = result.clone().into();
        assert_eq!(keys_list.len(), 4);
        assert_eq!(keys_list, [true, false, true, false]);

        let keys_dict: HashMap<String, bool> = result.into();

        // these dictionary entries should be true
        assert!(
            keys_dict[items[0].key()],
            "Key {} should exist",
            items[0].key()
        );
        assert!(
            keys_dict[items[2].key()],
            "Key {} should exist",
            items[2].key()
        );

        // these dictionary entries should be false
        assert!(
            !keys_dict[items[1].key()],
            "Key {} should not exist",
            items[1].key()
        );
        assert!(
            !keys_dict[items[3].key()],
            "Key {} should not exist",
            items[3].key()
        );

        Ok(())
    }
}
