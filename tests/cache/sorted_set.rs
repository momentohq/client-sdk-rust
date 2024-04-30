use std::sync::Arc;

use momento::cache::{
    IntoSortedSetElements, SortedSetElement, SortedSetElements, SortedSetFetch,
    SortedSetFetchByRankRequest, SortedSetFetchByScoreRequest,
    SortedSetOrder::{Ascending, Descending},
};
use momento::{CacheClient, MomentoErrorCode, MomentoResult};

use momento_test_util::{unique_cache_name, unique_key, TestSortedSet, CACHE_TEST_STATE};

fn assert_fetched_sorted_set_eq(
    sorted_set_fetch_result: SortedSetFetch,
    expected: Vec<(String, f64)>,
) -> MomentoResult<()> {
    let expected: SortedSetFetch = expected.into();
    assert_eq!(
        sorted_set_fetch_result, expected,
        "Expected SortedSetFetch::Hit to be equal to {:?}, but got {:?}",
        expected, sorted_set_fetch_result
    );
    Ok(())
}

fn assert_fetched_sorted_set_eq_after_sorting(
    sorted_set_fetch_result: SortedSetFetch,
    expected: Vec<(String, f64)>,
) -> MomentoResult<()> {
    let sort_by_score = |a: &(_, f64), b: &(_, f64)| -> std::cmp::Ordering {
        a.1.partial_cmp(&b.1)
            .expect("expected elements to be sortable")
    };

    let sorted_set_fetch_result = match sorted_set_fetch_result {
        SortedSetFetch::Hit { elements } => {
            let mut elements = elements.elements.clone();
            elements.sort_by(sort_by_score);
            SortedSetFetch::Hit {
                elements: SortedSetElements { elements },
            }
        }
        _ => sorted_set_fetch_result,
    };

    let mut expected = expected;
    expected.sort_by(|a, b| {
        a.1.partial_cmp(&b.1)
            .expect("expected elements to be sortable")
    });
    assert_fetched_sorted_set_eq(sorted_set_fetch_result, expected)
}

mod sorted_set_fetch_by_rank {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let item = TestSortedSet {
            name: unique_key(),
            value: vec![
                ("1".to_string(), 0.0),
                ("2".to_string(), 1.0),
                ("3".to_string(), 0.5),
                ("4".to_string(), 2.0),
                ("5".to_string(), 1.5),
            ],
        };

        let result = client
            .sorted_set_fetch_by_rank(cache_name, item.name(), Ascending, None, None)
            .await?;
        assert_eq!(result, SortedSetFetch::Miss);

        client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;

        // Full set ascending, end index larger than set
        let fetch_request = SortedSetFetchByRankRequest::new(cache_name, item.name())
            .order(Ascending)
            .start_rank(0)
            .end_rank(6);
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(
            result,
            vec![
                ("1".to_string(), 0.0),
                ("3".to_string(), 0.5),
                ("2".to_string(), 1.0),
                ("5".to_string(), 1.5),
                ("4".to_string(), 2.0),
            ],
        )?;

        // Up until rank 3
        let fetch_request = SortedSetFetchByRankRequest::new(cache_name, item.name())
            .order(Ascending)
            .end_rank(3);
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(
            result,
            vec![
                ("1".to_string(), 0.0),
                ("3".to_string(), 0.5),
                ("2".to_string(), 1.0),
            ],
        )?;

        // From rank 3
        let fetch_request = SortedSetFetchByRankRequest::new(cache_name, item.name())
            .order(Ascending)
            .start_rank(3);
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(result, vec![("5".to_string(), 1.5), ("4".to_string(), 2.0)])?;

        // Partial set descending
        let fetch_request = SortedSetFetchByRankRequest::new(cache_name, item.name())
            .order(Descending)
            .start_rank(1)
            .end_rank(4);
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(
            result,
            vec![
                ("5".to_string(), 1.5),
                ("2".to_string(), 1.0),
                ("3".to_string(), 0.5),
            ],
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();

        let result = client
            .sorted_set_fetch_by_rank(cache_name, "sorted-set", Ascending, None, None)
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);

        Ok(())
    }
}

mod sorted_set_fetch_by_score {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let item = TestSortedSet {
            name: unique_key(),
            value: vec![
                ("1".to_string(), 0.0),
                ("2".to_string(), 1.0),
                ("3".to_string(), 0.5),
                ("4".to_string(), 2.0),
                ("5".to_string(), 1.5),
            ],
        };

        let result = client
            .sorted_set_fetch_by_score(cache_name, item.name(), Ascending)
            .await?;
        assert_eq!(result, SortedSetFetch::Miss);

        client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;

        // Full set ascending, end score larger than set
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, item.name())
            .order(Ascending)
            .min_score(0.0)
            .max_score(9.9);
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(
            result,
            vec![
                ("1".to_string(), 0.0),
                ("3".to_string(), 0.5),
                ("2".to_string(), 1.0),
                ("5".to_string(), 1.5),
                ("4".to_string(), 2.0),
            ],
        )?;

        // Up until score 1.0
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, item.name())
            .order(Ascending)
            .max_score(1.0);
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(
            result,
            vec![
                ("1".to_string(), 0.0),
                ("3".to_string(), 0.5),
                ("2".to_string(), 1.0),
            ],
        )?;

        // From score 1.0
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, item.name())
            .order(Ascending)
            .min_score(1.0);
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(
            result,
            vec![
                ("2".to_string(), 1.0),
                ("5".to_string(), 1.5),
                ("4".to_string(), 2.0),
            ],
        )?;

        // Partial set descending
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, item.name())
            .order(Descending)
            .min_score(0.1)
            .max_score(1.9);
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(
            result,
            vec![
                ("5".to_string(), 1.5),
                ("2".to_string(), 1.0),
                ("3".to_string(), 0.5),
            ],
        )?;

        // Partial set limited by offset and count
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, item.name())
            .offset(1)
            .count(3);
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(
            result,
            vec![
                ("3".to_string(), 0.5),
                ("2".to_string(), 1.0),
                ("5".to_string(), 1.5),
            ],
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let sorted_set_name = "sorted-set";

        let result = client
            .sorted_set_fetch_by_score(cache_name, sorted_set_name, Ascending)
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }
}

mod sorted_set_get_rank {}

mod sorted_set_get_score {}

mod sorted_set_get_scores {}

mod sorted_set_increment_score {}

mod sorted_set_remove_element {}

mod sorted_set_put_element {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let item = TestSortedSet::new();

        let result = client
            .sorted_set_fetch_by_rank(cache_name, item.name(), Ascending, None, None)
            .await?;
        assert_eq!(result, SortedSetFetch::Miss);

        client
            .sorted_set_put_element(
                cache_name,
                item.name(),
                item.value[0].0.clone(),
                item.value[0].1,
            )
            .await?;

        let result = client
            .sorted_set_fetch_by_rank(cache_name, item.name(), Ascending, None, None)
            .await?;
        assert_fetched_sorted_set_eq(result, vec![item.value[0].clone()])?;

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let sorted_set_name = "sorted-set";

        let result = client
            .sorted_set_put_element(cache_name, sorted_set_name, "value", 1.0)
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }
}

mod sorted_set_put_elements {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        async fn test_put_elements_happy_path(
            client: &Arc<CacheClient>,
            cache_name: &String,
            to_put: impl IntoSortedSetElements<String> + Clone,
        ) -> MomentoResult<()> {
            let sorted_set_name = unique_key();
            let sorted_set_name = sorted_set_name.as_str();

            client
                .sorted_set_put_elements(cache_name, sorted_set_name, to_put.clone())
                .await?;

            let result = client
                .sorted_set_fetch_by_score(cache_name, sorted_set_name, Ascending)
                .await?;
            let expected = to_put
                .into_sorted_set_elements()
                .into_iter()
                .map(|e| (e.value, e.score))
                .collect::<Vec<_>>();

            assert_fetched_sorted_set_eq_after_sorting(result, expected)?;

            Ok(())
        }

        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;

        // Test putting multiple elements as a vector
        let to_put = vec![("element1".to_string(), 1.0), ("element2".to_string(), 2.0)];
        test_put_elements_happy_path(client, cache_name, to_put).await?;

        // Test putting multiple elements as a hashmap
        let to_put = std::collections::HashMap::from([
            ("element1".to_string(), 1.0),
            ("element2".to_string(), 2.0),
        ]);
        test_put_elements_happy_path(client, cache_name, to_put).await?;

        // Test passing a vec of `SortedSetElement`s
        let to_put = vec![("element1".to_string(), 1.0), ("element2".to_string(), 2.0)];
        test_put_elements_happy_path(client, cache_name, to_put).await?;

        let to_put = vec![
            SortedSetElement {
                value: "element1".to_string(),
                score: 1.0,
            },
            SortedSetElement {
                value: "element2".to_string(),
                score: 2.0,
            },
        ];
        test_put_elements_happy_path(client, cache_name, to_put).await?;
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_key();
        let sorted_set_name = "sorted-set";

        let result = client
            .sorted_set_put_elements(
                cache_name,
                sorted_set_name,
                vec![("element1".to_string(), 1.0), ("element2".to_string(), 2.0)],
            )
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }
}

mod sorted_set_length {}

mod sorted_set_length_by_score {}

mod delete_sorted_set {}
