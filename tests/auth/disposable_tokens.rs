use std::time::Duration;

use momento::auth::CacheSelector;
use momento::auth::{CachePermission, CacheRole, Permission, Permissions};
use momento::{
    auth::{
        DisposableTokenScope, DisposableTokenScopes, ExpiresIn, GenerateDisposableTokenResponse,
    },
    CacheClient, CredentialProvider, IntoBytes, MomentoResult, TopicClient,
};
use momento::{MomentoError, MomentoErrorCode};
use momento_test_util::{unique_key, TestScalar, CACHE_TEST_STATE};

// Helper function that generates a disposable token with the given scope
// that expires in 5 minutes.
async fn generate_disposable_token_success(
    scope: DisposableTokenScope<impl IntoBytes>,
) -> MomentoResult<GenerateDisposableTokenResponse> {
    let expiry = ExpiresIn::minutes(5);
    let response = CACHE_TEST_STATE
        .auth_client
        .generate_disposable_token(scope, expiry)
        .await?;
    assert!(!response.clone().auth_token().is_empty());
    Ok(response)
}

fn new_credential_provider_from_token(token: String) -> CredentialProvider {
    CredentialProvider::from_string(token).expect("auth token should be valid")
}

fn new_cache_client(credential_provider: CredentialProvider) -> CacheClient {
    CacheClient::builder()
        .default_ttl(Duration::from_secs(30))
        .configuration(momento::cache::configurations::Laptop::latest())
        .credential_provider(credential_provider)
        .build()
        .expect("Failed to create cache client")
}

fn new_topic_client(credential_provider: CredentialProvider) -> TopicClient {
    TopicClient::builder()
        .configuration(momento::topics::configurations::Laptop::latest())
        .credential_provider(credential_provider)
        .build()
        .expect("Failed to create topic client")
}

async fn assert_get_success(
    cache_client: &CacheClient,
    cache_name: String,
    key: String,
) -> MomentoResult<()> {
    match cache_client.get(cache_name.clone(), key.clone()).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!(
                "Expected to successfully get key '{}' from cache '{}'",
                key, cache_name
            );
            Err(e)
        }
    }
}

async fn assert_get_failure(
    cache_client: &CacheClient,
    cache_name: String,
    key: String,
) -> MomentoResult<()> {
    match cache_client.get(cache_name.clone(), key.clone()).await {
        Ok(_) => Err(MomentoError {
            message: format!(
                "Expected getting key '{}' from cache '{}' to fail",
                key, cache_name
            ),
            error_code: MomentoErrorCode::UnknownError,
            inner_error: None,
            details: None,
        }),
        Err(e) => {
            assert_eq!(e.error_code, MomentoErrorCode::PermissionError);
            Ok(())
        }
    }
}

async fn assert_set_success(
    cache_client: &CacheClient,
    cache_name: String,
    key: String,
    value: String,
) -> MomentoResult<()> {
    match cache_client
        .set(cache_name.clone(), key.clone(), value.clone())
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!(
                "Expected to successfully set value '{}' for key '{}' from cache '{}'",
                value, key, cache_name
            );
            Err(e)
        }
    }
}

async fn assert_set_failure(
    cache_client: &CacheClient,
    cache_name: String,
    key: String,
    value: String,
) -> MomentoResult<()> {
    match cache_client
        .set(cache_name.clone(), key.clone(), value.clone())
        .await
    {
        Ok(_) => Err(MomentoError {
            message: format!(
                "Expected setting value '{}' for key '{}' from cache '{}' to fail",
                value, key, cache_name
            ),
            error_code: MomentoErrorCode::UnknownError,
            inner_error: None,
            details: None,
        }),
        Err(e) => {
            assert_eq!(e.error_code, MomentoErrorCode::PermissionError);
            Ok(())
        }
    }
}

async fn assert_publish_success(
    topic_client: &TopicClient,
    cache_name: String,
    topic_name: String,
    value: String,
) -> MomentoResult<()> {
    match topic_client
        .publish(cache_name.clone(), topic_name.clone(), value.clone())
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!(
                "Expected to successfully publish value '{}' for topic '{}' in cache '{}'",
                value, topic_name, cache_name
            );
            Err(e)
        }
    }
}

async fn assert_publish_failure(
    topic_client: &TopicClient,
    cache_name: String,
    topic_name: String,
    value: String,
) -> MomentoResult<()> {
    match topic_client
        .publish(cache_name.clone(), topic_name.clone(), value.clone())
        .await
    {
        Ok(_) => Err(MomentoError {
            message: format!(
                "Expected publishing value '{}' for topic '{}' in cache '{}' to fail",
                value, topic_name, cache_name
            ),
            error_code: MomentoErrorCode::UnknownError,
            inner_error: None,
            details: None,
        }),
        Err(e) => {
            assert_eq!(e.error_code, MomentoErrorCode::PermissionError);
            Ok(())
        }
    }
}

async fn assert_subscribe_success(
    topic_client: &TopicClient,
    cache_name: String,
    topic_name: String,
) -> MomentoResult<()> {
    match topic_client
        .subscribe(cache_name.clone(), topic_name.clone())
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!(
                "Expected to successfully subscribe to topic '{}' in cache '{}'",
                topic_name, cache_name
            );
            Err(e)
        }
    }
}

async fn assert_subscribe_failure(
    topic_client: &TopicClient,
    cache_name: String,
    topic_name: String,
) -> MomentoResult<()> {
    match topic_client
        .subscribe(cache_name.clone(), topic_name.clone())
        .await
    {
        Ok(_) => Err(MomentoError {
            message: format!(
                "Expected subscribe to topic '{}' in cache '{}' to fail",
                topic_name, cache_name
            ),
            error_code: MomentoErrorCode::UnknownError,
            inner_error: None,
            details: None,
        }),
        Err(e) => {
            assert_eq!(e.error_code, MomentoErrorCode::PermissionError);
            Ok(())
        }
    }
}

mod disposable_tokens_cache_key {
    use super::*;

    #[tokio::test]
    async fn test_cache_key_read_only_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_item = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScopes::cache_key_read_only(CacheSelector::AllCaches, test_item.key()),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read this key in both caches
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cache_client, first_cache.to_string(), other_key.clone()).await?;
        assert_get_failure(&cache_client, second_cache.to_string(), other_key).await?;

        // should not be able to write the key in either cache
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_key_read_only_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_item = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScopes::cache_key_read_only(first_cache.clone(), test_item.key()),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read this key in only first cache
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cache_client, first_cache.to_string(), other_key.clone()).await?;
        assert_get_failure(&cache_client, second_cache.to_string(), other_key).await?;

        // should not be able to write the key in either cache
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_key_write_only_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_item = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScopes::cache_key_write_only(CacheSelector::AllCaches, test_item.key()),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should not be able to read this key in either cache
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cache_client, first_cache.to_string(), other_key.clone()).await?;
        assert_get_failure(&cache_client, second_cache.to_string(), other_key).await?;

        // should be able to write the key in both caches
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_key_write_only_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_item = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScopes::cache_key_write_only(first_cache.clone(), test_item.key()),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should not be able to read this key in either cache
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cache_client, first_cache.to_string(), other_key.clone()).await?;
        assert_get_failure(&cache_client, second_cache.to_string(), other_key).await?;

        // should be able to write the key in only first cache
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_key_read_write_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_item = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScopes::cache_key_read_write(CacheSelector::AllCaches, test_item.key()),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read this key in both caches
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cache_client, first_cache.to_string(), other_key.clone()).await?;
        assert_get_failure(&cache_client, second_cache.to_string(), other_key).await?;

        // should be able to write the key in both caches
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_key_read_write_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_item = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScopes::cache_key_read_write(first_cache.clone(), test_item.key()),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read this key in only first cache
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cache_client, first_cache.to_string(), other_key.clone()).await?;
        assert_get_failure(&cache_client, second_cache.to_string(), other_key).await?;

        // should be able to write the key in only first cache
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }
}

mod disposable_tokens_cache_key_prefix {
    use super::*;

    #[tokio::test]
    async fn test_cache_key_prefix_read_only_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_item = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScopes::cache_key_prefix_read_only(first_cache.clone(), test_item.key()),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read this key in only first cache
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should be able to read a prefixed key in only first cache
        let prefixed_key = format!("{}-smth-else", test_item.key());
        assert_get_success(&cache_client, first_cache.to_string(), prefixed_key.clone()).await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            prefixed_key.clone(),
        )
        .await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cache_client, first_cache.to_string(), other_key.clone()).await?;
        assert_get_failure(&cache_client, second_cache.to_string(), other_key).await?;

        // should not be able to write the key in either cache
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_key_prefix_read_only_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_item = TestScalar::new();
        let response =
            generate_disposable_token_success(DisposableTokenScopes::cache_key_prefix_read_only(
                CacheSelector::AllCaches,
                test_item.key(),
            ))
            .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read this key in both caches
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should be able to read a prefixed key in both caches
        let prefixed_key = format!("{}-smth-else", test_item.key());
        assert_get_success(&cache_client, first_cache.to_string(), prefixed_key.clone()).await?;
        assert_get_success(
            &cache_client,
            second_cache.to_string(),
            prefixed_key.clone(),
        )
        .await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cache_client, first_cache.to_string(), other_key.clone()).await?;
        assert_get_failure(&cache_client, second_cache.to_string(), other_key).await?;

        // should not be able to write the key in either cache
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_key_prefix_write_only_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_item = TestScalar::new();
        let response =
            generate_disposable_token_success(DisposableTokenScopes::cache_key_prefix_write_only(
                first_cache.clone(),
                test_item.key(),
            ))
            .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should not be able to read this key in either cache
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should not be able to read a prefixed key in either cache
        let prefixed_key = format!("{}-smth-else", test_item.key());
        assert_get_failure(&cache_client, first_cache.to_string(), prefixed_key.clone()).await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            prefixed_key.clone(),
        )
        .await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cache_client, first_cache.to_string(), other_key.clone()).await?;
        assert_get_failure(&cache_client, second_cache.to_string(), other_key).await?;

        // should be able to write the key in only first cache
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_key_prefix_write_only_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_item = TestScalar::new();
        let response =
            generate_disposable_token_success(DisposableTokenScopes::cache_key_prefix_write_only(
                CacheSelector::AllCaches,
                test_item.key(),
            ))
            .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should not be able to read this key in either cache
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should not be able to read a prefixed key in either cache
        let prefixed_key = format!("{}-smth-else", test_item.key());
        assert_get_failure(&cache_client, first_cache.to_string(), prefixed_key.clone()).await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            prefixed_key.clone(),
        )
        .await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cache_client, first_cache.to_string(), other_key.clone()).await?;
        assert_get_failure(&cache_client, second_cache.to_string(), other_key).await?;

        // should be able to write the key in both caches
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_key_prefix_read_write_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_item = TestScalar::new();
        let response =
            generate_disposable_token_success(DisposableTokenScopes::cache_key_prefix_read_write(
                first_cache.clone(),
                test_item.key(),
            ))
            .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read this key in only first cache
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should be able to read a prefixed key in only first cache
        let prefixed_key = format!("{}-smth-else", test_item.key());
        assert_get_success(&cache_client, first_cache.to_string(), prefixed_key.clone()).await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            prefixed_key.clone(),
        )
        .await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cache_client, first_cache.to_string(), other_key.clone()).await?;
        assert_get_failure(&cache_client, second_cache.to_string(), other_key).await?;

        // should be able to write the key in only first cache
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_key_prefix_read_write_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_item = TestScalar::new();
        let response =
            generate_disposable_token_success(DisposableTokenScopes::cache_key_prefix_read_write(
                CacheSelector::AllCaches,
                test_item.key(),
            ))
            .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read this key in both caches
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should be able to read a prefixed key in both caches
        let prefixed_key = format!("{}-smth-else", test_item.key());
        assert_get_success(&cache_client, first_cache.to_string(), prefixed_key.clone()).await?;
        assert_get_success(
            &cache_client,
            second_cache.to_string(),
            prefixed_key.clone(),
        )
        .await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cache_client, first_cache.to_string(), other_key.clone()).await?;
        assert_get_failure(&cache_client, second_cache.to_string(), other_key).await?;

        // should be able to write the key in both caches
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }
}

mod disposable_tokens_cache {
    use super::*;

    #[tokio::test]
    async fn test_cache_read_write_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: first_cache.to_string().into(),
                    role: CacheRole::ReadWrite,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read and write in only first cache
        let test_item = TestScalar::new();
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_read_write_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: CacheSelector::AllCaches,
                    role: CacheRole::ReadWrite,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read and write in both caches
        let test_item = TestScalar::new();
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_read_only_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: first_cache.to_string().into(),
                    role: CacheRole::ReadOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read in only first cache
        let test_item = TestScalar::new();
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should not be able to write in either cache
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_read_only_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: CacheSelector::AllCaches,
                    role: CacheRole::ReadOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read in both caches
        let test_item = TestScalar::new();
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should not be able to write in either cache
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_write_only_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: first_cache.to_string().into(),
                    role: CacheRole::WriteOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should not be able to read in either cache
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should be able to write in only first cache
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_write_only_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: CacheSelector::AllCaches,
                    role: CacheRole::WriteOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should not be able to read in either cache
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;

        // should be able to write in both caches
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
            topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            topic.key().to_string(),
        )
        .await?;

        Ok(())
    }
}

mod disposable_tokens_topics {
    use momento::auth::{TopicPermission, TopicRole, TopicSelector};

    use super::*;

    #[tokio::test]
    async fn test_topics_publish_subscribe_specific_topic_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: first_cache.to_string().into(),
                    topic: test_topic.key().to_string().into(),
                    role: TopicRole::PublishSubscribe,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let topic_client = new_topic_client(creds.clone());
        let cache_client = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should be able to publish and subscribe in only first cache on only the specific topic
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            test_topic.key().to_string(),
            test_topic.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            test_topic.key().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            test_topic.key().to_string(),
            test_topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            test_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_publish_subscribe_specific_topic_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: test_topic.key().to_string().into(),
                    role: TopicRole::PublishSubscribe,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let topic_client = new_topic_client(creds.clone());
        let cache_client = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should be able to publish and subscribe in both caches on only the specific topic
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            test_topic.key().to_string(),
            test_topic.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            test_topic.key().to_string(),
        )
        .await?;
        assert_publish_success(
            &topic_client,
            second_cache.to_string(),
            test_topic.key().to_string(),
            test_topic.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            second_cache.to_string(),
            test_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_publish_subscribe_all_topics_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: first_cache.to_string().into(),
                    topic: TopicSelector::AllTopics,
                    role: TopicRole::PublishSubscribe,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let topic_client = new_topic_client(creds.clone());
        let cache_client = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should be able to publish and subscribe in only first cache on all topics
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
            first_topic.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
            first_topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
            second_topic.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
            second_topic.value().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_publish_subscribe_all_topics_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: TopicSelector::AllTopics,
                    role: TopicRole::PublishSubscribe,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let topic_client = new_topic_client(creds.clone());
        let cache_client = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should be able to publish and subscribe in both caches on all topics
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
            first_topic.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;
        assert_publish_success(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
            first_topic.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
            second_topic.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
        )
        .await?;
        assert_publish_success(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
            second_topic.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_subscribe_only_specific_topic_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: first_cache.to_string().into(),
                    topic: test_topic.key().to_string().into(),
                    role: TopicRole::SubscribeOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let topic_client = new_topic_client(creds.clone());
        let cache_client = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to publish in either cache on only the specific topic
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            test_topic.key().to_string(),
            test_topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            test_topic.key().to_string(),
            test_topic.value().to_string(),
        )
        .await?;

        // should be able to subscribe in only first cache on only the specific topic
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            test_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            test_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_subscribe_only_specific_topic_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: test_topic.key().to_string().into(),
                    role: TopicRole::SubscribeOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let topic_client = new_topic_client(creds.clone());
        let cache_client = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to publish in either cache
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            test_topic.key().to_string(),
            test_topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            test_topic.key().to_string(),
            test_topic.value().to_string(),
        )
        .await?;

        // should be able to subscribe in both caches
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            test_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            second_cache.to_string(),
            test_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_subscribe_only_all_topics_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: first_cache.to_string().into(),
                    topic: TopicSelector::AllTopics,
                    role: TopicRole::SubscribeOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let topic_client = new_topic_client(creds.clone());
        let cache_client = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to publish in either cache on all topics
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
            first_topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
            first_topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
            second_topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
            second_topic.value().to_string(),
        )
        .await?;

        // should be able to subscribe in only first cache on all topics
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_subscribe_only_all_topics_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: TopicSelector::AllTopics,
                    role: TopicRole::SubscribeOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let topic_client = new_topic_client(creds.clone());
        let cache_client = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should not be able to publish in either cache on all topics
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
            first_topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
            first_topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
            second_topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
            second_topic.value().to_string(),
        )
        .await?;

        // should be able to subscribe in both caches on all topics
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_publish_only_specific_topic_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: first_cache.to_string().into(),
                    topic: test_topic.key().to_string().into(),
                    role: TopicRole::PublishOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let topic_client = new_topic_client(creds.clone());
        let cache_client = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should be able to publish in only first cache on only the specific topic
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            test_topic.key().to_string(),
            test_topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            test_topic.key().to_string(),
            test_topic.value().to_string(),
        )
        .await?;

        // should not be able to subscribe in either cache on only the specific topic
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            test_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            test_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_publish_only_specific_topic_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let test_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: test_topic.key().to_string().into(),
                    role: TopicRole::PublishOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let topic_client = new_topic_client(creds.clone());
        let cache_client = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should be able to publish in both caches on only the specific topic
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            test_topic.key().to_string(),
            test_topic.value().to_string(),
        )
        .await?;
        assert_publish_success(
            &topic_client,
            second_cache.to_string(),
            test_topic.key().to_string(),
            test_topic.value().to_string(),
        )
        .await?;

        // should not be able to subscribe in either cache on only the specific topic
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            test_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            test_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_publish_only_all_topics_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: first_cache.to_string().into(),
                    topic: TopicSelector::AllTopics,
                    role: TopicRole::PublishOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let topic_client = new_topic_client(creds.clone());
        let cache_client = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should be able to publish in only first cache on all topics
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
            first_topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
            first_topic.value().to_string(),
        )
        .await?;
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
            second_topic.value().to_string(),
        )
        .await?;
        assert_publish_failure(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
            second_topic.value().to_string(),
        )
        .await?;

        // should not be able to subscribe in either cache on all topics
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_publish_only_all_topics_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: TopicSelector::AllTopics,
                    role: TopicRole::PublishOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let topic_client = new_topic_client(creds.clone());
        let cache_client = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_failure(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should be able to publish in both caches on all topics
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
            first_topic.value().to_string(),
        )
        .await?;
        assert_publish_success(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
            first_topic.value().to_string(),
        )
        .await?;
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
            second_topic.value().to_string(),
        )
        .await?;
        assert_publish_success(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
            second_topic.value().to_string(),
        )
        .await?;

        // should not be able to subscribe in either cache on all topics
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
        )
        .await?;
        assert_subscribe_failure(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }
}

mod disposable_tokens_all_data {
    use super::*;

    #[tokio::test]
    async fn test_all_data_read_write() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions::all_data_read_write()),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cache_client = new_cache_client(creds.clone());
        let topic_client = new_topic_client(creds);

        // should be able to read and write in both caches
        let test_item = TestScalar::new();
        assert_get_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_get_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
        )
        .await?;
        assert_set_success(
            &cache_client,
            first_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_set_success(
            &cache_client,
            second_cache.to_string(),
            test_item.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;

        // should be able to publish and subscribe in both caches on both topics
        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;
        assert_publish_success(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            second_cache.to_string(),
            first_topic.key().to_string(),
        )
        .await?;

        assert_publish_success(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            first_cache.to_string(),
            second_topic.key().to_string(),
        )
        .await?;
        assert_publish_success(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
            test_item.value().to_string(),
        )
        .await?;
        assert_subscribe_success(
            &topic_client,
            second_cache.to_string(),
            second_topic.key().to_string(),
        )
        .await?;

        Ok(())
    }
}
