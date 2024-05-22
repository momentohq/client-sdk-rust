.PHONY: all
## Generate sync unit tests, format, lint, and test
all: precommit

.PHONY: lint
## Check the formatting of all files, run clippy on the source code, then run
## clippy on the tests (but allow expect to be used in tests)
lint:
	cd sdk && \
	cargo fmt -- --check && \
	cargo clippy --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W missing_docs && \
	cargo clippy --tests -- -D warnings -W clippy::unwrap_used

.PHONY: build
## Build project
build:
	cd sdk && cargo build --verbose

.PHONY: clean
## Remove build files
clean:
	cd sdk && cargo clean

.PHONY: clean-build
## Build project
clean-build: clean build

.PHONY: docs
## Build the docs, fail on warnings
docs:
	cd sdk && RUSTDOCFLAGS="-D warnings" cargo doc

.PHONY: precommit
## Run clean-build and test as a step before committing.
precommit: clean-build lint test build-examples


.PHONY: test-unit
test-unit:
	cd sdk && cargo test --lib

.PHONY: test-doctests
test-doctests:
	cd sdk && cargo test --doc

.PHONY: ci-test-setup
ci-test-setup:
	# This script relies on dev dependencies such as Tokio, so we run it with the --example flag
	cd sdk && cargo run --example test-setup

.PHONY: test-integration
## Run integration tests
test-integration:
	cd sdk && cargo test --tests

.PHONY: ci-test-teardown
ci-test-teardown:
	# This script relies on dev dependencies such as Tokio, so we run it with the --example flag
	cd sdk && cargo run --example test-teardown

.PHONY: test
## Run unit and integration tests
test: test-unit test-integration test-doctests

.PHONY: build-examples
## Build example code
build-examples:
	cd example/rust && make lint && make build

.PHONY: run-examples
## Run example code
run-examples:
	cd example/rust && make lint && cargo run --bin=readme && cargo run --bin=cache && cargo run --bin=topics && cargo run --bin=docs_examples && cargo run --bin=cheat_sheet_client_instantiation

# See <https://gist.github.com/klmr/575726c7e05d8780505a> for explanation.
.PHONY: help
help:
	@echo "$$(tput bold)Available rules:$$(tput sgr0)";echo;sed -ne"/^## /{h;s/.*//;:d" -e"H;n;s/^## //;td" -e"s/:.*//;G;s/\\n## /---/;s/\\n/ /g;p;}" ${MAKEFILE_LIST}|LC_ALL='C' sort -f|awk -F --- -v n=$$(tput cols) -v i=19 -v a="$$(tput setaf 6)" -v z="$$(tput sgr0)" '{printf"%s%*s%s ",a,-i,$$1,z;m=split($$2,w," ");l=n-i;for(j=1;j<=m;j++){l-=length(w[j])+1;if(l<= 0){l=n-i-length(w[j])-1;printf"\n%*s ",-i," ";}printf"%s ",w[j];}printf"\n";}'|more $(shell test $(shell uname) == Darwin && echo '-Xr')

.PHONY: publish
publish:
	cd sdk && cp ../README.md . && cargo publish
