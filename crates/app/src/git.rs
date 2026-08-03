//! Server-side git materialization with company-secret credential injection
//! (managed checkout). Credentials are injected via `http.extraheader` so the
//! token never appears in the process command line, and all captured error
//! text is redacted before it reaches logs or responses.

use std::path::PathBuf;
use std::process::Stdio;

use tokio::process::Command;

/// Builds the `http.extraheader` value for GitHub-style git hosts.
/// Format: `AUTHORIZATION: basic <base64("x-access-token:<token>")>`.
#[must_use]
pub fn basic_auth_header(token: &str) -> String {
    use base64::Engine;
    let raw = format!("x-access-token:{token}");
    let encoded = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes());
    format!("AUTHORIZATION: basic {encoded}")
}

/// Replaces the raw token and its base64-encoded form with `***` so error
/// text and logs never leak credentials.
#[must_use]
pub fn redact_credentials(text: &str, token: &str) -> String {
    use base64::Engine;
    let encoded =
        base64::engine::general_purpose::STANDARD.encode(format!("x-access-token:{token}"));
    let mut out = text.replace(token, "***");
    out = out.replace(&encoded, "***");
    out
}

/// Runs a git command with the credential extraheader; returns stdout+stderr
/// (redacted) or an error message (redacted).
async fn run_git(args: &[&str], token: &str, cwd: Option<&PathBuf>) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.args([
        "-c",
        &format!("http.extraheader={}", basic_auth_header(token)),
    ]);
    cmd.args(args);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = cmd
        .output()
        .await
        .map_err(|error| redact_credentials(&format!("failed to start git: {error}"), token))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        Ok(redact_credentials(&format!("{stdout}{stderr}"), token))
    } else {
        Err(redact_credentials(&format!("{stderr}{stdout}"), token))
    }
}

/// The checkout root under which managed checkouts are materialized.
/// Override with `STAPLE_CHECKOUT_ROOT` (used by tests to avoid writing into
/// the repository tree).
#[must_use]
pub fn checkout_root() -> PathBuf {
    std::env::var("STAPLE_CHECKOUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data/checkouts"))
}

/// The local destination for a workspace's materialized checkout.
#[must_use]
pub fn checkout_path(company_id: &str, workspace_id: &str) -> PathBuf {
    checkout_root().join(company_id).join(workspace_id)
}

/// Clones (first time) or fetches (already materialized) the workspace repo
/// into `data/checkouts/{company}/{workspace}` using `token` for auth.
///
/// Returns the destination path and the (redacted) git output.
pub async fn materialize_repo(
    repo_url: &str,
    token: &str,
    company_id: &str,
    workspace_id: &str,
    refresh: bool,
) -> Result<(PathBuf, String), String> {
    let dest = checkout_path(company_id, workspace_id);
    let dot_git = dest.join(".git");
    if refresh || dot_git.exists() {
        if dot_git.exists() {
            run_git(
                &["-C", dest.to_str().expect("utf8 path"), "fetch", "origin"],
                token,
                None,
            )
            .await?;
            Ok((dest, "fetched".to_owned()))
        } else {
            Err("workspace is not materialized; run materialize first".to_owned())
        }
    } else {
        let output = run_git(
            &["clone", repo_url, dest.to_str().expect("utf8 path")],
            token,
            None,
        )
        .await?;
        Ok((dest, output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_auth_header_is_well_formed() {
        let header = basic_auth_header("tok_123");
        assert!(header.starts_with("AUTHORIZATION: basic "));
        // The base64 decodes back to x-access-token:tok_123.
        use base64::Engine;
        let encoded = header.trim_start_matches("AUTHORIZATION: basic ");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .expect("valid base64");
        assert_eq!(
            String::from_utf8(decoded).unwrap(),
            "x-access-token:tok_123"
        );
    }

    #[test]
    fn redact_removes_raw_and_encoded_credentials() {
        let token = "ghp_supersecret";
        let encoded = {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .encode(format!("x-access-token:{token}").as_bytes())
        };
        let text = format!("clone failed for ghp_supersecret with auth {encoded} and more");
        let redacted = redact_credentials(&text, token);
        assert!(!redacted.contains("ghp_supersecret"));
        assert!(!redacted.contains(&encoded));
        assert_eq!(redacted, "clone failed for *** with auth *** and more");
    }

    #[test]
    fn checkout_path_is_scoped() {
        let path = checkout_path("c1", "w1");
        assert!(path.ends_with("data/checkouts/c1/w1"));
    }
}
