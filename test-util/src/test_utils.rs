use std::env;
use std::future::Future;
use std::time::Duration;

use momento::cache::configurations::{self, PrebuiltConfiguration};
use momento::CacheClient;
use momento::CredentialProvider;

use crate::unique_cache_name;

pub type DoctestResult = anyhow::Result<()>;

/// Doctest helper function.
///
/// This function takes care of common setup/cleanup tasks for doctests:
/// - Reading the auth token from the environment
/// - Creating a cache for the doctest to use.
/// - Ensuring that cache is deleted, even if the test case panics.
pub fn doctest<'ctx, Fn: 'ctx, Fut: 'ctx>(func: Fn) -> DoctestResult
where
    Fn: FnOnce(String, CredentialProvider) -> Fut,
    Fut: Future<Output = DoctestResult>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // The constructor for the cache client needs a tokio runtime to be active.
    let _guard = runtime.enter();

    let cache_name = unique_cache_name();
    let (client, credential_provider) = build_cache_client_and_credential_provider();
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
    let (cache_client, _) = build_cache_client_and_credential_provider();
    (cache_client, cache_name)
}

pub fn get_test_cache_name() -> String {
    env::var("TEST_CACHE_NAME").unwrap_or("rust-sdk-test-cache".to_string())
}

pub fn get_test_credential_provider() -> CredentialProvider {
    CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
        .expect("auth token should be valid")
}

pub fn build_cache_client_and_credential_provider() -> (CacheClient, CredentialProvider) {
    let credential_provider = get_test_credential_provider();
    let cache_client = momento::CacheClient::builder()
        .default_ttl(Duration::from_secs(5))
        .configuration(configurations::Laptop::latest())
        .credential_provider(credential_provider.clone())
        .build()
        .expect("cache client should be created");

    (cache_client, credential_provider)
}
