use std::convert::TryInto;

use momento::cache::{
    SetAddElementsRequest, SetAddElementsResponse, SetFetchResponse, SetRemoveElementsResponse,
};
use momento::{MomentoErrorCode, MomentoResult};

use momento_test_util::{unique_cache_name, unique_value, TestSet, CACHE_TEST_STATE};

fn assert_fetched_set_eq(
    set_fetch_result: SetFetchResponse,
    expected: Vec<String>,
) -> MomentoResult<()> {
    let expected: SetFetchResponse = expected.into();
    assert_eq!(
        set_fetch_result, expected,
        "Expected SetFetch::Hit to be equal to {expected:?}, but got {set_fetch_result:?}"
    );
    Ok(())
}

fn assert_fetched_set_eq_after_sorting(
    set_fetch_result: SetFetchResponse,
    expected: Vec<String>,
) -> MomentoResult<()> {
    let sort_by_value = |a: &String, b: &String| -> std::cmp::Ordering {
        a.partial_cmp(b).expect("expected elements to be sortable")
    };

    let set_fetch_result = match set_fetch_result {
        SetFetchResponse::Hit { values } => {
            let mut elements: Vec<String> = values.try_into()?;
            elements.sort_by(sort_by_value);
            let new_set_fetch: SetFetchResponse = elements.into();
            new_set_fetch
        }
        _ => set_fetch_result,
    };

    let mut expected = expected;
    expected.sort_by(|a, b| a.partial_cmp(b).expect("expected elements to be sortable"));

    assert_fetched_set_eq(set_fetch_result, expected)
}

mod set_add_element {}

mod set_add_elements {
    use std::time::Duration;

    use momento::cache::CollectionTtl;

    use super::*;

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();

        let result = client
            .set_add_elements(cache_name, "set", vec!["value1", "value2"])
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);

        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let test_set = TestSet::default();

        // add elements using convenience method
        let result = client
            .set_add_elements(cache_name, test_set.name(), test_set.value().to_vec())
            .await?;
        assert_eq!(result, SetAddElementsResponse {});
        assert_fetched_set_eq_after_sorting(
            client.set_fetch(cache_name, test_set.name()).await?,
            test_set.value().to_vec(),
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn happy_path_with_optional_arguments() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let test_set = TestSet::default();

        // add elements with optional ttl argument
        let request =
            SetAddElementsRequest::new(cache_name, test_set.name(), test_set.value().to_vec())
                .ttl(CollectionTtl::new(Some(Duration::from_secs(10)), false));
        let result = client.send_request(request).await?;
        assert_eq!(result, SetAddElementsResponse {});

        // only test_set elements should remain after 5 seconds
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert_fetched_set_eq_after_sorting(
            client.set_fetch(cache_name, test_set.name()).await?,
            test_set.value().to_vec(),
        )?;

        Ok(())
    }
}

mod set_contains_element {}

mod set_contains_elements {}

mod set_fetch {
    use super::*;

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();

        let result = client.set_fetch(cache_name, "set").await.unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);

        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let test_set = TestSet::default();

        // Should miss before set exists
        let result = client.set_fetch(cache_name, test_set.name()).await?;
        assert_eq!(result, SetFetchResponse::Miss {});

        // Should hit after set exists
        let result = client
            .set_add_elements(cache_name, test_set.name(), test_set.value().to_vec())
            .await?;
        assert_eq!(result, SetAddElementsResponse {});
        assert_fetched_set_eq_after_sorting(
            client.set_fetch(cache_name, test_set.name()).await?,
            test_set.value().to_vec(),
        )?;

        Ok(())
    }
}

mod set_length {}

mod set_remove_element {}

mod set_remove_elements {
    use super::*;

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();

        let result = client
            .set_remove_elements(cache_name, "set", vec!["value1", "value2"])
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);

        Ok(())
    }

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let test_set = TestSet::default();

        let result = client
            .set_add_elements(cache_name, test_set.name(), test_set.value().to_vec())
            .await?;
        assert_eq!(result, SetAddElementsResponse {});

        // Should do nothing when elements don't exist
        let result = client
            .set_remove_elements(cache_name, test_set.name(), vec![unique_value()])
            .await?;
        assert_eq!(result, SetRemoveElementsResponse {});

        // Should remove existing elements
        let result = client
            .set_remove_elements(
                cache_name,
                test_set.name(),
                vec![test_set.value()[0].clone()],
            )
            .await?;
        assert_eq!(result, SetRemoveElementsResponse {});
        assert_fetched_set_eq(
            client.set_fetch(cache_name, test_set.name()).await?,
            vec![test_set.value()[1].clone()],
        )?;

        // Should remove the set once it's emptied
        let result = client
            .set_remove_elements(
                cache_name,
                test_set.name(),
                vec![test_set.value()[1].clone()],
            )
            .await?;
        assert_eq!(result, SetRemoveElementsResponse {});
        let result = client.set_fetch(cache_name, test_set.name()).await?;
        assert_eq!(result, SetFetchResponse::Miss {});

        Ok(())
    }
}

mod set_sample {}
