# paperclip-rs

**paperclip-rs** is an independent, from-scratch rewrite of the Paperclip
control plane for AI-agent companies, built with **Rust + Topcoat + Turso**.

> This repository is **not a GitHub fork**. It is not linked to
> [paperclipai/paperclip](https://github.com/paperclipai/paperclip) and does not
> track it. The upstream codebase appears here only as an initial reference
> snapshot; the goal is to replace it entirely with a Rust implementation.

## Status

- **Phase 0 — done**: independent repo, upstream reference mirror, rewrite plan.
- **Phase 1 — next**: Rust workspace + Turso data layer.

See [doc/plans/2026-08-03-topcoat-turso-rewrite.md](doc/plans/2026-08-03-topcoat-turso-rewrite.md)
for the full roadmap.

## What we're building

- Companies, org structure, goals, projects, and hierarchical tasks (issues)
- Heartbeat execution control plane: atomic checkout, execution locks, budgets
  with hard-stop auto-pause, approval gates, activity audit
- Agent adapters: local CLI sessions, HTTP/webhook runtimes, external plugins
- Governance: secrets, audit log, decision desk, skills policy
- Board UI rendered by Topcoat (server-first, no WASM)

## Layout

The current tree is the upstream Node.js snapshot (`server/`, `ui/`,
`packages/`) kept for functional reference. Rust code lands in `crates/` as the
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
