{{ ossHeader }}

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
{% include "./example/rust/src/bin/readme.rs" %}
```

Note that the above code requires an environment variable named MOMENTO_API_KEY which must
be set to a valid [Momento authentication token](https://docs.momentohq.com/cache/develop/authentication/api-keys).

## Getting Started and Documentation

Documentation is available on the [Momento Docs website](https://docs.momentohq.com).

## Examples

Ready to dive right in? Just check out the [example](./example/README.md) directory for complete, working examples of how to use the SDK.

## Developing

If you are interested in contributing to the SDK, please see the [CONTRIBUTING](./CONTRIBUTING.md) docs.

{{ ossFooter }}
