# Momento client-sdk-rust

:warning: Experimental SDK :warning:

Rust SDK for Momento is experimental and under active development.
There could be non-backward compatible changes or removal in the future.
Please be aware that you may need to update your source code with the current version of the SDK when its version gets upgraded.

---

<br/>

Rust SDK for Momento, a serverless cache that automatically scales without any of the operational overhead required by traditional caching solutions.

<br/>

## Getting Started :running:

### Requirements

- A Momento Auth Token is required, you can generate one using the [Momento CLI](https://github.com/momentohq/momento-cli)

<br/>

### Using Momento

```rust
use std::env;
use momento::simple_cache_client::SimpleCacheClient;

// Initializing Momento
let auth_token = env::var("TEST_AUTH_TOKEN").expect("env var TEST_AUTH_TOKEN must be set");
let item_default_ttl_seconds = 60
let mut cache_client = SimpleCacheClient::new(auth_token, item_default_ttl_seconds).await.unwrap();

// Creating a cache named "cache"
let cache_name = String::from("cache");
cache_client.create_cache(&cache_name).await.unwrap()

// Sets key with default TTL and get value with that key
let key = String::from("my_key");
let value = String::from("my_value");
cache_client.set(&cache_name, key.clone(), value.clone(), None).await.unwrap();
let result = cache_client.get(&cache_name, key.clone()).await.unwrap();
println!("Looked up value: {}", result.value);

// Sets key with TTL of 5 seconds
cache_client.set(&cache_name, key.clone(), value.clone(), 5).await.unwrap();

// Permanently deletes cache
cache_client.delete_cache(&cache_name).await.unwrap();
```

<br/>

## Running Tests :zap:

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
