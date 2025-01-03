use std::time::Duration;

use momento::auth::CacheSelector;
use momento::auth::Expiration;
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
    cache_name: &str,
    key: &str,
) -> MomentoResult<()> {
    match cache_client.get(cache_name, key).await {
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
    cache_name: &str,
    key: &str,
) -> MomentoResult<()> {
    match cache_client.get(cache_name, key).await {
        Ok(_) => Err(MomentoError {
            message: format!(
                "Expected getting key '{}' from cache '{}' to fail but it did not",
                key, cache_name
            ),
            error_code: MomentoErrorCode::UnknownError,
            inner_error: None,
            details: None,
        }),
        Err(e) => {
            match e.error_code {
                MomentoErrorCode::PermissionError => {}
                MomentoErrorCode::AuthenticationError => {}
                _ => {
                    eprintln!(
                        "Expected getting key '{}' from cache '{}' to fail with permission or authentication error. Failed with error code '{:?}' instead",
                        key, cache_name, e.error_code
                    );
                    return Err(e);
                }
            }
            Ok(())
        }
    }
}

async fn assert_set_success(
    cache_client: &CacheClient,
    cache_name: &str,
    key: &str,
    value: &str,
) -> MomentoResult<()> {
    match cache_client.set(cache_name, key, value).await {
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
    cache_name: &str,
    key: &str,
    value: &str,
) -> MomentoResult<()> {
    match cache_client.set(cache_name, key, value).await {
        Ok(_) => Err(MomentoError {
            message: format!(
                "Expected setting value '{}' for key '{}' from cache '{}' to fail but it did not",
                value, key, cache_name
            ),
            error_code: MomentoErrorCode::UnknownError,
            inner_error: None,
            details: None,
        }),
        Err(e) => {
            match e.error_code {
                MomentoErrorCode::PermissionError => {}
                MomentoErrorCode::AuthenticationError => {}
                _ => {
                    eprintln!(
                        "Expected setting key '{}' in cache '{}' to fail with permission or authentication error. Failed with error code '{:?}' instead",
                        key, cache_name, e.error_code
                    );
                    return Err(e);
                }
            }
            Ok(())
        }
    }
}

async fn assert_publish_success(
    topic_client: &TopicClient,
    cache_name: &str,
    topic_name: &str,
    value: &str,
) -> MomentoResult<()> {
    match topic_client.publish(cache_name, topic_name, value).await {
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
    cache_name: &str,
    topic_name: &str,
    value: &str,
) -> MomentoResult<()> {
    match topic_client.publish(cache_name, topic_name, value).await {
        Ok(_) => Err(MomentoError {
            message: format!(
                "Expected publishing value '{}' for topic '{}' in cache '{}' to fail but it did not",
                value, topic_name, cache_name
            ),
            error_code: MomentoErrorCode::UnknownError,
            inner_error: None,
            details: None,
        }),
        Err(e) => {
            match e.error_code {
                MomentoErrorCode::PermissionError => {},
                MomentoErrorCode::AuthenticationError => {},
                _ => {
                    eprintln!(
                        "Expected publishing to topic '{}' in cache '{}' to fail with permission or authentication error. Failed with error code '{:?}' instead",
                        topic_name, cache_name, e.error_code
                    );
                    return Err(e);
                }
            }
            Ok(())
        }
    }
}

async fn assert_subscribe_success(
    topic_client: &TopicClient,
    cache_name: &str,
    topic_name: &str,
) -> MomentoResult<()> {
    match topic_client.subscribe(cache_name, topic_name).await {
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
    cache_name: &str,
    topic_name: &str,
) -> MomentoResult<()> {
    match topic_client.subscribe(cache_name, topic_name).await {
        Ok(_) => Err(MomentoError {
            message: format!(
                "Expected subscribe to topic '{}' in cache '{}' to fail but it did not",
                topic_name, cache_name
            ),
            error_code: MomentoErrorCode::UnknownError,
            inner_error: None,
            details: None,
        }),
        Err(e) => {
            match e.error_code {
                MomentoErrorCode::PermissionError => {}
                MomentoErrorCode::AuthenticationError => {}
                _ => {
                    eprintln!(
                        "Expected subscribing to topic '{}' cache '{}' to fail with permission or authentication error. Failed with error code '{:?}' instead",
                        topic_name, cache_name, e.error_code
                    );
                    return Err(e);
                }
            }
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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read this key in both caches
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_success(&cc, second_cache, test_item.key()).await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cc, first_cache, &other_key).await?;
        assert_get_failure(&cc, second_cache, &other_key).await?;

        // should not be able to write the key in either cache
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to write another key in either cache
        assert_set_failure(&cc, first_cache, &other_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &other_key, test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read this key in only first cache
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cc, first_cache, &other_key).await?;
        assert_get_failure(&cc, second_cache, &other_key).await?;

        // should not be able to write the key in either cache
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to write another key in either cache
        assert_set_failure(&cc, first_cache, &other_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &other_key, test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should not be able to read this key in either cache
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cc, first_cache, &other_key).await?;
        assert_get_failure(&cc, second_cache, &other_key).await?;

        // should be able to write the key in both caches
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_success(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to write another key in either cache
        assert_set_failure(&cc, first_cache, &other_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &other_key, test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should not be able to read this key in either cache
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cc, first_cache, &other_key).await?;
        assert_get_failure(&cc, second_cache, &other_key).await?;

        // should be able to write the key in only first cache
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to write another key in either cache
        assert_set_failure(&cc, first_cache, &other_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &other_key, test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read this key in both caches
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_success(&cc, second_cache, test_item.key()).await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cc, first_cache, &other_key).await?;
        assert_get_failure(&cc, second_cache, &other_key).await?;

        // should be able to write the key in both caches
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_success(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to write another key in both caches
        assert_set_failure(&cc, first_cache, &other_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &other_key, test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read this key in only first cache
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cc, first_cache, &other_key).await?;
        assert_get_failure(&cc, second_cache, &other_key).await?;

        // should be able to write the key in only first cache
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to write another key in either cache
        assert_set_failure(&cc, first_cache, &other_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &other_key, test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read this key in only first cache
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;

        // should be able to read a prefixed key in only first cache
        let prefixed_key = format!("{}-smth-else", test_item.key());
        assert_get_success(&cc, first_cache, &prefixed_key).await?;
        assert_get_failure(&cc, second_cache, &prefixed_key).await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cc, first_cache, &other_key).await?;
        assert_get_failure(&cc, second_cache, &other_key).await?;

        // should not be able to write the key in either cache
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to write another key in either cache
        assert_set_failure(&cc, first_cache, &other_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &other_key, test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read this key in both caches
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_success(&cc, second_cache, test_item.key()).await?;

        // should be able to read a prefixed key in both caches
        let prefixed_key = format!("{}-smth-else", test_item.key());
        assert_get_success(&cc, first_cache, &prefixed_key).await?;
        assert_get_success(&cc, second_cache, &prefixed_key).await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cc, first_cache, &other_key).await?;
        assert_get_failure(&cc, second_cache, &other_key).await?;

        // should not be able to write the key in either cache
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to write another key in either cache
        assert_set_failure(&cc, first_cache, &other_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &other_key, test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should not be able to read this key in either cache
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;

        // should not be able to read a prefixed key in either cache
        let prefixed_key = format!("{}-smth-else", test_item.key());
        assert_get_failure(&cc, first_cache, &prefixed_key).await?;
        assert_get_failure(&cc, second_cache, &prefixed_key).await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cc, first_cache, &other_key).await?;
        assert_get_failure(&cc, second_cache, &other_key).await?;

        // should be able to write the key in only first cache
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to write a prefixed key in only first cache
        assert_set_success(&cc, first_cache, &prefixed_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &prefixed_key, test_item.value()).await?;

        // should not be able to write another key in either cache
        assert_set_failure(&cc, first_cache, &other_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &other_key, test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should not be able to read this key in either cache
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;

        // should not be able to read a prefixed key in either cache
        let prefixed_key = format!("{}-smth-else", test_item.key());
        assert_get_failure(&cc, first_cache, &prefixed_key).await?;
        assert_get_failure(&cc, second_cache, &prefixed_key).await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cc, first_cache, &other_key).await?;
        assert_get_failure(&cc, second_cache, &other_key).await?;

        // should be able to write the key in both caches
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_success(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to write a prefixed key in both caches
        assert_set_success(&cc, first_cache, &prefixed_key, test_item.value()).await?;
        assert_set_success(&cc, second_cache, &prefixed_key, test_item.value()).await?;

        // should be able to write another key in either cache
        assert_set_failure(&cc, first_cache, &other_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &other_key, test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read this key in only first cache
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;

        // should be able to read a prefixed key in only first cache
        let prefixed_key = format!("{}-smth-else", test_item.key());
        assert_get_success(&cc, first_cache, &prefixed_key).await?;
        assert_get_failure(&cc, second_cache, &prefixed_key).await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cc, first_cache, &other_key).await?;
        assert_get_failure(&cc, second_cache, &other_key).await?;

        // should be able to write the key in only first cache
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to write a prefixed key in first cache
        assert_set_success(&cc, first_cache, &prefixed_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &prefixed_key, test_item.value()).await?;

        // should not be able to write another key in either cache
        assert_set_failure(&cc, first_cache, &other_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &other_key, test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read this key in both caches
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_success(&cc, second_cache, test_item.key()).await?;

        // should be able to read a prefixed key in both caches
        let prefixed_key = format!("{}-smth-else", test_item.key());
        assert_get_success(&cc, first_cache, &prefixed_key).await?;
        assert_get_success(&cc, second_cache, &prefixed_key).await?;

        // should not be able to read another key in either cache
        let other_key = unique_key();
        assert_get_failure(&cc, first_cache, &other_key).await?;
        assert_get_failure(&cc, second_cache, &other_key).await?;

        // should be able to write the key in both caches
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_success(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to write a prefixed key in both caches
        assert_set_success(&cc, first_cache, &prefixed_key, test_item.value()).await?;
        assert_set_success(&cc, second_cache, &prefixed_key, test_item.value()).await?;

        // should not be able to write another key in either cache
        assert_set_failure(&cc, first_cache, &other_key, test_item.value()).await?;
        assert_set_failure(&cc, second_cache, &other_key, test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
                    cache: (*first_cache).clone().into(),
                    role: CacheRole::ReadWrite,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read and write in only first cache
        let test_item = TestScalar::new();
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read and write in both caches
        let test_item = TestScalar::new();
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_success(&cc, second_cache, test_item.key()).await?;
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_success(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_read_only_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: (*first_cache).clone().into(),
                    role: CacheRole::ReadOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read in only first cache
        let test_item = TestScalar::new();
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;

        // should not be able to write in either cache
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read in both caches
        let test_item = TestScalar::new();
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_success(&cc, second_cache, test_item.key()).await?;

        // should not be able to write in either cache
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_write_only_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: (*first_cache).clone().into(),
                    role: CacheRole::WriteOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should not be able to read in either cache
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;

        // should be able to write in only first cache
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should not be able to read in either cache
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;

        // should be able to write in both caches
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_success(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to use topics
        let topic = TestScalar::new();
        assert_publish_failure(&tc, first_cache, topic.key(), topic.value()).await?;
        assert_publish_failure(&tc, second_cache, topic.key(), topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, topic.key()).await?;

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
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: (*first_cache).clone().into(),
                    topic: first_topic.key().into(),
                    role: TopicRole::PublishSubscribe,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let tc = new_topic_client(creds.clone());
        let cc = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to publish and subscribe in only first cache on only the specific topic
        assert_publish_success(&tc, first_cache, first_topic.key(), first_topic.value()).await?;
        assert_subscribe_success(&tc, first_cache, first_topic.key()).await?;
        assert_publish_failure(&tc, second_cache, first_topic.key(), first_topic.value()).await?;
        assert_subscribe_failure(&tc, second_cache, first_topic.key()).await?;
        assert_publish_failure(&tc, first_cache, second_topic.key(), second_topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, second_topic.key()).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_publish_subscribe_specific_topic_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: first_topic.key().into(),
                    role: TopicRole::PublishSubscribe,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let tc = new_topic_client(creds.clone());
        let cc = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to publish and subscribe in both caches on only the specific topic
        assert_publish_success(&tc, first_cache, first_topic.key(), first_topic.value()).await?;
        assert_subscribe_success(&tc, first_cache, first_topic.key()).await?;
        assert_publish_success(&tc, second_cache, first_topic.key(), first_topic.value()).await?;
        assert_subscribe_success(&tc, second_cache, first_topic.key()).await?;
        assert_publish_failure(&tc, first_cache, second_topic.key(), second_topic.value()).await?;
        assert_subscribe_failure(&tc, first_cache, second_topic.key()).await?;

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
                    cache: (*first_cache).clone().into(),
                    topic: TopicSelector::AllTopics,
                    role: TopicRole::PublishSubscribe,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let tc = new_topic_client(creds.clone());
        let cc = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to publish and subscribe in only first cache on all topics
        assert_publish_success(&tc, first_cache, first_topic.key(), first_topic.value()).await?;
        assert_subscribe_success(&tc, first_cache, first_topic.key()).await?;
        assert_publish_failure(&tc, second_cache, first_topic.key(), first_topic.value()).await?;
        assert_subscribe_failure(&tc, second_cache, first_topic.key()).await?;
        assert_publish_success(&tc, first_cache, second_topic.key(), second_topic.value()).await?;
        assert_subscribe_success(&tc, first_cache, second_topic.key()).await?;
        assert_publish_failure(&tc, second_cache, second_topic.key(), second_topic.value()).await?;
        assert_subscribe_failure(&tc, second_cache, second_topic.key()).await?;

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
        let tc = new_topic_client(creds.clone());
        let cc = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to publish and subscribe in both caches on all topics
        assert_publish_success(&tc, first_cache, first_topic.key(), first_topic.value()).await?;
        assert_subscribe_success(&tc, first_cache, first_topic.key()).await?;
        assert_publish_success(&tc, second_cache, first_topic.key(), first_topic.value()).await?;
        assert_subscribe_success(&tc, second_cache, first_topic.key()).await?;
        assert_publish_success(&tc, first_cache, second_topic.key(), second_topic.value()).await?;
        assert_subscribe_success(&tc, first_cache, second_topic.key()).await?;
        assert_publish_success(&tc, second_cache, second_topic.key(), second_topic.value()).await?;
        assert_subscribe_success(&tc, second_cache, second_topic.key()).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_subscribe_only_specific_topic_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: (*first_cache).clone().into(),
                    topic: first_topic.key().into(),
                    role: TopicRole::SubscribeOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let tc = new_topic_client(creds.clone());
        let cc = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to publish in either cache on either topic
        assert_publish_failure(&tc, first_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_failure(&tc, first_cache, second_topic.key(), second_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, second_topic.key(), second_topic.value()).await?;

        // should be able to subscribe in only first cache to the specific topic
        assert_subscribe_success(&tc, first_cache, first_topic.key()).await?;
        assert_subscribe_failure(&tc, first_cache, second_topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, first_topic.key()).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_subscribe_only_specific_topic_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: first_topic.key().into(),
                    role: TopicRole::SubscribeOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let tc = new_topic_client(creds.clone());
        let cc = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to publish in either cache on either topic
        assert_publish_failure(&tc, first_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_failure(&tc, first_cache, second_topic.key(), second_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, second_topic.key(), second_topic.value()).await?;

        // should be able to subscribe in both caches to only the specific topic
        assert_subscribe_success(&tc, first_cache, first_topic.key()).await?;
        assert_subscribe_failure(&tc, first_cache, second_topic.key()).await?;
        assert_subscribe_success(&tc, second_cache, first_topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, second_topic.key()).await?;

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
                    cache: (*first_cache).clone().into(),
                    topic: TopicSelector::AllTopics,
                    role: TopicRole::SubscribeOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let tc = new_topic_client(creds.clone());
        let cc = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to publish in either cache on either topic
        assert_publish_failure(&tc, first_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_failure(&tc, first_cache, second_topic.key(), second_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, second_topic.key(), second_topic.value()).await?;

        // should be able to subscribe in only first cache on either topic
        assert_subscribe_success(&tc, first_cache, first_topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, first_topic.key()).await?;
        assert_subscribe_success(&tc, first_cache, second_topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, second_topic.key()).await?;

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
        let tc = new_topic_client(creds.clone());
        let cc = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should not be able to publish in either cache on either topic
        assert_publish_failure(&tc, first_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_failure(&tc, first_cache, second_topic.key(), second_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, second_topic.key(), second_topic.value()).await?;

        // should be able to subscribe in both caches on both topics
        assert_subscribe_success(&tc, first_cache, first_topic.key()).await?;
        assert_subscribe_success(&tc, second_cache, first_topic.key()).await?;
        assert_subscribe_success(&tc, first_cache, second_topic.key()).await?;
        assert_subscribe_success(&tc, second_cache, second_topic.key()).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_publish_only_specific_topic_specific_cache() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: (*first_cache).clone().into(),
                    topic: first_topic.key().into(),
                    role: TopicRole::PublishOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let tc = new_topic_client(creds.clone());
        let cc = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to publish in only first cache on only the specific topic
        assert_publish_success(&tc, first_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_failure(&tc, first_cache, second_topic.key(), second_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, second_topic.key(), second_topic.value()).await?;

        // should not be able to subscribe in either cache on either topic
        assert_subscribe_failure(&tc, first_cache, first_topic.key()).await?;
        assert_subscribe_failure(&tc, first_cache, second_topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, first_topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, second_topic.key()).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_topics_publish_only_specific_topic_all_caches() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let first_topic = TestScalar::new();
        let second_topic = TestScalar::new();
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: first_topic.key().into(),
                    role: TopicRole::PublishOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let tc = new_topic_client(creds.clone());
        let cc = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to publish in both caches on only the specific topic
        assert_publish_success(&tc, first_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_success(&tc, second_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_failure(&tc, first_cache, second_topic.key(), second_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, second_topic.key(), second_topic.value()).await?;

        // should not be able to subscribe in either cache on either topic
        assert_subscribe_failure(&tc, first_cache, first_topic.key()).await?;
        assert_subscribe_failure(&tc, first_cache, second_topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, first_topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, second_topic.key()).await?;

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
                    cache: (*first_cache).clone().into(),
                    topic: TopicSelector::AllTopics,
                    role: TopicRole::PublishOnly,
                })],
            }),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let tc = new_topic_client(creds.clone());
        let cc = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to publish in only first cache on all topics
        assert_publish_success(&tc, first_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_success(&tc, first_cache, second_topic.key(), second_topic.value()).await?;
        assert_publish_failure(&tc, second_cache, second_topic.key(), second_topic.value()).await?;

        // should not be able to subscribe in either cache on all topics
        assert_subscribe_failure(&tc, first_cache, first_topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, first_topic.key()).await?;
        assert_subscribe_failure(&tc, first_cache, second_topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, second_topic.key()).await?;

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
        let tc = new_topic_client(creds.clone());
        let cc = new_cache_client(creds.clone());

        // should not be able to use cache directly
        let test_item = TestScalar::new();
        assert_get_failure(&cc, first_cache, test_item.key()).await?;
        assert_get_failure(&cc, second_cache, test_item.key()).await?;
        assert_set_failure(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_failure(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to publish in both caches on both topics
        assert_publish_success(&tc, first_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_success(&tc, second_cache, first_topic.key(), first_topic.value()).await?;
        assert_publish_success(&tc, first_cache, second_topic.key(), second_topic.value()).await?;
        assert_publish_success(&tc, second_cache, second_topic.key(), second_topic.value()).await?;

        // should not be able to subscribe in either cache on either topic
        assert_subscribe_failure(&tc, first_cache, first_topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, first_topic.key()).await?;
        assert_subscribe_failure(&tc, first_cache, second_topic.key()).await?;
        assert_subscribe_failure(&tc, second_cache, second_topic.key()).await?;

        Ok(())
    }
}

mod disposable_tokens_all_data {
    use super::*;

    #[tokio::test]
    async fn test_all_data_read_write() -> MomentoResult<()> {
        let first_cache = &CACHE_TEST_STATE.cache_name;
        let second_cache = &CACHE_TEST_STATE.auth_cache_name;
        let response = generate_disposable_token_success(
            DisposableTokenScope::Permissions::<String>(Permissions::all_data_read_write()),
        )
        .await?;
        let creds = new_credential_provider_from_token(response.auth_token());
        let cc = new_cache_client(creds.clone());
        let tc = new_topic_client(creds);

        // should be able to read and write in both caches
        let test_item = TestScalar::new();
        assert_get_success(&cc, first_cache, test_item.key()).await?;
        assert_get_success(&cc, second_cache, test_item.key()).await?;
        assert_set_success(&cc, first_cache, test_item.key(), test_item.value()).await?;
        assert_set_success(&cc, second_cache, test_item.key(), test_item.value()).await?;

        // should be able to publish and subscribe in both caches on both topics
        let first_topic = TestScalar::new();
        assert_publish_success(&tc, first_cache, first_topic.key(), test_item.value()).await?;
        assert_subscribe_success(&tc, first_cache, first_topic.key()).await?;
        assert_publish_success(&tc, second_cache, first_topic.key(), test_item.value()).await?;
        assert_subscribe_success(&tc, second_cache, first_topic.key()).await?;

        let second_topic = TestScalar::new();
        assert_publish_success(&tc, first_cache, second_topic.key(), test_item.value()).await?;
        assert_subscribe_success(&tc, first_cache, second_topic.key()).await?;
        assert_publish_success(&tc, second_cache, second_topic.key(), test_item.value()).await?;
        assert_subscribe_success(&tc, second_cache, second_topic.key()).await?;

        // cannot create caches
        match cc.create_cache(second_cache).await {
            Ok(_) => Err(MomentoError {
                message: "Expected creating cache using AllDataReadWrite disposable token to fail but it did not".into(),
                error_code: MomentoErrorCode::UnknownError,
                inner_error: None,
                details: None,
            }),
            Err(e) => {
                match e.error_code {
                    MomentoErrorCode::PermissionError => Ok(()),
                    _ => {
                        eprintln!(
                            "Expected creating cache using AllDataReadWrite disposable token to fail with permission error. Failed with error code '{:?}' instead",
                            e.error_code
                        );
                        Err(e)
                    }
                }
            }
        }?;

        // cannot delete caches
        match cc.delete_cache(second_cache).await {
            Ok(_) => Err(MomentoError {
                message: "Expected deleting cache using AllDataReadWrite disposable token to fail but it did not".into(),
                error_code: MomentoErrorCode::UnknownError,
                inner_error: None,
                details: None,
            }),
            Err(e) => {
                match e.error_code {
                    MomentoErrorCode::PermissionError => Ok(()),
                    _ => {
                        eprintln!(
                            "Expected deleting cache using AllDataReadWrite disposable token to fail with permission error. Failed with error code '{:?}' instead",
                            e.error_code
                        );
                        Err(e)
                    }
                }
            }
        }?;

        Ok(())
    }
}

mod disposable_tokens_expiry {
    use super::*;

    #[tokio::test]
    async fn test_expiry() -> MomentoResult<()> {
        // Generate a token that expires soon
        let expiry = ExpiresIn::seconds(5);
        let scope = momento::auth::DisposableTokenScope::Permissions::<String>(
            Permissions::all_data_read_write(),
        );
        let response = CACHE_TEST_STATE
            .auth_client
            .generate_disposable_token(scope, expiry)
            .await?;

        // Verify the received token exists and will expire
        let auth_token = response.clone().auth_token();
        assert!(!auth_token.clone().is_empty());
        let expires_at = response.expires_at();
        assert!(expires_at.does_expire());

        let creds = new_credential_provider_from_token(auth_token);
        let cc = new_cache_client(creds.clone());

        // Should be able to read and write in auth cache
        let test_item = TestScalar::new();
        assert_get_success(&cc, &CACHE_TEST_STATE.auth_cache_name, test_item.key()).await?;
        assert_set_success(
            &cc,
            &CACHE_TEST_STATE.auth_cache_name,
            test_item.key(),
            test_item.value(),
        )
        .await?;

        // Wait for token to expire (with some buffer time)
        let instant = tokio::time::Instant::now();
        let wait_time = tokio::time::Duration::from_secs(10);
        tokio::time::sleep(wait_time).await;
        assert!(instant.elapsed() >= wait_time);

        // Should not be able to read and write in auth cache
        assert_get_failure(&cc, &CACHE_TEST_STATE.auth_cache_name, test_item.key()).await?;
        assert_set_failure(
            &cc,
            &CACHE_TEST_STATE.auth_cache_name,
            test_item.key(),
            test_item.value(),
        )
        .await?;

        Ok(())
    }
}
