import { t } from "../i18n";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { brandChipBadge } from "@/lib/status-colors";
import type { BuiltInAgentStatus } from "@/api/builtInAgents";

/**
 * Derived lifecycle chip. Rendered for the amber attention states
 * (`needs_setup`, `pending_approval`). Kept separate from the real agent status
 * (`idle/active/…`) per ux-spec D1.
 */
export function BuiltInLifecycleChip({
  status,
  compact = false,
  className,
}: {
  status: BuiltInAgentStatus;
  compact?: boolean;
  className?: string;
}) {
  if (status !== "needs_setup" && status !== "pending_approval") return null;
  const isPendingApproval = status === "pending_approval";
  return (
    <Badge
      variant="outline"
      className={cn(
        brandChipBadge.amber,
        compact && "px-1.5 py-0 text-(length:--text-nano)",
        className,
      )}
      title={
        isPendingApproval
          ? t("ui.components.builtinagentbadges.waiting-board-hire-approval")
          : "Needs adapter/model setup before the feature can run"
      }
    >
      {isPendingApproval ? (compact ? t("components.issueProperties.approval") : t("components.resourceStatusChip.pendingApproval")) : compact ? t("ui.components.builtinagentbadges.setup") : t("components.resourceStatusChip.needsSetup")}
    </Badge>
  );
}
