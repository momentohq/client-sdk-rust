use momento::cache::{
    DeleteResponse, Get, Set, SetIfAbsent, SetIfAbsentOrEqual, SetIfAbsentOrEqualRequest,
    SetIfAbsentRequest, SetIfEqual, SetIfEqualRequest, SetIfNotEqual, SetIfNotEqualRequest,
    SetIfPresent, SetIfPresentAndNotEqual, SetIfPresentAndNotEqualRequest, SetIfPresentRequest,
    SetRequest,
};
use momento::{MomentoErrorCode, MomentoResult};
use momento_test_util::{
    unique_cache_name, unique_key, unique_string, TestScalar, CACHE_TEST_STATE,
};
use std::{convert::TryInto, time::Duration};

mod get_set_delete {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn get_set_string() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item = TestScalar::new();

        // Getting a key that doesn't exist should return a miss
        let result = client.get(cache_name, item.key()).await?;
        assert_eq!(
            result,
            Get::Miss,
            "Expected miss for key '{}' in cache {}, got {:?}",
            item.key(),
            cache_name,
            result
        );

        // Setting a key should return a hit
        let result = client.set(cache_name, item.key(), item.value()).await?;
        assert_eq!(
            result,
            Set {},
            "Expected successful Set of key '{}' in cache {}, got {:?}",
            item.key(),
            cache_name,
            result
        );

        // Getting the key should return a hit with the value
        let result = client.get(cache_name, item.key()).await?;
        assert_eq!(
            result,
            Get::from(&item),
            "Expected hit for key '{}' in cache {}, got {:?}",
            item.key(),
            cache_name,
            result
        );

        Ok(())
    }

    #[tokio::test]
    async fn set_with_nonnegative_ttl_is_ok() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        for ttl in [0, 1] {
            let item = TestScalar::new();
            let set_request =
                SetRequest::new(cache_name, item.key(), item.value()).ttl(Duration::from_secs(ttl));
            let result = client.send_request(set_request).await?;
            assert_eq!(result, Set {});
        }
        Ok(())
    }

    #[tokio::test]
    async fn set_with_ttl_is_miss_after_expiration() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item = TestScalar::new();
        let ttl = Duration::from_secs(1);
        let set_request = SetRequest::new(cache_name, item.key(), item.value()).ttl(ttl);
        let result = client.send_request(set_request).await?;
        assert_eq!(result, Set {});

        // Wait for the TTL to expire
        tokio::time::sleep(ttl).await;

        // Getting the key should return a miss
        let result = client.get(cache_name, item.key()).await?;
        assert_eq!(
            result,
            Get::Miss,
            "Expected miss for key '{}' in cache {}, got {:?}",
            item.key(),
            cache_name,
            result
        );

        Ok(())
    }

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
            DeleteResponse {},
            "Expected successful Delete of nonexistent key, got {:?}",
            result
        );

        // Deleting a key that exists should delete it
        client.set(cache_name, item.key(), item.value()).await?;
        let result = client.delete(cache_name, item.key()).await?;
        assert_eq!(
            result,
            DeleteResponse {},
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

    #[tokio::test]
    async fn fails_when_value_is_not_an_integer() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item = TestScalar::new();

        // Set a non-string value
        client.set(cache_name, item.key(), item.value()).await?;

        // Incrementing the key should fail
        let result = client
            .increment(cache_name, item.key(), 1)
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::FailedPreconditionError);

        Ok(())
    }
}

mod set_if_absent {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .set_if_absent("   ", "key", "value")
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
            .set_if_absent(cache_name, "key", "value")
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item1 = TestScalar::new();
        let item2 = TestScalar::new();

        // Setting a key that doesn't exist should create it
        let result = client
            .set_if_absent(cache_name, item1.key(), item1.value())
            .await?;
        assert_eq!(result, SetIfAbsent::Stored);
        let result = client.get(cache_name, item1.key()).await?;
        assert_eq!(result, Get::from(&item1));

        // Setting a key that exists should not overwrite it
        let result = client
            .set_if_absent(cache_name, item1.key(), item2.value())
            .await?;
        assert_eq!(result, SetIfAbsent::NotStored);
        let result = client.get(cache_name, item1.key()).await?;
        assert_eq!(result, Get::from(&item1));

        Ok(())
    }

    // string key and string value with ttl before and after expiration
    #[tokio::test]
    async fn happy_path_with_ttl() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item = TestScalar::new();

        let set_request = SetIfAbsentRequest::new(cache_name, item.key(), item.value())
            .ttl(Duration::from_secs(2));
        let result = client.send_request(set_request).await?;
        assert_eq!(result, SetIfAbsent::Stored);

        // Should have remaining ttl > 0
        let result_ttl: Duration = client
            .item_get_ttl(cache_name, item.key())
            .await?
            .try_into()?;
        assert!(result_ttl.as_millis() > 0);

        // Wait for ttl to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should get a cache miss
        let result = client.get(cache_name, item.key()).await?;
        assert_eq!(result, Get::Miss {});

        Ok(())
    }
}

mod set_if_present {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .set_if_present("   ", "key", "value")
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
            .set_if_present(cache_name, "key", "value")
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item1 = TestScalar::new();
        let item2 = TestScalar::new();

        // Setting a key that doesn't exist should not create it
        let result = client
            .set_if_present(cache_name, item1.key(), item1.value())
            .await?;
        assert_eq!(result, SetIfPresent::NotStored);

        let result = client.set(cache_name, item1.key(), item1.value()).await?;
        assert_eq!(result, Set {});

        // Setting a key that exists should overwrite it
        let result = client
            .set_if_present(cache_name, item1.key(), item2.value())
            .await?;
        assert_eq!(result, SetIfPresent::Stored);
        let result = client.get(cache_name, item1.key()).await?;
        assert_eq!(result, Get::from(&item2));

        Ok(())
    }

    // string key and string value with ttl before and after expiration
    #[tokio::test]
    async fn happy_path_with_ttl() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item = TestScalar::new();

        let result = client.set(cache_name, item.key(), item.value()).await?;
        assert_eq!(result, Set {});

        let set_request = SetIfPresentRequest::new(cache_name, item.key(), "second_value")
            .ttl(Duration::from_secs(2));
        let result = client.send_request(set_request).await?;
        assert_eq!(result, SetIfPresent::Stored);

        // Should have remaining ttl > 0
        let result_ttl: Duration = client
            .item_get_ttl(cache_name, item.key())
            .await?
            .try_into()?;
        assert!(result_ttl.as_millis() > 0);

        // Wait for ttl to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should get a cache miss
        let result = client.get(cache_name, item.key()).await?;
        assert_eq!(result, Get::Miss {});

        Ok(())
    }
}

mod set_if_equal {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .set_if_equal("   ", "key", "value", "equal")
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
            .set_if_equal(cache_name, "key", "value", "equal")
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item1 = TestScalar::new();
        let item2 = TestScalar::new();

        // Setting a key that doesn't exist should not create it
        let result = client
            .set_if_equal(cache_name, item1.key(), item1.value(), item1.value())
            .await?;
        assert_eq!(result, SetIfEqual::NotStored);

        let result = client.set(cache_name, item1.key(), item1.value()).await?;
        assert_eq!(result, Set {});

        // Setting a key that exists and equals the value should overwrite it
        let result = client
            .set_if_equal(cache_name, item1.key(), item2.value(), item1.value())
            .await?;
        assert_eq!(result, SetIfEqual::Stored);
        let result = client.get(cache_name, item1.key()).await?;
        assert_eq!(result, Get::from(&item2));

        // Setting a key that exists and does NOT equal the value should NOT overwrite it
        let result = client
            .set_if_equal(cache_name, item1.key(), item1.value(), item1.value())
            .await?;
        assert_eq!(result, SetIfEqual::NotStored);
        let result = client.get(cache_name, item1.key()).await?;
        assert_eq!(result, Get::from(&item2));

        Ok(())
    }

    // string key and string value with ttl before and after expiration
    #[tokio::test]
    async fn happy_path_with_ttl() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();

        let result = client.set(cache_name, key, "first_value").await?;
        assert_eq!(result, Set {});

        let set_request = SetIfEqualRequest::new(cache_name, key, "second_value", "first_value")
            .ttl(Duration::from_secs(2));
        let result = client.send_request(set_request).await?;
        assert_eq!(result, SetIfEqual::Stored);

        // Should have remaining ttl > 0
        let result_ttl: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected to get an item ttl");
        assert!(result_ttl.as_millis() > 0);

        // Wait for ttl to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should get a cache miss
        let result = client.get(cache_name, key).await?;
        assert_eq!(result, Get::Miss {});

        Ok(())
    }
}

mod set_if_not_equal {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .set_if_not_equal("   ", "key", "value", "not-equal")
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
            .set_if_not_equal(cache_name, "key", "value", "not-equal")
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item1 = TestScalar::new();
        let item2 = TestScalar::new();

        // Setting a key that doesn't exist should create it
        let result = client
            .set_if_not_equal(cache_name, item1.key(), item1.value(), item1.value())
            .await?;
        assert_eq!(result, SetIfNotEqual::Stored);

        let result = client.set(cache_name, item1.key(), item1.value()).await?;
        assert_eq!(result, Set {});

        // Setting a key that exists and does not equal the value should overwrite it
        let result = client
            .set_if_not_equal(cache_name, item1.key(), item2.value(), item2.value())
            .await?;
        assert_eq!(result, SetIfNotEqual::Stored);
        let result = client.get(cache_name, item1.key()).await?;
        assert_eq!(result, Get::from(&item2));

        // Setting a key that exists and equals the value should NOT overwrite it
        let result = client
            .set_if_not_equal(cache_name, item1.key(), item1.value(), item2.value())
            .await?;
        assert_eq!(result, SetIfNotEqual::NotStored);
        let result = client.get(cache_name, item1.key()).await?;
        assert_eq!(result, Get::from(&item2));

        Ok(())
    }

    // string key and string value with ttl before and after expiration
    #[tokio::test]
    async fn happy_path_with_ttl() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item = TestScalar::new();

        let result = client.set(cache_name, item.key(), item.value()).await?;
        assert_eq!(result, Set {});

        let set_request =
            SetIfNotEqualRequest::new(cache_name, item.key(), "second_value", "second_value")
                .ttl(Duration::from_secs(2));
        let result = client.send_request(set_request).await?;
        assert_eq!(result, SetIfNotEqual::Stored);

        // Should have remaining ttl > 0
        let result_ttl: Duration = client
            .item_get_ttl(cache_name, item.key())
            .await?
            .try_into()?;
        assert!(result_ttl.as_millis() > 0);

        // Wait for ttl to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should get a cache miss
        let result = client.get(cache_name, item.key()).await?;
        assert_eq!(result, Get::Miss {});

        Ok(())
    }
}

mod set_if_present_and_not_equal {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .set_if_present_and_not_equal("   ", "key", "value", "not-equal")
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
            .set_if_present_and_not_equal(cache_name, "key", "value", "not-equal")
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item1 = TestScalar::new();
        let item2 = TestScalar::new();

        // Setting a key that doesn't exist should not create it
        let result = client
            .set_if_present_and_not_equal(cache_name, item1.key(), item1.value(), item1.value())
            .await?;
        assert_eq!(result, SetIfPresentAndNotEqual::NotStored);

        let result = client.set(cache_name, item1.key(), item1.value()).await?;
        assert_eq!(result, Set {});

        // Setting a key that exists and does not equal the value should overwrite it
        let result = client
            .set_if_present_and_not_equal(cache_name, item1.key(), item2.value(), item2.value())
            .await?;
        assert_eq!(result, SetIfPresentAndNotEqual::Stored);
        let result = client.get(cache_name, item1.key()).await?;
        assert_eq!(result, Get::from(&item2));

        // Setting a key that exists and equals the value should NOT overwrite it
        let result = client
            .set_if_present_and_not_equal(cache_name, item1.key(), item1.value(), item2.value())
            .await?;
        assert_eq!(result, SetIfPresentAndNotEqual::NotStored);
        let result = client.get(cache_name, item1.key()).await?;
        assert_eq!(result, Get::from(&item2));

        Ok(())
    }

    // string key and string value with ttl before and after expiration
    #[tokio::test]
    async fn happy_path_with_ttl() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();

        let result = client.set(cache_name, key, "first_value").await?;
        assert_eq!(result, Set {});

        let set_request =
            SetIfPresentAndNotEqualRequest::new(cache_name, key, "second_value", "second_value")
                .ttl(Duration::from_secs(2));
        let result = client.send_request(set_request).await?;
        assert_eq!(result, SetIfPresentAndNotEqual::Stored);

        // Should have remaining ttl > 0
        let result_ttl: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected to get an item ttl");
        assert!(result_ttl.as_millis() > 0);

        // Wait for ttl to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should get a cache miss
        let result = client.get(cache_name, key).await?;
        assert_eq!(result, Get::Miss {});

        Ok(())
    }
}

mod set_if_absent_or_equal {
    use super::*;

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .set_if_absent_or_equal("   ", "key", "value", "equal")
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
            .set_if_absent_or_equal(cache_name, "key", "value", "equal")
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let item1 = TestScalar::new();
        let item2 = TestScalar::new();

        // Setting a key that doesn't exist should create it
        let result = client
            .set_if_absent_or_equal(cache_name, item1.key(), item1.value(), item1.value())
            .await?;
        assert_eq!(result, SetIfAbsentOrEqual::Stored);

        let result = client.set(cache_name, item1.key(), item1.value()).await?;
        assert_eq!(result, Set {});

        // Setting a key that exists and equals the value should overwrite it
        let result = client
            .set_if_absent_or_equal(cache_name, item1.key(), item2.value(), item1.value())
            .await?;
        assert_eq!(result, SetIfAbsentOrEqual::Stored);
        let result = client.get(cache_name, item1.key()).await?;
        assert_eq!(result, Get::from(&item2));

        // Setting a key that exists and does NOT equal the value should NOT overwrite it
        let result = client
            .set_if_absent_or_equal(cache_name, item1.key(), item1.value(), item1.value())
            .await?;
        assert_eq!(result, SetIfAbsentOrEqual::NotStored);
        let result = client.get(cache_name, item1.key()).await?;
        assert_eq!(result, Get::from(&item2));

        Ok(())
    }

    // string key and string value with ttl before and after expiration
    #[tokio::test]
    async fn happy_path_with_ttl() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let key_uuid = unique_string("key");
        let key = key_uuid.as_str();

        let set_request =
            SetIfAbsentOrEqualRequest::new(cache_name, key, "first_value", "first_value")
                .ttl(Duration::from_secs(2));
        let result = client.send_request(set_request).await?;
        assert_eq!(result, SetIfAbsentOrEqual::Stored);

        // Should have remaining ttl > 0
        let result_ttl: Duration = client
            .item_get_ttl(cache_name, key)
            .await?
            .try_into()
            .expect("Expected to get an item ttl");
        assert!(result_ttl.as_millis() > 0);

        // Wait for ttl to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should get a cache miss
        let result = client.get(cache_name, key).await?;
        assert_eq!(result, Get::Miss {});

        Ok(())
    }
}

mod when_readconcern_is_specified {}
