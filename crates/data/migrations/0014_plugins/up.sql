-- Plugin ecosystem (upstream plugins.ts family: plugins, config, state,
-- entities, jobs/runs, logs, webhook deliveries, database namespaces +
-- migration ledger, company settings, managed resources).

-- Instance-level plugin registry.
CREATE TABLE plugins (
  id            TEXT PRIMARY KEY,
  plugin_key    TEXT NOT NULL UNIQUE,
  package_name  TEXT NOT NULL,
  version       TEXT NOT NULL,
  api_version   INTEGER NOT NULL DEFAULT 1,
  categories    TEXT NOT NULL DEFAULT '[]',
  manifest_json TEXT NOT NULL,
  status        TEXT NOT NULL DEFAULT 'installed'
                CHECK (status IN ('installed', 'enabled', 'disabled', 'error', 'uninstalled')),
  install_order INTEGER,
  package_path  TEXT,
  last_error    TEXT,
  installed_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_plugins_status ON plugins (status);

-- Per-company operator configuration (one row per plugin/company).
CREATE TABLE plugin_config (
  id          TEXT PRIMARY KEY,
  plugin_id   TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  company_id  TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  config_json TEXT NOT NULL DEFAULT '{}',
  last_error  TEXT,
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (plugin_id, company_id),
  UNIQUE (company_id, id)
);

-- Scoped key-value storage for plugin workers.
CREATE TABLE plugin_state (
  id         TEXT PRIMARY KEY,
  plugin_id  TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  scope_kind TEXT NOT NULL,
  scope_id   TEXT,
  namespace  TEXT NOT NULL DEFAULT 'default',
  state_key  TEXT NOT NULL,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (plugin_id, scope_kind, scope_id, namespace, state_key)
);

CREATE INDEX idx_plugin_state_plugin_scope ON plugin_state (plugin_id, scope_kind);

-- Structured external-object mappings (company nullable for instance scope).
CREATE TABLE plugin_entities (
  id          TEXT PRIMARY KEY,
  plugin_id   TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  company_id  TEXT REFERENCES companies(id) ON DELETE CASCADE,
  entity_type TEXT NOT NULL,
  scope_kind  TEXT NOT NULL,
  scope_id    TEXT,
  external_id TEXT,
  title       TEXT,
  status      TEXT,
  data        TEXT NOT NULL DEFAULT '{}',
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, plugin_id, entity_type, external_id)
);

CREATE INDEX idx_plugin_entities_plugin ON plugin_entities (plugin_id);
CREATE INDEX idx_plugin_entities_company ON plugin_entities (company_id);
CREATE INDEX idx_plugin_entities_type ON plugin_entities (entity_type);
CREATE INDEX idx_plugin_entities_scope ON plugin_entities (scope_kind, scope_id);

-- Scheduled job definitions (instance-level).
CREATE TABLE plugin_jobs (
  id          TEXT PRIMARY KEY,
  plugin_id   TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  job_key     TEXT NOT NULL,
  schedule    TEXT NOT NULL,
  status      TEXT NOT NULL DEFAULT 'active'
              CHECK (status IN ('active', 'paused', 'error')),
  last_run_at TEXT,
  next_run_at TEXT,
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (plugin_id, job_key)
);

CREATE INDEX idx_plugin_jobs_plugin ON plugin_jobs (plugin_id);
CREATE INDEX idx_plugin_jobs_next_run ON plugin_jobs (next_run_at);

-- Immutable-ish job run history (company nullable).
CREATE TABLE plugin_job_runs (
  id          TEXT PRIMARY KEY,
  job_id      TEXT NOT NULL REFERENCES plugin_jobs(id) ON DELETE CASCADE,
  plugin_id   TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  company_id  TEXT REFERENCES companies(id) ON DELETE CASCADE,
  trigger     TEXT NOT NULL CHECK (trigger IN ('scheduled', 'manual')),
  status      TEXT NOT NULL DEFAULT 'pending'
              CHECK (status IN ('pending', 'running', 'succeeded', 'failed', 'cancelled')),
  duration_ms INTEGER,
  error       TEXT,
  logs        TEXT NOT NULL DEFAULT '[]',
  started_at  TEXT,
  finished_at TEXT,
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_plugin_job_runs_job ON plugin_job_runs (job_id);
CREATE INDEX idx_plugin_job_runs_plugin ON plugin_job_runs (plugin_id);
CREATE INDEX idx_plugin_job_runs_company ON plugin_job_runs (company_id);
CREATE INDEX idx_plugin_job_runs_status ON plugin_job_runs (status);

-- Worker logs (company nullable).
CREATE TABLE plugin_logs (
  id         TEXT PRIMARY KEY,
  plugin_id  TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  company_id TEXT REFERENCES companies(id) ON DELETE CASCADE,
  level      TEXT NOT NULL DEFAULT 'info'
             CHECK (level IN ('debug', 'info', 'warn', 'error')),
  message    TEXT NOT NULL,
  meta       TEXT,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_plugin_logs_plugin_time ON plugin_logs (plugin_id, created_at);
CREATE INDEX idx_plugin_logs_company ON plugin_logs (company_id);
CREATE INDEX idx_plugin_logs_level ON plugin_logs (level);

-- Inbound webhook deliveries (company nullable).
CREATE TABLE plugin_webhook_deliveries (
  id          TEXT PRIMARY KEY,
  plugin_id   TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  company_id  TEXT REFERENCES companies(id) ON DELETE CASCADE,
  webhook_key TEXT NOT NULL,
  external_id TEXT,
  status      TEXT NOT NULL DEFAULT 'pending'
              CHECK (status IN ('pending', 'processing', 'succeeded', 'failed')),
  duration_ms INTEGER,
  error       TEXT,
  payload     TEXT NOT NULL,
  headers     TEXT NOT NULL DEFAULT '{}',
  started_at  TEXT,
  finished_at TEXT,
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_plugin_webhook_deliveries_plugin ON plugin_webhook_deliveries (plugin_id);
CREATE INDEX idx_plugin_webhook_deliveries_company ON plugin_webhook_deliveries (company_id);
CREATE INDEX idx_plugin_webhook_deliveries_status ON plugin_webhook_deliveries (status);
CREATE INDEX idx_plugin_webhook_deliveries_key ON plugin_webhook_deliveries (webhook_key);

-- Per-plugin database namespaces (instance-level).
CREATE TABLE plugin_database_namespaces (
  id             TEXT PRIMARY KEY,
  plugin_id      TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  plugin_key     TEXT NOT NULL,
  namespace_name TEXT NOT NULL,
  namespace_mode TEXT NOT NULL DEFAULT 'schema'
                 CHECK (namespace_mode IN ('schema', 'table')),
  status         TEXT NOT NULL DEFAULT 'active'
                 CHECK (status IN ('active', 'inactive', 'error')),
  created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (plugin_id),
  UNIQUE (namespace_name)
);

CREATE INDEX idx_plugin_database_namespaces_status ON plugin_database_namespaces (status);

-- Per-plugin migration ledger (checksummed).
CREATE TABLE plugin_migrations (
  id             TEXT PRIMARY KEY,
  plugin_id      TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  plugin_key     TEXT NOT NULL,
  namespace_name TEXT NOT NULL,
  migration_key  TEXT NOT NULL,
  checksum       TEXT NOT NULL,
  plugin_version TEXT NOT NULL,
  status         TEXT NOT NULL
                 CHECK (status IN ('applied', 'failed', 'pending')),
  started_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  applied_at     TEXT,
  error_message  TEXT,
  UNIQUE (plugin_id, migration_key)
);

CREATE INDEX idx_plugin_migrations_plugin ON plugin_migrations (plugin_id);
CREATE INDEX idx_plugin_migrations_status ON plugin_migrations (status);

-- Company-level plugin enablement/settings.
CREATE TABLE plugin_company_settings (
  id           TEXT PRIMARY KEY,
  company_id   TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  plugin_id    TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  enabled      INTEGER NOT NULL DEFAULT 1,
  settings_json TEXT NOT NULL DEFAULT '{}',
  last_error   TEXT,
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, plugin_id),
  UNIQUE (company_id, id)
);

CREATE INDEX idx_plugin_company_settings_company ON plugin_company_settings (company_id);
CREATE INDEX idx_plugin_company_settings_plugin ON plugin_company_settings (plugin_id);

-- Plugins' managed core resources.
CREATE TABLE plugin_managed_resources (
  id           TEXT PRIMARY KEY,
  company_id   TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  plugin_id    TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  plugin_key   TEXT NOT NULL,
  resource_kind TEXT NOT NULL,
  resource_key TEXT NOT NULL,
  resource_id  TEXT NOT NULL,
  defaults_json TEXT NOT NULL DEFAULT '{}',
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, plugin_id, resource_kind, resource_key),
  UNIQUE (company_id, id)
);

CREATE INDEX idx_plugin_managed_resources_company ON plugin_managed_resources (company_id);
CREATE INDEX idx_plugin_managed_resources_plugin ON plugin_managed_resources (plugin_id);
CREATE INDEX idx_plugin_managed_resources_resource ON plugin_managed_resources (resource_kind, resource_id);
