# Welcome to Momento Rust SDK contributing guide :wave:

Thank you for taking your time to contribute to our Rust SDK!
<br/>
This guide will provide you information to start development and testing.
<br/>
Happy coding :dancer:
<br/>

## Requirements :coffee:

- A modern `cargo` tool chain is required; we love [rustup](https://rustup.rs/)
- A Momento API key is required. You can generate one using the [Momento Console](https://console.gomomento.com)

<br/>


### Installing Momento and Running the Example

Check out [example](./example/) directory!

<br/>

## Build :computer:

We use `cargo` to build and run other tasks such as linting. See the `Makefile` and the workflows in `.github/workflows` for the exact commands that are run in CI/CD.

Here are the basics:

```bash
# build
make build

# lint
make lint

# run all tests (unit, integration, and doctests)
MOMENTO_API_KEY=<api key> make test 

# shortcut to run all of the targets before committing:
MOMENTO_API_KEY=<api key> make
```

## Development 🔨

At Momento, we believe [exceptions are bugs](https://www.gomomento.com/blog/exceptions-are-bugs). In Rust, this means the
unchecked use of `.unwrap()` calls inherently fills your code with bugs and makes debugging `panic` calls a lot more difficult.

We rigorously check for proper formatting and use `clippy` to check for good code practices as well as avoiding `.unwrap()` calls. Instead, try to use
an alternative, including but not limited to:

- `.expect("descriptive error message")`
- `.unwrap_or_default("default string goes here")`
- Use `?` to have the caller handle the error

### Building

Run this command to verify everything passes so that you can save yourself some time when our GitHub actions are ran against the commit:

```bash
MOMENTO_API_KEY=<api key> make
```
