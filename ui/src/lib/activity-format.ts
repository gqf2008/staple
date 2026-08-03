import { t } from "../i18n";
import type { Agent } from "@paperclipai/shared";
import type { CompanyUserProfile } from "./company-members";

type ActivityDetails = Record<string, unknown> | null | undefined;

type ActivityParticipant = {
  type: "agent" | "user";
  agentId?: string | null;
  userId?: string | null;
};

type ActivityIssueReference = {
  id?: string | null;
  identifier?: string | null;
  title?: string | null;
};

interface ActivityFormatOptions {
  agentMap?: Map<string, Agent>;
  userProfileMap?: Map<string, CompanyUserProfile>;
  currentUserId?: string | null;
}

const ACTIVITY_ROW_VERBS: Record<string, string> = {
  "issue.created": "created",
  "issue.updated": "updated",
  "issue.checked_out": t("ui.lib.activity-format.checked"),
  "issue.released": "released",
  "issue.comment_added": t("ui.lib.activity-format.commented"),
  "issue.comment_cancelled": t("ui.lib.activity-format.cancelled-queued-comment"),
  "issue.comment_deleted": t("ui.lib.activity-format.deleted-comment"),
  "issue.attachment_added": t("ui.lib.activity-format.attached-file"),
  "issue.attachment_removed": t("ui.lib.activity-format.removed-attachment-from"),
  "issue.document_created": t("ui.lib.activity-format.created-document"),
  "issue.document_updated": t("ui.lib.activity-format.updated-document"),
  "issue.document_locked": t("ui.lib.activity-format.locked-document"),
  "issue.document_unlocked": t("ui.lib.activity-format.unlocked-document"),
  "issue.document_deleted": t("ui.lib.activity-format.deleted-document-from"),
  "issue.monitor_scheduled": t("ui.lib.activity-format.scheduled-monitor"),
  "issue.monitor_triggered": t("ui.lib.activity-format.triggered-monitor"),
  "issue.monitor_cleared": t("ui.lib.activity-format.cleared-monitor"),
  "issue.monitor_skipped": t("ui.lib.activity-format.skipped-monitor"),
  "issue.monitor_exhausted": t("ui.lib.activity-format.exhausted-monitor"),
  "issue.monitor_recovery_wake_queued": t("ui.lib.activity-format.queued-monitor-recovery"),
  "issue.monitor_recovery_issue_created": t("ui.lib.activity-format.created-monitor-recovery"),
  "issue.monitor_escalated_to_board": t("ui.lib.activity-format.escalated-monitor"),
  "issue.commented": t("ui.lib.activity-format.commented"),
  "issue.deleted": "deleted",
  "issue.successful_run_handoff_required": t("ui.lib.activity-format.flagged-missing-next-step"),
  "issue.successful_run_handoff_resolved": t("ui.lib.activity-format.recorded-next-step-chosen"),
  "issue.successful_run_handoff_escalated": t("ui.lib.activity-format.escalated-missing-next-step"),
  "issue.accepted_plan_decomposition_updated": t("ui.lib.activity-format.updated-accepted-plan-decomposition"),
  "issue.recovery_action_opened": t("ui.lib.activity-format.opened-recovery-action"),
  "issue.recovery_action_resolved": t("ui.lib.activity-format.resolved-recovery-action"),
  "issue.recovery_action_escalated": t("ui.lib.activity-format.escalated-recovery-action"),
  "agent.created": "created",
  "agent.updated": "updated",
  "agent.paused": "paused",
  "agent.resumed": "resumed",
  "agent.error_cleared": t("ui.lib.activity-format.cleared-error"),
  "agent.terminated": "terminated",
  "agent.key_created": t("ui.lib.activity-format.created-api-key"),
  "agent.budget_updated": t("ui.lib.activity-format.updated-budget"),
  "agent.runtime_session_reset": t("ui.lib.activity-format.reset-session"),
  "heartbeat.invoked": t("ui.lib.activity-format.invoked-heartbeat"),
  "heartbeat.cancelled": t("ui.lib.activity-format.cancelled-heartbeat"),
  "heartbeat.output_stale_source_resolved": t("ui.lib.activity-format.system-folded-stale-run"),
  "heartbeat.output_stale_recovery_recursion_refused": t("ui.lib.activity-format.refused-recovery-recovery"),
  "approval.created": t("ui.lib.activity-format.requested-approval"),
  "approval.approved": "approved",
  "approval.rejected": "rejected",
  "project.created": "created",
  "project.updated": "updated",
  "project.deleted": "deleted",
  "goal.created": "created",
  "goal.updated": "updated",
  "goal.deleted": "deleted",
  "cost.reported": t("ui.lib.activity-format.reported-cost"),
  "cost.recorded": t("ui.lib.activity-format.recorded-cost"),
  "company.created": t("ui.lib.activity-format.created-company"),
  "company.updated": t("ui.lib.activity-format.updated-company"),
  "company.archived": "archived",
  "company.reactivated": "reactivated",
  "company.budget_updated": t("ui.lib.activity-format.updated-budget"),
  "audit.exported": t("ui.lib.activity-format.exported-agent-audit-log"),
};

const ISSUE_ACTIVITY_LABELS: Record<string, string> = {
  "issue.created": t("ui.lib.activity-format.created-issue"),
  "issue.updated": t("ui.lib.activity-format.updated-issue"),
  "issue.checked_out": t("ui.lib.activity-format.checked-issue"),
  "issue.released": t("ui.lib.activity-format.released-issue"),
  "issue.comment_added": t("ui.lib.activity-format.added-comment"),
  "issue.comment_cancelled": t("ui.lib.activity-format.cancelled-queued-comment.2"),
  "issue.comment_deleted": t("ui.lib.activity-format.deleted-comment.2"),
  "issue.feedback_vote_saved": t("ui.lib.activity-format.saved-feedback-ai-output"),
  "issue.attachment_added": t("ui.lib.activity-format.added-attachment"),
  "issue.attachment_removed": t("ui.lib.activity-format.removed-attachment"),
  "issue.document_created": t("ui.lib.activity-format.created-document.2"),
  "issue.document_updated": t("ui.lib.activity-format.updated-document.2"),
  "issue.document_locked": t("ui.lib.activity-format.locked-document.2"),
  "issue.document_unlocked": t("ui.lib.activity-format.unlocked-document.2"),
  "issue.document_deleted": t("ui.lib.activity-format.deleted-document"),
  "issue.monitor_scheduled": t("ui.lib.activity-format.scheduled-monitor.2"),
  "issue.monitor_triggered": t("ui.lib.activity-format.triggered-monitor.2"),
  "issue.monitor_cleared": t("ui.lib.activity-format.cleared-monitor.2"),
  "issue.monitor_skipped": t("ui.lib.activity-format.skipped-monitor.2"),
  "issue.monitor_exhausted": t("ui.lib.activity-format.exhausted-monitor.2"),
  "issue.monitor_recovery_wake_queued": t("ui.lib.activity-format.queued-monitor-recovery-wake"),
  "issue.monitor_recovery_issue_created": t("ui.lib.activity-format.created-monitor-recovery-issue"),
  "issue.monitor_escalated_to_board": t("ui.lib.activity-format.escalated-monitor-board"),
  "issue.deleted": t("ui.lib.activity-format.deleted-issue"),
  "issue.successful_run_handoff_required": t("ui.lib.activity-format.run-finished-without-clear"),
  "issue.successful_run_handoff_resolved": t("ui.lib.activity-format.next-step-chosen"),
  "issue.successful_run_handoff_escalated": t("ui.lib.activity-format.run-finished-without-next"),
  "issue.recovery_action_opened": t("ui.lib.activity-format.opened-source-scoped-recovery"),
  "issue.recovery_action_resolved": t("ui.lib.activity-format.resolved-recovery-action.2"),
  "issue.recovery_action_escalated": t("ui.lib.activity-format.escalated-recovery-action.2"),
  "issue.accepted_plan_decomposition_updated": t("ui.lib.activity-format.updated-accepted-plan-decomposition.2"),
  "agent.created": t("ui.lib.activity-format.created-agent"),
  "agent.updated": t("ui.lib.activity-format.updated-agent"),
  "agent.paused": t("ui.lib.activity-format.paused-agent"),
  "agent.resumed": t("ui.lib.activity-format.resumed-agent"),
  "agent.error_cleared": t("ui.lib.activity-format.cleared-agent-error"),
  "agent.terminated": t("ui.lib.activity-format.terminated-agent"),
  "heartbeat.invoked": t("ui.lib.activity-format.invoked-heartbeat.2"),
  "heartbeat.cancelled": t("ui.lib.activity-format.cancelled-heartbeat.2"),
  "heartbeat.output_stale_source_resolved": t("ui.lib.activity-format.system-folded-stale-run.2"),
  "heartbeat.output_stale_recovery_recursion_refused": t("ui.lib.activity-format.refused-recovery-recovery-escalation"),
  "approval.created": t("ui.lib.activity-format.requested-approval"),
  "approval.approved": "approved",
  "approval.rejected": "rejected",
};

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  return value as Record<string, unknown>;
}

function humanizeValue(value: unknown): string {
  if (typeof value !== "string") return String(value ?? "none");
  return value.replace(/_/g, " ");
}

function isActivityParticipant(value: unknown): value is ActivityParticipant {
  const record = asRecord(value);
  if (!record) return false;
  return record.type === "agent" || record.type === "user";
}

function isActivityIssueReference(value: unknown): value is ActivityIssueReference {
  return asRecord(value) !== null;
}

function readParticipants(details: ActivityDetails, key: string): ActivityParticipant[] {
  const value = details?.[key];
  if (!Array.isArray(value)) return [];
  return value.filter(isActivityParticipant);
}

function readIssueReferences(details: ActivityDetails, key: string): ActivityIssueReference[] {
  const value = details?.[key];
  if (!Array.isArray(value)) return [];
  return value.filter(isActivityIssueReference);
}

function formatUserLabel(userId: string | null | undefined, options: ActivityFormatOptions = {}): string {
  if (!userId || userId === "local-board") return t("components.activityRow.board");
  if (options.currentUserId && userId === options.currentUserId) return t("components.activityRow.you", { defaultValue: "You" });
  const profile = options.userProfileMap?.get(userId);
  if (profile) return profile.label;
  return `user ${userId.slice(0, 5)}`;
}

function formatParticipantLabel(participant: ActivityParticipant, options: ActivityFormatOptions): string {
  if (participant.type === "agent") {
    const agentId = participant.agentId ?? "";
    return options.agentMap?.get(agentId)?.name ?? "agent";
  }
  return formatUserLabel(participant.userId, options);
}

function formatIssueReferenceLabel(reference: ActivityIssueReference): string {
  if (reference.identifier) return reference.identifier;
  if (reference.title) return reference.title;
  if (reference.id) return reference.id.slice(0, 8);
  return "task";
}

function formatChangedEntityLabel(
  singular: string,
  plural: string,
  labels: string[],
): string {
  if (labels.length <= 0) return plural;
  if (labels.length === 1) return `${singular} ${labels[0]}`;
  return `${labels.length} ${plural}`;
}

function readNumber(value: unknown): number | null {
  if (typeof value === "number" && Number.isFinite(value)) return value;
  return null;
}

function readStringArrayLength(value: unknown): number {
  if (!Array.isArray(value)) return 0;
  return value.filter((entry) => typeof entry === "string" && entry.length > 0).length;
}

function formatAcceptedPlanDecompositionDetail(details: ActivityDetails): string | null {
  if (!details) return null;
  const status = typeof details.status === "string" ? details.status : null;
  const requested = readNumber(details.requestedChildCount);
  const totalChildren = readStringArrayLength(details.childIssueIds);
  const newlyCreated = readStringArrayLength(details.newlyCreatedChildIssueIds);
  const reused = Math.max(0, totalChildren - newlyCreated);
  const parts: string[] = [];
  if (newlyCreated > 0) parts.push(`created ${newlyCreated} new`);
  if (reused > 0) parts.push(`reused ${reused} existing`);
  if (parts.length === 0 && requested !== null) parts.push(`${requested} requested`);
  const summary = parts.length > 0 ? parts.join(", ") : null;
  if (status === "completed" && summary) return `decomposition completed (${summary})`;
  if (status === "completed") return t("ui.lib.activity-format.decomposition-completed");
  if (status === "in_flight" && summary) return `decomposition in flight (${summary})`;
  return summary;
}

function formatIssueUpdatedVerb(details: ActivityDetails): string | null {
  if (!details) return null;
  const previous = asRecord(details._previous) ?? {};
  if (details.status !== undefined) {
    const from = previous.status;
    return from
      ? `changed status from ${humanizeValue(from)} to ${humanizeValue(details.status)} on`
      : `changed status to ${humanizeValue(details.status)} on`;
  }
  if (details.priority !== undefined) {
    const from = previous.priority;
    return from
      ? `changed priority from ${humanizeValue(from)} to ${humanizeValue(details.priority)} on`
      : `changed priority to ${humanizeValue(details.priority)} on`;
  }
  return null;
}

function formatAssigneeName(details: ActivityDetails, options: ActivityFormatOptions): string | null {
  if (!details) return null;
  const agentId = details.assigneeAgentId;
  const userId = details.assigneeUserId;
  if (typeof agentId === "string" && agentId) {
    return options.agentMap?.get(agentId)?.name ?? "agent";
  }
  if (typeof userId === "string" && userId) {
    return formatUserLabel(userId, options);
  }
  return null;
}

function formatIssueUpdatedAction(details: ActivityDetails, options: ActivityFormatOptions = {}): string | null {
  if (!details) return null;
  const previous = asRecord(details._previous) ?? {};
  const parts: string[] = [];

  if (details.status !== undefined) {
    const from = previous.status;
    parts.push(
      from
        ? `changed the status from ${humanizeValue(from)} to ${humanizeValue(details.status)}`
        : `changed the status to ${humanizeValue(details.status)}`,
    );
  }
  if (details.priority !== undefined) {
    const from = previous.priority;
    parts.push(
      from
        ? `changed the priority from ${humanizeValue(from)} to ${humanizeValue(details.priority)}`
        : `changed the priority to ${humanizeValue(details.priority)}`,
    );
  }
  if (details.assigneeAgentId !== undefined || details.assigneeUserId !== undefined) {
    const assigneeName = formatAssigneeName(details, options);
    parts.push(assigneeName ? `made ${assigneeName} responsible for the task` : t("ui.lib.activity-format.cleared-responsible"));
  }
  if (details.title !== undefined) parts.push("updated the title");
  if (details.description !== undefined) parts.push("updated the description");

  return parts.length > 0 ? parts.join(", ") : null;
}

function formatStructuredIssueChange(input: {
  action: string;
  details: ActivityDetails;
  options: ActivityFormatOptions;
  forIssueDetail: boolean;
}): string | null {
  const details = input.details;
  if (!details) return null;

  if (input.action === "issue.blockers_updated") {
    const added = readIssueReferences(details, "addedBlockedByIssues").map(formatIssueReferenceLabel);
    const removed = readIssueReferences(details, "removedBlockedByIssues").map(formatIssueReferenceLabel);
    if (added.length > 0 && removed.length === 0) {
      const changed = formatChangedEntityLabel("blocker", "blockers", added);
      return input.forIssueDetail ? `added ${changed}` : `added ${changed} to`;
    }
    if (removed.length > 0 && added.length === 0) {
      const changed = formatChangedEntityLabel("blocker", "blockers", removed);
      return input.forIssueDetail ? `removed ${changed}` : `removed ${changed} from`;
    }
    return input.forIssueDetail ? t("ui.lib.activity-format.updated-blockers") : t("ui.lib.activity-format.updated-blockers.2");
  }

  if (input.action === "issue.reviewers_updated" || input.action === "issue.approvers_updated") {
    const added = readParticipants(details, "addedParticipants").map((participant) => formatParticipantLabel(participant, input.options));
    const removed = readParticipants(details, "removedParticipants").map((participant) => formatParticipantLabel(participant, input.options));
    const singular = input.action === "issue.reviewers_updated" ? "reviewer" : "approver";
    const plural = input.action === "issue.reviewers_updated" ? "reviewers" : "approvers";
    if (added.length > 0 && removed.length === 0) {
      const changed = formatChangedEntityLabel(singular, plural, added);
      return input.forIssueDetail ? `added ${changed}` : `added ${changed} to`;
    }
    if (removed.length > 0 && added.length === 0) {
      const changed = formatChangedEntityLabel(singular, plural, removed);
      return input.forIssueDetail ? `removed ${changed}` : `removed ${changed} from`;
    }
    return input.forIssueDetail ? `updated ${plural}` : `updated ${plural} on`;
  }

  return null;
}

export function formatActivityVerb(
  action: string,
  details?: Record<string, unknown> | null,
  options: ActivityFormatOptions = {},
): string {
  if (action === "issue.updated") {
    const issueUpdatedVerb = formatIssueUpdatedVerb(details);
    if (issueUpdatedVerb) return issueUpdatedVerb;
  }

  const structuredChange = formatStructuredIssueChange({
    action,
    details,
    options,
    forIssueDetail: false,
  });
  if (structuredChange) return structuredChange;

  return ACTIVITY_ROW_VERBS[action] ?? action.replace(/[._]/g, " ");
}

export function formatIssueActivityAction(
  action: string,
  details?: Record<string, unknown> | null,
  options: ActivityFormatOptions = {},
): string {
  if (action === "issue.updated") {
    const issueUpdatedAction = formatIssueUpdatedAction(details, options);
    if (issueUpdatedAction) return issueUpdatedAction;
  }

  const structuredChange = formatStructuredIssueChange({
    action,
    details,
    options,
    forIssueDetail: true,
  });
  if (structuredChange) return structuredChange;

  if (action === "issue.accepted_plan_decomposition_updated") {
    const detail = formatAcceptedPlanDecompositionDetail(details);
    if (detail) return detail;
  }

  if (action.startsWith("issue.monitor_") && details) {
    const serviceName = typeof details.serviceName === "string" && details.serviceName.trim()
      ? details.serviceName.trim()
      : null;
    const base = ISSUE_ACTIVITY_LABELS[action] ?? action.replace(/[._]/g, " ");
    return serviceName ? `${base} for ${serviceName}` : base;
  }

  if (
    (
      action === "issue.document_created" ||
      action === "issue.document_updated" ||
      action === "issue.document_locked" ||
      action === "issue.document_unlocked" ||
      action === "issue.document_deleted"
    ) &&
    details
  ) {
    const key = typeof details.key === "string" ? details.key : "document";
    const title = typeof details.title === "string" && details.title ? ` (${details.title})` : "";
    return `${ISSUE_ACTIVITY_LABELS[action] ?? action} ${key}${title}`;
  }

  return ISSUE_ACTIVITY_LABELS[action] ?? action.replace(/[._]/g, " ");
}
