-- Company profile fields aligned with the upstream Company JSON contract
-- (doc/SPEC-implementation.md §7.1) plus the issue-prefix uniqueness index
-- (upstream `companies_issue_prefix_idx`).

ALTER TABLE companies ADD COLUMN default_responsible_user_id TEXT;
ALTER TABLE companies ADD COLUMN feedback_data_sharing_enabled INTEGER NOT NULL DEFAULT 0;
ALTER TABLE companies ADD COLUMN feedback_data_sharing_consent_at TEXT;
ALTER TABLE companies ADD COLUMN feedback_data_sharing_consent_by_user_id TEXT;
ALTER TABLE companies ADD COLUMN feedback_data_sharing_terms_version TEXT;
ALTER TABLE companies ADD COLUMN logo_asset_id TEXT;
ALTER TABLE companies ADD COLUMN logo_url TEXT;

CREATE UNIQUE INDEX companies_issue_prefix_idx ON companies (issue_prefix);
