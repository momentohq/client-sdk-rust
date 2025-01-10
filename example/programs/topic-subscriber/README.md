<img src="https://docs.momentohq.com/img/momento-logo-forest.svg" alt="logo" width="400"/>

[![project status](https://momentohq.github.io/standards-and-practices/badges/project-status-official.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)
[![project stability](https://momentohq.github.io/standards-and-practices/badges/project-stability-beta.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)


# Momento Rust SDK - Topic Subscription

This project is a Rust-based example program that subscribes to a Momento Topic and prints the output.  The `main` function handles the topic subscription, deserializes the payloads that it receives into a Rust struct, and prints the output.

This is Rust program can be taken into a container or run locally.

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

- `src/models.rs` - this file holds the `struct` definition that will be deserialized.  Make sure when posting sample payloads that your example JSON has the below shape and data types.

  ```json
  {
    "keyOne": "Key One",
    "keyTwo": "Key Two",
    "keyThree": 100,
    "timestamp": "2024-12-28T14:21:45Z"
  }
  ```

  ```rust
  pub struct MomentoModel {
      pub key_one: String,
      pub key_two: String,
      pub key_three: i64,
      #[serde()]
      #[serde(rename(deserialize = "timestamp"))]
      pub published_timestamp: DateTime<Utc>,
  }
  ```

- `src/main.rs` - holds the Rust `main` function which is the entry point of execution for your code.  The function also establishes a topic subscription that responds to data posted to the Momento Topic.  From there, the data is deserialized and printed to the console.

----------------------------------------------------------------------------------------
For more info, visit our website at [https://gomomento.com](https://gomomento.com)!
