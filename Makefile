.DEFAULT_GOAL := help

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

# -- variables -------------------------------------------------------------------------------------

BACKTRACE=RUST_BACKTRACE=1
WARNINGS=RUSTDOCFLAGS="-D warnings"

# -- linting --------------------------------------------------------------------------------------

.PHONY: clippy
clippy: ## Runs Clippy with configs
	cargo +nightly clippy --workspace --all-targets --all-features -- -D warnings


.PHONY: fix
fix: ## Runs Fix with configs
	cargo +nightly fix --allow-staged --allow-dirty --all-targets --all-features


.PHONY: format
format: ## Runs Format using nightly toolchain
	cargo +nightly fmt --all


.PHONY: format-check
format-check: ## Runs Format using nightly toolchain but only in check mode
	cargo +nightly fmt --all --check


.PHONY: lint
lint: format fix clippy ## Runs all linting tasks at once (Clippy, fixing, formatting)

# --- docs ----------------------------------------------------------------------------------------

.PHONY: doc
doc: ## Generates & checks documentation
	$(WARNINGS) cargo doc --all-features --keep-going --release
# NOTE: This target currently fails due to rustdoc warnings. Requires fixing documentation issues.


.PHONY: book
book: ## Builds the book & serves documentation site
	mdbook serve --open docs
# NOTE: This target requires mdbook to be installed: cargo install mdbook

# --- testing -------------------------------------------------------------------------------------

.PHONY: test-build
test-build: ## Build the test binary
	cargo test --workspace --all-features --no-run


.PHONY: test
test: ## Run all tests
	$(BACKTRACE) cargo test --workspace --all-features


.PHONY: test-docs
test-docs: ## Run documentation tests
	cargo test --doc --all-features


.PHONY: test-fast
test-fast: ## Runs all tests with the debug profile
	cargo test --workspace


.PHONY: test-package
test-package: ## Tests specific package: make test-package package=air-script
	$(BACKTRACE) cargo test -p $(package)

# --- checking ------------------------------------------------------------------------------------

.PHONY: check
check: ## Checks all targets and features for errors without code generation
	cargo check --all-targets --all-features

# --- building ------------------------------------------------------------------------------------

.PHONY: build
build: ## Builds with default parameters
	cargo build --release --all-features


.PHONY: build-no-std
build-no-std: ## Builds without the standard library
# cargo build --no-default-features --target wasm32-unknown-unknown --workspace
# NOTE: Issue #393 needs to be fixed before uncommenting this target
# Requires: rustup target add wasm32-unknown-unknown

# --- executable ----------------------------------------------------------------------------------

.PHONY: exec
exec: ## Builds an executable with optimized profile
	cargo build --release


.PHONY: exec-debug
exec-debug: ## Builds a debug executable
	cargo build