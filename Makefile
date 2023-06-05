all: ci
test-all:
	cargo test --release
build-all:
	cargo build --release
build-docs:
	cargo doc --release --no-deps
ci: test-all build-all lint-all build-docs
lint-all: clippy fmt-check
clippy:
	cargo clippy
fmt: 
	cargo fmt
fmt-check:
	cargo fmt --all -- --check
