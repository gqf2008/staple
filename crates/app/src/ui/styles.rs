//! Design token layer (aligned with `DESIGN.md`: every visual value lives in
//! the token layer; components only reference `var(--token)`).

/// Token definitions plus the small component class set used by the board UI.
/// No component may contain a bare hex/px value — all values resolve through
/// these custom properties.
pub const TOKENS_CSS: &str = r#"
:root {
  /* color */
  --color-background: #fafaf9;
  --color-foreground: #1c1917;
  --color-card: #ffffff;
  --color-card-foreground: #1c1917;
  --color-primary: #2563eb;
  --color-primary-foreground: #ffffff;
  --color-muted: #f5f5f4;
  --color-muted-foreground: #78716c;
  --color-border: #e7e5e4;
  --color-destructive: #dc2626;
  --color-status-running: #16a34a;
  --color-status-paused: #d97706;
  --color-status-blocked: #dc2626;
  --color-status-done: #16a34a;
  --color-status-todo: #f59e0b;
  --color-status-in-progress: #2563eb;
  --color-status-in-review: #8b5cf6;
  --color-status-cancelled: #78716c;
  --color-priority-critical: #dc2626;
  --color-priority-high: #ea580c;
  --color-priority-medium: #2563eb;
  --color-priority-low: #78716c;

  /* spacing */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;
  --space-6: 1.5rem;
  --space-8: 2rem;
  --space-12: 3rem;

  /* radius */
  --radius-sm: 0.3rem;
  --radius-md: 0.5rem;
  --radius-lg: 0.75rem;

  /* typography */
  --font-sans: ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif;
  --font-mono: ui-monospace, SFMono-Regular, Menlo, monospace;
  --font-size-xs: 0.75rem;
  --font-size-sm: 0.875rem;
  --font-size-md: 1rem;
  --font-size-lg: 1.125rem;
  --font-size-xl: 1.5rem;

  /* shadow */
  --shadow-sm: 0 1px 2px rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px rgb(0 0 0 / 0.07);

  /* motion */
  --motion-duration-fast: 120ms;
  --motion-duration-base: 200ms;
  --motion-ease-base: cubic-bezier(0.4, 0, 0.2, 1);
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

.app-shell { display: flex; gap: var(--space-6); align-items: flex-start; }
.app-sidebar { width: 220px; flex-shrink: 0; border-right: 1px solid var(--color-border); padding-right: var(--space-4); }
.app-sidebar a { display: block; padding: var(--space-1) 0; color: var(--color-muted-foreground); text-decoration: none; }
.app-sidebar a:hover { text-decoration: underline; color: var(--color-primary); }
.app-sidebar a.brand { font-weight: 600; color: var(--color-primary-foreground); margin-bottom: var(--space-3); }
.app-sidebar h3 { font-size: var(--font-size-sm); text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-muted-foreground); margin: var(--space-3) 0 var(--space-1); }
.app-main { flex: 1; min-width: 0; padding: var(--space-6); max-width: 960px; margin: 0 auto; }

.card {
  background: var(--color-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  margin-bottom: var(--space-4);
  box-shadow: var(--shadow-sm);
  transition: box-shadow var(--motion-duration-fast) var(--motion-ease-base);
}

.card:hover { box-shadow: var(--shadow-md); }

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
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
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
.board-column-backlog .board-column-header { color: var(--color-status-cancelled); }
.board-column-backlog .board-column-body { background: color-mix(in srgb, var(--color-status-cancelled) 8%, var(--color-background)); }
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
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 9999px;
  flex: none;
}
.status-dot-backlog, .status-dot-cancelled { background: var(--color-status-cancelled); }
.status-dot-todo { background: var(--color-status-todo); }
.status-dot-in_progress { background: var(--color-status-in-progress); }
.status-dot-in_review { background: var(--color-status-in-review); }
.status-dot-blocked { background: var(--color-status-blocked); }
.status-dot-done { background: var(--color-status-done); }
.board-card {
  display: block;
  background: var(--color-card);
  border: 1px solid var(--color-border);
  border-left-width: 3px;
  border-radius: var(--radius-sm);
  padding: var(--space-2) var(--space-2) var(--space-2) var(--space-3);
  cursor: grab;
  transition: box-shadow 120ms ease;
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
  border-radius: 9999px;
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
.issue-status-pill-backlog, .issue-status-pill-cancelled { background: color-mix(in srgb, var(--color-status-cancelled) 16%, var(--color-background)); color: var(--color-status-cancelled); }
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
"#;
