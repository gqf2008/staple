import { t } from "../../i18n";
/**
 * Canonical state inventory for the Task Chat Redesign (flag:
 * enableTaskChatRedesign).
 *
 * This list is the single source of truth for:
 *   - the dev harness state switcher (/dev/task-chat-lab), and
 *   - the finish-line test that asserts every state renders without error.
 *
 * Each id traces to a real agent-protocol state (plan Deliverable 1). `tier`
 * marks whether the state already streams live ("live") or is emitted upstream
 * but dropped by acpx today ("tier-b", driven by synthetic events in the
 * harness, live wiring flagged). `surface` says where the state renders.
 */

export const TASK_CHAT_STATES = [
  "session-start",
  "human-message",
  "agent-message",
  "thinking",
  "responding",
  "tool-call",
  "diff",
  "working",
  "running",
  "completed",
  "awaiting-approval",
  "plan-todo",
  "interrupted",
  "refused",
  "truncated",
  "live-token-cost",
] as const;

export type TaskChatStateId = (typeof TASK_CHAT_STATES)[number];

export type TaskChatStateTier = "live" | "tier-b";
export type TaskChatStateSurface = "thread" | "plan";

export interface TaskChatStateMeta {
  id: TaskChatStateId;
  label: string;
  tier: TaskChatStateTier;
  surface: TaskChatStateSurface;
  /** Real protocol source, quoted for the harness inspector. */
  protocol: string;
}

export const TASK_CHAT_STATE_META: Record<TaskChatStateId, TaskChatStateMeta> = {
  "session-start": {
    id: "session-start",
    label: t("ui.components.task-chat.task-chat-states.session-start"),
    tier: "live",
    surface: "thread",
    protocol: 'acpx.session → TranscriptEntry kind:"init"',
  },
  "human-message": {
    id: "human-message",
    label: t("ui.components.task-chat.task-chat-states.human-message"),
    tier: "live",
    surface: "thread",
    protocol: 'IssueComment authorType:"user"',
  },
  "agent-message": {
    id: "agent-message",
    label: t("ui.components.task-chat.task-chat-states.agent-message"),
    tier: "live",
    surface: "thread",
    protocol: t("ui.components.task-chat.task-chat-states.text-delta-stream-output"),
  },
  thinking: {
    id: "thinking",
    label: t("ui.components.task-chat.task-chat-states.thinking"),
    tier: "live",
    surface: "thread",
    protocol: t("ui.components.task-chat.task-chat-states.text-delta-stream-thought"),
  },
  responding: {
    id: "responding",
    label: t("ui.components.task-chat.task-chat-states.responding-streaming"),
    tier: "live",
    surface: "thread",
    protocol: t("ui.components.task-chat.task-chat-states.text-delta-stream-output.2"),
  },
  "tool-call": {
    id: "tool-call",
    label: t("ui.components.task-chat.task-chat-states.tool-call"),
    tier: "live",
    surface: "thread",
    protocol: "acpx.tool_call (ACP tool_call / tool_call_update)",
  },
  diff: {
    id: "diff",
    label: t("ui.components.task-chat.task-chat-states.diff"),
    tier: "live",
    surface: "thread",
    protocol: 'ToolCallContent type:"diff" → TranscriptEntry kind:"diff"',
  },
  working: {
    id: "working",
    label: "Working",
    tier: "live",
    surface: "thread",
    protocol: t("ui.components.task-chat.task-chat-states.heartbeat-run-progress-acpx"),
  },
  running: {
    id: "running",
    label: "Running",
    tier: "live",
    surface: "thread",
    protocol: 'message.status.type === "running"',
  },
  completed: {
    id: "completed",
    label: t("ui.components.task-chat.task-chat-states.completed-collapsed"),
    tier: "live",
    surface: "thread",
    protocol: "acpx.result (StopReason in subtype)",
  },
  "awaiting-approval": {
    id: "awaiting-approval",
    label: t("components.issueThreadInteraction.awaitingApproval"),
    tier: "tier-b",
    surface: "thread",
    protocol: "ACP RequestPermissionRequest + PermissionOptionKind",
  },
  "plan-todo": {
    id: "plan-todo",
    label: "Plan / todo",
    tier: "tier-b",
    surface: "plan",
    protocol: "ACP Plan { entries: PlanEntry[] }, PlanEntryStatus",
  },
  interrupted: {
    id: "interrupted",
    label: t("ui.components.task-chat.task-chat-states.interrupted"),
    tier: "tier-b",
    surface: "thread",
    protocol: 'AcpRuntimeTurnResult.status:"cancelled" / StopReason "cancelled"',
  },
  refused: {
    id: "refused",
    label: t("ui.components.task-chat.task-chat-states.refused"),
    tier: "tier-b",
    surface: "thread",
    protocol: 'StopReason "refusal"',
  },
  truncated: {
    id: "truncated",
    label: t("ui.components.task-chat.task-chat-states.truncated"),
    tier: "tier-b",
    surface: "thread",
    protocol: 'StopReason "max_tokens" | "max_turn_requests"',
  },
  "live-token-cost": {
    id: "live-token-cost",
    label: "Live token / cost",
    tier: "tier-b",
    surface: "thread",
    protocol: "ACP UsageUpdate { used, size, cost }",
  },
};

export const TASK_CHAT_STATE_LIST: TaskChatStateMeta[] = TASK_CHAT_STATES.map(
  (id) => TASK_CHAT_STATE_META[id],
);
