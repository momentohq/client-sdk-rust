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
* Rust SDK Documentation: [https://docs.momentohq.com/develop/sdks/rust](https://docs.momentohq.com/develop/sdks/rust)
* Discuss: [Momento Discord](https://discord.gg/3HkAKjUZGq)

## Packages

The Momento Rust SDK package is available on `crates.io`: [momento](https://crates.io/crates/momento).

## Usage

Here is a quickstart you can use in your own project:

```csharp
use std::time::Duration;
use momento::{CacheClient, CredentialProvider};
use momento::config::configurations::laptop;

const CACHE_NAME: String = "cache";

pub async fn main() {
    let cache_client = CacheClient::new(
        CredentialProvider::from_env_var("MOMENTO_API_KEY")?,
        laptop::latest(),
        Duration::from_secs(60)
    )?;

    cache_client.create_cache(CACHE_NAME).await?;
    
    match(cache_client.set(CACHE_NAME, "mykey", "myvalue").await) {
        Ok(_) => println!("Successfully stored key 'mykey' with value 'myvalue'"),
        Err(e) => println!("Error: {}", e)
    }
    
    let value = match(cache_client.get(CACHE_NAME, "mykey").await?) {
        
    };


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
