use uuid::Uuid;

use momento::requests::cache::sorted_set::sorted_set_fetch_by_rank::SortOrder::{
    Ascending, Descending,
};
use momento::requests::cache::sorted_set::sorted_set_fetch_by_rank::SortedSetFetchByRankRequest;
use momento::requests::cache::sorted_set::sorted_set_fetch_by_score::SortedSetFetchByScoreRequest;
use momento::requests::cache::sorted_set::sorted_set_fetch_response::SortedSetFetch;

use momento_test_util::CACHE_TEST_STATE;

#[tokio::test]
async fn sorted_set_put_element_happy_path() {
    let client = CACHE_TEST_STATE.client.clone();
    let cache_name = CACHE_TEST_STATE.cache_name.clone();
    let sorted_set_name = "sorted-set-".to_string() + &Uuid::new_v4().to_string();
    let value = "value";
    let score = 1.0;

    let result = client
        .sorted_set_fetch_by_rank(cache_name.clone(), sorted_set_name.clone(), Ascending)
        .await
        .unwrap();
    assert_eq!(result, SortedSetFetch::Miss);

    client
        .sorted_set_put_element(cache_name.clone(), sorted_set_name.clone(), "value", 1.0)
        .await
        .unwrap();

    let result = client
        .sorted_set_fetch_by_rank(cache_name.clone(), sorted_set_name.clone(), Ascending)
        .await
        .unwrap();

    match result {
        SortedSetFetch::Hit { elements } => {
            assert_eq!(elements.len(), 1);
            let string_elements = elements.into_strings().unwrap();
            assert_eq!(string_elements[0].0, value);
            assert_eq!(string_elements[0].1, score)
        }
        _ => panic!("Expected SortedSetFetch::Hit, but got {:?}", result),
    }
}

#[tokio::test]
async fn sorted_set_put_element_nonexistent_cache() {
    let client = CACHE_TEST_STATE.client.clone();
    let cache_name = "fake-cache-".to_string() + &Uuid::new_v4().to_string();
    let sorted_set_name = "sorted-set";

    let result = client
        .sorted_set_put_element(cache_name, sorted_set_name, "value", 1.0)
        .await
        .unwrap_err();

    let _err_msg = "Cache name cannot be empty".to_string();
    assert!(matches!(result.to_string(), _err_message))
}

#[tokio::test]
async fn sorted_set_put_elements_happy_path() {
    let client = CACHE_TEST_STATE.client.clone();
    let cache_name = CACHE_TEST_STATE.cache_name.clone();
    let sorted_set_name = "sorted-set-".to_string() + &Uuid::new_v4().to_string();
    let to_put = vec![("element1".to_string(), 1.0), ("element2".to_string(), 2.0)];

    let result = client
        .sorted_set_fetch_by_score(cache_name.clone(), sorted_set_name.clone(), Ascending)
        .await
        .unwrap();
    assert_eq!(result, SortedSetFetch::Miss);

    client
        .sorted_set_put_elements(cache_name.clone(), sorted_set_name.clone(), to_put.clone())
        .await
        .unwrap();

    let result = client
        .sorted_set_fetch_by_score(cache_name.clone(), sorted_set_name.clone(), Ascending)
        .await
        .unwrap();

    match result {
        SortedSetFetch::Hit { elements } => {
            assert_eq!(elements.len(), 2);
            let string_elements = elements.into_strings().unwrap();
            assert_eq!(to_put, string_elements)
        }
        _ => panic!("Expected SortedSetFetch::Hit, but got {:?}", result),
    }
}

#[tokio::test]
async fn sorted_set_put_elements_nonexistent_cache() {
    let client = CACHE_TEST_STATE.client.clone();
    let cache_name = "fake-cache-".to_string() + &Uuid::new_v4().to_string();
    let sorted_set_name = "sorted-set";

    let result = client
        .sorted_set_put_elements(
            cache_name.clone(),
            sorted_set_name,
            vec![("element1".to_string(), 1.0), ("element2".to_string(), 2.0)],
        )
        .await
        .unwrap_err();

    let _err_msg = "Cache name cannot be empty".to_string();
    assert!(matches!(result.to_string(), _err_message))
}

#[tokio::test]
async fn sorted_set_fetch_by_rank_happy_path() {
    let client = CACHE_TEST_STATE.client.clone();
    let cache_name = CACHE_TEST_STATE.cache_name.clone();
    let sorted_set_name = "sorted-set-".to_string() + &Uuid::new_v4().to_string();
    let to_put = vec![
        ("1".to_string(), 0.0),
        ("2".to_string(), 1.0),
        ("3".to_string(), 0.5),
        ("4".to_string(), 2.0),
        ("5".to_string(), 1.5),
    ];

    let result = client
        .sorted_set_fetch_by_rank(cache_name.clone(), sorted_set_name.clone(), Ascending)
        .await
        .unwrap();
    assert_eq!(result, SortedSetFetch::Miss);

    client
        .sorted_set_put_elements(cache_name.clone(), sorted_set_name.clone(), to_put.clone())
        .await
        .unwrap();

    // Full set ascending, end index larger than set
    let fetch_request =
        SortedSetFetchByRankRequest::new(cache_name.clone(), sorted_set_name.clone())
            .with_order(Ascending)
            .with_start_rank(0)
            .with_end_rank(6);

    let result = client.send_request(fetch_request).await.unwrap();

    match result {
        SortedSetFetch::Hit { elements } => {
            assert_eq!(elements.len(), 5);
            let string_elements: Vec<String> = elements
                .into_strings()
                .unwrap()
                .into_iter()
                .map(|e| e.0)
                .collect();

            assert_eq!(string_elements, vec!["1", "3", "2", "5", "4"])
        }
        _ => panic!("Expected SortedSetFetch::Hit, but got {:?}", result),
    }

    // Partial set descending
    let fetch_request =
        SortedSetFetchByRankRequest::new(cache_name.clone(), sorted_set_name.clone())
            .with_order(Descending)
            .with_start_rank(1)
            .with_end_rank(4);

    let result = client.send_request(fetch_request).await.unwrap();

    match result {
        SortedSetFetch::Hit { elements } => {
            assert_eq!(elements.len(), 3);
            let string_elements: Vec<String> = elements
                .into_strings()
                .unwrap()
                .into_iter()
                .map(|e| e.0)
                .collect();

            assert_eq!(string_elements, vec!["5", "2", "3"])
        }
        _ => panic!("Expected SortedSetFetch::Hit, but got {:?}", result),
    }
}

#[tokio::test]
async fn sorted_set_fetch_by_rank_nonexistent_cache() {
    let client = CACHE_TEST_STATE.client.clone();
    let cache_name = "fake-cache-".to_string() + &Uuid::new_v4().to_string();
    let sorted_set_name = "sorted-set";

    let result = client
        .sorted_set_fetch_by_rank(cache_name.clone(), sorted_set_name, Ascending)
        .await
        .unwrap_err();

    let _err_msg = "Cache name cannot be empty".to_string();
    assert!(matches!(result.to_string(), _err_message))
}

#[tokio::test]
async fn sorted_set_fetch_by_score_happy_path() {
    let client = CACHE_TEST_STATE.client.clone();
    let cache_name = CACHE_TEST_STATE.cache_name.clone();
    let sorted_set_name = "sorted-set-".to_string() + &Uuid::new_v4().to_string();
    let to_put = vec![
        ("1".to_string(), 0.0),
        ("2".to_string(), 1.0),
        ("3".to_string(), 0.5),
        ("4".to_string(), 2.0),
        ("5".to_string(), 1.5),
    ];

    let result = client
        .sorted_set_fetch_by_score(cache_name.clone(), sorted_set_name.clone(), Ascending)
        .await
        .unwrap();
    assert_eq!(result, SortedSetFetch::Miss);

    client
        .sorted_set_put_elements(cache_name.clone(), sorted_set_name.clone(), to_put.clone())
        .await
        .unwrap();

    // Full set ascending, end score larger than set
    let fetch_request =
        SortedSetFetchByScoreRequest::new(cache_name.clone(), sorted_set_name.clone())
            .with_order(Ascending)
            .with_min_score(0.0)
            .with_max_score(9.9);

    let result = client.send_request(fetch_request).await.unwrap();

    match result {
        SortedSetFetch::Hit { elements } => {
            assert_eq!(elements.len(), 5);
            let string_elements: Vec<String> = elements
                .into_strings()
                .unwrap()
                .into_iter()
                .map(|e| e.0)
                .collect();

            assert_eq!(string_elements, vec!["1", "3", "2", "5", "4"])
        }
        _ => panic!("Expected SortedSetFetch::Hit, but got {:?}", result),
    }

    // Partial set descending
    let fetch_request =
        SortedSetFetchByScoreRequest::new(cache_name.clone(), sorted_set_name.clone())
            .with_order(Descending)
            .with_min_score(0.1)
            .with_max_score(1.9);

    let result = client.send_request(fetch_request).await.unwrap();

    match result {
        SortedSetFetch::Hit { elements } => {
            assert_eq!(elements.len(), 3);
            let string_elements: Vec<String> = elements
                .into_strings()
                .unwrap()
                .into_iter()
                .map(|e| e.0)
                .collect();

            assert_eq!(string_elements, vec!["5", "2", "3"])
        }
        _ => panic!("Expected SortedSetFetch::Hit, but got {:?}", result),
    }

    // Partial set limited by offset and count
    let fetch_request =
        SortedSetFetchByScoreRequest::new(cache_name.clone(), sorted_set_name.clone())
            .with_offset(1)
            .with_count(3);

    let result = client.send_request(fetch_request).await.unwrap();

    match result {
        SortedSetFetch::Hit { elements } => {
            assert_eq!(elements.len(), 3);
            let string_elements: Vec<String> = elements
                .into_strings()
                .unwrap()
                .into_iter()
                .map(|e| e.0)
                .collect();

            assert_eq!(string_elements, vec!["3", "2", "5"])
        }
        _ => panic!("Expected SortedSetFetch::Hit, but got {:?}", result),
    }
}

#[tokio::test]
async fn sorted_set_fetch_by_score_nonexistent_cache() {
    let client = CACHE_TEST_STATE.client.clone();
    let cache_name = "fake-cache-".to_string() + &Uuid::new_v4().to_string();
    let sorted_set_name = "sorted-set";

    let result = client
        .sorted_set_fetch_by_score(cache_name.clone(), sorted_set_name, Ascending)
        .await
        .unwrap_err();

    let _err_msg = "Cache name cannot be empty".to_string();
    assert!(matches!(result.to_string(), _err_message))
}
