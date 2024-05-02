use momento::cache::{
    CollectionTtl, ListConcatenateBack, ListConcatenateBackRequest, ListConcatenateFront,
    ListConcatenateFrontRequest, ListFetch, ListLength, ListPopBack, ListPopFront, ListRemoveValue,
};
use momento::{MomentoErrorCode, MomentoResult};

use momento_test_util::{unique_cache_name, TestList, CACHE_TEST_STATE};

use std::convert::TryInto;
use std::time::Duration;

fn assert_list_eq(list_fetch_result: ListFetch, expected: Vec<String>) -> MomentoResult<()> {
    let expected: ListFetch = expected.into();
    assert_eq!(
        list_fetch_result, expected,
        "Expected ListFetch::Hit to be equal to {:?}, but got {:?}",
        expected, list_fetch_result
    );
    Ok(())
}

mod list_concatenate_back {
    use super::*;

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();

        let result = client
            .list_concatenate_back(cache_name, "list", vec!["value1", "value2"])
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);

        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let test_list = TestList::default();

        let result = client
            .list_concatenate_back(cache_name, test_list.name(), test_list.values().to_vec())
            .await?;
        assert_eq!(result, ListConcatenateBack {});
        assert_list_eq(
            client.list_fetch(cache_name, test_list.name()).await?,
            test_list.values().to_vec(),
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn happy_path_with_optional_arguments() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let test_list = TestList::default();

        // Concatenates with truncation and collection ttl
        let request = ListConcatenateBackRequest::new(
            cache_name.to_string(),
            test_list.name(),
            [test_list.values().to_vec(), test_list.values().to_vec()].concat(),
        )
        .truncate_back_to_size(2)
        .ttl(CollectionTtl::new(Some(Duration::from_secs(3)), false));
        let result = client.send_request(request).await?;
        assert_eq!(result, ListConcatenateBack {});

        // Should have truncated to only 2 elements
        assert_list_eq(
            client.list_fetch(cache_name, test_list.name()).await?,
            test_list.values().to_vec(),
        )?;

        tokio::time::sleep(Duration::from_secs(3)).await;

        // Expect a miss after collection ttl expires
        let result = client.list_fetch(cache_name, test_list.name()).await?;
        assert_eq!(result, ListFetch::Miss);

        Ok(())
    }
}

mod list_concatenate_front {
    use super::*;

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();

        let result = client
            .list_concatenate_front(cache_name, "list", vec!["value1", "value2"])
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);

        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let test_list = TestList::default();

        // Concatenates string values
        let result = client
            .list_concatenate_front(cache_name, test_list.name(), test_list.values().to_vec())
            .await?;
        assert_eq!(result, ListConcatenateFront {});
        assert_list_eq(
            client.list_fetch(cache_name, test_list.name()).await?,
            test_list.values().to_vec(),
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn happy_path_with_optional_arguments() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let test_list = TestList::default();

        // Concatenates with truncation and collection ttl
        let request = ListConcatenateFrontRequest::new(
            cache_name.to_string(),
            test_list.name(),
            [test_list.values().to_vec(), test_list.values().to_vec()].concat(),
        )
        .truncate_back_to_size(2)
        .ttl(CollectionTtl::new(Some(Duration::from_secs(3)), false));
        let result = client.send_request(request).await?;
        assert_eq!(result, ListConcatenateFront {});

        // Should have truncated to only 2 elements
        assert_list_eq(
            client.list_fetch(cache_name, test_list.name()).await?,
            test_list.values().to_vec(),
        )?;

        tokio::time::sleep(Duration::from_secs(3)).await;

        // Expect a miss after collection ttl expires
        let result = client.list_fetch(cache_name, test_list.name()).await?;
        assert_eq!(result, ListFetch::Miss);

        Ok(())
    }
}

mod list_length {
    use super::*;

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();

        let result = client.list_length(cache_name, "list").await.unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_list() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let list_name = unique_cache_name();

        let result = client.list_length(cache_name, list_name).await?;

        assert_eq!(result, ListLength::Miss {});

        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let test_list = TestList::default();

        // Concatenates some values first
        let result = client
            .list_concatenate_back(cache_name, test_list.name(), test_list.values().to_vec())
            .await?;
        assert_eq!(result, ListConcatenateBack {});

        // Fetch list length
        let result = client.list_length(cache_name, test_list.name()).await?;
        assert_eq!(result, ListLength::Hit { length: 2 });

        Ok(())
    }
}

mod list_fetch {
    use momento::cache::ListFetchRequest;

    use super::*;

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();

        let result = client.list_fetch(cache_name, "list").await.unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_list() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let list_name = unique_cache_name();

        let result = client.list_fetch(cache_name, list_name).await?;

        assert_eq!(result, ListFetch::Miss {});

        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let list1 = TestList::default();
        let list2 = TestList::default();

        // Concatenates some values first
        let result = client
            .list_concatenate_back(
                cache_name,
                list1.name(),
                [list1.values().to_vec(), list2.values().to_vec()].concat(),
            )
            .await?;
        assert_eq!(result, ListConcatenateBack {});

        // Fetch entire list
        let fetch_full_list = client.list_fetch(cache_name, list1.name()).await?;
        assert_list_eq(
            fetch_full_list,
            [list1.values().to_vec(), list2.values().to_vec()].concat(),
        )?;

        // Fetch a list slice
        let request = ListFetchRequest::new(cache_name, list1.name())
            .start_index(2)
            .end_index(4);
        let fetch_slice = client.send_request(request).await?;
        assert_list_eq(fetch_slice, list2.values().to_vec())?;

        Ok(())
    }
}

mod list_pop_back {
    use std::convert::TryInto;

    use super::*;

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();

        let result = client.list_pop_back(cache_name, "list").await.unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_list() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let list_name = unique_cache_name();

        let result = client.list_pop_back(cache_name, list_name).await?;

        assert_eq!(result, ListPopBack::Miss {});

        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let test_list = TestList::default();

        // Concatenates some values first
        let result = client
            .list_concatenate_back(cache_name, test_list.name(), test_list.values().to_vec())
            .await?;
        assert_eq!(result, ListConcatenateBack {});

        // Pop first value from the back
        let popped_first: String = client
            .list_pop_back(cache_name, test_list.name())
            .await?
            .try_into()
            .expect("Expected a popped list value!");
        assert_eq!(popped_first, test_list.values().last().unwrap().to_string());

        // Pop second value from the back
        let popped_second: String = client
            .list_pop_back(cache_name, test_list.name())
            .await?
            .try_into()
            .expect("Expected a popped list value!");
        assert_eq!(
            popped_second,
            test_list.values().first().unwrap().to_string()
        );

        Ok(())
    }
}

mod list_pop_front {
    use super::*;

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();

        let result = client.list_pop_front(cache_name, "list").await.unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_list() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let list_name = unique_cache_name();

        let result = client.list_pop_front(cache_name, list_name).await?;

        assert_eq!(result, ListPopFront::Miss {});

        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let test_list = TestList::default();

        // Concatenates some values first
        let result = client
            .list_concatenate_back(cache_name, test_list.name(), test_list.values().to_vec())
            .await?;
        assert_eq!(result, ListConcatenateBack {});

        // Pop first value from the front
        let popped_first: String = client
            .list_pop_front(cache_name, test_list.name())
            .await?
            .try_into()
            .expect("Expected a popped list value!");
        assert_eq!(
            popped_first,
            test_list.values().first().unwrap().to_string()
        );

        // Pop second value from the front
        let popped_second: String = client
            .list_pop_front(cache_name, test_list.name())
            .await?
            .try_into()
            .expect("Expected a popped list value!");
        assert_eq!(
            popped_second,
            test_list.values().last().unwrap().to_string()
        );

        Ok(())
    }
}

mod list_remove {
    use super::*;

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();

        let result = client
            .list_remove_value(cache_name, "list", "value")
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_list() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let list_name = unique_cache_name();

        let result = client
            .list_remove_value(cache_name, list_name, "value")
            .await?;

        assert_eq!(result, ListRemoveValue {});

        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let test_list = TestList::default();

        // Concatenates some values first
        let result = client
            .list_concatenate_back(cache_name, test_list.name(), test_list.values().to_vec())
            .await?;
        assert_eq!(result, ListConcatenateBack {});

        let first_value = test_list
            .values()
            .first()
            .expect("Expected first value from TestList")
            .to_string();
        let second_value = test_list
            .values()
            .last()
            .expect("Expected last value from TestList")
            .to_string();

        // Remove one of the values and only the other should remain
        let result = client
            .list_remove_value(cache_name, test_list.name(), first_value)
            .await?;
        assert_eq!(result, ListRemoveValue {});
        assert_list_eq(
            client.list_fetch(cache_name, test_list.name()).await?,
            vec![second_value],
        )?;

        Ok(())
    }
}
