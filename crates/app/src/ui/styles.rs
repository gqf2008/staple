//! Design token layer (aligned with `DESIGN.md`: every visual value lives in
//! the token layer; components only reference `var(--token)`).

/// Token definitions plus the small component class set used by the board UI.
/// No component may contain a bare hex/px value — all values resolve through
/// these custom properties.
pub const TOKENS_CSS: &str = r#"
@font-face {
  font-family: "InterVariable";
  src: url("/static/fonts/InterVariable.woff2") format("woff2");
  font-display: swap;
  font-style: normal;
  font-weight: 100 900;
}

@font-face {
  font-family: "InterVariable";
  src: url("/static/fonts/InterVariable-Italic.woff2") format("woff2");
  font-display: swap;
  font-style: italic;
  font-weight: 100 900;
}

:root {
  /* color — aligned with upstream ui/src/index.css (light theme, OKLCH) */
  --color-background: oklch(1 0 0);
  --color-foreground: oklch(0.145 0 0);
  --color-card: oklch(1 0 0);
  --color-card-foreground: oklch(0.145 0 0);
  --color-popover: oklch(1 0 0);
  --color-popover-foreground: oklch(0.145 0 0);
  --color-primary: oklch(0.205 0 0);
  --color-primary-foreground: oklch(0.985 0 0);
  --color-secondary: oklch(0.97 0 0);
  --color-secondary-foreground: oklch(0.205 0 0);
  --color-muted: oklch(0.97 0 0);
  --color-muted-foreground: oklch(0.556 0 0);
  --color-accent: oklch(0.97 0 0);
  --color-accent-foreground: oklch(0.205 0 0);
  --color-destructive: oklch(0.577 0.245 27.325);
  --color-destructive-foreground: oklch(0.577 0.245 27.325);
  --color-border: oklch(0.922 0 0);
  --color-input: oklch(0.922 0 0);
  --color-ring: oklch(0.708 0 0);
  --color-sidebar: oklch(0.985 0 0);
  --color-sidebar-foreground: oklch(0.145 0 0);
  --color-sidebar-primary: oklch(0.205 0 0);
  --color-sidebar-primary-foreground: oklch(0.985 0 0);
  --color-sidebar-accent: oklch(0.97 0 0);
  --color-sidebar-accent-foreground: oklch(0.205 0 0);
  --color-sidebar-border: oklch(0.922 0 0);
  --color-sidebar-ring: oklch(0.708 0 0);

  /* status colors — aligned with upstream --status-task-* / --status-agent-* */
  --color-status-running: #2563eb;
  --color-status-paused: #f59e0b;
  --color-status-blocked: #dc2626;
  --color-status-done: #22c55e;
  --color-status-todo: #f59e0b;
  --color-status-in-progress: #2563eb;
  --color-status-in-review: #7c3aed;
  --color-status-cancelled: #a8aeb2;
  --color-status-backlog: #a8aeb2;
  --color-status-idle: #a8aeb2;
  --color-status-error: #dc2626;

  /* priority colors — Staple extension (upstream has no priority token set;
     kept for board/issue priority chips, documented in ui-pixel-baseline) */
  --color-priority-critical: #dc2626;
  --color-priority-high: #ea580c;
  --color-priority-medium: #2563eb;
  --color-priority-low: #78716c;

  /* spacing */
  --space-0-5: 0.125rem;
  --space-1: 0.25rem;
  --space-1-5: 0.375rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;
  --space-6: 1.5rem;
  --space-8: 2rem;
  --space-10: 2.5rem;
  --space-12: 3rem;
  --border-width: 0.125rem;

  /* radius — upstream ladder: --radius 0.5rem anchor, sm/md/lg/xl/2xl/3xl/4xl
     = 0.6/0.8/1.0/1.4/1.8/2.2/2.6 x anchor */
  --radius-sm: 0.3rem;
  --radius-md: 0.4rem;
  --radius-lg: 0.5rem;
  --radius-xl: 0.7rem;
  --radius-2xl: 0.9rem;
  --radius-3xl: 1.1rem;
  --radius-4xl: 1.3rem;
  --radius-full: 9999px;

  /* typography — upstream stack, InterVariable self-hosted (see @font-face) */
  --font-sans: "InterVariable", "Inter", ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  --font-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  --font-size-xs: 0.75rem;
  --font-size-sm: 0.875rem;
  --font-size-md: 1rem;
  --font-size-lg: 1.125rem;
  --font-size-xl: 1.5rem;

  /* shadow */
  --shadow-sm: 0 1px 2px rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px rgb(0 0 0 / 0.07);
  --shadow-lg: 0 12px 32px rgba(0, 0, 0, 0.18);

  /* motion — upstream durations (fast/base/slow/deliberate); pulse is a
     Staple extension for chat/thinking dots */
  --motion-duration-instant: 80ms;
  --motion-duration-fast: 160ms;
  --motion-duration-base: 240ms;
  --motion-duration-slow: 360ms;
  --motion-duration-deliberate: 520ms;
  --motion-duration-pulse: 1.2s;
  --motion-ease-base: cubic-bezier(0.4, 0, 0.2, 1);

  --toast-max-width: 28rem;
}

@media (prefers-reduced-motion: reduce) {
  :root {
    --motion-duration-instant: 0ms;
    --motion-duration-fast: 0ms;
    --motion-duration-base: 0ms;
    --motion-duration-slow: 0ms;
    --motion-duration-deliberate: 0ms;
    --motion-duration-pulse: 0ms;
  }
}
* { box-sizing: border-box; }

body {
  margin: 0;
  font-family: var(--font-sans);
  font-size: var(--font-size-md);
  color: var(--color-foreground);
  background: var(--color-background);
}

.app-nav {
  display: flex;
  gap: var(--space-4);
  align-items: center;
  padding: var(--space-4) var(--space-6);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-card);
}

.app-nav a {
  color: var(--color-primary);
  text-decoration: none;
  font-weight: 600;
}

.app-nav a:hover { text-decoration: underline; }

.app-shell { display: flex; align-items: flex-start; min-height: 100vh; }
.app-sidebar {
  width: 240px;
  flex-shrink: 0;
  background: var(--color-sidebar);
  border-right: 1px solid var(--color-border);
  padding: var(--space-3) 0;
  transition: width var(--motion-duration-fast) var(--motion-ease-base);
}
.app-sidebar.collapsed { width: 64px; overflow: hidden; }
.app-sidebar a {
  display: block;
  padding: var(--space-1) var(--space-3);
  color: var(--color-muted-foreground);
  text-decoration: none;
  white-space: nowrap;
  overflow: hidden;
}
.app-sidebar a:hover { text-decoration: underline; color: var(--color-primary); }
.app-sidebar a.brand { font-weight: 600; color: var(--color-foreground); margin-bottom: var(--space-3); }
.app-sidebar h3 {
  font-size: var(--font-size-sm);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-muted-foreground);
  margin: var(--space-3) 0 var(--space-1);
  padding: 0 var(--space-3);
  white-space: nowrap;
  overflow: hidden;
}
.sidebar-toggle {
  width: calc(100% - var(--space-6));
  margin: 0 var(--space-3) var(--space-2);
  height: var(--space-8);
  font-size: var(--font-size-sm);
}
.app-main { flex: 1; min-width: 0; padding: var(--space-4); width: 100%; }
@media (min-width: 48rem) {
  .app-main { padding: var(--space-6); }
}

.card {
  background: var(--color-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  margin-bottom: var(--space-4);
}

.card h3 { margin: 0 0 var(--space-2); font-size: var(--font-size-lg); }

.card p { margin: var(--space-1) 0; color: var(--color-muted-foreground); }

.badge {
  display: inline-block;
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-xs);
  font-weight: 600;
}

.badge-running { background: var(--color-status-running); color: var(--color-primary-foreground); }
.badge-paused { background: var(--color-status-paused); color: var(--color-primary-foreground); }
.badge-blocked { background: var(--color-status-blocked); color: var(--color-primary-foreground); }
.badge-done { background: var(--color-status-done); color: var(--color-primary-foreground); }
.badge-default { background: var(--color-muted); color: var(--color-muted-foreground); }

.mono { font-family: var(--font-mono); font-size: var(--font-size-xs); }

.page-title { font-size: var(--font-size-xl); margin: 0 0 var(--space-6); }

.list { list-style: none; padding: 0; margin: 0; }

.list li { padding: var(--space-2) 0; border-bottom: 1px solid var(--color-border); }

.empty { color: var(--color-muted-foreground); padding: var(--space-4); }

form.inline-form { display: flex; gap: var(--space-2); align-items: center; margin: var(--space-2) 0; }

input[type="text"], textarea, select {
  font: inherit;
  padding: var(--space-2);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-card);
  color: var(--color-foreground);
}

button {
  font: inherit;
  font-size: var(--font-size-sm);
  height: var(--space-10);
  padding: 0 var(--space-4);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-primary);
  color: var(--color-primary-foreground);
  cursor: pointer;
  transition: opacity var(--motion-duration-fast) var(--motion-ease-base);
}

button:hover { opacity: 0.9; }

button.secondary { background: var(--color-card); color: var(--color-foreground); }
button.destructive { background: var(--color-destructive); color: var(--color-primary-foreground); }

.meta-row { color: var(--color-muted-foreground); font-size: var(--font-size-sm); }

section { margin-bottom: var(--space-8); }

.nav-row { display: flex; gap: var(--space-3); flex-wrap: wrap; margin: var(--space-4) 0; }
.nav-row a { color: var(--color-primary); text-decoration: none; font-size: var(--font-size-sm); }
.nav-row a:hover { text-decoration: underline; }
.muted-link { color: var(--color-muted-foreground); text-decoration: none; font-size: var(--font-size-sm); }

form.stack-form { display: flex; flex-direction: column; gap: var(--space-2); max-width: 24rem; }
form.stack-form label { font-size: var(--font-size-sm); color: var(--color-muted-foreground); }

/* Board (upstream KanbanBoard parity): horizontal scroll, status-hued
   columns, card with priority rail + assignee. All values from tokens. */
.board-grid {
  display: flex;
  gap: var(--space-3);
  overflow-x: auto;
  padding-bottom: var(--space-4);
  align-items: flex-start;
}
.board-column {
  flex: 0 0 16rem;
  min-width: 16rem;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
}
.board-column-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  font-size: var(--font-size-xs);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.board-column-count {
  margin-left: auto;
  font-size: var(--font-size-xs);
  font-variant-numeric: tabular-nums;
  opacity: 0.75;
}
.board-column-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  padding: var(--space-2);
  min-height: 6rem;
  border-top: 1px solid var(--color-border);
}
.board-column.drag-over { outline: 2px solid var(--color-primary); outline-offset: -2px; }
.board-column-backlog .board-column-header { color: var(--color-status-backlog); }
.board-column-backlog .board-column-body { background: color-mix(in srgb, var(--color-status-backlog) 8%, var(--color-background)); }
.board-column-todo .board-column-header { color: var(--color-status-todo); }
.board-column-todo .board-column-body { background: color-mix(in srgb, var(--color-status-todo) 10%, var(--color-background)); }
.board-column-in_progress .board-column-header { color: var(--color-status-in-progress); }
.board-column-in_progress .board-column-body { background: color-mix(in srgb, var(--color-status-in-progress) 10%, var(--color-background)); }
.board-column-in_review .board-column-header { color: var(--color-status-in-review); }
.board-column-in_review .board-column-body { background: color-mix(in srgb, var(--color-status-in-review) 10%, var(--color-background)); }
.board-column-blocked .board-column-header { color: var(--color-status-blocked); }
.board-column-blocked .board-column-body { background: color-mix(in srgb, var(--color-status-blocked) 10%, var(--color-background)); }
.board-column-done .board-column-header { color: var(--color-status-done); }
.board-column-done .board-column-body { background: color-mix(in srgb, var(--color-status-done) 8%, var(--color-background)); }
.board-column-cancelled .board-column-header,
.board-column-cancelled .board-column-body { opacity: 0.7; }
.status-dot {
  display: inline-block;
  width: var(--space-2);
  height: var(--space-2);
  border-radius: var(--radius-full);
  flex: none;
  background: var(--color-muted-foreground);
}
.status-dot-backlog { background: var(--color-status-backlog); }
.status-dot-cancelled { background: var(--color-status-cancelled); }
.status-dot-todo { background: var(--color-status-todo); }
.status-dot-in_progress { background: var(--color-status-in-progress); }
.status-dot-in_review { background: var(--color-status-in-review); }
.status-dot-blocked { background: var(--color-status-blocked); }
.status-dot-done { background: var(--color-status-done); }
.status-dot-pending { background: var(--color-status-todo); }
.status-dot-failed, .status-dot-timed_out { background: var(--color-status-blocked); }
.status-dot-over_budget { background: var(--color-priority-critical); }
.status-dot-open { background: var(--color-status-in-progress); }
.status-dot-approved, .status-dot-decided, .status-dot-active,
.status-dot-achieved, .status-dot-completed { background: var(--color-status-done); }
.status-dot-rejected, .status-dot-expired, .status-dot-terminated { background: var(--color-status-cancelled); }
.status-dot-revision_requested { background: var(--color-status-in-review); }
.status-dot-paused { background: var(--color-status-paused); }
.status-dot-planned, .status-dot-todo { background: var(--color-status-todo); }
.status-dot-error { background: var(--color-status-blocked); }
.status-dot-idle { background: var(--color-muted-foreground); }
.board-card {
  display: block;
  background: var(--color-card);
  border: 1px solid var(--color-border);
  border-left-width: 3px;
  border-radius: var(--radius-sm);
  padding: var(--space-2) var(--space-2) var(--space-2) var(--space-3);
  cursor: grab;
  transition: box-shadow var(--motion-duration-fast) var(--motion-ease-base);
}
.board-card:hover { box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08); }
.board-card.dragging { opacity: 0.35; }
.board-card-priority-critical { border-left-color: var(--color-priority-critical); }
.board-card-priority-high { border-left-color: var(--color-priority-high); }
.board-card-priority-medium { border-left-color: var(--color-priority-medium); }
.board-card-priority-low { border-left-color: var(--color-priority-low); }
.board-card-id {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--color-muted-foreground);
}
.board-card-title {
  margin: var(--space-1) 0 var(--space-2);
  font-size: var(--font-size-sm);
  line-height: 1.35;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.board-card-footer {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--font-size-xs);
}
.board-priority-label { font-weight: 600; font-size: var(--font-size-xs); }
.board-priority-critical { color: var(--color-priority-critical); }
.board-priority-high { color: var(--color-priority-high); }
.board-priority-medium { color: var(--color-priority-medium); }
.board-priority-low { color: var(--color-priority-low); }
.board-assignee {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  color: var(--color-muted-foreground);
}
.board-assignee-dot {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.25rem;
  height: 1.25rem;
  border-radius: var(--radius-full);
  background: var(--color-primary);
  color: var(--color-primary-foreground);
  font-size: var(--font-size-xs);
  font-weight: 600;
}
.board-column form { margin: var(--space-1) 0 0; }

/* Issue detail (upstream IssueDetail parity, B3): header pill + two-column
   layout + timeline comments + work product/attachment chips. */
.issue-header {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-4);
  background: var(--color-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  margin-bottom: var(--space-4);
}
.issue-header-top {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-wrap: wrap;
}
.issue-title {
  font-size: var(--font-size-xl);
  line-height: 1.3;
  margin: 0;
}
.issue-meta {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  color: var(--color-muted-foreground);
  font-size: var(--font-size-sm);
}
.issue-status-pill {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: 0.125rem 0.5rem;
  border-radius: 9999px;
  font-size: var(--font-size-xs);
  font-weight: 600;
  text-transform: capitalize;
}
.issue-status-pill-backlog { background: color-mix(in srgb, var(--color-status-backlog) 16%, var(--color-background)); color: var(--color-status-backlog); }
.issue-status-pill-cancelled { background: color-mix(in srgb, var(--color-status-cancelled) 16%, var(--color-background)); color: var(--color-status-cancelled); }
.issue-status-pill-todo { background: color-mix(in srgb, var(--color-status-todo) 16%, var(--color-background)); color: var(--color-status-todo); }
.issue-status-pill-in_progress { background: color-mix(in srgb, var(--color-status-in-progress) 16%, var(--color-background)); color: var(--color-status-in-progress); }
.issue-status-pill-in_review { background: color-mix(in srgb, var(--color-status-in-review) 16%, var(--color-background)); color: var(--color-status-in-review); }
.issue-status-pill-blocked { background: color-mix(in srgb, var(--color-status-blocked) 16%, var(--color-background)); color: var(--color-status-blocked); }
.issue-status-pill-done { background: color-mix(in srgb, var(--color-status-done) 16%, var(--color-background)); color: var(--color-status-done); }
.issue-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 20rem;
  gap: var(--space-4);
  align-items: start;
}
@media (max-width: 60rem) { .issue-layout { grid-template-columns: 1fr; } }
.issue-sidebar { display: flex; flex-direction: column; gap: var(--space-4); }
.issue-section {
  background: var(--color-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-4);
}
.issue-section h2 { margin: 0 0 var(--space-3); font-size: var(--font-size-sm); }
.issue-description { white-space: pre-wrap; font-size: var(--font-size-sm); line-height: 1.6; }
.comment-card {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--space-3);
  margin-bottom: var(--space-2);
}
.comment-card .comment-meta {
  font-size: var(--font-size-xs);
  color: var(--color-muted-foreground);
  margin-bottom: var(--space-1);
}
.comment-card .comment-body { font-size: var(--font-size-sm); white-space: pre-wrap; margin: 0; }
.chip {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: 0.125rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 9999px;
  font-size: var(--font-size-xs);
  color: var(--color-muted-foreground);
  background: var(--color-background);
}
.chip-row { display: flex; flex-wrap: wrap; gap: var(--space-2); }
.interaction-card {
  border: 1px solid var(--color-border);
  border-left-width: 3px;
  border-left-color: var(--color-status-in-progress);
  border-radius: var(--radius-sm);
  padding: var(--space-3);
  margin-bottom: var(--space-2);
}
.interaction-card .interaction-kind {
  font-size: var(--font-size-xs);
  font-weight: 600;
  text-transform: capitalize;
  color: var(--color-muted-foreground);
}
.interaction-card .interaction-payload { font-size: var(--font-size-sm); white-space: pre-wrap; margin: var(--space-1) 0 0; }

/* List rows (B4: inbox / my-issues / what-needs-me card rows). */
.row-card {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  background: var(--color-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-3);
  margin-bottom: var(--space-2);
}
.row-card-main { flex: 1 1 auto; min-width: 0; }
.row-card-title { font-weight: 600; font-size: var(--font-size-sm); }
.row-card-meta {
  font-size: var(--font-size-xs);
  color: var(--color-muted-foreground);
  margin-top: var(--space-1);
}
.row-card-actions { display: flex; gap: var(--space-2); flex: none; align-items: center; }

/* Approval / decision cards (B5). */
.card-excerpt {
  font-size: var(--font-size-xs);
  color: var(--color-muted-foreground);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

/* Board chat (B6): bubbles, streaming cursor, thinking, tool accordions. */
.chat-log {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  max-height: 60vh;
  overflow-y: auto;
  padding: var(--space-3);
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  margin-bottom: var(--space-4);
}
.chat-bubble {
  display: flex;
  flex-direction: column;
  max-width: 80%;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}
.chat-bubble-user {
  align-self: flex-end;
  background: color-mix(in srgb, var(--color-primary) 14%, var(--color-background));
  border: 1px solid color-mix(in srgb, var(--color-primary) 35%, var(--color-border));
}
.chat-bubble-agent {
  align-self: flex-start;
  background: var(--color-card);
  border: 1px solid var(--color-border);
}
.chat-bubble-header {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-muted-foreground);
  margin-bottom: var(--space-1);
}
.chat-cursor {
  display: inline-block;
  width: var(--space-2);
  height: 1rem;
  margin-left: var(--space-0-5);
  vertical-align: text-bottom;
  background: var(--color-primary);
  animation: chat-blink var(--motion-duration-slow) step-end infinite;
}
@keyframes chat-blink { 50% { opacity: 0; } }
.chat-thinking {
  display: inline-flex;
  gap: var(--space-1);
  padding: var(--space-2) 0;
}
.chat-thinking span {
  width: var(--space-1-5);
  height: var(--space-1-5);
  border-radius: var(--radius-full);
  background: var(--color-muted-foreground);
  animation: chat-pulse var(--motion-duration-pulse) ease-in-out infinite;
}
.chat-thinking span:nth-child(2) { animation-delay: 0.2s; }
.chat-thinking span:nth-child(3) { animation-delay: 0.4s; }
@keyframes chat-pulse { 0%, 80%, 100% { opacity: 0.25; } 40% { opacity: 1; } }
@media (prefers-reduced-motion: reduce) {
  .chat-cursor, .chat-thinking span { animation: none; }
  .chat-cursor { opacity: 1; }
}
.chat-tool {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  margin-top: var(--space-1);
  background: var(--color-background);
}
.chat-tool-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-1) var(--space-2);
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-muted-foreground);
  cursor: pointer;
  user-select: none;
}
.chat-tool-body {
  display: none;
  padding: var(--space-1) var(--space-2);
  font-size: var(--font-size-xs);
  font-family: var(--font-mono);
  white-space: pre-wrap;
  border-top: 1px solid var(--color-border);
}
.chat-tool.open .chat-tool-body { display: block; }
.chat-tool-stderr {
  background: color-mix(in srgb, var(--color-status-blocked) 8%, var(--color-background));
  color: var(--color-status-blocked);
}

/* Command palette (Cmd/Ctrl+K, upstream CommandPalette parity). */
.command-palette {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 12vh;
  background: color-mix(in srgb, var(--color-foreground) 35%, transparent);
}
.command-palette[hidden] { display: none; }
.command-palette-panel {
  width: min(32rem, calc(100vw - var(--space-8)));
  background: var(--color-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}
.command-palette-input {
  width: 100%;
  box-sizing: border-box;
  border: none;
  border-bottom: 1px solid var(--color-border);
  padding: var(--space-3);
  font-size: var(--font-size-md);
  background: transparent;
  color: var(--color-foreground);
  outline: none;
}
.command-palette-list {
  max-height: 48vh;
  overflow-y: auto;
  padding: var(--space-1);
}
.command-item {
  display: flex;
  gap: var(--space-2);
  align-items: center;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-sm);
  color: var(--color-foreground);
  text-decoration: none;
  font-size: var(--font-size-sm);
  cursor: pointer;
}
.command-item:hover, .command-item.active { background: var(--color-muted); }
.command-item[hidden] { display: none; }
.command-item .command-id { font-family: var(--font-mono); font-size: var(--font-size-xs); color: var(--color-muted-foreground); }
.command-empty { padding: var(--space-3); font-size: var(--font-size-sm); color: var(--color-muted-foreground); }

/* UI feedback (issue #231): mutating form loading state + flash toast.
   All values come from the token layer; no bare hex/px. */
button[disabled] { opacity: 0.6; cursor: default; }
.btn-loading { opacity: 0.85; }
.btn-loading .spinner { margin-left: var(--space-1); }
.spinner {
  display: inline-block;
  width: var(--space-3);
  height: var(--space-3);
  border: var(--border-width) solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: var(--radius-full);
  animation: spinner-rotate var(--motion-duration-slow) linear infinite;
  vertical-align: text-bottom;
}
@keyframes spinner-rotate { to { transform: rotate(360deg); } }
.toast {
  position: fixed;
  top: var(--space-4);
  left: 50%;
  transform: translateX(-50%);
  z-index: 2000;
  display: flex;
  gap: var(--space-2);
  align-items: center;
  max-width: min(var(--toast-max-width), calc(100vw - var(--space-8)));
  padding: var(--space-2) var(--space-4);
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-card);
  box-shadow: var(--shadow-lg);
  font-size: var(--font-size-sm);
  transition: opacity var(--motion-duration-base) var(--motion-ease-base);
}
.toast[hidden] { display: none; }
.toast.hide { opacity: 0; }
.toast-success { border-color: var(--color-status-done); color: var(--color-status-done); }
.toast-error { border-color: var(--color-destructive); color: var(--color-destructive); }
@media (prefers-reduced-motion: reduce) {
  .spinner { animation: none; }
  .toast { transition: none; }
}
"#;
