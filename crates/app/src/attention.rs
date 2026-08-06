//! Issue-based attention feed (upstream `server/src/services/attention.ts`
//! parity).
//!
//! Source kinds:
//! - `approval` (pending approvals + issue links)
//! - `issue_thread_interaction` (pending interactions; `request_confirmation`
//!   collapsed to the newest per issue)
//! - `blocker_attention` (blocked issues; multi-level blocker tree with
//!   #10785 triage fields)
//! - `budget_alert` (companies whose monthly budget is exhausted)
//! - `failed_run` (latest failed/timed-out heartbeat run per agent)
//! - `decision` (open decision-desk decisions)
//!
//! The feed envelope matches upstream (`companyId`, `generatedAt`,
//! `totalCount`, `deskBadgeCount`, `countsBySourceKind`, `nextCursor`,
//! `items`) with `activity`/`decide` sorts, cursor pagination, dismissals
//! (dismiss/snooze), activity boundaries, and decision-queue filtering.

use serde::Serialize;
use staple_data::{
    ApprovalRecord, DecisionRecord, HeartbeatRunRecord, IssueRecord, ThreadInteractionRecord,
};

use crate::error::ApiError;
use crate::state::AppState;

/// One decision verb offered by an attention item.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttentionDecisionVerb {
    /// Verb id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Description.
    pub description: String,
}

/// The subject of an attention item.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttentionSubject {
    /// Subject kind (`approval` | `interaction` | `issue` | `company`).
    pub kind: String,
    /// Subject id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Title.
    pub title: String,
    /// Identifier (e.g. `ALPHA-3`), when available.
    pub identifier: Option<String>,
    /// Status.
    pub status: String,
    /// UI href.
    pub href: Option<String>,
    /// Kind-specific metadata.
    pub metadata: serde_json::Value,
}

/// One attention item.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttentionItem {
    /// Stable item id (`{sourceKind}:{dedupKey}`).
    pub id: String,
    /// Source kind.
    pub source_kind: String,
    /// Subject.
    pub subject: AttentionSubject,
    /// Why this item needs attention now.
    pub why_now: String,
    /// Decision verbs.
    pub decision_verbs: Vec<AttentionDecisionVerb>,
    /// Whether the item can be resolved inline.
    pub inline_resolvable: bool,
    /// Entry rule.
    pub entry_rule: String,
    /// Exit rule.
    pub exit_rule: String,
    /// Dedup key.
    pub dedup_key: String,
    /// Severity (`critical` | `high` | `medium` | `low`).
    pub severity: String,
    /// Activity timestamp.
    pub activity_at: String,
    /// Creation timestamp.
    pub created_at: String,
    /// Update timestamp.
    pub updated_at: String,
    /// Related issue, when applicable.
    pub related_issue: Option<AttentionSubject>,
    /// Kind-specific detail.
    pub detail: serde_json::Value,
    /// Active dismissal/snooze state for this item, when present.
    pub dismissal: Option<DismissalInfo>,
    /// Decision queues containing this item's subject.
    pub queues: Vec<AttentionQueueRef>,
}

/// Dismissal/snooze state attached to an item.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DismissalInfo {
    /// `dismiss` | `snooze`.
    pub kind: String,
    /// Dismissal timestamp.
    pub dismissed_at: String,
    /// Snooze expiry, when snoozing.
    pub snoozed_until: Option<String>,
}

/// A decision queue reference.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttentionQueueRef {
    /// Queue key.
    pub key: String,
    /// Queue title.
    pub title: String,
}

/// The attention feed.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttentionFeed {
    /// Company id.
    pub company_id: String,
    /// Generation timestamp.
    pub generated_at: String,
    /// Total distinct items before pagination.
    pub total_count: usize,
    /// Sidebar badge: distinct items surfaced today (pre-pagination).
    pub desk_badge_count: usize,
    /// Counts by source kind.
    pub counts_by_source_kind: serde_json::Value,
    /// Next cursor, when more items exist.
    pub next_cursor: Option<String>,
    /// Page items.
    pub items: Vec<AttentionItem>,
}

/// Feed query options.
#[derive(Debug, Clone, Default)]
pub struct AttentionQuery {
    /// Page size (1..=100).
    pub limit: usize,
    /// Cursor returned by a previous page.
    pub cursor: Option<String>,
    /// Sort mode (`activity` | `decide`).
    pub sort: String,
    /// Include dismissed/snoozed items.
    pub include_dismissed: bool,
    /// Only items with activity at/after this ISO boundary.
    pub activity_since: Option<String>,
    /// Only items with activity at/before this ISO boundary.
    pub activity_until: Option<String>,
    /// Include all items (requires `queue`).
    pub all: bool,
    /// Restrict to items in a decision queue (by queue key).
    pub queue: Option<String>,
    /// Acting board user id (dismissal scope).
    pub user_id: String,
}

/// Builds the attention feed for a company.
///
/// # Errors
///
/// Returns [`ApiError`] on database failure or an invalid cursor.
pub async fn build_attention_feed(
    state: &AppState,
    company_id: &str,
    query: &AttentionQuery,
) -> Result<AttentionFeed, ApiError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    if query.all && query.queue.is_none() {
        return Err(ApiError::bad_request("all requires a queue filter"));
    }
    let mut items = Vec::new();

    collect_approvals(state, company_id, &mut items).await?;
    collect_interactions(state, company_id, &mut items).await?;
    collect_blockers(state, company_id, &mut items).await?;
    collect_budget_alerts(state, company_id, &mut items).await?;
    collect_failed_runs(state, company_id, &mut items).await?;
    collect_decisions(state, company_id, &mut items).await?;

    // Dismissals (dismiss/snooze) filter + attach, unless include_dismissed.
    let dismissals = state
        .attention_dismissals
        .list(company_id, &query.user_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let by_key: std::collections::HashMap<String, staple_data::AttentionDismissalRecord> =
        dismissals
            .into_iter()
            .map(|row| (row.item_key.clone(), row))
            .collect();
    items.retain_mut(|item| {
        let Some(dismissal) = by_key.get(&item.dedup_key) else {
            return true;
        };
        let active = dismissal_active(dismissal, &item.activity_at, now);
        if active && !query.include_dismissed {
            return false;
        }
        item.dismissal = Some(DismissalInfo {
            kind: dismissal.kind.clone(),
            dismissed_at: dismissal.dismissed_at.clone(),
            snoozed_until: dismissal.snoozed_until.clone(),
        });
        true
    });

    // Activity boundaries.
    if let Some(since) = &query.activity_since {
        items.retain(|item| item.activity_at >= *since);
    }
    if let Some(until) = &query.activity_until {
        items.retain(|item| item.activity_at <= *until);
    }

    // Decision-queue filtering.
    if let Some(queue_key) = &query.queue {
        let queue_ids = state
            .decisions
            .list_queues(company_id)
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?
            .into_iter()
            .filter(|queue| queue.key.as_deref() == Some(queue_key.as_str()))
            .map(|queue| queue.id)
            .collect::<Vec<_>>();
        let mut allowed: std::collections::HashSet<String> = std::collections::HashSet::new();
        for queue_id in queue_ids {
            for item in state
                .decisions
                .list_items(company_id, &queue_id)
                .await
                .map_err(|error| ApiError::internal(error.to_string()))?
            {
                allowed.insert(item.source_id);
            }
        }
        items.retain(|item| allowed.contains(&item.subject.id));
    }

    let total_count = items.len();
    let desk_badge_count = items
        .iter()
        .filter(|item| is_new_today(&item.created_at, now))
        .count();

    let mut counts = serde_json::Map::new();
    for kind in [
        "approval",
        "issue_thread_interaction",
        "blocker_attention",
        "budget_alert",
        "failed_run",
        "decision",
    ] {
        let count = items.iter().filter(|item| item.source_kind == kind).count();
        counts.insert(kind.to_owned(), serde_json::json!(count));
    }

    let sort = if query.sort == "decide" {
        "decide"
    } else {
        "activity"
    };
    sort_items(&mut items, sort);
    reorder_blocked_by_weight(&mut items);

    let start = match &query.cursor {
        Some(cursor) => {
            let cursor_id = decode_cursor(cursor).ok_or_else(|| {
                ApiError::bad_request(format!("invalid attention cursor: {cursor}"))
            })?;
            items
                .iter()
                .position(|item| item.id == cursor_id)
                .map(|index| index + 1)
                .unwrap_or(0)
        }
        None => 0,
    };
    let end = (start + query.limit).min(items.len());
    let page = items[start..end].to_vec();
    let next_cursor = if end < items.len() {
        page.last().map(|item| encode_cursor(sort, &item.id))
    } else {
        None
    };

    Ok(AttentionFeed {
        company_id: company_id.to_owned(),
        generated_at: iso_now(now),
        total_count,
        desk_badge_count,
        counts_by_source_kind: serde_json::Value::Object(counts),
        next_cursor,
        items: page,
    })
}

// -- source collectors -----------------------------------------------------

async fn collect_approvals(
    state: &AppState,
    company_id: &str,
    items: &mut Vec<AttentionItem>,
) -> Result<(), ApiError> {
    let approvals = state
        .approvals
        .list(company_id, Some("pending"))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    if approvals.is_empty() {
        return Ok(());
    }
    let links = state
        .issue_structure
        .list_company_issue_approvals(company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let issue_by_approval: std::collections::HashMap<String, String> = links
        .iter()
        .map(|link| (link.approval_id.clone(), link.issue_id.clone()))
        .collect();
    for approval in approvals {
        items.push(approval_item(
            company_id,
            &approval,
            issue_by_approval.get(&approval.id),
        ));
    }
    Ok(())
}

fn approval_item(
    company_id: &str,
    approval: &ApprovalRecord,
    issue_id: Option<&String>,
) -> AttentionItem {
    let title = approval_title(&approval.r#type);
    let id = format!("approval:{}", approval.id);
    AttentionItem {
        id: id.clone(),
        source_kind: "approval".to_owned(),
        subject: AttentionSubject {
            kind: "approval".to_owned(),
            id: approval.id.clone(),
            company_id: company_id.to_owned(),
            title: title.clone(),
            identifier: None,
            status: approval.status.clone(),
            href: Some(format!("/companies/{company_id}/approvals/{}", approval.id)),
            metadata: serde_json::json!({
                "type": approval.r#type,
                "requestedByAgentId": approval.requested_by_agent_id,
                "requestedByUserId": approval.requested_by_user_id,
                "issueId": issue_id,
            }),
        },
        why_now: "Approval is pending a board decision.".to_owned(),
        decision_verbs: vec![
            verb("approve", "Approve", "Approve the request."),
            verb("reject", "Reject", "Reject the request."),
            verb(
                "request_revision",
                "Request revision",
                "Send the request back for changes.",
            ),
        ],
        inline_resolvable: approval.r#type != "request_board_approval",
        entry_rule: "approvals.status = 'pending'".to_owned(),
        exit_rule: "Approval leaves pending status.".to_owned(),
        dedup_key: id,
        severity: "medium".to_owned(),
        activity_at: approval.created_at.clone(),
        created_at: approval.created_at.clone(),
        updated_at: approval.created_at.clone(),
        related_issue: None,
        detail: serde_json::json!({
            "kind": "approval",
            "approvalType": approval.r#type,
            "requestedByAgentId": approval.requested_by_agent_id,
            "requestedByUserId": approval.requested_by_user_id,
            "issueId": issue_id,
        }),
        dismissal: None,
        queues: Vec::new(),
    }
}

async fn collect_interactions(
    state: &AppState,
    company_id: &str,
    items: &mut Vec<AttentionItem>,
) -> Result<(), ApiError> {
    let interactions = state
        .issue_structure
        .list_company_thread_interactions(company_id, Some("pending"))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    if interactions.is_empty() {
        return Ok(());
    }
    let issues = state
        .issues
        .list(company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let issue_by_id: std::collections::HashMap<String, IssueRecord> = issues
        .into_iter()
        .map(|issue| (issue.id.clone(), issue))
        .collect();
    // Collapse pending request confirmations to the newest per issue
    // (#10785 `collapsePendingConfirmationsToNewest`).
    let mut newest_by_issue: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for (index, interaction) in interactions.iter().enumerate() {
        if interaction.kind != "request_confirmation" {
            continue;
        }
        let current = newest_by_issue.get(&interaction.issue_id).copied();
        let replace = match current {
            None => true,
            Some(existing) => {
                let newer = interaction.created_at > interactions[existing].created_at;
                newer
                    || (interaction.created_at == interactions[existing].created_at
                        && interaction.id > interactions[existing].id)
            }
        };
        if replace {
            newest_by_issue.insert(interaction.issue_id.clone(), index);
        }
    }
    for (index, interaction) in interactions.iter().enumerate() {
        if interaction.kind == "request_confirmation"
            && newest_by_issue.get(&interaction.issue_id).copied() != Some(index)
        {
            continue;
        }
        let issue = issue_by_id.get(&interaction.issue_id);
        items.push(interaction_item(company_id, interaction, issue));
    }
    Ok(())
}

fn interaction_item(
    company_id: &str,
    interaction: &ThreadInteractionRecord,
    issue: Option<&IssueRecord>,
) -> AttentionItem {
    let label = interaction_label(&interaction.kind);
    let title = interaction
        .title
        .clone()
        .or_else(|| interaction.summary.clone())
        .unwrap_or_else(|| label.clone());
    let id = format!("interaction:{}", interaction.id);
    AttentionItem {
        id: id.clone(),
        source_kind: "issue_thread_interaction".to_owned(),
        subject: AttentionSubject {
            kind: "interaction".to_owned(),
            id: interaction.id.clone(),
            company_id: company_id.to_owned(),
            title: title.clone(),
            identifier: None,
            status: interaction.status.clone(),
            href: issue.map(|issue| {
                format!(
                    "/companies/{company_id}/issues/{}#interaction-{}",
                    issue.id, interaction.id
                )
            }),
            metadata: serde_json::json!({
                "kind": interaction.kind,
                "issueId": interaction.issue_id,
                "createdByAgentId": interaction.created_by_agent_id,
            }),
        },
        why_now: format!("{label} on an issue thread."),
        decision_verbs: interaction_verbs(&interaction.kind),
        inline_resolvable: true,
        entry_rule: "issue_thread_interactions.status = 'pending'".to_owned(),
        exit_rule: "Interaction resolves, expires, fails, or is cancelled.".to_owned(),
        dedup_key: id,
        severity: "medium".to_owned(),
        activity_at: interaction.created_at.clone(),
        created_at: interaction.created_at.clone(),
        updated_at: interaction.created_at.clone(),
        related_issue: issue.map(issue_subject),
        detail: serde_json::json!({
            "kind": "generic",
            "summaryExcerpt": title,
            "images": [],
        }),
        dismissal: None,
        queues: Vec::new(),
    }
}

/// Blocked-tree traversal limits (upstream `BLOCKER_ATTENTION_MAX_DEPTH` /
/// `BLOCKER_ATTENTION_MAX_NODES`).
const BLOCKER_MAX_DEPTH: usize = 8;
const BLOCKER_MAX_NODES: usize = 200;

async fn collect_blockers(
    state: &AppState,
    company_id: &str,
    items: &mut Vec<AttentionItem>,
) -> Result<(), ApiError> {
    let issues = state
        .issues
        .list(company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let blocked: Vec<&IssueRecord> = issues
        .iter()
        .filter(|issue| issue.status == "blocked")
        .collect();
    if blocked.is_empty() {
        return Ok(());
    }
    let issue_by_id: std::collections::HashMap<&str, &IssueRecord> = issues
        .iter()
        .map(|issue| (issue.id.as_str(), issue))
        .collect();
    let is_open = |issue: &IssueRecord| issue.status != "done" && issue.status != "cancelled";

    for issue in blocked {
        // Walk the blocker tree breadth-first from the blocked issue, bounded
        // by depth and total-node limits. `chain` collects every issue that
        // (transitively) blocks the subject.
        let mut chain: Vec<&IssueRecord> = Vec::new();
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut frontier: Vec<String> = vec![issue.id.clone()];
        let mut depth = 0usize;
        while !frontier.is_empty() && depth < BLOCKER_MAX_DEPTH {
            depth += 1;
            let mut next: Vec<String> = Vec::new();
            for id in frontier {
                if chain.len() >= BLOCKER_MAX_NODES || !visited.insert(id.clone()) {
                    continue;
                }
                let blockers = state
                    .relations
                    .list_blockers(&id)
                    .await
                    .map_err(|error| ApiError::internal(error.to_string()))?;
                for relation in blockers {
                    if let Some(blocker) = issue_by_id.get(relation.issue_id.as_str()).copied() {
                        chain.push(blocker);
                        next.push(blocker.id.clone());
                    }
                }
            }
            frontier = next;
        }
        if chain.is_empty() {
            continue;
        }
        let any_live = chain.iter().any(|blocker| is_open(blocker));
        let sample = chain
            .first()
            .map(|blocker| blocker.identifier.clone())
            .unwrap_or_else(|| issue.id.clone());
        let terminal_id = chain
            .last()
            .map(|blocker| blocker.id.clone())
            .unwrap_or_else(|| issue.id.clone());
        let chain_ids: std::collections::HashSet<&str> =
            chain.iter().map(|blocker| blocker.id.as_str()).collect();
        // Number of open issues held up by the chain: open issues (excluding
        // the subject and chain members) that are directly blocked by any
        // chain member.
        let mut blocked_task_count = 0usize;
        for candidate in issues.iter().filter(|candidate| {
            candidate.id != issue.id
                && is_open(candidate)
                && !chain_ids.contains(candidate.id.as_str())
        }) {
            let candidate_blockers = state
                .relations
                .list_blockers(&candidate.id)
                .await
                .map_err(|error| ApiError::internal(error.to_string()))?;
            if candidate_blockers
                .iter()
                .any(|relation| chain_ids.contains(relation.issue_id.as_str()))
            {
                blocked_task_count += 1;
            }
        }
        items.push(blocker_item(
            issue,
            &chain,
            any_live,
            sample,
            terminal_id,
            blocked_task_count,
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn blocker_item(
    issue: &IssueRecord,
    chain: &[&IssueRecord],
    any_live: bool,
    sample: String,
    terminal_blocker_id: String,
    blocked_task_count: usize,
) -> AttentionItem {
    let first = chain[0];
    let blocking_issue = serde_json::json!({
        "id": first.id,
        "identifier": first.identifier,
        "title": first.title,
        "status": first.status,
        "priority": first.priority,
    });
    let id = format!("blocker:{}:{}", issue.id, sample);
    AttentionItem {
        id: id.clone(),
        source_kind: "blocker_attention".to_owned(),
        subject: issue_subject(issue),
        why_now: if any_live {
            "Blocked dependency chain needs human attention.".to_owned()
        } else {
            "Blocked dependency chain is stalled and needs a human to choose the next owner or action."
                .to_owned()
        },
        decision_verbs: vec![
            verb(
                "unblock",
                "Unblock",
                "Repair or replace the stalled blocker path.",
            ),
            verb(
                "reassign",
                "Reassign",
                "Assign the stalled blocker to a live owner.",
            ),
            verb("nudge", "Nudge", "Wake or prompt the current owner."),
        ],
        inline_resolvable: false,
        entry_rule: "blocked issue has an active blocker chain".to_owned(),
        exit_rule: "Blocker chain is no longer stalled or the issue leaves blocked status."
            .to_owned(),
        dedup_key: id,
        severity: "high".to_owned(),
        activity_at: issue.updated_at.clone(),
        created_at: issue.created_at.clone(),
        updated_at: issue.updated_at.clone(),
        related_issue: None,
        detail: serde_json::json!({
            "kind": "blocker",
            "blockingIssue": blocking_issue,
            "blockedTaskCount": blocked_task_count,
            "blockingTreeLive": any_live,
            "terminalBlockerIssueId": terminal_blocker_id,
        }),
        dismissal: None,
        queues: Vec::new(),
    }
}

async fn collect_budget_alerts(
    state: &AppState,
    company_id: &str,
    items: &mut Vec<AttentionItem>,
) -> Result<(), ApiError> {
    let companies = state
        .companies
        .list()
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let Some(company) = companies
        .into_iter()
        .find(|company| company.id == company_id)
    else {
        return Ok(());
    };
    if company.budget_monthly_cents <= 0
        || company.spent_monthly_cents < company.budget_monthly_cents
    {
        return Ok(());
    }
    let percent = if company.budget_monthly_cents > 0 {
        (company.spent_monthly_cents * 100) / company.budget_monthly_cents
    } else {
        100
    };
    items.push(AttentionItem {
        id: format!("budget:{}", company.id),
        source_kind: "budget_alert".to_owned(),
        subject: AttentionSubject {
            kind: "company".to_owned(),
            id: company.id.clone(),
            company_id: company.id.clone(),
            title: format!("Budget exhausted - {}", company.name),
            identifier: None,
            status: "over_budget".to_owned(),
            href: Some(format!("/companies/{}/settings", company.id)),
            metadata: serde_json::json!({
                "budgetMonthlyCents": company.budget_monthly_cents,
                "spentMonthlyCents": company.spent_monthly_cents,
                "percent": percent,
            }),
        },
        why_now: "Budget is exhausted; agents are paused until it is reset.".to_owned(),
        decision_verbs: vec![
            verb(
                "review",
                "Review budget",
                "Review and reset the monthly budget.",
            ),
            verb("pause", "Pause company", "Pause the company to stop spend."),
        ],
        inline_resolvable: false,
        entry_rule: "company spent_monthly_cents >= budget_monthly_cents".to_owned(),
        exit_rule: "Budget is reset or the company leaves over-budget state.".to_owned(),
        dedup_key: format!("budget:{}", company.id),
        severity: "critical".to_owned(),
        activity_at: company.updated_at.clone(),
        created_at: company.created_at.clone(),
        updated_at: company.updated_at.clone(),
        related_issue: None,
        detail: serde_json::json!({
            "kind": "budget",
            "budgetMonthlyCents": company.budget_monthly_cents,
            "spentMonthlyCents": company.spent_monthly_cents,
            "percent": percent,
        }),
        dismissal: None,
        queues: Vec::new(),
    });
    Ok(())
}

async fn collect_failed_runs(
    state: &AppState,
    company_id: &str,
    items: &mut Vec<AttentionItem>,
) -> Result<(), ApiError> {
    let runs = state
        .heartbeat
        .list(company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    if runs.is_empty() {
        return Ok(());
    }
    let agents = state
        .agents
        .list(company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let agent_name: std::collections::HashMap<&str, &str> = agents
        .iter()
        .map(|agent| (agent.id.as_str(), agent.name.as_str()))
        .collect();
    // Keep only the latest run per agent; emit a failed_run item when that
    // latest run is failed/timed_out (a newer successful run clears it).
    let mut latest_by_agent: std::collections::HashMap<&str, &HeartbeatRunRecord> =
        std::collections::HashMap::new();
    for run in runs.iter() {
        let current = latest_by_agent.get(run.agent_id.as_str()).copied();
        if current.is_none_or(|existing| run.created_at > existing.created_at) {
            latest_by_agent.insert(run.agent_id.as_str(), run);
        }
    }
    for run in latest_by_agent.values() {
        if run.status != "failed" && run.status != "timed_out" {
            continue;
        }
        let name = agent_name
            .get(run.agent_id.as_str())
            .copied()
            .unwrap_or("agent");
        let id = format!("run:{}", run.id);
        items.push(AttentionItem {
            id: id.clone(),
            source_kind: "failed_run".to_owned(),
            subject: AttentionSubject {
                kind: "run".to_owned(),
                id: run.id.clone(),
                company_id: company_id.to_owned(),
                title: format!("{name} run {}", run.status),
                identifier: None,
                status: run.status.clone(),
                href: Some(format!(
                    "/companies/{company_id}/agents/{}/runs/{}",
                    run.agent_id, run.id
                )),
                metadata: serde_json::json!({
                    "agentId": run.agent_id,
                    "agentName": name,
                    "error": run.error,
                }),
            },
            why_now: "Run failed after automatic retries were exhausted.".to_owned(),
            decision_verbs: vec![
                verb("retry", "Retry", "Retry the failed run or issue."),
                verb("reassign", "Reassign", "Move the work to another owner."),
                verb(
                    "dismiss",
                    "Dismiss",
                    "Dismiss this failed-run attention row.",
                ),
            ],
            inline_resolvable: true,
            entry_rule: "latest failed/timed_out run has no newer run".to_owned(),
            exit_rule: "A newer run exists for the same agent or the row is dismissed.".to_owned(),
            dedup_key: id,
            severity: "high".to_owned(),
            activity_at: run
                .finished_at
                .clone()
                .unwrap_or_else(|| run.updated_at.clone()),
            created_at: run.created_at.clone(),
            updated_at: run.updated_at.clone(),
            related_issue: None,
            detail: serde_json::json!({
                "kind": "failed_run",
                "agentName": name,
                "failureReasonExcerpt": run.error,
            }),
            dismissal: None,
            queues: Vec::new(),
        });
    }
    Ok(())
}

async fn collect_decisions(
    state: &AppState,
    company_id: &str,
    items: &mut Vec<AttentionItem>,
) -> Result<(), ApiError> {
    let decisions = state
        .decision_actions
        .list_decisions(company_id, Some("open"))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    for decision in decisions {
        let id = format!("decision:{}", decision.id);
        items.push(AttentionItem {
            id: id.clone(),
            source_kind: "decision".to_owned(),
            subject: AttentionSubject {
                kind: "decision".to_owned(),
                id: decision.id.clone(),
                company_id: company_id.to_owned(),
                title: decision.title.clone(),
                identifier: None,
                status: decision.status.clone(),
                href: Some(format!(
                    "/companies/{company_id}/decisions?decisionId={}",
                    decision.id
                )),
                metadata: serde_json::json!({
                    "originIssueId": decision.origin_issue_id,
                    "originAgentId": decision.origin_agent_id,
                    "bundleId": decision.bundle_id,
                }),
            },
            why_now: "An agent decision is waiting for a board response.".to_owned(),
            decision_verbs: vec![verb("decide", "Review", "Review and choose an option.")],
            inline_resolvable: true,
            entry_rule: "decisions.status = 'open'".to_owned(),
            exit_rule: "Decision is decided, expired, or cancelled.".to_owned(),
            dedup_key: id,
            severity: "medium".to_owned(),
            activity_at: decision.updated_at.clone(),
            created_at: decision.created_at.clone(),
            updated_at: decision.updated_at.clone(),
            related_issue: None,
            detail: serde_json::json!({
                "kind": "generic",
                "summaryExcerpt": decision.body,
                "images": [],
            }),
            dismissal: None,
            queues: Vec::new(),
        });
    }
    Ok(())
}

/// Whether a dismissal is currently active for an item.
fn dismissal_active(
    dismissal: &staple_data::AttentionDismissalRecord,
    activity_at: &str,
    now_secs: i64,
) -> bool {
    if dismissal.kind == "snooze" {
        return dismissal
            .snoozed_until
            .as_deref()
            .is_some_and(|until| iso_to_unix(until).unwrap_or(0) > now_secs);
    }
    // Dismiss: active when dismissed after the item's last activity.
    iso_to_unix(&dismissal.dismissed_at).unwrap_or(0) >= iso_to_unix(activity_at).unwrap_or(0)
}

fn iso_to_unix(iso: &str) -> Option<i64> {
    let iso = iso.strip_suffix('Z').unwrap_or(iso);
    let (date, time) = iso.split_once('T')?;
    let mut parts = date.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    let mut time_parts = time.split(':');
    let hour: i64 = time_parts.next()?.parse().ok()?;
    let minute: i64 = time_parts.next()?.parse().ok()?;
    let second: f64 = time_parts.next()?.parse().ok()?;
    let days = days_from_civil(year, month, day);
    Some(days * 86_400 + hour * 3600 + minute * 60 + second as i64)
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = i64::from(month) + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `orderBlockedAttentionByWeight` parity: blocker items keep their sorted
/// slots but are ordered among themselves by blocked-task weight (descending).
fn reorder_blocked_by_weight(items: &mut [AttentionItem]) {
    let slots: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.source_kind == "blocker_attention")
        .map(|(index, _)| index)
        .collect();
    if slots.len() < 2 {
        return;
    }
    let mut blockers: Vec<AttentionItem> =
        slots.iter().map(|&index| items[index].clone()).collect();
    blockers.sort_by(|left, right| {
        let weight = |item: &AttentionItem| {
            item.detail
                .get("blockedTaskCount")
                .and_then(|value| value.as_u64())
                .unwrap_or(0)
        };
        weight(right)
            .cmp(&weight(left))
            .then_with(|| left.updated_at.cmp(&right.updated_at))
    });
    for (slot, item) in slots.iter().zip(blockers) {
        items[*slot] = item;
    }
}

// -- helpers -----------------------------------------------------------------

fn issue_subject(issue: &IssueRecord) -> AttentionSubject {
    AttentionSubject {
        kind: "issue".to_owned(),
        id: issue.id.clone(),
        company_id: issue.company_id.clone(),
        title: issue.title.clone(),
        identifier: Some(issue.identifier.clone()),
        status: issue.status.clone(),
        href: Some(format!(
            "/companies/{}/issues/{}",
            issue.company_id, issue.id
        )),
        metadata: serde_json::json!({
            "priority": issue.priority,
            "assigneeAgentId": issue.assignee_agent_id,
            "assigneeUserId": issue.assignee_user_id,
        }),
    }
}

fn verb(id: &str, label: &str, description: &str) -> AttentionDecisionVerb {
    AttentionDecisionVerb {
        id: id.to_owned(),
        label: label.to_owned(),
        description: description.to_owned(),
    }
}

fn approval_title(r#type: &str) -> String {
    match r#type {
        "hire_agent" => "Hire agent approval".to_owned(),
        "approve_ceo_strategy" => "CEO strategy approval".to_owned(),
        "budget_override_required" => "Budget override approval".to_owned(),
        "request_board_approval" => "Board approval".to_owned(),
        other => format!("Approval - {other}"),
    }
}

fn interaction_label(kind: &str) -> String {
    match kind {
        "request_confirmation" => "Request confirmation".to_owned(),
        "ask_user_questions" => "Question".to_owned(),
        "suggest_tasks" => "Suggested tasks".to_owned(),
        "checkbox_confirmation" => "Checkbox confirmation".to_owned(),
        "item_verdicts" => "Item verdicts".to_owned(),
        other => other.replace('_', " "),
    }
}

fn interaction_verbs(kind: &str) -> Vec<AttentionDecisionVerb> {
    match kind {
        "ask_user_questions" => vec![verb("answer", "Answer", "Answer the questions.")],
        "suggest_tasks" => vec![verb("review", "Review", "Review the suggested tasks.")],
        "checkbox_confirmation" => vec![
            verb("confirm", "Confirm", "Confirm the checkboxes."),
            verb("reject", "Reject", "Reject the confirmation."),
        ],
        "item_verdicts" => vec![verb("decide", "Decide", "Choose a verdict for each item.")],
        _ => vec![verb("respond", "Respond", "Respond to the request.")],
    }
}

/// Sorts items in place. `activity` = newest update first; `decide` = oldest
/// first (the A1 approximation of upstream decide ordering, which prefers
/// due/past-due items).
fn sort_items(items: &mut [AttentionItem], sort: &str) {
    if sort == "decide" {
        items.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then_with(|| left.id.cmp(&right.id))
        });
    } else {
        items.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| right.id.cmp(&left.id))
        });
    }
}

/// Whether an ISO timestamp is within the current UTC day.
fn is_new_today(iso: &str, now_secs: i64) -> bool {
    let start = start_of_utc_day(now_secs);
    iso >= start.as_str()
}

/// ISO-8601 timestamp for the current UTC day boundary.
fn start_of_utc_day(now_secs: i64) -> String {
    let days = now_secs.div_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T00:00:00.000Z")
}

fn iso_now(now_secs: i64) -> String {
    let days = now_secs.div_euclid(86_400);
    let secs_of_day = now_secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.000Z")
}

/// Howard Hinnant's days-from-civil inverse: days since epoch -> (y, m, d).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

fn encode_cursor(sort: &str, id: &str) -> String {
    use base64::Engine;
    let payload = serde_json::json!({ "v": 1, "sort": sort, "id": id });
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload.to_string())
}

fn decode_cursor(cursor: &str) -> Option<String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if value.get("v")?.as_i64()? != 1 {
        return None;
    }
    value.get("id")?.as_str().map(str::to_owned)
}
