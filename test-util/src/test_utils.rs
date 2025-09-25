use std::env;
use std::future::Future;
use std::time::Duration;

use momento::cache::configurations;
use momento::{
    protosocket, AuthClient, CacheClient, CredentialProvider, FunctionClient, LeaderboardClient,
    ProtosocketCacheClient, TopicClient,
};

use crate::unique_cache_name;

pub type DoctestResult = anyhow::Result<()>;

/// Doctest helper function.
///
/// This function takes care of common setup/cleanup tasks for doctests:
/// - Reading the auth token from the environment
/// - Creating a cache for the doctest to use.
/// - Ensuring that cache is deleted, even if the test case panics.
pub fn doctest<'ctx, Fn, Fut>(func: Fn) -> DoctestResult
where
    Fn: 'ctx + FnOnce(String, CredentialProvider) -> Fut,
    Fut: 'ctx + Future<Output = DoctestResult>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // The constructor for the cache client needs a tokio runtime to be active.
    let _guard = runtime.enter();

    let cache_name = unique_cache_name();
    let (client, _, _, _, credential_provider) = build_clients_and_credential_provider();
    runtime.block_on(client.create_cache(&cache_name))?;

    let runtime = scopeguard::guard(runtime, {
        let cache_name = cache_name.clone();
        move |runtime| {
            let _ = runtime.block_on(client.delete_cache(&cache_name));

            // If any background tasks were spawned we give them some time to exit cleanly.
            runtime.shutdown_timeout(Duration::from_secs(1));
        }
    });

    runtime.block_on(func(cache_name, credential_provider))
}

pub fn create_doctest_cache_client() -> (CacheClient, String) {
    let cache_name = get_test_cache_name();
    let (cache_client, _, _, _, _) = build_clients_and_credential_provider();
    (cache_client, cache_name)
}

pub fn create_doctest_topic_client() -> (TopicClient, String) {
    let cache_name = get_test_cache_name();
    let (_, _, topic_client, _, _) = build_clients_and_credential_provider();
    (topic_client, cache_name)
}

pub fn create_doctest_auth_client() -> AuthClient {
    let (_, _, _, auth_client, _) = build_clients_and_credential_provider();
    auth_client
}

pub fn create_doctest_function_client() -> (FunctionClient, String) {
    let cache_name = get_test_cache_name();
    let credential_provider = get_test_credential_provider();
    let client = momento::FunctionClient::builder()
        .credential_provider(credential_provider.clone())
        .build()
        .expect("cache client should be created");
    (client, cache_name)
}

pub async fn create_doctest_protosocket_cache_client() -> (ProtosocketCacheClient, String) {
    let cache_name = get_test_cache_name();
    let credential_provider = get_test_credential_provider();
    let client = momento::ProtosocketCacheClient::builder()
        .default_ttl(Duration::from_secs(5))
        .configuration(protosocket::cache::configurations::Laptop::latest())
        .credential_provider(credential_provider.clone())
        .runtime(tokio::runtime::Handle::current())
        .build()
        .await
        .expect("cache client should be created");
    (client, cache_name)
}

pub fn get_test_cache_name() -> String {
    env::var("TEST_CACHE_NAME").unwrap_or("rust-sdk-test-cache".to_string())
}

pub fn get_test_store_name() -> String {
    env::var("TEST_STORE_NAME").unwrap_or("rust-sdk-test-store".to_string())
}

pub fn get_test_auth_cache_name() -> String {
    env::var("TEST_AUTH_CACHE_NAME").unwrap_or("rust-sdk-test-cache-auth".to_string())
}

#[allow(clippy::expect_used)] // we want to panic if the env var is not set
pub fn get_test_credential_provider() -> CredentialProvider {
    CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
        .expect("auth token should be valid")
}

pub fn build_clients_and_credential_provider() -> (
    CacheClient,
    LeaderboardClient,
    TopicClient,
    AuthClient,
    CredentialProvider,
) {
    let credential_provider = get_test_credential_provider();
    let cache_client = momento::CacheClient::builder()
        .default_ttl(Duration::from_secs(5))
        .configuration(configurations::Laptop::latest())
        .credential_provider(credential_provider.clone())
        .build()
        .expect("cache client should be created");
    let leaderboard_client = momento::LeaderboardClient::builder()
        .configuration(momento::leaderboard::configurations::Laptop::latest())
        .credential_provider(credential_provider.clone())
        .build()
        .expect("leaderboard client should be created");
    let topic_client = momento::TopicClient::builder()
        .configuration(momento::topics::configurations::Laptop::latest())
        .credential_provider(credential_provider.clone())
        .build()
        .expect("topic client should be created");
    let auth_client = momento::AuthClient::builder()
        .credential_provider(credential_provider.clone())
        .build()
        .expect("auth client should be created");

    (
        cache_client,
        leaderboard_client,
        topic_client,
        auth_client,
        credential_provider,
    )
}
