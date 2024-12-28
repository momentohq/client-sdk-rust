<img src="https://docs.momentohq.com/img/momento-logo-forest.svg" alt="logo" width="400"/>

[![project status](https://momentohq.github.io/standards-and-practices/badges/project-status-official.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)
[![project stability](https://momentohq.github.io/standards-and-practices/badges/project-stability-beta.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)


# Momento Rust SDK - Topic Subscription

This directory contains an example project defining a Rust-based program that subscribes to a Momento Cache's Topic, deserializes the payload into. Rust struct, and prints the output.

This is Rust program which can be taken into a container or run locally.

## Example Requirements

- Follow the [installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install Rust and Cargo.
- To get started with Momento you will need a Momento API key. You can get one from the [Momento Console](https://console.gomomento.com).

## Building and Running the Example

Several environment variables are required to be set before getting started.

```bash
export MOMENTO_API_KEY=<Your API Key>
export CACHE=<Name of your Momento Cache>
export TOPIC=<Name of your Topic on the Cache>
```

With those variables set, execute

```bash
cargo run
```

## Interesting Files in this Example

- `src/models.rs` - this file holds the `struct` definition that will be deserialized.  Make sure when posting sample payloads, that your example JSON looks like this

  ```json
  {
    "keyOne": "Key One",
    "keyTwo": "Key Two",
    "keyThree": 100,
    "timestamp": "2024-12-28T14:21:45Z"
  }
  ```

- `src/main.rs` - holds the Rust `main` function which is the entry point and the execution for your code.  Establishes a loop that subscribes to the Momento Topic and responds to change

----------------------------------------------------------------------------------------
For more info, visit our website at [https://gomomento.com](https://gomomento.com)!
