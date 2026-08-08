.PHONY: dev test lint build check js-test ui-e2e

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

# JS behavior tests for the Topcoat UI (Node built-in test runner, zero
# dependencies; runs scripts/tests/*.test.mjs against the real palette script)
js-test:
	node --test scripts/tests/*.test.mjs

# UI/UX alignment end-to-end tests (issue #254). Builds the app, boots it on
# 3109 with a fresh DB (the suite self-seeds a company/issue via the API),
# runs Playwright computed-style assertions against the upstream-aligned
# specs, and saves screenshots + JSON report under target/ui-e2e-*.
# Requirements: node with playwright available (NODE_PATH auto-added), and a
# chromium browser (`npx playwright install chromium`, or pass PW_EXECUTABLE).
ui-e2e:
	cargo build -p staple-app
	@mkdir -p target
	@rm -f /tmp/staple-ui-e2e.db
	@STAPLE_DB_PATH=$(or $(STAPLE_DB_PATH),/tmp/staple-ui-e2e.db) PORT=3109 ./target/debug/staple-app > target/ui-e2e.log 2>&1 & echo $$! > target/ui-e2e.pid
	@sleep 1
	@for i in $$(seq 1 90); do if curl -sf http://127.0.0.1:3109/api/health >/dev/null; then echo "ui-e2e: server up on 3109"; break; fi; sleep 1; done; 	 curl -sf http://127.0.0.1:3109/api/health >/dev/null || { echo "ui-e2e: server failed to start (see target/ui-e2e.log)"; kill $$(cat target/ui-e2e.pid) 2>/dev/null; exit 1; }
	@NODE_PATH=$$(npm root -g 2>/dev/null || true) BASE_URL=http://127.0.0.1:3109 E2E_OUT_DIR=$(CURDIR)/target/ui-e2e-screenshots E2E_REPORT=$(CURDIR)/target/ui-e2e-report.json node --test scripts/ui_e2e/alignment.test.mjs; status=$$?; kill $$(cat target/ui-e2e.pid) 2>/dev/null; rm -f target/ui-e2e.pid; exit $$status

# Node reference runtime was frozen and removed (Phase 5). The reference
# mirror (gqf2008/paperclip) retains the Node code for behavior comparison.
# Local pre-push check
check: lint test
