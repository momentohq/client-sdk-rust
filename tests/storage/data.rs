use momento::storage::{
    DeleteResponse, GetResponse, SetRequest, SetResponse,
};
use momento::{MomentoErrorCode, MomentoResult};
use momento_test_util::{
    unique_store_name, unique_key, unique_string, TestScalar, CACHE_TEST_STATE,
};
use std::convert::TryInto;

mod get_set_delete {
    use super::*;

    #[tokio::test]
    async fn get_set_string() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.storage_client;
        let store_name = CACHE_TEST_STATE.store_name.as_str();
        let item = TestScalar::new();

        // Getting a key that doesn't exist should return a miss
        let result = client.get(store_name, item.key()).await?;
        assert_eq!(
            result,
            GetResponse::Miss,
            "Expected miss for key '{}' in store {}, got {:?}",
            item.key(),
            store_name,
            result
        );

        // Setting a key should return a hit
        let result = client.set(store_name, item.key(), item.value()).await?;
        assert_eq!(
            result,
            SetResponse {},
            "Expected successful Set of key '{}' in store {}, got {:?}",
            item.key(),
            store_name,
            result
        );

        // Getting the key should return a hit with the value
        let result = client.get(store_name, item.key()).await?;
        assert_eq!(
            result,
            GetResponse::from(&item),
            "Expected hit for key '{}' in store {}, got {:?}",
            item.key(),
            store_name,
            result
        );

        Ok(())
    }

    #[tokio::test]
    async fn delete_invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.storage_client;
        let result = client.delete("   ", "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn delete_nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.storage_client;
        let store_name = unique_store_name();
        let result = client.delete(store_name, "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn delete_happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.storage_client;
        let store_name = CACHE_TEST_STATE.store_name.as_str();
        let item = TestScalar::new();

        // Deleting a key that doesn't exist should not error
        let result = client.delete(store_name, item.key()).await?;
        assert_eq!(
            result,
            DeleteResponse {},
            "Expected successful Delete of nonexistent key, got {:?}",
            result
        );

        // Deleting a key that exists should delete it
        client.set(store_name, item.key(), item.value()).await?;
        let result = client.delete(store_name, item.key()).await?;
        assert_eq!(
            result,
            DeleteResponse {},
            "Expected successful Delete of existing key, got {:?}",
            result
        );

        // Key should not exist after deletion
        let result = client.key_exists(store_name, item.key()).await?;
        assert!(
            !result.exists,
            "Expected key 'key' to not exist in store {} after deletion, but it does",
            store_name
        );

        Ok(())
    }
}
