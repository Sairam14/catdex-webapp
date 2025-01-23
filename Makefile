format:
	cargo fmt --quiet

format-check:
	@rustup component add rustfmt 2> /dev/null
	cargo fmt --all -- --check

run:
	cargo run -- dedupe --path tests --pattern .txt

lint:
	@rustup component add clippy 2> /dev/null
	@cargo-clippy --all-targets --all-features -- -D warnings

test:
	cargo test

build-release:
	@echo "Building release version $(shell uname -s)"
	cargo build --release

build-debug:
	@echo "Building debug version $(shell uname -s)"
	cargo build

all:
	@echo "Building all versions"
	@make build-release
	@make build-debug

clean:
	cargo clean
	