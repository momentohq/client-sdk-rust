<img src="https://docs.momentohq.com/img/momento-logo-forest.svg" alt="logo" width="400"/>

[![project status](https://momentohq.github.io/standards-and-practices/badges/project-status-official.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)
[![project stability](https://momentohq.github.io/standards-and-practices/badges/project-stability-beta.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)


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

Note: to see a non-null `publisher_id` on a received [`SubscriptionValue`](https://docs.rs/momento/0.46.1/momento/topics/struct.SubscriptionValue.html), you'll need to set the optional `token_id` argument when programmatically creating a [disposable token](https://docs.momentohq.com/cache/develop/api-reference/auth#generatedisposabletoken). An example of this is provided in [topics.rs](src/bin/topics.rs).

----------------------------------------------------------------------------------------
For more info, visit our website at [https://gomomento.com](https://gomomento.com)!
