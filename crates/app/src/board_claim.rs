//! Board ownership claim challenge.
//!
//! Mirrors upstream `server/src/board-claim.ts`: a single in-memory challenge
//! (token + code, 24h TTL) is generated only while the instance has no real
//! admin (only the local board user holds `instance_admin`). Claiming
//! promotes the claiming user to `instance_admin`. This fork has no
//! email/password sessions, so the board actor claims directly.

use serde::Serialize;
use std::sync::Mutex;

/// The board user id used for local/implicit board identity.
pub const LOCAL_BOARD_USER_ID: &str = "local-board";

const CLAIM_TTL_MS: i64 = 24 * 60 * 60 * 1000;

/// An active board-claim challenge.
#[derive(Debug, Clone)]
pub struct ClaimChallenge {
    /// Public token (URL path segment).
    pub token: String,
    /// Secret code (query parameter).
    pub code: String,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 expiry.
    pub expires_at: String,
    /// ISO 8601 claim time.
    pub claimed_at: Option<String>,
    /// User id that claimed the challenge.
    pub claimed_by_user_id: Option<String>,
}

/// Public claim status payload.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimStatus {
    /// `available` | `claimed` | `expired` | `invalid`.
    pub status: String,
    /// Whether claiming requires a signed-in session (always true upstream;
    /// this fork treats the board actor as the session).
    pub requires_sign_in: bool,
    /// ISO 8601 expiry, when the challenge is valid.
    pub expires_at: Option<String>,
    /// Claiming user id, when claimed.
    pub claimed_by_user_id: Option<String>,
}

/// Claim errors.
#[derive(Debug)]
pub enum ClaimError {
    /// Token/code do not match an active challenge.
    Invalid,
    /// The challenge expired.
    Expired,
    /// The challenge was already claimed.
    Claimed,
}

/// In-memory board claim manager.
#[derive(Default)]
pub struct BoardClaimManager {
    challenge: Mutex<Option<ClaimChallenge>>,
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

impl BoardClaimManager {
    /// Creates an empty manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// (Re)initializes the challenge per the upstream rule: only when the
    /// instance has exactly one `instance_admin` and it is the local board
    /// user. Pass `false` to clear any challenge.
    pub fn initialize(&self, only_local_board_admin: bool) {
        let mut guard = self.challenge.lock().expect("challenge lock");
        if !only_local_board_admin {
            *guard = None;
            return;
        }
        let now = chrono::Utc::now();
        let expired = guard.as_ref().is_none_or(|challenge| {
            challenge.claimed_at.is_some()
                || challenge.expires_at.as_str()
                    <= now
                        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
                        .as_str()
        });
        if expired {
            let token = format!("board-claim-{}", uuid::Uuid::new_v4());
            let code = uuid::Uuid::new_v4().to_string().replace('-', "");
            let created_at = now_iso();
            let expires_at = (now + chrono::Duration::milliseconds(CLAIM_TTL_MS))
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            *guard = Some(ClaimChallenge {
                token,
                code,
                created_at,
                expires_at,
                claimed_at: None,
                claimed_by_user_id: None,
            });
        }
    }

    /// Inspects the challenge for `token`/`code` without mutating state.
    #[must_use]
    pub fn inspect(&self, token: &str, code: Option<&str>) -> ClaimStatus {
        let guard = self.challenge.lock().expect("challenge lock");
        let Some(challenge) = guard.as_ref() else {
            return ClaimStatus {
                status: "invalid".to_owned(),
                requires_sign_in: true,
                expires_at: None,
                claimed_by_user_id: None,
            };
        };
        if challenge.token != token || challenge.code != code.unwrap_or_default() {
            return ClaimStatus {
                status: "invalid".to_owned(),
                requires_sign_in: true,
                expires_at: None,
                claimed_by_user_id: None,
            };
        }
        if challenge.claimed_at.is_some() {
            return ClaimStatus {
                status: "claimed".to_owned(),
                requires_sign_in: true,
                expires_at: Some(challenge.expires_at.clone()),
                claimed_by_user_id: challenge.claimed_by_user_id.clone(),
            };
        }
        if challenge.expires_at.as_str() <= now_iso().as_str() {
            return ClaimStatus {
                status: "expired".to_owned(),
                requires_sign_in: true,
                expires_at: Some(challenge.expires_at.clone()),
                claimed_by_user_id: None,
            };
        }
        ClaimStatus {
            status: "available".to_owned(),
            requires_sign_in: true,
            expires_at: Some(challenge.expires_at.clone()),
            claimed_by_user_id: None,
        }
    }

    /// Claims the challenge for `user_id`.
    ///
    /// # Errors
    ///
    /// Returns [`ClaimError`] when invalid, expired, or already claimed.
    pub fn claim(&self, token: &str, code: &str, user_id: &str) -> Result<ClaimStatus, ClaimError> {
        let mut guard = self.challenge.lock().expect("challenge lock");
        let Some(challenge) = guard.as_mut() else {
            return Err(ClaimError::Invalid);
        };
        if challenge.token != token || challenge.code != code {
            return Err(ClaimError::Invalid);
        }
        if let Some(claimed_by) = &challenge.claimed_by_user_id {
            return Ok(ClaimStatus {
                status: "claimed".to_owned(),
                requires_sign_in: true,
                expires_at: Some(challenge.expires_at.clone()),
                claimed_by_user_id: Some(claimed_by.clone()),
            });
        }
        if challenge.expires_at.as_str() <= now_iso().as_str() {
            return Err(ClaimError::Expired);
        }
        challenge.claimed_at = Some(now_iso());
        challenge.claimed_by_user_id = Some(user_id.to_owned());
        Ok(ClaimStatus {
            status: "claimed".to_owned(),
            requires_sign_in: true,
            expires_at: Some(challenge.expires_at.clone()),
            claimed_by_user_id: Some(user_id.to_owned()),
        })
    }

    /// Returns the active challenge (used to seed tests).
    #[must_use]
    pub fn active(&self) -> Option<ClaimChallenge> {
        self.challenge.lock().expect("challenge lock").clone()
    }

    /// Seeds a challenge directly (test helper).
    pub fn seed(&self, token: &str, code: &str) {
        let now = chrono::Utc::now();
        let expires_at = (now + chrono::Duration::milliseconds(CLAIM_TTL_MS))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        *self.challenge.lock().expect("challenge lock") = Some(ClaimChallenge {
            token: token.to_owned(),
            code: code.to_owned(),
            created_at: now_iso(),
            expires_at,
            claimed_at: None,
            claimed_by_user_id: None,
        });
    }
}
