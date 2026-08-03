.PHONY: dev test lint build check

# Run the app (dev mode)
dev:
	cargo run -p staple-app

# Run all workspace tests
test:
	cargo test --workspace

# Format check + clippy (must be clean for CI)
lint:
	cargo fmt --check
	cargo clippy --workspace --all-targets -- -D warnings

# Release build
build:
	cargo build --release --workspace

# Local pre-push check
check: lint test
