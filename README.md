# Staple

**Staple** is an independent, from-scratch rewrite of the Paperclip
control plane for AI-agent companies, built with **Rust + Topcoat + Turso**.

> This repository is **not a GitHub fork**. It is not linked to
> [paperclipai/paperclip](https://github.com/paperclipai/paperclip) and does not
> track it. The upstream codebase appears here only as an initial reference
> snapshot; the goal is to replace it entirely with a Rust implementation.

## Status

- **Phase 0–4 — done**: Rust workspace, Turso data layer, core APIs, governance,
  board UI, adapters, and the plugin mechanism are implemented and tested
  (see [doc/plans/parity-checklist.md](doc/plans/parity-checklist.md)).
- **Phase 5 — in progress**: parity tracking, data migration tool, and the
  dual-stack switch.

## Runtime switch (dual-stack)

The Rust binary is the default dev entrypoint (`make dev`). The Node reference
snapshot was frozen and removed from this repository (Phase 5); the reference mirror keeps the Node code
for behavior comparison.

- Switch to Rust: `make dev` (default).
- Rollback: use the reference mirror (`gqf2008/paperclip`) if a Node runtime is ever needed.
- Smoke the core flow through the Rust binary: `make smoke`.

See [doc/plans/2026-08-03-topcoat-turso-rewrite.md](doc/plans/2026-08-03-topcoat-turso-rewrite.md)
for the full roadmap.

## What we're building

- Companies, org structure, goals, projects, and hierarchical tasks (issues)
- Heartbeat execution control plane: atomic checkout, execution locks, budgets
  with hard-stop auto-pause, approval gates, activity audit
- Agent adapters: local CLI sessions, HTTP/webhook runtimes, external plugins
- Governance: secrets, audit log, decision desk, skills policy
- Board UI rendered by Topcoat (server-first, no WASM). Press Cmd/Ctrl+K anywhere for the global command palette (pages + task search).

## Layout

The Node.js snapshot (`server/`, `ui/`, `packages/`) was frozen and removed kept for functional reference. Rust code lands in `crates/` as the
rewrite progresses; Node code is removed as it is replaced.

- `doc/` — product docs, implementation spec, rewrite plan
- `crates/` — Rust workspace (Topcoat app, domain services, Turso data layer)
- `tools/migrate/` — Postgres → Turso migration tooling

## Upstream reference mirror

The upstream codebase stays readable and up to date at
[gqf2008/paperclip](https://github.com/gqf2008/paperclip), which is a fork with
automatic sync (`sync/upstream` branch + fast-forward/PR workflow). This repo
only consumes it as a reference; changes never merge back.

## License

MIT (kept from the upstream snapshot for attribution; see [LICENSE](LICENSE)).

## For Codex agents

Starting work on this repo? Read
[doc/plans/2026-08-03-codex-onboarding.md](doc/plans/2026-08-03-codex-onboarding.md)
first — it covers the workspace, issue-driven workflow, and coding rules.

## Rust development

The rewrite lives in `crates/` as a Cargo workspace. The toolchain is pinned
by `rust-toolchain.toml` (stable, edition 2024).

- `make dev` — run the app (defaults to `127.0.0.1:3100`; `HOST`/`PORT`/`RUST_LOG` override)
- `make test` — run all workspace tests
- `make lint` — `cargo fmt --check` + `cargo clippy -- -D warnings`
- `make build` — release build
- `make js-test` — run the UI JS behavior tests (Node built-in test runner, zero dependencies)
- `make ui-e2e` — run the UI/UX alignment end-to-end tests (Playwright computed-style assertions vs upstream-aligned specs; builds + boots the app on 3109 with a fresh self-seeding DB, saves screenshots + JSON report under `target/ui-e2e-*`; requires `npx playwright install chromium`, or set `PW_EXECUTABLE`)

### Data layer

- Embedded dev database at `data/staple.db` by default (override with `STAPLE_DB_PATH`)
- Remote Turso: set `TURSO_URL` and `TURSO_AUTH_TOKEN`
- Schema is versioned via SQL migrations in `crates/data/migrations/` (up/down, idempotent)

Quick check while the app runs:

```sh
curl http://localhost:3100/api/health
# {"status":"ok"}
```

CI (`.github/workflows/ci.yml`) runs fmt, clippy, tests, and the release build
on every push/PR.
