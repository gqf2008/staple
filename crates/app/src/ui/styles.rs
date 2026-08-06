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

.board-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(14rem, 1fr)); gap: var(--space-4); }
.board-column { background: var(--color-muted); border: 1px solid var(--color-border); border-radius: var(--radius-md); padding: var(--space-3); }
.board-column-title { font-size: var(--font-size-sm); margin: 0 0 var(--space-3); }
.board-column .list li { border-bottom: 1px solid var(--color-border); }
.board-column form { margin: var(--space-1) 0 0; }

form.stack-form { display: flex; flex-direction: column; gap: var(--space-2); max-width: 24rem; }
form.stack-form label { font-size: var(--font-size-sm); color: var(--color-muted-foreground); }

.board-card { cursor: grab; }
.board-card.dragging { opacity: 0.5; }
.board-column.drag-over { outline: 2px dashed var(--color-primary); outline-offset: -2px; }
"#;
