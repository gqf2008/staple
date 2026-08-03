export type {
  AskUserQuestionsAnswer,
  AskUserQuestionsInteraction,
  AskUserQuestionsPayload,
  AskUserQuestionsQuestion,
  AskUserQuestionsQuestionOption,
  AskUserQuestionsResult,
  IssueThreadInteraction,
  IssueThreadInteractionActorFields,
  IssueThreadInteractionBase,
  IssueThreadInteractionContinuationPolicy,
  IssueThreadInteractionStatus,
  RequestCheckboxConfirmationInteraction,
  RequestCheckboxConfirmationOption,
  RequestCheckboxConfirmationPayload,
  RequestCheckboxConfirmationResult,
  RequestConfirmationInteraction,
  RequestConfirmationIssueDocumentTarget,
  RequestConfirmationPayload,
  RequestConfirmationResult,
  RequestConfirmationTarget,
  RequestConfirmationToolActionPayload,
  RequestConfirmationToolActionResult,
  RequestItemVerdictsInteraction,
  RequestItemVerdictsItem,
  RequestItemVerdictsPayload,
  RequestItemVerdictsResult,
  RequestItemVerdictsResultItem,
  RequestItemVerdictValue,
  SubmitIssueThreadInteractionVerdicts,
  SuggestedTaskDraft,
  SuggestTasksInteraction,
  SuggestTasksPayload,
  SuggestTasksResult,
  SuggestTasksResultCreatedTask,
} from "@paperclipai/shared";
import { t } from "../i18n";
import type {
  AskUserQuestionsAnswer,
  AskUserQuestionsInteraction,
  AskUserQuestionsQuestion,
  IssueThreadInteraction,
  RequestCheckboxConfirmationPayload,
  RequestCheckboxConfirmationResult,
  RequestConfirmationInteraction,
  RequestConfirmationTarget,
  RequestItemVerdictsInteraction,
  RequestItemVerdictsPayload,
  RequestItemVerdictsResult,
  RequestItemVerdictValue,
  SuggestedTaskDraft,
  SuggestTasksInteraction,
  SuggestTasksResultCreatedTask,
} from "@paperclipai/shared";

export interface SuggestedTaskTreeNode {
  task: SuggestedTaskDraft;
  children: SuggestedTaskTreeNode[];
}

export function isIssueThreadInteraction(
  value: unknown,
): value is IssueThreadInteraction {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<IssueThreadInteraction>;
  return typeof candidate.id === "string"
    && typeof candidate.companyId === "string"
    && typeof candidate.issueId === "string"
    && (
      candidate.kind === "suggest_tasks"
      || candidate.kind === "ask_user_questions"
      || candidate.kind === "request_confirmation"
      || candidate.kind === "request_checkbox_confirmation"
      || candidate.kind === "request_item_verdicts"
    );
}

export interface ItemVerdictProgress {
  total: number;
  decided: number;
  approved: number;
  rejected: number;
  deferred: number;
  /** ids in payload order that still have no verdict. */
  pendingItemIds: string[];
}

/**
 * Derive the `M of N decided` progress for a per-item verdict interaction from
 * its payload (the full item roster) and result (verdicts accumulated so far).
 * Present-tense verdict values (`approve`/`reject`/`defer`) are what the server
 * stores in `result.items[].verdict` (see PAP-13247).
 */
export function getItemVerdictProgress(args: {
  payload: RequestItemVerdictsPayload;
  result?: RequestItemVerdictsResult | null;
}): ItemVerdictProgress {
  const { payload, result } = args;
  const resolvedById = new Map<string, RequestItemVerdictValue>(
    (result?.items ?? []).map((item) => [item.id, item.verdict] as const),
  );
  let approved = 0;
  let rejected = 0;
  let deferred = 0;
  const pendingItemIds: string[] = [];
  for (const item of payload.items) {
    const verdict = resolvedById.get(item.id);
    if (verdict === "approve") approved += 1;
    else if (verdict === "reject") rejected += 1;
    else if (verdict === "defer") deferred += 1;
    else pendingItemIds.push(item.id);
  }
  const decided = approved + rejected + deferred;
  return { total: payload.items.length, decided, approved, rejected, deferred, pendingItemIds };
}

export function buildItemVerdictsSummary(
  interaction: RequestItemVerdictsInteraction,
): string {
  const progress = getItemVerdictProgress({
    payload: interaction.payload,
    result: interaction.result,
  });
  if (interaction.status === "answered") {
    const parts = [`${progress.decided} decided`];
    if (progress.approved > 0) parts.push(`${progress.approved} approved`);
    if (progress.rejected > 0) parts.push(`${progress.rejected} rejected`);
    if (progress.deferred > 0) parts.push(`${progress.deferred} deferred`);
    return parts.join(" · ");
  }
  if (interaction.status === "expired") {
    const outcome = interaction.result?.outcome;
    if (outcome === "superseded_by_comment") return t("ui.lib.issue-thread-interactions.verdicts-expired-after-comment");
    if (outcome === "stale_target") return t("ui.lib.issue-thread-interactions.verdicts-expired-after-target");
    return t("ui.lib.issue-thread-interactions.verdicts-expired");
  }
  return `${progress.decided} of ${progress.total} decided`;
}

export function getCheckboxConfirmationSelectedLabels(args: {
  payload: RequestCheckboxConfirmationPayload;
  result?: RequestCheckboxConfirmationResult | null;
}): string[] {
  const { payload, result } = args;
  const selectedIds = result?.selectedOptionIds ?? [];
  const optionLabelById = new Map(
    payload.options.map((option) => [option.id, option.label] as const),
  );
  return selectedIds
    .map((optionId) => optionLabelById.get(optionId))
    .filter((label): label is string => typeof label === "string");
}

export function normalizeRequestConfirmationTargetHref(href: string) {
  const value = href.trim();
  if (value.startsWith("#")) return value;
  if (value.startsWith("/")) return value.startsWith("//") ? null : value;
  return /^https?:\/\//i.test(value) ? value : null;
}

export function getRequestConfirmationTargetHref({
  issueId,
  target,
}: {
  issueId: string;
  target: RequestConfirmationTarget;
}) {
  if (target.href) {
    const safeHref = normalizeRequestConfirmationTargetHref(target.href);
    if (safeHref) return safeHref;
  }
  if (target.type === "issue_document") {
    const targetIssueId = target.issueId ?? issueId;
    return `/issues/${targetIssueId}#document-${encodeURIComponent(target.key)}`;
  }
  return null;
}

export function buildIssueThreadInteractionSummary(
  interaction: IssueThreadInteraction,
) {
  const administrativeOutcome = interaction.result && "outcome" in interaction.result
    ? interaction.result.outcome
    : null;
  if (administrativeOutcome === "withdrawn") return t("ui.lib.issue-thread-interactions.withdrawn-interaction");
  if (administrativeOutcome === "issue_closed") return t("components.issueThreadInteraction.expiredWhenClosed");
  if (interaction.kind === "suggest_tasks") {
    const count = interaction.payload.tasks.length;
    if (interaction.status === "accepted") {
      const createdCount = interaction.result?.createdTasks?.length ?? 0;
      const skippedCount = interaction.result?.skippedClientKeys?.length ?? 0;
      if (skippedCount > 0) {
        return `Accepted ${createdCount} of ${count} tasks`;
      }
      return createdCount === 1 ? t("ui.lib.issue-thread-interactions.accepted-task") : `Accepted ${createdCount} tasks`;
    }
    if (interaction.status === "rejected") {
      return count === 1 ? t("ui.lib.issue-thread-interactions.rejected-task") : `Rejected ${count} tasks`;
    }
    return count === 1 ? t("ui.lib.issue-thread-interactions.suggested-task") : `Suggested ${count} tasks`;
  }

  if (interaction.kind === "request_confirmation") {
    if (interaction.status === "accepted") return t("ui.lib.issue-thread-interactions.confirmed-request");
    if (interaction.status === "rejected") return t("ui.lib.issue-thread-interactions.declined-request");
    if (interaction.status === "expired") {
      const outcome = interaction.result?.outcome;
      if (outcome === "superseded_by_comment") return t("ui.lib.issue-thread-interactions.confirmation-expired-after-comment");
      if (outcome === "stale_target") return t("ui.lib.issue-thread-interactions.confirmation-expired-after-target");
      return t("ui.lib.issue-thread-interactions.confirmation-expired");
    }
    return t("ui.lib.issue-thread-interactions.requested-confirmation");
  }

  if (interaction.kind === "request_checkbox_confirmation") {
    const optionCount = interaction.payload.options.length;
    if (interaction.status === "accepted") {
      const selectedCount = interaction.result?.selectedOptionIds?.length ?? 0;
      if (selectedCount === 0) return t("components.issueThreadInteraction.confirmedNoOptions");
      return selectedCount === 1
        ? `Confirmed 1 of ${optionCount} options`
        : `Confirmed ${selectedCount} of ${optionCount} options`;
    }
    if (interaction.status === "rejected") return t("ui.lib.issue-thread-interactions.declined-selection");
    if (interaction.status === "expired") {
      const outcome = interaction.result?.outcome;
      if (outcome === "superseded_by_comment") return t("ui.lib.issue-thread-interactions.selection-expired-after-comment");
      if (outcome === "stale_target") return t("ui.lib.issue-thread-interactions.selection-expired-after-target");
      return t("ui.lib.issue-thread-interactions.selection-expired");
    }
    return optionCount === 1
      ? t("ui.lib.issue-thread-interactions.requested-selection-from-option")
      : `Requested a selection from ${optionCount} options`;
  }

  if (interaction.kind === "request_item_verdicts") {
    return buildItemVerdictsSummary(interaction);
  }

  const count = interaction.payload.questions.length;
  if (interaction.status === "answered") {
    return count === 1 ? t("ui.lib.issue-thread-interactions.answered-question") : `Answered ${count} questions`;
  }
  if (interaction.status === "cancelled") {
    return count === 1 ? t("ui.lib.issue-thread-interactions.cancelled-question") : `Cancelled ${count} questions`;
  }
  if (interaction.status === "expired") {
    if (interaction.result?.expirationReason === "superseded_by_comment") {
      return count === 1 ? t("ui.lib.issue-thread-interactions.question-expired-after-comment") : t("ui.lib.issue-thread-interactions.questions-expired-after-comment");
    }
    return count === 1 ? t("ui.lib.issue-thread-interactions.question-expired") : t("ui.lib.issue-thread-interactions.questions-expired");
  }
  return count === 1 ? t("ui.lib.issue-thread-interactions.asked-question") : `Asked ${count} questions`;
}

export function buildSuggestedTaskTree(
  tasks: readonly SuggestedTaskDraft[],
): SuggestedTaskTreeNode[] {
  const nodes = new Map<string, SuggestedTaskTreeNode>();
  for (const task of tasks) {
    nodes.set(task.clientKey, { task, children: [] });
  }

  const roots: SuggestedTaskTreeNode[] = [];
  for (const task of tasks) {
    const node = nodes.get(task.clientKey);
    if (!node) continue;
    const parentNode = task.parentClientKey ? nodes.get(task.parentClientKey) : null;
    if (parentNode) {
      parentNode.children.push(node);
      continue;
    }
    roots.push(node);
  }

  return roots;
}

export function countSuggestedTaskNodes(node: SuggestedTaskTreeNode): number {
  return 1 + node.children.reduce((sum, child) => sum + countSuggestedTaskNodes(child), 0);
}

export function collectSuggestedTaskClientKeys(node: SuggestedTaskTreeNode): string[] {
  return [
    node.task.clientKey,
    ...node.children.flatMap((child) => collectSuggestedTaskClientKeys(child)),
  ];
}

export function getQuestionAnswerLabels(args: {
  question: AskUserQuestionsQuestion;
  answers: readonly AskUserQuestionsAnswer[];
}) {
  const { question, answers } = args;
  const answer = answers.find((candidate) => candidate.questionId === question.id);
  const selectedIds = answer?.optionIds ?? [];
  const optionLabelById = new Map(
    question.options.map((option) => [option.id, option.label] as const),
  );
  const labels = selectedIds
    .map((optionId) => optionLabelById.get(optionId))
    .filter((label): label is string => typeof label === "string");
  const otherText = answer?.otherText?.trim();
  if (otherText) labels.push(`Other: ${otherText}`);
  return labels;
}
