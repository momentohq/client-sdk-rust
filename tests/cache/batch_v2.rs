use momento::cache::messages::data::scalar::get::Value;
use momento::cache::{GetResponse, SetResponse};
use momento::{MomentoErrorCode, MomentoResult};
use momento_test_util::{unique_cache_name, unique_key, TestScalar, CACHE_TEST_STATE};
use std::collections::HashMap;
use std::convert::TryInto;

mod batch_get_set {
    use momento::cache::SetBatchRequest;

    use super::*;

    #[tokio::test]
    async fn get_batch_invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client_v2;
        let keys = vec!["a", "b", "c"];
        let result = client.get_batch("   ", keys).await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn get_batch_nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client_v2;
        let cache_name = unique_cache_name();
        let keys = vec!["a", "b", "c"];
        let result = client.get_batch(cache_name, keys).await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn get_batch_happy_path_with_all_misses() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client_v2;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();
        let keys = vec![unique_key(), unique_key(), unique_key()];
        let response = client.get_batch(cache_name, keys.clone()).await?;

        // Then we verify that each possible GetBatchResponse conversion works as expected

        let byte_keys_get_responses: HashMap<Vec<u8>, GetResponse> = response.clone().into();
        assert_eq!(byte_keys_get_responses.len(), keys.len());
        for key in keys.iter() {
            let byte_key = key.as_bytes();
            let retrieved_value = byte_keys_get_responses.get(byte_key).unwrap();
            assert_eq!(*retrieved_value, GetResponse::Miss {});
        }

        let str_keys_get_responses: HashMap<String, GetResponse> =
            response.clone().try_into().expect("string keys");
        assert_eq!(str_keys_get_responses.len(), keys.len());
        for key in keys {
            let retrieved_value = str_keys_get_responses.get(&key).unwrap();
            assert_eq!(*retrieved_value, GetResponse::Miss {});
        }

        let byte_keys_hit_values: HashMap<Vec<u8>, Value> = response.clone().into();
        assert_eq!(byte_keys_hit_values.len(), 0);

        let str_keys_hit_values: HashMap<String, Value> =
            response.clone().try_into().expect("string keys");
        assert_eq!(str_keys_hit_values.len(), 0);

        let byte_keys_hit_bytes: HashMap<Vec<u8>, Vec<u8>> = response.clone().into();
        assert_eq!(byte_keys_hit_bytes.len(), 0);

        let str_keys_hit_bytes: HashMap<String, Vec<u8>> =
            response.clone().try_into().expect("stored string keys");
        assert_eq!(str_keys_hit_bytes.len(), 0);

        let byte_keys_hit_strings: HashMap<Vec<u8>, String> =
            response.clone().try_into().expect("stored string values");
        assert_eq!(byte_keys_hit_strings.len(), 0);

        let str_keys_hit_strings: HashMap<String, String> = response
            .clone()
            .try_into()
            .expect("stored string keys and values");
        assert_eq!(str_keys_hit_strings.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn get_set_batch_happy_path_with_some_hits_and_misses() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client_v2;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();

        let items = [TestScalar::new(), TestScalar::new(), TestScalar::new()];
        let items_map = HashMap::from([
            (items[0].key(), items[0].value()),
            (items[1].key(), items[1].value()),
            (items[2].key(), items[2].value()),
        ]);
        let set_batch_response = client.set_batch(cache_name, items_map.clone()).await?;

        // Then we verify that each possible SetBatchResponse conversion works as expected

        let byte_keys_set_responses: HashMap<Vec<u8>, SetResponse> =
            set_batch_response.clone().into();
        assert_eq!(byte_keys_set_responses.len(), items.len());
        for item in items.iter() {
            let set_value = byte_keys_set_responses.get(item.key().as_bytes()).unwrap();
            assert_eq!(*set_value, SetResponse {});
        }

        let str_keys_set_responses: HashMap<String, SetResponse> =
            set_batch_response.try_into().expect("string keys");
        assert_eq!(str_keys_set_responses.len(), items.len());
        for item in items.iter() {
            let set_value = str_keys_set_responses.get(item.key()).unwrap();
            assert_eq!(*set_value, SetResponse {});
        }

        // Now create some keys that won't exist in the cache

        let nonexistent_keys = vec![unique_key(), unique_key(), unique_key()];
        let all_keys = [
            nonexistent_keys.clone(),
            items.iter().map(|item| item.key().to_string()).collect(),
        ]
        .concat();
        let response = client.get_batch(cache_name, all_keys.clone()).await?;

        // Then we verify that each possible GetBatchResponse conversion works as expected

        let byte_keys_get_responses: HashMap<Vec<u8>, GetResponse> = response.clone().into();
        assert_eq!(byte_keys_get_responses.len(), all_keys.len());
        for key in all_keys.iter() {
            let byte_key = key.as_bytes();
            let retrieved_value = byte_keys_get_responses.get(byte_key).unwrap();
            if nonexistent_keys.contains(key) {
                assert_eq!(*retrieved_value, GetResponse::Miss {});
            } else {
                for item in items.iter() {
                    if item.key() == key {
                        assert_eq!(*retrieved_value, GetResponse::from(item));
                    }
                }
            }
        }

        let str_keys_get_responses: HashMap<String, GetResponse> =
            response.clone().try_into().expect("string keys");
        assert_eq!(str_keys_get_responses.len(), all_keys.len());
        for key in all_keys.iter() {
            let retrieved_value = str_keys_get_responses.get(key).unwrap();
            if nonexistent_keys.contains(key) {
                assert_eq!(*retrieved_value, GetResponse::Miss {});
            } else {
                for item in items.iter() {
                    if item.key() == key {
                        assert_eq!(*retrieved_value, GetResponse::from(item));
                    }
                }
            }
        }

        let byte_keys_hit_values: HashMap<Vec<u8>, Value> = response.clone().into();
        assert_eq!(byte_keys_hit_values.len(), items.len());
        for item in items.iter() {
            let byte_key = item.key().as_bytes();
            let retrieved_value = byte_keys_hit_values.get(byte_key).unwrap();
            let item_value = Value::new(item.value().as_bytes().to_vec());
            assert_eq!(*retrieved_value, item_value);
        }

        let str_keys_hit_values: HashMap<String, Value> =
            response.clone().try_into().expect("string keys");
        assert_eq!(str_keys_hit_values.len(), items.len());
        for item in items.iter() {
            let retrieved_value = str_keys_hit_values.get(item.key()).unwrap();
            let item_value = Value::new(item.value().as_bytes().to_vec());
            assert_eq!(*retrieved_value, item_value);
        }

        let byte_keys_hit_bytes: HashMap<Vec<u8>, Vec<u8>> = response.clone().into();
        assert_eq!(byte_keys_hit_bytes.len(), items.len());
        for item in items.iter() {
            let byte_key = item.key().as_bytes();
            let retrieved_value = byte_keys_hit_bytes.get(byte_key).unwrap();
            assert_eq!(retrieved_value, item.value().as_bytes());
        }

        let str_keys_hit_bytes: HashMap<String, Vec<u8>> =
            response.clone().try_into().expect("stored string keys");
        assert_eq!(str_keys_hit_bytes.len(), items.len());
        for item in items.iter() {
            let retrieved_value = str_keys_hit_bytes.get(item.key()).unwrap();
            assert_eq!(retrieved_value, item.value().as_bytes());
        }

        let byte_keys_hit_strings: HashMap<Vec<u8>, String> =
            response.clone().try_into().expect("stored string values");
        for item in items.iter() {
            let byte_key = item.key().as_bytes();
            let retrieved_value = byte_keys_hit_strings.get(byte_key).unwrap();
            assert_eq!(retrieved_value, item.value());
        }

        let str_keys_hit_strings: HashMap<String, String> = response
            .clone()
            .try_into()
            .expect("stored string keys and values");
        assert_eq!(str_keys_hit_strings.len(), items.len());
        for item in items.iter() {
            let retrieved_value = str_keys_hit_strings.get(item.key()).unwrap();
            assert_eq!(retrieved_value, item.value());
        }

        Ok(())
    }

    #[tokio::test]
    async fn set_batch_invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client_v2;
        let items: HashMap<&str, &str> = HashMap::from([("k1", "v1"), ("k1", "v1")]);
        let result = client.set_batch("   ", items).await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn set_batch_nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client_v2;
        let cache_name = unique_cache_name();
        let items: HashMap<&str, &str> = HashMap::from([("k1", "v1"), ("k1", "v1")]);
        let result = client.set_batch(cache_name, items).await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);
        Ok(())
    }

    #[tokio::test]
    async fn set_batch_with_ttl() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client_v2;
        let cache_name = CACHE_TEST_STATE.cache_name.as_str();

        let items = [TestScalar::new(), TestScalar::new(), TestScalar::new()];
        let items_map = HashMap::from([
            (items[0].key(), items[0].value()),
            (items[1].key(), items[1].value()),
            (items[2].key(), items[2].value()),
        ]);

        let set_batch_request = SetBatchRequest::new(cache_name, items_map.clone())
            .ttl(std::time::Duration::from_secs(60));
        let set_batch_response = client.send_request(set_batch_request).await?;

        // Then we verify that each possible SetBatchResponse conversion works as expected

        let byte_keys_set_responses: HashMap<Vec<u8>, SetResponse> =
            set_batch_response.clone().into();
        assert_eq!(byte_keys_set_responses.len(), items.len());
        for item in items.iter() {
            let set_value = byte_keys_set_responses.get(item.key().as_bytes()).unwrap();
            assert_eq!(*set_value, SetResponse {});
        }

        let str_keys_set_responses: HashMap<String, SetResponse> =
            set_batch_response.try_into().expect("string keys");
        assert_eq!(str_keys_set_responses.len(), items.len());
        for item in items.iter() {
            let set_value = str_keys_set_responses.get(item.key()).unwrap();
            assert_eq!(*set_value, SetResponse {});
        }

        Ok(())
    }
}
