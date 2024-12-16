use momento::storage::{DeleteResponse, GetResponse, PutResponse};
use momento::{MomentoErrorCode, MomentoResult};
use momento_test_util::{unique_store_name, TestScalar, CACHE_TEST_STATE};

mod get_set_delete {
    use super::*;
    use momento_test_util::unique_key;

    #[tokio::test]
    async fn get_set_string() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.storage_client;
        let store_name = CACHE_TEST_STATE.store_name.as_str();
        let item = TestScalar::new();

        // Getting a key that doesn't exist should return a not found
        let result = client.get(store_name, item.key()).await?;
        assert_eq!(
            result,
            GetResponse::NotFound {},
            "Expected NotFound for key '{}' in store {}, got {:?}",
            item.key(),
            store_name,
            result
        );

        // Setting a key should return a success
        let result = client.put(store_name, item.key(), item.value()).await?;
        assert_eq!(
            result,
            PutResponse {},
            "Expected successful Set of key '{}' in store {}, got {:?}",
            item.key(),
            store_name,
            result
        );

        // Getting the key should return a success with the value
        let result = client.get(store_name, item.key()).await?;
        assert_eq!(
            result,
            GetResponse::from(&item.value),
            "Expected hit for key '{}' in store {}, got {:?}",
            item.key(),
            store_name,
            result
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_set_integer() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.storage_client;
        let store_name = CACHE_TEST_STATE.store_name.as_str();
        let key = unique_key();
        let value: i64 = 1;

        // Getting a key that doesn't exist should return a not found
        let result = client.get(store_name, &key).await?;
        assert_eq!(
            result,
            GetResponse::NotFound {},
            "Expected NotFound for key '{}' in store {}, got {:?}",
            key,
            store_name,
            result
        );

        // Setting a key should return a success
        let result = client.put(store_name, &key, &value).await?;
        assert_eq!(
            result,
            PutResponse {},
            "Expected successful Set of key '{}' in store {}, got {:?}",
            value,
            store_name,
            result
        );

        // Getting the key should return a success with the value
        let result = client.get(store_name, &key).await?;
        assert_eq!(
            result,
            GetResponse::from(&value),
            "Expected hit for key '{}' in store {}, got {:?}",
            key,
            store_name,
            result
        );

        Ok(())
    }

    #[tokio::test]
    async fn delete_invalid_store_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.storage_client;
        let result = client.delete("   ", "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn delete_nonexistent_store() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.storage_client;
        let store_name = unique_store_name();
        let result = client.delete(store_name, "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::StoreNotFoundError);
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
        client.put(store_name, item.key(), item.value()).await?;
        let result = client.delete(store_name, item.key()).await?;
        assert_eq!(
            result,
            DeleteResponse {},
            "Expected successful Delete of existing key, got {:?}",
            result
        );

        // Key should not exist after deletion
        let result = client.get(store_name, item.key()).await?;
        assert_eq!(
            result,
            GetResponse::NotFound {},
            "Expected NotFound for key '{}' in store {}, got {:?}",
            item.key(),
            store_name,
            result
        );

        Ok(())
    }
}
