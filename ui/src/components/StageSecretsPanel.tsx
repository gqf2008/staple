import { t } from "../i18n";
import { KeyRound, Save } from "lucide-react";
import type { CompanySecret, RoutineEnvConfig } from "@paperclipai/shared";
import { Button } from "@/components/ui/button";
import { EmptyState } from "./EmptyState";
import { EnvironmentVariablesEditor } from "./environment-variables-editor";
import { AgentIcon } from "./AgentIconPicker";

export interface StageSecretsPanelProps {
  /** Whether the stage has a backing automation routine with an assignee. */
  hasAutomation: boolean;
  /** Display name + icon of the agent that runs this step (when automation exists). */
  agentName?: string | null;
  agentIcon?: string | null;
  /** Company secret inventory (shared, not stage-scoped). */
  secrets: CompanySecret[];
  secretsLoading: boolean;
  value: RoutineEnvConfig;
  onChange: (env: RoutineEnvConfig) => void;
  onCreateSecret: (name: string, value: string) => Promise<CompanySecret>;
  /** Jump to the Automation section so the user can pick an agent. */
  onSetupAutomation: () => void;
  onSave: () => void;
  saving: boolean;
  dirty: boolean;
}

/**
 * Stage Secrets tab body. Stage secrets are env bindings on the step's backing
 * automation routine — the same company-secret backbone used by routines,
 * agents, and projects. This panel is intentionally dense and reuses
 * `EnvironmentVariablesEditor` for secret refs, inline secret creation, version
 * selection, and missing/disabled-secret warnings.
 */
export function StageSecretsPanel({
  hasAutomation,
  agentName,
  agentIcon,
  secrets,
  secretsLoading,
  value,
  onChange,
  onCreateSecret,
  onSetupAutomation,
  onSave,
  saving,
  dirty,
}: StageSecretsPanelProps) {
  // No backing automation/assignee → nothing can receive secrets at runtime.
  // Point the user at Automation instead of creating a hidden routine just
  // because the Secrets tab was opened.
  if (!hasAutomation) {
    return (
      <EmptyState
        icon={KeyRound}
        message={t("ui.components.stagesecretspanel.secrets-available-only-step")}
        action={t("ui.components.stagesecretspanel.set-up-automation")}
        onAction={onSetupAutomation}
      />
    );
  }

  const displayName = agentName?.trim() || t("ui.components.stagesecretspanel.fallback-responsible-agent");

  return (
    <div className="space-y-5">
      <div className="flex items-start gap-2 rounded-md border border-border bg-muted/20 px-4 py-3 text-xs text-muted-foreground">
        {agentName ? (
          <AgentIcon icon={agentIcon} className="h-3.5 w-3.5 mt-0.5 shrink-0" />
        ) : (
          <KeyRound className="h-3.5 w-3.5 mt-0.5 shrink-0" />
        )}
        <p>
          {t("ui.components.stagesecretspanel.these-env-vars-injected")}{" "}
          <span className="font-medium text-foreground">{displayName}</span> {t("ui.components.stagesecretspanel.runs-step-they-override")}<span className="font-mono">{t("ui.components.stagesecretspanel.paperclip")}</span> {t("ui.components.stagesecretspanel.names-reserved")}</p>
      </div>

      {secretsLoading ? (
        <p className="text-sm text-muted-foreground">{t("ui.components.stagesecretspanel.loading-secrets")}</p>
      ) : (
        <EnvironmentVariablesEditor
          value={value}
          secrets={secrets}
          onCreateSecret={onCreateSecret}
          onChange={(env) => onChange((env ?? {}) as RoutineEnvConfig)}
        />
      )}

      <div className="flex items-center gap-3">
        <Button type="button" onClick={onSave} disabled={!dirty || saving}>
          <Save className="h-4 w-4 mr-1.5" />
          {saving ? t("components.agentMultiSelect.saving") : t("ui.components.stagesecretspanel.save-secrets")}
        </Button>
        {dirty && !saving ? <span className="text-xs text-muted-foreground">{t("components.agentConfigForm.unsavedChanges")}</span> : null}
      </div>
    </div>
  );
}
