use std::sync::Arc;

use momento::cache::{
    IntoSortedSetElements, ScoreBound, SortedSetAggregateFunction, SortedSetElement,
    SortedSetElements, SortedSetFetchByRankRequest, SortedSetFetchByScoreRequest,
    SortedSetFetchResponse, SortedSetGetRankResponse, SortedSetGetScoreResponse,
    SortedSetLengthByScoreRequest, SortedSetLengthByScoreResponse, SortedSetLengthResponse,
    SortedSetOrder::{Ascending, Descending},
    SortedSetPutElementsResponse, SortedSetRemoveElementsResponse, SortedSetUnionStoreRequest,
    SortedSetUnionStoreResponse, SortedSetUnionStoreSource,
};
use momento::{CacheClient, MomentoErrorCode, MomentoResult};

use momento_test_util::{
    unique_cache_name, unique_key, unique_value, TestSortedSet, CACHE_TEST_STATE,
};

fn assert_fetched_sorted_set_eq(
    sorted_set_fetch_result: SortedSetFetchResponse,
    expected: Vec<(String, f64)>,
) -> MomentoResult<()> {
    let expected: SortedSetFetchResponse = expected.into();
    assert_eq!(
        sorted_set_fetch_result, expected,
        "Expected SortedSetFetch::Hit to be equal to {expected:?}, but got {sorted_set_fetch_result:?}"
    );
    Ok(())
}

fn assert_fetched_sorted_set_eq_after_sorting(
    sorted_set_fetch_result: SortedSetFetchResponse,
    expected: Vec<(String, f64)>,
) -> MomentoResult<()> {
    let sort_by_score = |a: &(_, f64), b: &(_, f64)| -> std::cmp::Ordering {
        a.1.partial_cmp(&b.1)
            .expect("expected elements to be sortable")
    };

    let sorted_set_fetch_result = match sorted_set_fetch_result {
        SortedSetFetchResponse::Hit { value } => {
            let mut elements = value.elements.clone();
            elements.sort_by(sort_by_score);
            SortedSetFetchResponse::Hit {
                value: SortedSetElements { elements },
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
        assert_eq!(result, SortedSetFetchResponse::Miss);

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

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);

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
        assert_eq!(result, SortedSetFetchResponse::Miss);

        client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;

        // Full set ascending, end score larger than set
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, item.name())
            .order(Ascending)
            .min_score(ScoreBound::Inclusive(0.0))
            .max_score(ScoreBound::Inclusive(9.9));
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

        // Up until score 1.0 (inclusive)
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, item.name())
            .order(Ascending)
            .max_score(ScoreBound::Inclusive(1.0));
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(
            result,
            vec![
                ("1".to_string(), 0.0),
                ("3".to_string(), 0.5),
                ("2".to_string(), 1.0),
            ],
        )?;

        // Up until score 1.0 (exclusive)
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, item.name())
            .order(Ascending)
            .max_score(ScoreBound::Exclusive(1.0));
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(result, vec![("1".to_string(), 0.0), ("3".to_string(), 0.5)])?;

        // From score 1.0 (inclusive)
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, item.name())
            .order(Ascending)
            .min_score(ScoreBound::Inclusive(1.0));
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(
            result,
            vec![
                ("2".to_string(), 1.0),
                ("5".to_string(), 1.5),
                ("4".to_string(), 2.0),
            ],
        )?;

        // From score 1.0 (exclusive)
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, item.name())
            .order(Ascending)
            .min_score(ScoreBound::Exclusive(1.0));
        let result = client.send_request(fetch_request).await?;
        assert_fetched_sorted_set_eq(result, vec![("5".to_string(), 1.5), ("4".to_string(), 2.0)])?;

        // Partial set descending
        let fetch_request = SortedSetFetchByScoreRequest::new(cache_name, item.name())
            .order(Descending)
            .min_score(ScoreBound::Inclusive(0.1))
            .max_score(ScoreBound::Inclusive(1.9));
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

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }
}

mod sorted_set_get_rank {
    use momento::cache::SortedSetGetRankRequest;

    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let item = TestSortedSet::new();

        let result = client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Hit for existing value
        let result = client
            .sorted_set_get_rank(cache_name, item.name(), item.value[0].0.as_str())
            .await?;
        assert_eq!(result, SortedSetGetRankResponse::Hit { rank: 0 });

        // Miss for nonexistent value
        let result = client
            .sorted_set_get_rank(cache_name, item.name(), "nonexistent")
            .await?;
        assert_eq!(result, SortedSetGetRankResponse::Miss);

        Ok(())
    }

    #[tokio::test]
    async fn happy_path_explicit_ascending_order() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let item = TestSortedSet::new();

        let result = client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Hit for existing value 1
        let request =
            SortedSetGetRankRequest::new(cache_name, item.name(), item.value[0].0.as_str())
                .order(Ascending);
        let result = client.send_request(request).await?;
        assert_eq!(result, SortedSetGetRankResponse::Hit { rank: 0 });

        // Hit for existing value 2
        let request =
            SortedSetGetRankRequest::new(cache_name, item.name(), item.value[1].0.as_str())
                .order(Ascending);
        let result = client.send_request(request).await?;
        assert_eq!(result, SortedSetGetRankResponse::Hit { rank: 1 });

        // Miss for nonexistent value
        let request =
            SortedSetGetRankRequest::new(cache_name, item.name(), "nonexistent").order(Descending);
        let result = client.send_request(request).await?;
        assert_eq!(result, SortedSetGetRankResponse::Miss);

        Ok(())
    }

    #[tokio::test]
    async fn happy_path_descending_order() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let item = TestSortedSet::new();

        let result = client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Hit for existing value 1
        let request =
            SortedSetGetRankRequest::new(cache_name, item.name(), item.value[0].0.as_str())
                .order(Descending);
        let result = client.send_request(request).await?;
        assert_eq!(result, SortedSetGetRankResponse::Hit { rank: 1 });

        // Hit for existing value 2
        let request =
            SortedSetGetRankRequest::new(cache_name, item.name(), item.value[1].0.as_str())
                .order(Descending);
        let result = client.send_request(request).await?;
        assert_eq!(result, SortedSetGetRankResponse::Hit { rank: 0 });

        // Miss for nonexistent value
        let request =
            SortedSetGetRankRequest::new(cache_name, item.name(), "nonexistent").order(Descending);
        let result = client.send_request(request).await?;
        assert_eq!(result, SortedSetGetRankResponse::Miss);

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_key();
        let sorted_set_name = "sorted-set";

        let result = client
            .sorted_set_get_rank(cache_name, sorted_set_name, "element1")
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_sorted_set() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let sorted_set_name = unique_key();

        let result = client
            .sorted_set_get_rank(cache_name, sorted_set_name, "element1")
            .await?;
        assert_eq!(result, SortedSetGetRankResponse::Miss);
        Ok(())
    }
}

mod sorted_set_get_score {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let item = TestSortedSet::new();

        let result = client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Hit for existing value
        let result = client
            .sorted_set_get_score(cache_name, item.name(), item.value[0].0.as_str())
            .await?;
        assert_eq!(result, SortedSetGetScoreResponse::Hit { score: 1.0 });

        // Miss for nonexistent value
        let result = client
            .sorted_set_get_score(cache_name, item.name(), unique_value())
            .await?;
        assert_eq!(result, SortedSetGetScoreResponse::Miss);

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_key();
        let sorted_set_name = "sorted-set";

        let result = client
            .sorted_set_get_score(cache_name, sorted_set_name, "element1")
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_sorted_set() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let sorted_set_name = unique_key();

        let result = client
            .sorted_set_get_score(cache_name, sorted_set_name, "element1")
            .await?;
        assert_eq!(result, SortedSetGetScoreResponse::Miss);
        Ok(())
    }
}

mod sorted_set_get_scores {
    use std::convert::TryInto;

    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let item = TestSortedSet::new();

        let result = client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Hit for existing value
        let result: Vec<SortedSetElement<String>> = client
            .sorted_set_get_scores(
                cache_name,
                item.name(),
                vec![item.value[0].0.as_str(), item.value[1].0.as_str()],
            )
            .await?
            .try_into()
            .expect("should be able to convert to Vec<SortedSetElement<String>>");
        assert_eq!(result.len(), 2);
        let first = result.first().unwrap();
        let second = result.last().unwrap();
        assert_eq!(first.score, item.value[0].1);
        assert_eq!(first.value, item.value[0].0);

        assert_eq!(second.score, item.value[1].1);
        assert_eq!(second.value, item.value[1].0);

        // Miss for nonexistent value
        let result = client
            .sorted_set_get_score(cache_name, item.name(), unique_value())
            .await?;
        assert_eq!(result, SortedSetGetScoreResponse::Miss);

        Ok(())
    }
}

mod sorted_set_increment_score {
    use momento::cache::SortedSetIncrementScoreResponse;

    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let item = TestSortedSet::new();

        let result = client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Hit for existing value
        let result = client
            .sorted_set_increment_score(cache_name, item.name(), item.value[0].0.as_str(), 0.5)
            .await?;
        assert_eq!(result, SortedSetIncrementScoreResponse { score: 1.5 });

        // Set a non-existent value with passed in score
        let result = client
            .sorted_set_increment_score(cache_name, unique_key(), unique_value(), 100.0)
            .await?;
        assert_eq!(result, SortedSetIncrementScoreResponse { score: 100.0 });

        Ok(())
    }
}

mod sorted_set_remove_element {}

mod sorted_set_remove_elements {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let item = TestSortedSet::new();

        // put some values
        let result = client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // does nothing for nonexistent values
        let result = client
            .sorted_set_remove_elements(
                cache_name,
                item.name(),
                vec![unique_value(), unique_value()],
            )
            .await?;
        assert_eq!(result, SortedSetRemoveElementsResponse {});
        assert_fetched_sorted_set_eq(
            client
                .sorted_set_fetch_by_score(cache_name, item.name(), Ascending)
                .await?,
            item.value().to_vec(),
        )?;

        // removes existing values
        let values = vec![item.value[0].0.to_string()];
        let result = client
            .sorted_set_remove_elements(cache_name, item.name(), values)
            .await?;
        assert_eq!(result, SortedSetRemoveElementsResponse {});
        assert_fetched_sorted_set_eq(
            client
                .sorted_set_fetch_by_score(cache_name, item.name(), Ascending)
                .await?,
            vec![item.value()[1].clone()],
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_key();
        let sorted_set_name = "sorted-set";

        let result = client
            .sorted_set_remove_elements(cache_name, sorted_set_name, vec!["element1", "element2"])
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_sorted_set() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let sorted_set_name = unique_key();

        let result = client
            .sorted_set_remove_elements(cache_name, sorted_set_name, vec!["element1", "element2"])
            .await?;
        assert_eq!(result, SortedSetRemoveElementsResponse {});
        Ok(())
    }
}

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
        assert_eq!(result, SortedSetFetchResponse::Miss);

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

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
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

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }
}

mod sorted_set_length {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let item = TestSortedSet::new();

        // Miss before sorted set exists
        let result = client.sorted_set_length(cache_name, item.name()).await?;
        assert_eq!(result, SortedSetLengthResponse::Miss);

        let result = client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Nonzero length after sorted set exists
        let result = client.sorted_set_length(cache_name, item.name()).await?;
        assert_eq!(result, SortedSetLengthResponse::Hit { length: 2 });

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_key();
        let sorted_set_name = "sorted-set";

        let result = client
            .sorted_set_length(cache_name, sorted_set_name)
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }
}

mod sorted_set_length_by_score {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let item = TestSortedSet::new();

        // Miss before sorted set exists
        let result = client
            .sorted_set_length_by_score(cache_name, item.name())
            .await?;
        assert_eq!(result, SortedSetLengthByScoreResponse::Miss);

        let result = client
            .sorted_set_put_elements(cache_name, item.name(), item.value().to_vec())
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Nonzero length after sorted set exists
        let result = client
            .sorted_set_length_by_score(cache_name, item.name())
            .await?;
        assert_eq!(result, SortedSetLengthByScoreResponse::Hit { length: 2 });

        // Nonzero length after specifying inclusive min and max score
        let request = SortedSetLengthByScoreRequest::new(cache_name, item.name())
            .min_score(ScoreBound::Inclusive(0.0))
            .max_score(ScoreBound::Inclusive(2.0));
        let result = client.send_request(request).await?;
        assert_eq!(result, SortedSetLengthByScoreResponse::Hit { length: 2 });

        // Nonzero length after specifying exclusive min and max score
        let request = SortedSetLengthByScoreRequest::new(cache_name, item.name())
            .min_score(ScoreBound::Exclusive(0.0))
            .max_score(ScoreBound::Exclusive(2.0));
        let result = client.send_request(request).await?;
        assert_eq!(result, SortedSetLengthByScoreResponse::Hit { length: 1 });

        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_key();
        let sorted_set_name = "sorted-set";

        let result = client
            .sorted_set_length_by_score(cache_name, sorted_set_name)
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }
}

mod delete_sorted_set {}

mod sorted_set_union_store {
    use std::collections::HashMap;

    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let destination_sorted_set_name = unique_key();
        let sorted_set_one = TestSortedSet::new();
        let sorted_set_two = TestSortedSet::new();
        let sources = vec![
            SortedSetUnionStoreSource::new(sorted_set_one.name(), 1.0),
            SortedSetUnionStoreSource::new(sorted_set_two.name(), 1.0),
        ];

        // Destination set should be empty
        let result = client
            .sorted_set_length(cache_name, destination_sorted_set_name.clone())
            .await?;
        assert_eq!(result, SortedSetLengthResponse::Miss);

        // Union of empty (nonexistent) sets produces destination set with length 0
        let result = client
            .sorted_set_union_store(
                cache_name,
                destination_sorted_set_name.clone(),
                sources.clone(),
            )
            .await?;
        assert_eq!(result, SortedSetUnionStoreResponse { length: 0 });

        // Insert two sets
        let result = client
            .sorted_set_put_elements(
                cache_name,
                sorted_set_one.name(),
                sorted_set_one.value().to_vec(),
            )
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});
        let result = client
            .sorted_set_put_elements(
                cache_name,
                sorted_set_two.name(),
                sorted_set_two.value().to_vec(),
            )
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Nonzero length after sorted set exists
        let result = client
            .sorted_set_union_store(cache_name, destination_sorted_set_name.clone(), sources)
            .await?;
        assert_eq!(result, SortedSetUnionStoreResponse { length: 4 });

        // Destination set should be non-empty
        let result = client
            .sorted_set_length(cache_name, destination_sorted_set_name)
            .await?;
        assert_eq!(result, SortedSetLengthResponse::Hit { length: 4 });
        Ok(())
    }

    #[tokio::test]
    async fn happy_path_with_hash_map_of_sources() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let destination_sorted_set_name = unique_key();
        let sorted_set_one = TestSortedSet::new();
        let sorted_set_two = TestSortedSet::new();
        let mut sources: HashMap<&str, f32> = HashMap::new();
        sources.insert(sorted_set_one.name(), 1.0);
        sources.insert(sorted_set_two.name(), 2.0);

        // Destination set should be empty
        let result = client
            .sorted_set_length(cache_name, destination_sorted_set_name.clone())
            .await?;
        assert_eq!(result, SortedSetLengthResponse::Miss);

        // Union of empty (nonexistent) sets produces destination set with length 0
        let result = client
            .sorted_set_union_store(
                cache_name,
                destination_sorted_set_name.clone(),
                sources.clone(),
            )
            .await?;
        assert_eq!(result, SortedSetUnionStoreResponse { length: 0 });

        // Insert two sets
        let result = client
            .sorted_set_put_elements(
                cache_name,
                sorted_set_one.name(),
                sorted_set_one.value().to_vec(),
            )
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});
        let result = client
            .sorted_set_put_elements(
                cache_name,
                sorted_set_two.name(),
                sorted_set_two.value().to_vec(),
            )
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Nonzero length after sorted set exists
        let result = client
            .sorted_set_union_store(cache_name, destination_sorted_set_name.clone(), sources)
            .await?;
        assert_eq!(result, SortedSetUnionStoreResponse { length: 4 });

        // Destination set should be non-empty
        let result = client
            .sorted_set_length(cache_name, destination_sorted_set_name)
            .await?;
        assert_eq!(result, SortedSetLengthResponse::Hit { length: 4 });
        Ok(())
    }

    #[tokio::test]
    async fn happy_path_with_sum_aggregation() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let destination_sorted_set_name = unique_key();
        let sorted_set_one = TestSortedSet::new();
        let sorted_set_two = TestSortedSet::new();
        let sources = vec![
            SortedSetUnionStoreSource::new(sorted_set_one.name(), 1.0),
            SortedSetUnionStoreSource::new(sorted_set_two.name(), 1.0),
        ];

        // Destination set should be empty
        let result = client
            .sorted_set_length(cache_name, destination_sorted_set_name.clone())
            .await?;
        assert_eq!(result, SortedSetLengthResponse::Miss);

        // Insert two sets, but duplicate the values from the first set.
        let result = client
            .sorted_set_put_elements(
                cache_name,
                sorted_set_one.name(),
                sorted_set_one.value().to_vec(),
            )
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});
        let result = client
            .sorted_set_put_elements(
                cache_name,
                sorted_set_two.name(),
                sorted_set_one.value().to_vec(),
            )
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Nonzero length after sorted set exists
        let request = SortedSetUnionStoreRequest::new(
            cache_name,
            destination_sorted_set_name.clone(),
            sources.clone(),
        )
        .aggregate(SortedSetAggregateFunction::Sum);
        let result = client.send_request(request).await?;
        assert_eq!(result, SortedSetUnionStoreResponse { length: 2 });

        // Test that the destination set has the correct elements
        let result = client
            .sorted_set_fetch_by_score(cache_name, destination_sorted_set_name, Ascending)
            .await?;
        assert_fetched_sorted_set_eq_after_sorting(
            result,
            vec![
                (
                    sorted_set_one.value()[0].0.clone(),
                    sorted_set_one.value()[0].1 * 2.0,
                ),
                (
                    sorted_set_one.value()[1].0.clone(),
                    sorted_set_one.value()[1].1 * 2.0,
                ),
            ],
        )?;
        Ok(())
    }

    #[tokio::test]
    async fn happy_path_with_min_aggregation() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let destination_sorted_set_name = unique_key();
        let sorted_set_one = TestSortedSet::new();
        let sorted_set_two = TestSortedSet::new();
        let sources = vec![
            SortedSetUnionStoreSource::new(sorted_set_one.name(), 1.0),
            SortedSetUnionStoreSource::new(sorted_set_two.name(), 2.0),
        ];

        // Destination set should be empty
        let result = client
            .sorted_set_length(cache_name, destination_sorted_set_name.clone())
            .await?;
        assert_eq!(result, SortedSetLengthResponse::Miss);

        // Insert two sets, but duplicate the values from the first set.
        let result = client
            .sorted_set_put_elements(
                cache_name,
                sorted_set_one.name(),
                sorted_set_one.value().to_vec(),
            )
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});
        let result = client
            .sorted_set_put_elements(
                cache_name,
                sorted_set_two.name(),
                sorted_set_one.value().to_vec(),
            )
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Nonzero length after sorted set exists
        let request = SortedSetUnionStoreRequest::new(
            cache_name,
            destination_sorted_set_name.clone(),
            sources.clone(),
        )
        .aggregate(SortedSetAggregateFunction::Min);
        let result = client.send_request(request).await?;
        assert_eq!(result, SortedSetUnionStoreResponse { length: 2 });

        // Test that the destination set has the correct elements
        let result = client
            .sorted_set_fetch_by_score(cache_name, destination_sorted_set_name, Ascending)
            .await?;
        assert_fetched_sorted_set_eq_after_sorting(
            result,
            vec![
                (
                    sorted_set_one.value()[0].0.clone(),
                    sorted_set_one.value()[0].1,
                ),
                (
                    sorted_set_one.value()[1].0.clone(),
                    sorted_set_one.value()[1].1,
                ),
            ],
        )?;
        Ok(())
    }

    #[tokio::test]
    async fn happy_path_with_max_aggregation() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let destination_sorted_set_name = unique_key();
        let sorted_set_one = TestSortedSet::new();
        let sorted_set_two = TestSortedSet::new();
        let sources = vec![
            SortedSetUnionStoreSource::new(sorted_set_one.name(), 1.0),
            SortedSetUnionStoreSource::new(sorted_set_two.name(), 0.0),
        ];

        // Destination set should be empty
        let result = client
            .sorted_set_length(cache_name, destination_sorted_set_name.clone())
            .await?;
        assert_eq!(result, SortedSetLengthResponse::Miss);

        // Insert two sets, but duplicate the values from the first set.
        let result = client
            .sorted_set_put_elements(
                cache_name,
                sorted_set_one.name(),
                sorted_set_one.value().to_vec(),
            )
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});
        let result = client
            .sorted_set_put_elements(
                cache_name,
                sorted_set_two.name(),
                sorted_set_one.value().to_vec(),
            )
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Nonzero length after sorted set exists
        let request = SortedSetUnionStoreRequest::new(
            cache_name,
            destination_sorted_set_name.clone(),
            sources.clone(),
        )
        .aggregate(SortedSetAggregateFunction::Max);
        let result = client.send_request(request).await?;
        assert_eq!(result, SortedSetUnionStoreResponse { length: 2 });

        // Test that the destination set has the correct elements
        let result = client
            .sorted_set_fetch_by_score(cache_name, destination_sorted_set_name, Ascending)
            .await?;
        assert_fetched_sorted_set_eq_after_sorting(
            result,
            vec![
                (
                    sorted_set_one.value()[0].0.clone(),
                    sorted_set_one.value()[0].1,
                ),
                (
                    sorted_set_one.value()[1].0.clone(),
                    sorted_set_one.value()[1].1,
                ),
            ],
        )?;
        Ok(())
    }

    #[tokio::test]
    async fn duplicate_sources() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let destination_sorted_set_name = unique_key();
        let sorted_set_one = TestSortedSet::new();
        let sources = vec![(sorted_set_one.name(), 1.0), (sorted_set_one.name(), 2.0)];

        // Destination set should be empty
        let result = client
            .sorted_set_length(cache_name, destination_sorted_set_name.clone())
            .await?;
        assert_eq!(result, SortedSetLengthResponse::Miss);

        // Insert the set
        let result = client
            .sorted_set_put_elements(
                cache_name,
                sorted_set_one.name(),
                sorted_set_one.value().to_vec(),
            )
            .await?;
        assert_eq!(result, SortedSetPutElementsResponse {});

        // Union of the set with itself should produce a set with length 2
        let result = client
            .sorted_set_union_store(cache_name, destination_sorted_set_name.clone(), sources)
            .await?;
        assert_eq!(result, SortedSetUnionStoreResponse { length: 2 });

        // Test that the destination set has the correct elements
        let result = client
            .sorted_set_fetch_by_score(cache_name, destination_sorted_set_name, Ascending)
            .await?;
        // Should have unioned the values with values*2 using sum aggregation
        let expected_elements = sorted_set_one
            .value()
            .iter()
            .cloned()
            .map(|(k, v)| (k, v + v * 2.0))
            .collect();
        assert_fetched_sorted_set_eq_after_sorting(result, expected_elements)?;
        Ok(())
    }

    #[tokio::test]
    async fn empty_sources_list() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let sorted_set_name = "sorted-set";

        let empty_sources: Vec<SortedSetUnionStoreSource<&str>> = vec![];
        let result = client
            .sorted_set_union_store(cache_name, sorted_set_name, empty_sources)
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_key();
        let sorted_set_name = "sorted-set";
        let sources = vec![
            SortedSetUnionStoreSource::new("one_sorted_set", 1.0),
            SortedSetUnionStoreSource::new("two_sorted_set", 2.0),
        ];

        let result = client
            .sorted_set_union_store(cache_name, sorted_set_name, sources)
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }
}
