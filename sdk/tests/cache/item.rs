use momento::{
    cache::{ItemGetTypeResponse, ItemType},
    MomentoErrorCode, MomentoResult,
};
use momento_test_util::{unique_cache_name, TestScalar, TestSet, TestSortedSet, CACHE_TEST_STATE};

mod item_get_type {
    use std::convert::TryInto;

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
        let cache_name = unique_cache_name();
        let result = client.item_get_type(cache_name, "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn happy_path_scalar() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item = TestScalar::new();

        // Expect miss when key is not set
        let result = client.item_get_type(cache_name, item.key()).await?;
        assert_eq!(result, ItemGetTypeResponse::Miss {});

        // Expect Scalar after using set
        client.set(cache_name, item.key(), item.value()).await?;
        let result = client.item_get_type(cache_name, item.key()).await?;
        let item_type: ItemType = result.try_into().expect("Expected ItemType, got Miss");
        assert_eq!(
            item_type,
            ItemType::Scalar,
            "Expected Scalar, got {:?}",
            item_type
        );
        Ok(())
    }

    #[tokio::test]
    async fn happy_path_set() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();

        // Expect Set after using setAddElements
        let item = TestSet::new();
        client
            .set_add_elements(cache_name, item.name(), item.value().to_vec())
            .await?;
        let result = client.item_get_type(cache_name, item.name()).await?;
        let item_type: ItemType = result.try_into().expect("Expected ItemType, got Miss");
        assert_eq!(
            item_type,
            ItemType::Set,
            "Expected Set, got {:?}",
            item_type
        );
        Ok(())
    }

    #[tokio::test]
    async fn happy_path_sorted_set() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();

        // Expect SortedSet after using sortedSetPutElements
        let item = TestSortedSet::new();
        client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;
        let result = client.item_get_type(cache_name, item.name()).await?;
        let item_type: ItemType = result.try_into().expect("Expected ItemType, got Miss");
        assert_eq!(
            item_type,
            ItemType::SortedSet,
            "Expected SortedSet, got {:?}",
            item_type
        );
        Ok(())
    }
}
