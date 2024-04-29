use momento::cache::DictionaryFetch;
use momento::{MomentoErrorCode, MomentoResult};
use momento_test_util::{unique_cache_name, unique_key, CACHE_TEST_STATE};

mod dictionary_fetch {
    use super::*;

    #[tokio::test]
    async fn happy_path() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let dictionary_name = unique_key();
        let result = client.dictionary_fetch(cache_name, dictionary_name).await?;
        assert_eq!(result, DictionaryFetch::Miss);
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

mod dictionary_get_fields {}

mod dictionary_increment {}

mod dictionary_remove_field {}

mod dictionary_remove_fields {}

mod dictionary_set_field {}

mod dictionary_set_fields {}

mod dictionary_length {}
