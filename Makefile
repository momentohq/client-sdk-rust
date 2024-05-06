.PHONY: all
## Generate sync unit tests, format, lint, and test
all: precommit

.PHONY: lint
lint:
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings -W clippy::unwrap_used

.PHONY: build
## Build project
build:
	cargo build --verbose

.PHONY: clean
## Remove build files
clean:
	cargo clean

.PHONY: clean-build
## Build project
clean-build: clean build

.PHONY: docs
## Build the docs, fail on warnings
docs:
	RUSTDOCFLAGS="-D warnings" cargo doc

.PHONY: precommit
## Run clean-build and test as a step before committing.
precommit: clean-build lint test build-examples


.PHONY: test-unit
test-unit:
	cargo test --lib

.PHONY: test-doctests
test-doctests:
	cargo test --doc

.PHONY: test-integration
## Run integration tests
test-integration:
	# Reason for "--ignored":
	# The flush_cache test first so as not to introduce a race condition that
	# might cause the other tests to fail since they all use the same test cache.
	cargo test --tests -- --ignored
	cargo test --tests

.PHONY: test
## Run unit and integration tests
test: test-unit test-integration test-doctests

.PHONY: build-examples
## Build example code
build-examples:
	cd example && make lint && make build

.PHONY: run-examples
## Run example code
run-examples:
	cd example && cargo run --bin=rust
	cd example && cargo run --bin=readme
	cd example && cargo run --bin=docs_examples

# See <https://gist.github.com/klmr/575726c7e05d8780505a> for explanation.
.PHONY: help
help:
	@echo "$$(tput bold)Available rules:$$(tput sgr0)";echo;sed -ne"/^## /{h;s/.*//;:d" -e"H;n;s/^## //;td" -e"s/:.*//;G;s/\\n## /---/;s/\\n/ /g;p;}" ${MAKEFILE_LIST}|LC_ALL='C' sort -f|awk -F --- -v n=$$(tput cols) -v i=19 -v a="$$(tput setaf 6)" -v z="$$(tput sgr0)" '{printf"%s%*s%s ",a,-i,$$1,z;m=split($$2,w," ");l=n-i;for(j=1;j<=m;j++){l-=length(w[j])+1;if(l<= 0){l=n-i-length(w[j])-1;printf"\n%*s ",-i," ";}printf"%s ",w[j];}printf"\n";}'|more $(shell test $(shell uname) == Darwin && echo '-Xr')
