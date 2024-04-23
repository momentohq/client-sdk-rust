use std::collections::HashMap;

use momento::{MomentoErrorCode, MomentoResult};
use momento_test_util::{unique_string, CACHE_TEST_STATE};

mod key_exists {
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
        let cache_name = unique_string("fake-cache");
        let result = client.key_exists(cache_name, "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();

        // Key should not exist yet
        let result = client.key_exists(cache_name, key).await?;
        assert!(
            !result.exists,
            "Expected key {} to not exist in cache {}, but it does",
            key, cache_name
        );

        // Key should exist after setting a key
        client.set(cache_name, key, "value").await?;
        let result = client.key_exists(cache_name, key).await?;
        assert!(
            result.exists,
            "Expected key {} to exist in cache {}, but it does not",
            key, cache_name
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
        let cache_name = unique_string("fake-cache");
        let result = client
            .keys_exist(cache_name, vec!["key"])
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
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
        let unique_key1 = unique_string("keys-exist");
        let key1 = unique_key1.as_str();

        let unique_key2 = unique_string("keys-exist");
        let key2 = unique_key2.as_str();

        let unique_key3 = unique_string("keys-exist");
        let key3 = unique_key3.as_str();

        let unique_key4 = unique_string("keys-exist");
        let key4 = unique_key4.as_str();

        client.set(cache_name, key1, key1).await?;
        client.set(cache_name, key3, key3).await?;

        let result = client
            .keys_exist(cache_name, vec![key1, key2, key3, key4])
            .await?;

        let keys_list: Vec<bool> = result.clone().into();
        assert_eq!(keys_list.len(), 4);
        assert_eq!(keys_list, [true, false, true, false]);

        let keys_dict: HashMap<String, bool> = result.into();

        // these dictionary entries should be true
        assert!(keys_dict[key1]);
        assert!(keys_dict[key3]);

        // these dictionary entries should be false
        assert!(!keys_dict[key2]);
        assert!(!keys_dict[key4]);

        Ok(())
    }
}
