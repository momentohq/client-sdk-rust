use std::convert::TryInto;
use std::time::Duration;

use momento::cache::{
    DecreaseTtl, IncreaseTtl, ItemGetTtl, SetRequest, SortedSetPutElementsRequest, UpdateTtl,
};
use momento::{CollectionTtl, MomentoErrorCode, MomentoResult};
use momento_test_util::{unique_string, CACHE_TEST_STATE};

mod item_get_ttl {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client.item_get_ttl("   ", "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_string("fake-cache");
        let result = client.item_get_ttl(cache_name, "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_key() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();
        let result = client.item_get_ttl(cache_name, key).await?;
        assert_eq!(result, ItemGetTtl::Miss {});
        Ok(())
    }

    #[tokio::test]
    async fn get_ttl_for_a_scalar() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();

        client
            .send_request(SetRequest::new(cache_name, key, "value").ttl(Duration::from_secs(2)))
            .await?;

        // Should get a HIT before ttl expires
        let ttl: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected an item ttl!");
        assert!(ttl.as_secs() > 0);

        // Sleep for 2 seconds
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should get a MISS after ttl expires
        let result = client.item_get_ttl(cache_name, key).await?;
        assert_eq!(result, ItemGetTtl::Miss {});
        Ok(())
    }

    #[tokio::test]
    async fn get_ttl_for_a_sorted_set() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();

        // Create a sorted set that expires in 2 seconds
        client
            .send_request(
                SortedSetPutElementsRequest::new(
                    cache_name,
                    key,
                    vec![("abc", 12.3), ("xyz", 32.1)],
                )
                .ttl(CollectionTtl::new(Some(Duration::from_secs(2)), true)),
            )
            .await?;

        // Should get a HIT before ttl expires
        let ttl: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected an item ttl!");
        assert!(ttl.as_secs() > 0);

        // Sleep for 2 seconds
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should get a MISS after ttl expires
        let result = client.item_get_ttl(cache_name, key).await?;
        assert_eq!(result, ItemGetTtl::Miss {});
        Ok(())
    }
}

mod increase_ttl {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .increase_ttl("   ", "key", Duration::from_secs(5))
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_string("fake-cache");
        let result = client
            .increase_ttl(cache_name, "key", Duration::from_secs(5))
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_key() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();
        let result = client
            .increase_ttl(cache_name, key, Duration::from_secs(10))
            .await?;
        assert_eq!(result, IncreaseTtl::Miss {});
        Ok(())
    }

    #[tokio::test]
    async fn only_increases_ttl_for_existing_key() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();

        // Set a low TTL
        client
            .send_request(SetRequest::new(cache_name, key, "value").ttl(Duration::from_secs(5)))
            .await?;

        let ttl_before: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected an item ttl!");
        assert!(ttl_before.as_secs() > 0 && ttl_before.as_secs() < 5);

        // Set a higher TTL
        let result = client
            .increase_ttl(cache_name, key, Duration::from_secs(20))
            .await?;
        assert_eq!(result, IncreaseTtl::Set {});

        let ttl_after: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected an item ttl!");
        assert!(ttl_after.as_secs() > 15 && ttl_after.as_secs() < 20);

        // Setting TTL lower than current TTL should not change the TTL
        let result = client
            .increase_ttl(cache_name, key, Duration::from_secs(10))
            .await?;
        assert_eq!(result, IncreaseTtl::NotSet {});

        let ttl_lower: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected an item ttl!");
        assert!(ttl_lower.as_secs() > 15 && ttl_lower.as_secs() < 20);
        Ok(())
    }
}

mod decrease_ttl {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .decrease_ttl("   ", "key", Duration::from_secs(5))
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_string("fake-cache");
        let result = client
            .decrease_ttl(cache_name, "key", Duration::from_secs(5))
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_key() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();
        let result = client
            .decrease_ttl(cache_name, key, Duration::from_secs(1))
            .await?;
        assert_eq!(result, DecreaseTtl::Miss {});
        Ok(())
    }

    #[tokio::test]
    async fn only_decreases_ttl_for_existing_key() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();

        // Set a high TTL
        client
            .send_request(SetRequest::new(cache_name, key, "value").ttl(Duration::from_secs(20)))
            .await?;

        let ttl_before: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected an item ttl!");
        assert!(ttl_before.as_secs() > 15 && ttl_before.as_secs() < 20);

        // Set a lower TTL
        let result = client
            .decrease_ttl(cache_name, key, Duration::from_secs(5))
            .await?;
        assert_eq!(result, DecreaseTtl::Set {});

        let ttl_after: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected an item ttl!");
        assert!(ttl_after.as_secs() > 0 && ttl_after.as_secs() < 5);

        // Setting TTL higher than current TTL should not change the TTL
        let result = client
            .decrease_ttl(cache_name, key, Duration::from_secs(10))
            .await?;
        assert_eq!(result, DecreaseTtl::NotSet {});

        let ttl_lower: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected an item ttl!");
        assert!(ttl_lower.as_secs() > 0 && ttl_lower.as_secs() < 5);
        Ok(())
    }
}

mod update_ttl {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .update_ttl("   ", "key", Duration::from_secs(5))
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_string("fake-cache");
        let result = client
            .update_ttl(cache_name, "key", Duration::from_secs(5))
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_key() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();
        let result = client
            .update_ttl(cache_name, key, Duration::from_secs(5))
            .await?;
        assert_eq!(result, UpdateTtl::Miss {});
        Ok(())
    }

    #[tokio::test]
    async fn overwrites_ttl_for_existing_key() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();

        client
            .send_request(SetRequest::new(cache_name, key, "value").ttl(Duration::from_secs(10)))
            .await?;

        let ttl_before: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected an item ttl!");
        assert!(ttl_before.as_secs() > 0 && ttl_before.as_secs() < 10);

        client
            .update_ttl(cache_name, key, Duration::from_secs(20))
            .await?;

        let ttl_after: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected an item ttl!");
        assert!(ttl_after.as_secs() > 10 && ttl_after.as_secs() < 20);
        Ok(())
    }
}
