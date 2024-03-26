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

> :bulb: **Tip** When installing with `rustup`, which we recommend, ensure no other versions of the Rust toolchain are installed on your system (eg via package managers like homebrew). This can lead to puzzling issues with the `cargo` command. Check `which cargo` to ensure you are using the correct version. Uninstall any other versions of Rust if necessary (`brew uninstall rust`).

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

### Tests

There are three different kinds of tests in this repo:

- Unit tests: these are defined via the `#[test]` macro, in the same file as the code they are testing. For an example see `credential_provider.rs`.
- Doc tests: these are defined inside of docstrings. They are intended more for documentation of how users should interact with our SDK than for exhaustive test coverage.
- Integration tests: these live in the `tests` directory, and this is where the lion's share of our test coverage will be.

To run all the tests you can do:

```
MOMENTO_API_KEY=<api key> make test
```

To run only the unit tests:

```
MOMENTO_API_KEY=<api key> make test-unit
```

To run only the doc tests:

```
MOMENTO_API_KEY=<api key> make test-doctests
```

To run only the integration tests:

```
MOMENTO_API_KEY=<api key> make test-integration
```

To run a single file of integration tests:

```
MOMENTO_API_KEY=<api key> cargo test --test 'cache_sorted_set'
```

### More details on doctests

Docstring are denoted via `///` comment blocks preceding a function or struct.

Inside of these docstrings you can include code blocks, via `/// \`\`\``.

The code inside of these code blocks will be compiled and executed as part of the doctest execution. This ensures that our docs always have working code; the compile will fail if the docstring code is not valid.j

The resulting docs will be automatically published to https://docs.rs/momento/latest/momento/ . However you may hide certain lines from the published docs by prefixing them with a `#` (`/// #`). Use this liberally to remove extraneous details from the documentation while keeping the code blocks self-contained and syntactically valid.

You can build and examine the docs locally at any time via:

```
cargo doc --open
```

The goal of these code snippets is to show users how to use the APIs; they are not intended for
exhaustive test coverage. You can (and should) add `assert` statements if they will help ensure
the code is correct, but these `assert` statements should always be excluded from the published docs
via a `#` comment.
