-- Heartbeat run fields aligned with the upstream run model: failure
-- attribution (infrastructure vs agent), trigger detail, and log size.

ALTER TABLE heartbeat_runs ADD COLUMN error_kind TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN trigger_detail TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN log_bytes INTEGER NOT NULL DEFAULT 0;
