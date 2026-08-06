# AGENTS.md

Guidance for human and AI contributors working in this repository.

## 1. Purpose

**Staple** is an independent, from-scratch rewrite of the Paperclip control
plane (AI-agent companies), built with **Rust + Topcoat + Turso/libSQL**.

> This repository is **not a fork** and does not track upstream. The Node.js
> reference snapshot (`server/`, `ui/`, `packages/`) was frozen and removed
> (Phase 5); behavior comparison happens against the read-only reference mirror
> `gqf2008/paperclip`.

## 2. Read This First

Before making changes, read in this order:

1. `doc/plans/2026-08-03-codex-onboarding.md` — workspace, issue-driven workflow, coding rules
2. `doc/plans/2026-08-03-topcoat-turso-rewrite.md` — roadmap, feature list, phased plan
3. `doc/plans/parity-checklist.md` — upstream parity status and deferred items
4. `README.md` — status, runtime switch, Rust dev commands
5. For behavior baselines: the reference mirror `gqf2008/paperclip` docs
   (`doc/SPEC-implementation.md`, `doc/PRODUCT.md`, `doc/DATABASE.md`).

Topcoat source lives locally at `/Volumes/Workspace/GitHub/topcoat` — check its
API/examples before writing UI code; do not invent APIs.

## 3. Repo Map

- `crates/app/` — Topcoat application entry, routes, UI pages, scheduler, team catalog
- `crates/domain/` — domain models and business rules (pure Rust, no I/O)
- `crates/data/` — Turso/libSQL data layer: SQL migrations, repositories
- `crates/adapters/` — agent adapter contracts and implementations (CLI/HTTP/webhook/plugins)
- `tools/migrate/` — Postgres → Turso migration tooling
- `doc/` — operational and product docs (plans under `doc/plans/`)
- `skills/` — Paperclip skills (artifact upload helper etc.)
- `scripts/` — operational helper scripts

## 4. Dev Setup (Embedded Turso)

The Rust binary is the default dev entrypoint. Embedded Turso (file database)
is used when `TURSO_URL` is unset.

```sh
make dev        # or: cargo run -p staple-app
```

This starts:

- API + Topcoat UI: `http://127.0.0.1:3100` (`HOST`/`PORT`/`RUST_LOG` override)
- Embedded database: `data/staple.db` (`STAPLE_DB_PATH` override)

Quick checks:

```sh
curl http://localhost:3100/api/health
curl http://localhost:3100/api/companies
```

Reset the local dev database:

```sh
# stop the app, then delete data/staple.db and restart
```

Production/remote Turso:

```sh
export TURSO_URL=libsql://...
export TURSO_AUTH_TOKEN=...
```

## 5. Core Engineering Rules

1. Keep changes company-scoped.
Every domain entity is scoped to a company; company boundaries must be
enforced in routes/services (`enforce_company_scope`).

2. Keep contracts synchronized.
If you change schema/API behavior, update all impacted layers:
- `crates/data` migrations and repositories
- `crates/domain` models
- `crates/app` routes/services and Topcoat UI pages
- tests (`crates/app/tests/*`, `crates/data/tests/*`)

3. Preserve control-plane invariants.
- Single-assignee task model
- Atomic issue checkout semantics
- Approval gates for governed actions
- Budget hard-stop auto-pause behavior
- Activity logging for mutating actions

4. Do not replace strategic docs wholesale unless asked.
Prefer additive updates. Keep plan docs dated and centralized in `doc/plans/`
as `YYYY-MM-DD-slug.md`; do not use repo markdown files as a substitute for
Paperclip issue planning.

5. Attach inspectable generated artifacts.
When a task produces a user-inspectable deliverable file, follow the Paperclip
skill's "Generated Artifacts and Work Products" workflow
(`skills/paperclip/scripts/paperclip-upload-artifact.sh`; see
`doc/AGENT-ARTIFACTS.md`) before final disposition, and link the artifact in
the final issue comment.

## 6. Database Change Workflow

Data model changes are SQL migrations plus repository code:

1. Add `crates/data/migrations/NNNN_name/up.sql` + `down.sql` (auto-discovered,
   versioned; idempotent `schema_migrations` tracking)
2. Update/extend repositories in `crates/data/src/repositories/` and export
   them from `repositories/mod.rs` / `crates/data/src/lib.rs`
3. Wire into `crates/app/src/state.rs` if a new repository is added
4. Validate:

```sh
cargo test -p staple-data
cargo check -p staple-app
```

## 7. Verification Before Hand-off

Default local checks:

```sh
make lint      # cargo fmt --check + cargo clippy --workspace --all-targets -- -D warnings
make test      # cargo test --workspace
make smoke     # cargo test -p staple-app --test release_smoke
make build     # cargo build --release --workspace
```

Run the smallest relevant check first for normal issue work; run the full
`make lint && make test` (plus `make smoke`/`make build` when UI or release
flows are touched) before claiming repo work PR-ready. If anything cannot be
run, report explicitly what was not run and why.

## 8. API and Auth Expectations

- Base path: `/api`
- Board access is treated as full-control operator context
- Agent access uses bearer API keys (`agent_api_keys`), hashed at rest
- Agent keys must not access other companies

When adding endpoints:

- apply company access checks (`enforce_company_scope`)
- enforce actor permissions (board vs agent, `require_board`)
- write activity log entries for mutations (`crate::audit::log_activity`)
- return consistent HTTP errors (`400/401/403/404/409/422/500`)

## 9. UI Expectations

- Topcoat server-rendered pages (no WASM); routes in `crates/app/src/ui/`
- Keep routes and nav aligned with the available API surface
- Use company selection context for company-scoped pages
- Surface failures clearly; do not silently ignore API errors
- All visual values come from the token layer (`crates/app/src/ui/styles.rs`,
  aligned with `DESIGN.md`); no bare hex/px in components

## 10. Pull Request Requirements

When creating a pull request (via `gh pr create` or any other method), you
**must** read and fill in every section of
[`.github/PULL_REQUEST_TEMPLATE.md`](.github/PULL_REQUEST_TEMPLATE.md). Do not
craft ad-hoc PR bodies — use the template as the structure. Required sections:

- **Thinking Path** — trace reasoning from project context to this change
- **What Changed** — bullet list of concrete changes
- **Verification** — how a reviewer can confirm it works
- **Risks** — what could go wrong
- **Model Used** — the AI model that produced or assisted with the change
  (provider, exact model ID, context window, capabilities). Write
  "None — human-authored" if no AI was used.
- **Checklist** — all items checked

## 11. Definition of Done

A change is done when all are true:

1. Behavior matches the issue acceptance criteria and the parity checklist
2. `cargo fmt --check`, `cargo clippy -- -D warnings`, and relevant tests pass
3. Contracts are synced across data/domain/app/UI layers
4. Docs updated when behavior or commands change
5. PR description follows the [PR template](.github/PULL_REQUEST_TEMPLATE.md)
   with all sections filled in (including Model Used)
