-- Attention inbox dismissals/snoozes (upstream inbox_dismissals.ts parity,
-- issue #204 A2). Rows are scoped by company + user and keyed by the
-- attention item key; `kind` marks a permanent dismiss or a snooze with a
-- future `snoozed_until`.
--
-- NOTE: migration 0026 already creates an `inbox_dismissals` table for the
-- infrastructure domain with the same columns/unique key, so `IF NOT EXISTS`
-- keeps this migration a no-op on databases migrated through 0026 while
-- still defining the canonical schema if that table is retired later.

CREATE TABLE IF NOT EXISTS inbox_dismissals (
  id            TEXT PRIMARY KEY,
  company_id    TEXT NOT NULL REFERENCES companies(id),
  user_id       TEXT NOT NULL,
  item_key      TEXT NOT NULL,
  kind          TEXT NOT NULL CHECK (kind IN ('dismiss', 'snooze')),
  dismissed_at  TEXT NOT NULL,
  snoozed_until TEXT,
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  UNIQUE (company_id, user_id, item_key)
);

CREATE INDEX IF NOT EXISTS idx_inbox_dismissals_company_user
  ON inbox_dismissals (company_id, user_id);
