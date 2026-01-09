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

## Prerequisites

- Follow the [installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install Rust and Cargo.
- To get started with Momento you will need a Momento API key. You can get one from the [Momento Console](https://console.gomomento.com).
- A Momento service endpoint is required. You can find a [list of them here](https://docs.momentohq.com/platform/regions)

## Usage

Here is a quickstart you can use in your own project:

```rust
{% include "./example/rust/src/bin/readme.rs" %}
```

## Getting Started and Documentation

Documentation is available on the [Momento Docs website](https://docs.momentohq.com).

## Examples

Ready to dive right in? Just check out the [example](./example/README.md) directory for complete, working examples of how to use the SDK.

## Developing

If you are interested in contributing to the SDK, please see the [CONTRIBUTING](./CONTRIBUTING.md) docs.

{{ ossFooter }}
