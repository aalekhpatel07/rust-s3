all: ci
test-all:
	cargo test --release
build-all:
	cargo build --release
build-docs:
	cargo doc --release --no-deps
	rm -rf ./docs
	echo "<meta http-equiv=\"refresh\" content=\"0; url=rust-s3-async\">" > target/doc/index.html
	cp -r target/doc ./docs
ci: test-all build-all lint-all build-docs
lint-all: clippy fmt-check
clippy:
	cargo clippy
fmt: 
	cargo fmt
fmt-check:
	cargo fmt --all -- --check
