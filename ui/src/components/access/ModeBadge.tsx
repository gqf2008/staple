import { t } from "../../i18n";
import type { DeploymentExposure, DeploymentMode } from "@paperclipai/shared";
import { Badge } from "@/components/ui/badge";

export function ModeBadge({
  deploymentMode,
  deploymentExposure,
}: {
  deploymentMode?: DeploymentMode;
  deploymentExposure?: DeploymentExposure;
}) {
  if (!deploymentMode) return null;

  const label =
    deploymentMode === "local_trusted"
      ? t("ui.components.access.modebadge.local-trusted")
      : `Authenticated ${deploymentExposure ?? "private"}`;

  return <Badge variant="outline">{label}</Badge>;
}
