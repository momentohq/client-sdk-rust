use momento::{
    cache::{ItemGetType, ItemType},
    MomentoErrorCode, MomentoResult,
};
use momento_test_util::{unique_string, CACHE_TEST_STATE};

mod item_get_type {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client.item_get_type("   ", "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_string("fake-cache");
        let result = client.item_get_type(cache_name, "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();

        // Expect miss when key is not set
        let result = client.item_get_type(cache_name, key).await?;
        assert_eq!(result, ItemGetType::Miss {});
        client.delete(cache_name, key).await?;

        // Expect Scalar after using set
        client.set(cache_name, key, "value").await?;
        let result = client.item_get_type(cache_name, key).await?;
        match result {
            ItemGetType::Hit { key_type } => assert_eq!(
                key_type,
                ItemType::Scalar,
                "Expected Scalar, got {:?}",
                key_type
            ),
            _ => panic!("Expected Hit, got {:?}", result),
        }
        client.delete(cache_name, key).await?;

        // Expect Set after using setAddElements
        client
            .set_add_elements(cache_name, key, vec!["value1", "value2"])
            .await?;
        let result = client.item_get_type(cache_name, key).await?;
        match result {
            ItemGetType::Hit { key_type } => {
                assert_eq!(key_type, ItemType::Set, "Expected Set, got {:?}", key_type)
            }
            _ => panic!("Expected Hit, got {:?}", result),
        }
        client.delete(cache_name, key).await?;

        // Expect SortedSet after using sortedSetPutElements
        client
            .sorted_set_put_elements(cache_name, key, vec![("value1", 1.0), ("value2", 2.0)])
            .await?;
        let result = client.item_get_type(cache_name, key).await?;
        match result {
            ItemGetType::Hit { key_type } => assert_eq!(
                key_type,
                ItemType::SortedSet,
                "Expected SortedSet, got {:?}",
                key_type
            ),
            _ => panic!("Expected Hit, got {:?}", result),
        }
        client.delete(cache_name, key).await?;

        Ok(())
    }
}

mod item_get_ttl {}
