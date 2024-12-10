use momento::cache::messages::data::scalar::get::Value;
use momento::cache::GetResponse;
use momento::{MomentoErrorCode, MomentoResult};
use momento_test_util::{unique_cache_name, unique_key, TestScalar, CACHE_TEST_STATE};
use std::collections::HashMap;
use std::convert::TryInto;
use std::iter::zip;

mod batch_get_set {
    use super::*;

    #[tokio::test]
    async fn get_batch_invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let keys = vec!["a", "b", "c"];
        let result = client.get_batch("   ", keys).await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn get_batch_nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let keys = vec!["a", "b", "c"];
        let result = client.get_batch(cache_name, keys).await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn get_batch_happy_path_with_all_misses() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let keys = vec![unique_key(), unique_key(), unique_key()];
        let response = client.get_batch(cache_name, keys.clone()).await?;

        let results_list: Vec<GetResponse> = response.clone().into();
        assert_eq!(results_list.len(), 3);
        for (result, key) in zip(results_list, keys.clone()) {
            assert_eq!(
                result,
                GetResponse::Miss,
                "Expected miss for key '{}' in cache {}, got {:?}",
                key,
                cache_name,
                result
            );
        }

        let results_map: HashMap<String, GetResponse> = response.clone().into();
        assert_eq!(results_map.len(), keys.len());
        for key in keys {
            let retrieved_value = results_map.get(&key).unwrap();
            assert_eq!(*retrieved_value, GetResponse::Miss {});
        }

        let results_values_map: HashMap<String, Value> = response.clone().into();
        assert_eq!(results_values_map.len(), 0);

        let results_values_map_string: HashMap<String, String> = response
            .clone()
            .try_into()
            .expect("Expected a HashMap<String, String>");
        assert_eq!(results_values_map_string.len(), 0);

        let results_values_map_bytes: HashMap<String, Vec<u8>> = response.clone().into();
        assert_eq!(results_values_map_bytes.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn get_batch_happy_path_some_hits_and_misses() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();

        let items = [TestScalar::new(), TestScalar::new(), TestScalar::new()];
        for item in items.iter() {
            client.set(cache_name, item.key(), item.value()).await?;
        }

        let nonexistent_keys = vec![unique_key(), unique_key(), unique_key()];
        let all_keys = [
            nonexistent_keys.clone(),
            items.iter().map(|item| item.key().to_string()).collect(),
        ]
        .concat();
        let response = client.get_batch(cache_name, all_keys.clone()).await?;

        let results_list: Vec<GetResponse> = response.clone().into();
        assert_eq!(results_list.len(), all_keys.len());
        for (result, key) in zip(results_list, all_keys.clone()) {
            if nonexistent_keys.contains(&key) {
                assert_eq!(
                    result,
                    GetResponse::Miss,
                    "Expected miss for key '{}' in cache {}, got {:?}",
                    key,
                    cache_name,
                    result
                );
            } else {
                for item in items.iter() {
                    if item.key() == key {
                        assert_eq!(result, GetResponse::from(item));
                    }
                }
            }
        }

        let results_map: HashMap<String, GetResponse> = response.clone().into();
        assert_eq!(results_map.len(), all_keys.len());
        for key in all_keys {
            let retrieved_value = results_map.get(&key).unwrap();
            if nonexistent_keys.contains(&key) {
                assert_eq!(*retrieved_value, GetResponse::Miss {});
            } else {
                for item in items.iter() {
                    if item.key() == key {
                        assert_eq!(*retrieved_value, GetResponse::from(item));
                    }
                }
            }
        }

        let results_values_map: HashMap<String, Value> = response.clone().into();
        assert_eq!(results_values_map.len(), items.len());

        let results_values_map_string: HashMap<String, String> = response
            .clone()
            .try_into()
            .expect("Expected a HashMap<String, String>");
        assert_eq!(results_values_map_string.len(), items.len());
        for item in items.iter() {
            let retrieved_value = results_values_map_string.get(item.key()).unwrap();
            assert_eq!(*retrieved_value, item.value().to_string());
        }

        let results_values_map_bytes: HashMap<String, Vec<u8>> = response.clone().into();
        assert_eq!(results_values_map_bytes.len(), items.len());
        for item in items.iter() {
            let retrieved_value = results_values_map_bytes.get(item.key()).unwrap();
            assert_eq!(*retrieved_value, item.value().as_bytes());
        }

        Ok(())
    }
}
