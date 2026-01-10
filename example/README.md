<img src="https://docs.momentohq.com/img/momento-logo-forest.svg" alt="logo" width="400"/>

[![project status](https://momentohq.github.io/standards-and-practices/badges/project-status-official.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)
[![project stability](https://momentohq.github.io/standards-and-practices/badges/project-stability-beta.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)


# Momento Rust SDK - Examples

This directory contains fully-functioning examples that demonstrate how to use the Momento Rust SDK.

## Example Requirements

- Follow the [installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install Rust and Cargo.
- To get started with Momento you will need a Momento API key. You can get one from the [Momento Console](https://console.gomomento.com).
- A Momento service endpoint is required. You can find a [list of them here](https://docs.momentohq.com/platform/regions)

Here are the different examples available:

- [Simple Rust Examples](./rust) - Basic Cache and Topics examples using the Momento Rust SDK; for server-side use, etc.
- [CDK-based AWS Lambda Example](./aws/cdk-lambda) - Example of using the Momento Rust SDK in an AWS Lambda Function, deployed via AWS CDK.
- [Zip-based AWS Lambda Example](./aws/zip-lambda) - Example of using the Momento Rust SDK in an AWS Lambda Function, deployed via a zip archive.

----------------------------------------------------------------------------------------
For more info, visit our website at [https://gomomento.com](https://gomomento.com)!
