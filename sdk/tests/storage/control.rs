use momento::storage::CreateStoreResponse;
use momento::MomentoErrorCode;
use momento::MomentoResult;
use momento_test_util::unique_store_name;
use momento_test_util::CACHE_TEST_STATE;

mod create_delete_list_store {
    use super::*;

    #[tokio::test]
    async fn delete_nonexistent_store_returns_not_found() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.storage_client;
        let store_name = unique_store_name();
        let result = client.delete_store(store_name).await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn create_existing_store_returns_already_exists() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.storage_client;
        let store_name = &CACHE_TEST_STATE.store_name;
        let result = client.create_store(store_name).await?;
        assert_eq!(result, CreateStoreResponse::AlreadyExists {});
        Ok(())
    }

    #[tokio::test]
    async fn list_stores_contains_test_store() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.storage_client;
        let store_name = &CACHE_TEST_STATE.store_name;
        let result = client.list_stores().await?;
        assert!(
            result.stores.iter().any(|x| x.name == *store_name),
            "The list of stores does not contain the test store"
        );
        Ok(())
    }
}
