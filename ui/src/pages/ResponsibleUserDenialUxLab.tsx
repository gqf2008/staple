import type { ReactNode } from "react";
import { t } from "../i18n";
import { ResponsibleUserDenialNotice } from "@/components/ResponsibleUserDenialNotice";
import { cn } from "@/lib/utils";
import { Card } from "@/components/ui/card";

/**
 * UX lab for PAP-12462 (P7): run "on behalf of {user}" surfacing + responsible-user
 * denial copy. Renders before/after of both surfaces with real design tokens so the
 * states can be captured for UX review. Route: /ux-lab/responsible-user-denial
 */

function LabSection({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <section className="rounded-2xl border border-border/70 bg-background/85 p-5 shadow-sm">
      <div className="mb-4">
        <h2 className="text-base font-semibold text-foreground">{title}</h2>
        <p className="mt-1 text-sm text-muted-foreground">{description}</p>
      </div>
      <div className="grid gap-4 lg:grid-cols-2">{children}</div>
    </section>
  );
}

function BeforeAfter({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="space-y-2">
      <div className="text-(length:--text-micro) font-semibold uppercase tracking-(--tracking-caps) text-muted-foreground">
        {label}
      </div>
      <Card className="block border-border/60 p-3">{children}</Card>
    </div>
  );
}

/** A faithful copy of a run ledger row header (see IssueRunLedger.tsx). */
function RunLedgerRow({
  onBehalfOf,
  denial,
}: {
  onBehalfOf?: string | null;
  denial?: ReactNode;
}) {
  return (
    <article className="space-y-1.5 rounded-lg border border-border/60 px-3 py-2 text-xs text-muted-foreground">
      <div className="flex flex-wrap items-center gap-1.5">
        <span className="font-medium text-foreground">{t("pages.responsibleUserDenialUxLab.run", { defaultValue: "Run" })}</span>
        <span className="min-w-0 max-w-full truncate font-mono text-foreground">a1b2c3d4</span>
        <span>{t("ui.pages.responsibleuserdenialuxlab.codexcoder")}</span>
        {onBehalfOf ? (
          <span className="min-w-0 max-w-full truncate text-muted-foreground">
            {t("ui.components.issuerunledger.behalf")}<span className="text-foreground">{onBehalfOf}</span>
          </span>
        ) : null}
        <span className="rounded-md border border-border px-1.5 py-0.5 text-(length:--text-micro) capitalize text-muted-foreground">
          {denial ? t("pages.responsibleUserDenialUxLab.failed", { defaultValue: "Failed" }) : t("pages.responsibleUserDenialUxLab.succeeded", { defaultValue: "Succeeded" })}
        </span>
        <span className="ml-auto shrink-0">{t("ui.pages.responsibleuserdenialuxlab.2m-ago")}</span>
      </div>
      <div className="grid gap-2 text-xs text-muted-foreground sm:grid-cols-3">
        <div className="min-w-0">
          <span className="text-foreground">{t("pages.responsibleUserDenialUxLab.elapsed", { defaultValue: "Elapsed" })}</span> 1m 4s
        </div>
        <div className="min-w-0">
          <span className="text-foreground">{t("pages.responsibleUserDenialUxLab.lastAction", { defaultValue: "Last useful action" })}</span> {t("ui.pages.responsibleuserdenialuxlab.2m-ago")}</div>
        <div className="min-w-0">
          <span className="text-foreground">{t("pages.responsibleUserDenialUxLab.stop", { defaultValue: "Stop" })}</span> {denial ? t("pages.responsibleUserDenialUxLab.denied", { defaultValue: "Denied" }) : t("pages.responsibleUserDenialUxLab.completed", { defaultValue: "Completed" })}
        </div>
      </div>
      {denial}
    </article>
  );
}

/** A faithful copy of the run-detail header identity block (see AgentDetail.tsx RunDetail). */
function RunDetailHeader({ onBehalfOf, denial }: { onBehalfOf?: string | null; denial?: ReactNode }) {
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <span className="text-lg font-semibold text-foreground">{t("pages.responsibleUserDenialUxLab.runId", { defaultValue: "Run a1b2c3d4" })}</span>
        <span className="rounded-md border border-border px-1.5 py-0.5 text-(length:--text-micro) capitalize text-muted-foreground">
          {denial ? "failed" : "succeeded"}
        </span>
      </div>
      <div className="flex flex-wrap items-center gap-1.5 font-mono text-(length:--text-micro) text-muted-foreground">
        <span className="rounded bg-muted px-1.5 py-0.5 text-(length:--text-nano) font-medium uppercase tracking-wide">
          {t("ui.pages.responsibleuserdenialuxlab.codex-local")}</span>
        <span>{t("ui.pages.responsibleuserdenialuxlab.anthropic-claude-opus")}</span>
      </div>
      {onBehalfOf ? (
        <div className="text-xs text-muted-foreground">
          {t("ui.pages.agentdetail.behalf")}<span className="text-foreground">{onBehalfOf}</span>
        </div>
      ) : null}
      {denial}
    </div>
  );
}

export function ResponsibleUserDenialUxLab() {
  return (
    <div className="min-h-screen bg-muted/20 p-6">
      <div className="mx-auto max-w-5xl space-y-6">
        <header>
          <div className="text-(length:--text-micro) font-semibold uppercase tracking-(--tracking-caps) text-muted-foreground">
            PAP-12462 · P7
          </div>
          <h1 className="mt-1 text-xl font-semibold text-foreground">
            {t("ui.pages.responsibleuserdenialuxlab.run-behalf-surfacing-denial")}</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            {t("ui.pages.responsibleuserdenialuxlab.before-after-two-run")}</p>
        </header>

        <LabSection
          title="1 · Run identity — “on behalf of {user}”"
          description={t("ui.pages.responsibleuserdenialuxlab.run-acting-human-now")}
        >
          <BeforeAfter label={t("pages.responsibleUserDenialUxLab.beforeLedger", { defaultValue: "Before — run ledger" })}>
            <RunLedgerRow />
          </BeforeAfter>
          <BeforeAfter label={t("pages.responsibleUserDenialUxLab.afterLedger", { defaultValue: "After — run ledger" })}>
            <RunLedgerRow onBehalfOf={t("ui.pages.responsibleuserdenialuxlab.ada-lovelace.2")} />
          </BeforeAfter>
          <BeforeAfter label={t("pages.responsibleUserDenialUxLab.beforeDetail", { defaultValue: "Before — run detail" })}>
            <RunDetailHeader />
          </BeforeAfter>
          <BeforeAfter label={t("pages.responsibleUserDenialUxLab.afterDetail", { defaultValue: "After — run detail" })}>
            <RunDetailHeader onBehalfOf={t("ui.pages.responsibleuserdenialuxlab.ada-lovelace.2")} />
          </BeforeAfter>
        </LabSection>

        <LabSection
          title={t("ui.pages.responsibleuserdenialuxlab.denial-state-responsible-user")}
          description={t("ui.pages.responsibleuserdenialuxlab.agent-allowed-but-user")}
        >
          <BeforeAfter label={t("pages.responsibleUserDenialUxLab.beforeGeneric", { defaultValue: "Before — generic failure text" })}>
            <div className="text-xs">
              <span className="text-red-600 dark:text-red-400">
                {t("ui.pages.responsibleuserdenialuxlab.forbidden-action-not-permitted")}</span>
              <span className="ml-1 text-muted-foreground">{t("ui.pages.responsibleuserdenialuxlab.responsible-user-unauthorized")}</span>
            </div>
          </BeforeAfter>
          <BeforeAfter label={t("pages.responsibleUserDenialUxLab.afterActionable", { defaultValue: "After — actionable denial copy" })}>
            <ResponsibleUserDenialNotice
              code="RESPONSIBLE_USER_UNAUTHORIZED"
              userName={t("ui.pages.responsibleuserdenialuxlab.ada-lovelace.2")}
            />
          </BeforeAfter>
        </LabSection>

        <LabSection
          title={t("ui.pages.responsibleuserdenialuxlab.denial-state-agent-lacks")}
          description={t("ui.pages.responsibleuserdenialuxlab.denial-not-responsible-user")}
        >
          <BeforeAfter label={t("pages.responsibleUserDenialUxLab.agentLacksPermission", { defaultValue: "Agent-lacks-permission failure" })}>
            <div className="text-xs">
              <span className="text-red-600 dark:text-red-400">
                {t("ui.pages.responsibleuserdenialuxlab.forbidden-agent-not-permitted")}</span>
              <span className="ml-1 text-muted-foreground">{t("ui.pages.responsibleuserdenialuxlab.deny-missing-membership")}</span>
            </div>
          </BeforeAfter>
          <BeforeAfter label={t("pages.responsibleUserDenialUxLab.noNotice", { defaultValue: "No responsible-user notice rendered" })}>
            <div className="text-xs text-muted-foreground">
              {t("ui.pages.responsibleuserdenialuxlab.responsible-user-denial-notice")}</div>
          </BeforeAfter>
        </LabSection>

        <LabSection
          title={t("ui.pages.responsibleuserdenialuxlab.denial-state-responsible-user-alt")}
          description={t("ui.pages.responsibleuserdenialuxlab.user-run-acts-was")}
        >
          <BeforeAfter label={t("pages.responsibleUserDenialUxLab.beforeGeneric", { defaultValue: "Before — generic failure text" })}>
            <div className="text-xs">
              <span className="text-red-600 dark:text-red-400">
                {t("ui.pages.responsibleuserdenialuxlab.forbidden-responsible-user-unavailable")}</span>
              <span className="ml-1 text-muted-foreground">{t("ui.pages.responsibleuserdenialuxlab.responsible-user-unavailable")}</span>
            </div>
          </BeforeAfter>
          <BeforeAfter label={t("pages.responsibleUserDenialUxLab.afterActionable", { defaultValue: "After — actionable denial copy" })}>
            <ResponsibleUserDenialNotice
              code="RESPONSIBLE_USER_UNAVAILABLE"
              userName={t("ui.pages.responsibleuserdenialuxlab.grace-hopper.2")}
            />
          </BeforeAfter>
        </LabSection>

        <LabSection
          title={t("pages.responsibleUserDenialUxLab.inContext", { defaultValue: "In-context — denial inside a failed run ledger row" })}
          description={t("pages.responsibleUserDenialUxLab.inContextDesc", { defaultValue: "How the notice reads within a run row on the issue timeline." })}
        >
          <BeforeAfter label={t("pages.responsibleUserDenialUxLab.unauthorized", { defaultValue: "Unauthorized" })}>
            <RunLedgerRow
              onBehalfOf={t("ui.pages.responsibleuserdenialuxlab.ada-lovelace.2")}
              denial={
                <ResponsibleUserDenialNotice
                  code="RESPONSIBLE_USER_UNAUTHORIZED"
                  userName={t("ui.pages.responsibleuserdenialuxlab.ada-lovelace.2")}
                />
              }
            />
          </BeforeAfter>
          <BeforeAfter label={t("pages.responsibleUserDenialUxLab.unavailable", { defaultValue: "Unavailable" })}>
            <RunLedgerRow
              onBehalfOf={t("ui.pages.responsibleuserdenialuxlab.grace-hopper.2")}
              denial={
                <ResponsibleUserDenialNotice
                  code="RESPONSIBLE_USER_UNAVAILABLE"
                  userName={t("ui.pages.responsibleuserdenialuxlab.grace-hopper.2")}
                />
              }
            />
          </BeforeAfter>
        </LabSection>

        <p className={cn("text-center text-(length:--text-micro) text-muted-foreground")}>
          {t("ui.pages.responsibleuserdenialuxlab.copy-sourced-from-shared")}<code>{t("ui.pages.responsibleuserdenialuxlab.describeresponsibleuserdenial")}</code> contract.
        </p>
      </div>
    </div>
  );
}
