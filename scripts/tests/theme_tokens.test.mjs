// Theme token parity tests (issue #242).
//
// Reproducible value-level check that the token values embedded in
// crates/app/src/ui/styles.rs match the upstream reference mirror
// (ui/src/index.css :root + .dark blocks). Snapshots are taken from
// /Volumes/Workspace/GitHub/paperclip at the 2026-08-07 baseline; update them
// when the reference mirror advances.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const STYLES = readFileSync(
  new URL("../../crates/app/src/ui/styles.rs", import.meta.url),
  "utf8",
);

function block(source, name) {
  const start = source.indexOf(`${name} {`);
  assert.notEqual(start, -1, `block ${name} not found`);
  const brace = source.indexOf("{", start);
  const end = source.indexOf("}", brace);
  return source.slice(brace + 1, end);
}

function tokens(source, name) {
  const out = {};
  for (const line of block(source, name).split("\n")) {
    const m = line.match(/^\s*(--[\w-]+)\s*:\s*([^;]+);\s*$/);
    if (m) out[m[1]] = m[2].trim();
  }
  return out;
}

const root = tokens(STYLES, ":root");
const dark = tokens(STYLES, ".dark");

const LIGHT_EXPECTED = {
  "--color-background": "oklch(1 0 0)",
  "--color-foreground": "oklch(0.145 0 0)",
  "--color-card": "oklch(1 0 0)",
  "--color-card-foreground": "oklch(0.145 0 0)",
  "--color-primary": "oklch(0.205 0 0)",
  "--color-primary-foreground": "oklch(0.985 0 0)",
  "--color-muted": "oklch(0.97 0 0)",
  "--color-muted-foreground": "oklch(0.556 0 0)",
  "--color-border": "oklch(0.922 0 0)",
  "--color-destructive": "oklch(0.577 0.245 27.325)",
  "--radius-sm": "0.3rem",
  "--radius-md": "0.4rem",
  "--radius-lg": "0.5rem",
  "--motion-duration-fast": "160ms",
  "--motion-duration-base": "240ms",
  "--motion-duration-slow": "360ms",
  // status-icon light values (upstream --status-task-icon-*, PAP-238)
  "--color-status-icon-todo": "#cc7a00",
  "--color-status-icon-done": "#16a34a",
  "--color-status-icon-in-progress": "#2563eb",
  "--color-status-icon-in-review": "#7c3aed",
  "--color-status-icon-blocked": "#dc2626",
  "--color-status-icon-cancelled": "#52585d",
  "--color-status-icon-backlog": "#52585d",
};

const DARK_EXPECTED = {
  "--color-background": "oklch(0.145 0 0)",
  "--color-foreground": "oklch(0.985 0 0)",
  "--color-card": "oklch(0.205 0 0)",
  "--color-card-foreground": "oklch(0.985 0 0)",
  "--color-popover": "oklch(0.205 0 0)",
  "--color-popover-foreground": "oklch(0.985 0 0)",
  "--color-primary": "oklch(0.922 0 0)",
  "--color-primary-foreground": "oklch(0.205 0 0)",
  "--color-secondary": "oklch(0.269 0 0)",
  "--color-secondary-foreground": "oklch(0.985 0 0)",
  "--color-muted": "oklch(0.269 0 0)",
  "--color-muted-foreground": "oklch(0.708 0 0)",
  "--color-accent": "oklch(0.269 0 0)",
  "--color-accent-foreground": "oklch(0.985 0 0)",
  "--color-destructive": "oklch(0.637 0.237 25.331)",
  "--color-destructive-foreground": "oklch(0.985 0 0)",
  "--color-border": "oklch(1 0 0 / 10%)",
  "--color-input": "oklch(1 0 0 / 15%)",
  "--color-ring": "oklch(0.556 0 0)",
  "--color-sidebar": "oklch(0.205 0 0)",
  "--color-sidebar-foreground": "oklch(0.985 0 0)",
  "--color-sidebar-primary": "oklch(0.488 0.243 264.376)",
  "--color-sidebar-primary-foreground": "oklch(0.985 0 0)",
  "--color-sidebar-accent": "oklch(0.269 0 0)",
  "--color-sidebar-accent-foreground": "oklch(0.985 0 0)",
  "--color-sidebar-border": "oklch(1 0 0 / 10%)",
  "--color-sidebar-ring": "oklch(0.556 0 0)",
  // dark status-icon overrides (upstream .dark, PAP-238)
  "--color-status-icon-todo": "#fbbf24",
  "--color-status-icon-done": "#34d06f",
  "--color-status-icon-in-review": "#9474f0",
  "--color-status-icon-cancelled": "#9a958a",
  "--color-status-icon-backlog": "#9a958a",
  "--color-status-icon-paused": "#fbbf24",
  "--color-status-icon-idle": "#9a958a",
};

test("light theme tokens match upstream :root", () => {
  for (const [name, expected] of Object.entries(LIGHT_EXPECTED)) {
    assert.equal(root[name], expected, `light token ${name}`);
  }
});

test("dark theme tokens match upstream .dark", () => {
  for (const [name, expected] of Object.entries(DARK_EXPECTED)) {
    assert.equal(dark[name], expected, `dark token ${name}`);
  }
});

test("dark block carries color-scheme dark and root carries light", () => {
  assert.ok(block(STYLES, ".dark").includes("color-scheme: dark"));
  assert.ok(block(STYLES, ":root").includes("color-scheme: light"));
});
