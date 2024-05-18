<head>
  <meta name="Momento Rust Client Library Documentation" content="Rust client software development kit for Momento Cache">
</head>
<img src="https://docs.momentohq.com/img/momento-logo-forest.svg" alt="logo" width="400"/>

[![project status](https://momentohq.github.io/standards-and-practices/badges/project-status-official.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)
[![project stability](https://momentohq.github.io/standards-and-practices/badges/project-stability-alpha.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)

# Momento Rust Client Library

Momento Cache is a fast, simple, pay-as-you-go caching solution without any of the operational overhead
required by traditional caching solutions.  This repo contains the source code for the Momento Rust client library.

To get started with Momento you will need a Momento Auth Token. You can get one from the [Momento Console](https://console.gomomento.com).

* Website: [https://www.gomomento.com/](https://www.gomomento.com/)
* Momento Documentation: [https://docs.momentohq.com/](https://docs.momentohq.com/)
* Getting Started: [https://docs.momentohq.com/getting-started](https://docs.momentohq.com/getting-started)
* Rust SDK Documentation: [https://docs.momentohq.com/sdks/rust](https://docs.momentohq.com/sdks/rust)
* Discuss: [Momento Discord](https://discord.gg/3HkAKjUZGq)

## Packages

The Momento Rust SDK package is available on `crates.io`: [momento](https://crates.io/crates/momento).

You will need to install additional dependencies to make full use of our SDK:

```bash
cargo add momento
cargo add tokio --features full
cargo add futures
```

Note: you will only need to install `futures` if you use Momento Topics.

## Usage

Here is a quickstart you can use in your own project:

```rust
use momento::cache::{configurations::Laptop, GetResponse};
use momento::{CacheClient, CredentialProvider, MomentoError};
use std::time::Duration;

const CACHE_NAME: &str = "cache";

#[tokio::main]
pub async fn main() -> Result<(), MomentoError> {
    let cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(60))
        .configuration(Laptop::latest())
        .credential_provider(CredentialProvider::from_env_var(
            "MOMENTO_API_KEY".to_string(),
        )?)
        .build()?;

    cache_client.create_cache(CACHE_NAME.to_string()).await?;

    match cache_client.set(CACHE_NAME, "mykey", "myvalue").await {
        Ok(_) => println!("Successfully stored key 'mykey' with value 'myvalue'"),
        Err(e) => println!("Error: {}", e),
    }

    let value: String = match cache_client.get(CACHE_NAME, "mykey").await? {
        GetResponse::Hit { value } => value.try_into().expect("I stored a string!"),
        GetResponse::Miss => {
            println!("Cache miss!");
            return Ok(()); // probably you'll do something else here
        }
    };
    println!("Successfully retrieved value: {}", value);

    Ok(())
}

```

Note that the above code requires an environment variable named MOMENTO_API_KEY which must
be set to a valid [Momento authentication token](https://docs.momentohq.com/cache/develop/authentication/api-keys).

## Getting Started and Documentation

Documentation is available on the [Momento Docs website](https://docs.momentohq.com).

## Examples

Ready to dive right in? Just check out the [example](./example/README.md) directory for complete, working examples of how to use the SDK.

## Developing

If you are interested in contributing to the SDK, please see the [CONTRIBUTING](./CONTRIBUTING.md) docs.

----------------------------------------------------------------------------------------
For more info, visit our website at [https://gomomento.com](https://gomomento.com)!
