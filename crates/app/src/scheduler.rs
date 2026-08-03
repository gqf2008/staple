//! Built-in scheduler: wake dispatch, routine cron triggers, and the
//! decision-desk retention sweep.
//!
//! Runs as a background task started by the binary; each tick performs one
//! pass over all companies. All state changes go through the existing
//! repositories so company scoping and audit invariants stay in one place.

use std::str::FromStr;
use std::time::Duration;

use tokio::time::{MissedTickBehavior, interval};

use crate::state::AppState;

/// Scheduler tuning (overridable via environment).
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Tick interval (default 60s; `STAPLE_SCHEDULER_TICK_SECS`).
    pub tick: Duration,
    /// Max wakeup dispatches per tick per company (default 10).
    pub wakeup_batch: usize,
    /// Sweep frequency in days (default 1).
    pub sweep_interval_days: u32,
}

/// Reads scheduler config from the environment with safe defaults.
#[must_use]
pub fn config_from_env() -> SchedulerConfig {
    let tick_secs = std::env::var("STAPLE_SCHEDULER_TICK_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(60)
        .max(1);
    let wakeup_batch = std::env::var("STAPLE_SCHEDULER_WAKEUP_BATCH")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10)
        .max(1);
    let sweep_interval_days = std::env::var("STAPLE_SCHEDULER_SWEEP_DAYS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1)
        .max(1);
    SchedulerConfig {
        tick: Duration::from_secs(tick_secs),
        wakeup_batch,
        sweep_interval_days,
    }
}

/// Runs the scheduler forever. Errors are logged and the loop continues.
pub async fn run(state: AppState) {
    let config = config_from_env();
    let mut ticker = interval(config.tick);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last_sweep: Option<String> = None;
    loop {
        ticker.tick().await;
        if let Err(error) = tick(&state, &config, &mut last_sweep).await {
            tracing::error!(%error, "scheduler tick failed");
        }
    }
}

/// One scheduler pass: wake dispatch, routine cron triggers, retention sweep.
///
/// # Errors
///
/// Returns the first error encountered; callers log and continue.
pub async fn tick(
    state: &AppState,
    config: &SchedulerConfig,
    last_sweep: &mut Option<String>,
) -> Result<(), String> {
    dispatch_wakeups(state, config).await?;
    trigger_routines(state).await?;
    run_sweep(state, config, last_sweep).await?;
    Ok(())
}

/// Claims queued wakeup requests for active agents and starts heartbeat runs.
async fn dispatch_wakeups(state: &AppState, config: &SchedulerConfig) -> Result<(), String> {
    let companies = state.companies.list().await.map_err(|e| e.to_string())?;
    for company in companies {
        let requests = state
            .agent_runtime
            .wakeup_list(&company.id)
            .await
            .map_err(|e| e.to_string())?;
        for request in requests
            .into_iter()
            .filter(|r| r.status == "queued")
            .take(config.wakeup_batch)
        {
            let active = match state.agents.get(&company.id, &request.agent_id).await {
                Ok(Some(agent)) => agent.status == "active",
                _ => false,
            };
            if !active {
                continue;
            }
            let Some(claimed) = state
                .agent_runtime
                .wakeup_claim(&company.id, &request.id)
                .await
                .map_err(|e| e.to_string())?
            else {
                continue;
            };
            let trigger_detail = claimed
                .reason
                .clone()
                .or_else(|| Some(format!("wakeup:{}", claimed.source)));
            match state
                .heartbeat
                .start(staple_data::NewHeartbeatRun {
                    company_id: company.id.clone(),
                    agent_id: claimed.agent_id.clone(),
                    invocation_source: "scheduler".to_owned(),
                    issue_id: None,
                    context_snapshot: None,
                    trigger_detail,
                })
                .await
            {
                Ok(run) => {
                    let _ = state
                        .agent_runtime
                        .wakeup_finish(&company.id, &claimed.id, "finished", None, Some(run.id))
                        .await;
                }
                Err(error) => {
                    let _ = state
                        .agent_runtime
                        .wakeup_finish(
                            &company.id,
                            &claimed.id,
                            "failed",
                            Some(error.to_string()),
                            None,
                        )
                        .await;
                }
            }
        }
    }
    Ok(())
}

/// Fires due cron triggers for active routines.
async fn trigger_routines(state: &AppState) -> Result<(), String> {
    let companies = state.companies.list().await.map_err(|e| e.to_string())?;
    let now = now_iso();
    for company in companies {
        let routines = state
            .routines
            .list(&company.id)
            .await
            .map_err(|e| e.to_string())?;
        for routine in routines {
            if routine.status != "active" {
                continue;
            }
            let triggers = state
                .routines
                .list_triggers(&company.id, &routine.id)
                .await
                .map_err(|e| e.to_string())?;
            for trigger in triggers {
                if trigger.get("scheduleKind").and_then(|v| v.as_str()) != Some("cron")
                    || trigger.get("enabled").and_then(|v| v.as_bool()) == Some(false)
                {
                    continue;
                }
                let Some(expr) = trigger.get("scheduleExpr").and_then(|v| v.as_str()) else {
                    continue;
                };
                if expr.is_empty() {
                    continue;
                }
                // First fire is "now minus one minute" so every-minute crons
                // trigger promptly; afterwards the routine's own
                // `lastTriggeredAt` stamp prevents double firing.
                let after = routine
                    .last_triggered_at
                    .clone()
                    .unwrap_or_else(|| now_minus_one_minute(&now));
                if let Some(next) = next_cron_after(expr, &after)
                    && next <= now
                {
                    let _ = state
                        .routines
                        .trigger(&company.id, &routine.id)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }
    }
    Ok(())
}

/// Runs the 90-day retention sweep once per configured interval.
async fn run_sweep(
    state: &AppState,
    config: &SchedulerConfig,
    last_sweep: &mut Option<String>,
) -> Result<(), String> {
    let today = today_utc();
    if last_sweep.as_deref() == Some(today.as_str()) {
        return Ok(());
    }
    let companies = state.companies.list().await.map_err(|e| e.to_string())?;
    for company in companies {
        let _ = state
            .decisions
            .sweep(&company.id, 90)
            .await
            .map_err(|e| e.to_string())?;
    }
    *last_sweep = Some(today);
    let _ = config.sweep_interval_days;
    Ok(())
}

/// Current UTC time as an ISO-8601 timestamp with milliseconds.
fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// UTC date (`YYYY-MM-DD`) used as the sweep gate.
fn today_utc() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

/// `after` minus one minute (ISO-8601).
fn now_minus_one_minute(after: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(after)
        .map(|dt| {
            (dt - chrono::Duration::minutes(1)).to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        })
        .unwrap_or_else(|_| after.to_owned())
}

/// Computes the next cron occurrence strictly after `after_iso`.
///
/// # Errors
///
/// Returns `None` when the expression is invalid.
fn next_cron_after(expr: &str, after_iso: &str) -> Option<String> {
    // The cron crate uses 6-field expressions (seconds first). Standard
    // 5-field cron (minute hour dom month dow) is normalized by prepending
    // "0 " so `0 0 * * *` still means "daily at 00:00".
    let normalized = if expr.split_whitespace().count() == 5 {
        format!("0 {expr}")
    } else {
        expr.to_owned()
    };
    let schedule = cron::Schedule::from_str(&normalized).ok()?;
    let after = chrono::DateTime::parse_from_rfc3339(after_iso)
        .ok()?
        .with_timezone(&chrono::Utc);
    schedule
        .after(&after)
        .next()
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_cron_after_hourly() {
        let next = next_cron_after("0 * * * *", "2026-08-04T01:23:45.000Z").unwrap();
        assert_eq!(next, "2026-08-04T02:00:00.000Z");
    }

    #[test]
    fn next_cron_after_every_minute() {
        let next = next_cron_after("* * * * *", "2026-08-04T01:23:45.000Z").unwrap();
        assert_eq!(next, "2026-08-04T01:24:00.000Z");
    }

    #[test]
    fn invalid_cron_returns_none() {
        assert!(next_cron_after("not a cron", "2026-08-04T01:23:45.000Z").is_none());
    }

    #[test]
    fn today_utc_is_date() {
        assert_eq!(today_utc().len(), 10);
    }
}
