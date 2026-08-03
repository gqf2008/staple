DROP INDEX IF EXISTS companies_issue_prefix_idx;

ALTER TABLE companies DROP COLUMN logo_url;
ALTER TABLE companies DROP COLUMN logo_asset_id;
ALTER TABLE companies DROP COLUMN feedback_data_sharing_terms_version;
ALTER TABLE companies DROP COLUMN feedback_data_sharing_consent_by_user_id;
ALTER TABLE companies DROP COLUMN feedback_data_sharing_consent_at;
ALTER TABLE companies DROP COLUMN feedback_data_sharing_enabled;
ALTER TABLE companies DROP COLUMN default_responsible_user_id;
