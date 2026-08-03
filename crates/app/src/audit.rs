//! Audit helper: every mutating route writes an activity entry.

use std::sync::Arc;

use serde_json::Value;
use staple_data::{ActivityRepository, NewActivity};

use crate::error::ApiError;

/// Writes an activity-log entry as the board user (authentication lands in
/// #28; until then all mutations are attributed to the board actor).
///
/// # Errors
///
/// Returns [`ApiError`] when the audit write fails — auditing is a hard
/// invariant, so a failed write fails the mutation.
pub async fn log_activity(
    activity: &Arc<dyn ActivityRepository>,
    company_id: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    details: Option<Value>,
) -> Result<(), ApiError> {
    activity
        .log(NewActivity {
            company_id: company_id.to_owned(),
            actor_type: "user".to_owned(),
            actor_id: "board".to_owned(),
            action: action.to_owned(),
            entity_type: entity_type.to_owned(),
            entity_id: entity_id.to_owned(),
            details: details.map(|value| value.to_string()),
        })
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(())
}
