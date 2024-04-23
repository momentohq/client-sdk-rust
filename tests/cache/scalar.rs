use momento::{cache::Delete, MomentoErrorCode, MomentoResult};
use momento_test_util::{unique_cache_name, unique_key, TestScalar, CACHE_TEST_STATE};

mod get_set_delete {
    use super::*;

    #[tokio::test]
    async fn delete_invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client.delete("   ", "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn delete_nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let result = client.delete(cache_name, "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn delete_happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item = TestScalar::new();

        // Deleting a key that doesn't exist should not error
        let result = client.delete(cache_name, item.key()).await?;
        assert_eq!(
            result,
            Delete {},
            "Expected successful Delete of nonexistent key, got {:?}",
            result
        );

        // Deleting a key that exists should delete it
        client.set(cache_name, item.key(), item.value()).await?;
        let result = client.delete(cache_name, item.key()).await?;
        assert_eq!(
            result,
            Delete {},
            "Expected successful Delete of existing key, got {:?}",
            result
        );

        // Key should not exist after deletion
        let result = client.key_exists(cache_name, item.key()).await?;
        assert!(
            !result.exists,
            "Expected key 'key' to not exist in cache {} after deletion, but it does",
            cache_name
        );

        Ok(())
    }
}

mod increment {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client.increment("   ", "key", 1).await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let result = client.increment(cache_name, "key", 1).await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key = unique_key();
        let key = key.as_str();

        // Incrementing a key that doesn't exist should create it
        let result = client.increment(cache_name, key, 1).await?;
        assert_eq!(result.value, 1);

        // Incrementing an existing key should increment it
        let result = client.increment(cache_name, key, 1).await?;
        assert_eq!(result.value, 2);

        // Incrementing by a negative number should decrement the value
        let result = client.increment(cache_name, key, -2).await?;
        assert_eq!(result.value, 0);

        Ok(())
    }
}

mod set_if_not_exists {}

mod set_if_absent {}

mod set_if_present {}

mod set_if_equal {}

mod set_if_not_equal {}

mod set_if_present_and_not_equal {}

mod set_if_absent_or_equal {}

mod when_readconcern_is_specified {}
