-- Company skills with opt-in restriction policies (SPEC §9.10).

CREATE TABLE company_skills (
  id                 TEXT PRIMARY KEY,
  company_id         TEXT NOT NULL REFERENCES companies(id),
  name               TEXT NOT NULL,
  description        TEXT,
  restriction_policy TEXT NOT NULL DEFAULT '{}',
  status             TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
  created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, name)
);
