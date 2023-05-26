# Momento client-sdk-rust

⚠️ Experimental SDK ⚠️

Rust SDK for Momento is experimental and under active development.
There could be non-backward compatible changes or removal in the future.
Please be aware that you may need to update your source code with the current version of the SDK when its version gets upgraded.

---

<br/>

Rust SDK for Momento, a serverless cache that automatically scales without any of the operational overhead required by traditional caching solutions.

<br/>

## Getting Started 🏃

### Requirements

- A Momento Auth Token is required, you can generate one using the [Momento Console](https://console.gomomento.com)

<br/>

### Installing Momento and Running the Example

Check out [example](./example/) directory!

<br/>

### Using Momento

```rust
use momento::{CredentialProviderBuilder, SimpleCacheClientBuilder};
use std::env;
use std::num::NonZeroU64;

async fn demo_cache_usage() {
    // Initialize Momento
    let credential_provider =
        CredentialProviderBuilder::new_from_environment_variable("MOMENTO_AUTH_TOKEN")
            .build()
            .expect("env var MOMENTO_AUTH_TOKEN must be set to your auth token");
    let item_default_ttl_seconds = 60;
    let mut cache_client = SimpleCacheClientBuilder::new(
        credential_provider,
        NonZeroU64::new(item_default_ttl_seconds).unwrap(),
    )
    .unwrap()
    .build();

    // Create a cache named "cache"
    let cache_name = String::from("cache");
    cache_client.create_cache(&cache_name).await.unwrap();

    // Set key with default TTL and get value with that key
    let key = String::from("my_key");
    let value = String::from("my_value");
    cache_client
        .set(&cache_name, key.clone(), value.clone(), None)
        .await
        .unwrap();
    let result = cache_client.get(&cache_name, key.clone()).await.unwrap();
    println!("Looked up value: {:?}", result.as_string());

    // Set key with TTL of 5 seconds
    cache_client
        .set(&cache_name, key.clone(), value.clone(), NonZeroU64::new(5))
        .await
        .unwrap();

    // Permanently delete cache
    cache_client.delete_cache(&cache_name).await.unwrap();
}
```

<br/>

## Running Tests ⚡

Doc and integration tests require an auth token for testing. Set the env var `TEST_AUTH_TOKEN` to
provide it.

Running unit tests:

```
cargo test --lib
```

Running doc tests:

```
TEST_AUTH_TOKEN=<auth token> cargo test --doc
```

Running integration tests:

```
TEST_AUTH_TOKEN=<auth token> cargo test --tests
```

## Development 🔨

At Momento, we believe [exceptions are bugs](https://www.gomomento.com/blog/exceptions-are-bugs). In Rust, this means the
unchecked use of `.unwrap()` calls inherently fills your code with bugs and makes debugging `panic` calls a lot more difficult.

We rigorously check for proper formatting and use `clippy` to check for good code practices as well as avoiding `.unwrap()` calls. Instead, try to use
an alternative, including but not limited to:

- `.expect("descriptive error message")`
- `.unwrap_or_default("default string goes here")`
- Use `?` to have the caller handle the error

### Building

Run this command to verify everything passes so that you can save yourself some time when our GitHub actions are ran against the commit:

```bash
cargo build && cargo clippy --all-targets --all-features -- -D warnings -W clippy::unwrap_used
```
