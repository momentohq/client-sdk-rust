{{ ossHeader }}

# Momento Rust SDK - Examples

_Read this in other languages_: [日本語](README.ja.md)

<br>

This directory contains fully-functioning examples that demonstrate how to use the Momento Rust SDK.

## Example Requirements

- Follow the [installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install Rust and Cargo.
- To get started with Momento you will need a Momento API key. You can get one from the [Momento Console](https://console.gomomento.com).

## Running the Cache Example

This example demonstrates a basic set and get from a cache.

```bash
# Run example code
MOMENTO_API_KEY=<YOUR API KEY> cargo run --bin=cache
```

Example Code: [cache.rs](src/bin/cache.rs)

## Running the Topics Example

This example demonstrates subscribing to and publishing to a Topic.

```bash
# Run example code
MOMENTO_API_KEY=<YOUR API KEY> cargo run --bin=topics
```

Example Code: [topics.rs](src/bin/topics.rs)

{{ ossFooter }}
