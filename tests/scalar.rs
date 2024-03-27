use momento::requests::cache::basic::set::Set;
use momento_test_util::CACHE_TEST_STATE;
use uuid::Uuid;

#[tokio::test]
async fn set_happy_path() {
    let client = CACHE_TEST_STATE.client.clone();
    let cache_name = CACHE_TEST_STATE.cache_name.clone();
    let key = &Uuid::new_v4().to_string();
    let value = &Uuid::new_v4().to_string();

    let result = client
        .set(cache_name.clone(), key.clone(), value.clone())
        .await
        .expect("Failed to execute set operation");

    assert_eq!(result, Set {});
}
