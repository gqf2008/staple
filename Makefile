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

# Smoke-test the core business flow through the Rust binary surface
smoke:
	cargo test -p staple-app --test release_smoke

# Node reference runtime was frozen and removed (Phase 5). The reference
# mirror (gqf2008/paperclip) retains the Node code for behavior comparison.
# Local pre-push check
check: lint test
