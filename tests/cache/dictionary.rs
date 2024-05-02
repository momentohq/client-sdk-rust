use momento::cache::{
    DictionaryFetch, DictionaryGetFields, DictionaryRemoveFields, DictionarySetField,
    DictionarySetFields,
};
use momento::{MomentoError, MomentoErrorCode, MomentoResult};
use momento_test_util::{
    unique_cache_name, unique_key, unique_value, TestDictionary, CACHE_TEST_STATE,
};
use std::collections::HashMap;
use std::convert::TryInto;

fn assert_fetched_dictionary_equals_test_data(
    dictionary_fetch_result: DictionaryFetch,
    expected: &TestDictionary,
) -> Result<(), MomentoError> {
    let actual: HashMap<String, String> = dictionary_fetch_result.try_into()?;
    assert_eq!(actual, *expected.value());
    Ok(())
}

mod dictionary_fetch {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;

        // Test a miss
        let dictionary_name = unique_key();
        let result = client.dictionary_fetch(cache_name, dictionary_name).await?;
        assert_eq!(result, DictionaryFetch::Miss);

        // Test a hit
        let item = TestDictionary::new();
        let dictionary_set_response = client
            .dictionary_set_fields(cache_name, item.name(), item.value().clone())
            .await?;
        assert_eq!(dictionary_set_response, DictionarySetFields {});
        let result = client.dictionary_fetch(cache_name, item.name()).await?;
        assert_fetched_dictionary_equals_test_data(result, &item)?;
        Ok(())
    }

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client.dictionary_fetch("   ", "key").await.unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let result = client
            .dictionary_fetch(cache_name, "my-dictionary")
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }
}

mod dictionary_get_field {}

mod dictionary_get_fields {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;

        // Add some test data
        let item = TestDictionary::new();
        let response = client
            .dictionary_set_fields(cache_name, item.name(), item.value().clone())
            .await?;
        assert_eq!(response, DictionarySetFields {});

        let item2 = TestDictionary::new();
        let response = client
            .dictionary_set_fields(cache_name, item2.name(), item2.value().clone())
            .await?;
        assert_eq!(response, DictionarySetFields {});

        // Now get the values relevant to the first dictionary
        let result = client
            .dictionary_get_fields(
                cache_name,
                item.name(),
                item.value().keys().cloned().collect(),
            )
            .await?;
        match result {
            DictionaryGetFields::Hit { .. } => {
                let actual: HashMap<String, String> = result.try_into().expect("Stored string-string field-value pairs but could not convert into HashMap<String, String>");
                assert_eq!(actual, *item.value());
            }
            DictionaryGetFields::Miss => panic!("I expected a hit!"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .dictionary_get_fields("   ", "my-dictionary", vec!["my-field".to_string()])
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let result = client
            .dictionary_get_fields(cache_name, "my-dictionary", vec!["my-field".to_string()])
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }
}

mod dictionary_increment {}

mod dictionary_remove_field {}

mod dictionary_remove_fields {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;

        let item = TestDictionary::new();
        let response = client
            .dictionary_set_fields(cache_name, item.name(), item.value().clone())
            .await?;
        assert_eq!(response, DictionarySetFields {});

        let item2 = TestDictionary::new();
        let response = client
            .dictionary_set_fields(cache_name, item.name(), item2.value().clone())
            .await?;
        assert_eq!(response, DictionarySetFields {});

        let response = client
            .dictionary_remove_fields(
                cache_name,
                item.name(),
                item.value().keys().cloned().collect(),
            )
            .await?;
        assert_eq!(response, DictionaryRemoveFields {});

        let result = client.dictionary_fetch(cache_name, item.name()).await?;
        assert_fetched_dictionary_equals_test_data(result, &item2)?;
        Ok(())
    }

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .dictionary_remove_fields("   ", "my-dictionary", vec!["my-field".to_string()])
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let result = client
            .dictionary_remove_fields(cache_name, "my-dictionary", vec!["my-field".to_string()])
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }
}

mod dictionary_set_field {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;

        let pair = (unique_key(), unique_value());
        let item = TestDictionary {
            name: unique_key(),
            value: HashMap::from([pair.clone()]),
        };

        let response = client
            .dictionary_set_field(cache_name, item.name(), pair.0, pair.1)
            .await?;
        assert_eq!(response, DictionarySetField {});

        let result = client.dictionary_fetch(cache_name, item.name()).await?;
        assert_fetched_dictionary_equals_test_data(result, &item)?;
        Ok(())
    }

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .dictionary_set_field("   ", "my-dictionary", "my-field", "my-value")
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let result = client
            .dictionary_set_field(cache_name, "my-dictionary", "my-field", "my-value")
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }
}

mod dictionary_set_fields {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;

        let item = TestDictionary::new();
        let response = client
            .dictionary_set_fields(cache_name, item.name(), item.value().clone())
            .await?;
        assert_eq!(response, DictionarySetFields {});

        let result = client.dictionary_fetch(cache_name, item.name()).await?;
        assert_fetched_dictionary_equals_test_data(result, &item)?;
        Ok(())
    }

    #[tokio::test]
    async fn invalid_cache_name() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let result = client
            .dictionary_set_fields(
                "   ",
                "my-dictionary",
                TestDictionary::default().value().clone(),
            )
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::InvalidArgumentError);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_cache() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = unique_cache_name();
        let result = client
            .dictionary_set_fields(
                cache_name,
                "my-dictionary",
                TestDictionary::default().value().clone(),
            )
            .await
            .unwrap_err();

        assert_eq!(result.error_code, MomentoErrorCode::NotFoundError);
        Ok(())
    }
}

mod dictionary_length {}
