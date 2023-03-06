use std::future::Future;
use std::time::Duration;

use momento::SimpleCacheClientBuilder;
use uuid::Uuid;

pub type DoctestResult = anyhow::Result<()>;

/// Doctest helper function.
///
/// This function takes care of common setup/cleanup tasks for doctests:
/// - Reading the auth token from the environment
/// - Creating a cache for the doctest to use.
/// - Ensuring that cache is deleted, even if the test case panics.
pub fn doctest<'ctx, Fn: 'ctx, Fut: 'ctx>(func: Fn) -> DoctestResult
where
    Fn: FnOnce(String, String) -> Fut,
    Fut: Future<Output = DoctestResult>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // The constructor for the cache client needs a tokio runtime to be active.
    let _guard = runtime.enter();

    let cache_name = "rust-sdk-".to_string() + &Uuid::new_v4().to_string();
    let auth_token = std::env::var("TEST_AUTH_TOKEN").expect("TEST_AUTH_TOKEN must be set");

    let mut client =
        SimpleCacheClientBuilder::new(auth_token.clone(), Duration::from_secs(5))?.build();
    runtime.block_on(client.create_cache(&cache_name))?;

    let runtime = scopeguard::guard(runtime, {
        let cache_name = cache_name.clone();
        move |runtime| {
            let _ = runtime.block_on(client.delete_cache(&cache_name));

            // If any background tasks were spawned we give them some time to exit cleanly.
            runtime.shutdown_timeout(Duration::from_secs(1));
        }
    });

    runtime.block_on(func(cache_name, auth_token))
}
