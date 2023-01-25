# Rust Client SDK

_Read this in other languages_: [日本語](README.ja.md)

<br>

## Running the Example

- Rust and Cargo are needed. [Installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html).
- A Momento Auth Token is required, you can generate one using the [Momento CLI](https://github.com/momentohq/momento-cli)

```bash
# Run example code
MOMENTO_AUTH_TOKEN=<YOUR AUTH TOKEN> cargo run
```

Example Code: [main.rs](src/main.rs)

## Using the SDK in your projects

Add the following line to your Cargo.toml file: `momento = "0.3.1"`, or you can check out the latest version [here](https://crates.io/crates/momento).
