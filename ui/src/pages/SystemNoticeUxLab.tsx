// token-extraction: allowlisted — intentional one-off decoration (DECISION-SHEET.md B1
// user ruling). The bg-[...gradient...] / shadow-[...] literals in this demo/UX-lab page
// are deliberate one-off decoration, reverted from --gradient-extract-*/--shadow-extract-*
// tokens; the file is on the check-token-gates allowlist in ui/src/index.css.
import type { ReactNode } from "react";
import { t } from "../i18n";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { SystemNotice } from "@/components/SystemNotice";
import { systemNoticeFixtures } from "@/fixtures/systemNoticeFixtures";
import { cn } from "@/lib/utils";
import {
  CircleDashed,
  FlaskConical,
  Layers,
  ListChecks,
  Sparkles,
} from "lucide-react";

function LabSection({
  id,
  eyebrow,
  title,
  description,
  accentClassName,
  children,
}: {
  id?: string;
  eyebrow: string;
  title: string;
  description: string;
  accentClassName?: string;
  children: ReactNode;
}) {
  return (
    <section
      id={id}
      className={cn(
        "rounded-(--rad-28) border border-border/70 bg-background/85 p-4 shadow-[0_24px_60px_rgba(15,23,42,0.08)] sm:p-5",
        accentClassName,
      )}
    >
      <div className="mb-4 flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="text-(length:--text-micro) font-semibold uppercase tracking-(--tracking-caps) text-muted-foreground">
            {eyebrow}
          </div>
          <h2 className="mt-1 text-xl font-semibold tracking-tight">{title}</h2>
          <p className="mt-2 max-w-3xl text-sm text-muted-foreground">{description}</p>
        </div>
      </div>
      {children}
    </section>
  );
}

function FixtureFrame({ caption, children }: { caption: string; children: ReactNode }) {
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 text-(length:--text-micro) font-semibold uppercase tracking-(--tracking-eyebrow) text-muted-foreground">
        <CircleDashed className="h-3.5 w-3.5" />
        {caption}
      </div>
      {children}
    </div>
  );
}

function MockUserBubble({
  authorName,
  body,
  alignEnd,
}: {
  authorName: string;
  body: string;
  alignEnd?: boolean;
}) {
  return (
    <div className={cn("flex items-start gap-2.5", alignEnd && "justify-end")}>
      {!alignEnd ? (
        <Avatar size="sm" className="shrink-0">
          <AvatarFallback>{authorName.slice(0, 2).toUpperCase()}</AvatarFallback>
        </Avatar>
      ) : null}
      <div className={cn("flex min-w-0 max-w-(--pct-85) flex-col", alignEnd && "items-end")}>
        <div
          className={cn(
            "mb-1 px-1 text-sm font-medium text-foreground",
            alignEnd ? "text-right" : "text-left",
          )}
        >
          {authorName}
        </div>
        <div className="min-w-0 max-w-full rounded-2xl bg-muted px-4 py-2.5 text-sm leading-6 text-foreground">
          {body}
        </div>
      </div>
      {alignEnd ? (
        <Avatar size="sm" className="shrink-0">
          <AvatarFallback>{authorName.slice(0, 2).toUpperCase()}</AvatarFallback>
        </Avatar>
      ) : null}
    </div>
  );
}

function MockAgentBubble({ agentName, body }: { agentName: string; body: string }) {
  return (
    <div className="flex items-start gap-2.5">
      <Avatar size="sm" className="shrink-0">
        <AvatarFallback>{agentName.slice(0, 2).toUpperCase()}</AvatarFallback>
      </Avatar>
      <div className="flex min-w-0 max-w-(--pct-85) flex-col">
        <div className="mb-1 px-1 text-sm font-medium text-foreground">{agentName}</div>
        <div className="min-w-0 max-w-full rounded-2xl border border-border/70 bg-background px-4 py-2.5 text-sm leading-6 text-foreground">
          {body}
        </div>
      </div>
    </div>
  );
}

const checklist = [
  t("pages.systemNoticeUxLab.oneContainer", { defaultValue: "One container per system notice — no nested chat bubble" }),
  t("pages.systemNoticeUxLab.toneByIcon", { defaultValue: "Tone communicated by icon + label, never color alone" }),
  t("pages.systemNoticeUxLab.evidenceBehindDetails", { defaultValue: "Operational evidence hidden behind Details, expanded only on demand" }),
  t("pages.systemNoticeUxLab.typedLinkRows", { defaultValue: "Issue, agent, and run metadata render as typed link rows, not raw markdown" }),
  t("pages.systemNoticeUxLab.hierarchyDistinct", { defaultValue: "Hierarchy visibly distinct from user (right-aligned) and agent (left-aligned) bubbles" }),
];

export function SystemNoticeUxLab() {
  const fixtureById = new Map(systemNoticeFixtures.map((f) => [f.id, f] as const));

  const warningCollapsed = fixtureById.get("warning-collapsed")!;
  const warningExpanded = fixtureById.get("warning-expanded")!;
  const dangerCollapsed = fixtureById.get("danger-collapsed")!;
  const dangerExpanded = fixtureById.get("danger-expanded")!;
  const neutralCollapsed = fixtureById.get("neutral-collapsed")!;
  const neutralExpanded = fixtureById.get("neutral-expanded")!;
  const warningNoDetails = fixtureById.get("warning-no-details")!;

  return (
    <div className="space-y-6">
      <div className="overflow-hidden rounded-(--rad-32) border border-border/70 bg-[linear-gradient(135deg,rgba(245,158,11,0.10),transparent_28%),linear-gradient(180deg,rgba(8,145,178,0.08),transparent_44%),var(--background)] shadow-[0_30px_80px_rgba(15,23,42,0.10)]">
        <div className="grid gap-6 lg:grid-cols-(--gtc-39)">
          <div className="p-6 sm:p-7">
            <div className="inline-flex items-center gap-2 rounded-full border border-amber-500/25 bg-amber-500/[0.08] px-3 py-1 text-(length:--text-nano) font-semibold uppercase tracking-(--tracking-caps) text-amber-700 dark:text-amber-300">
              <FlaskConical className="h-3.5 w-3.5" />
              {t("ui.pages.systemnoticeuxlab.system-notice-lab")}</div>
            <h1 className="mt-4 text-3xl font-semibold tracking-tight">
              {t("ui.pages.systemnoticeuxlab.first-class-system-notice")}</h1>
            <p className="mt-3 max-w-3xl text-sm leading-6 text-muted-foreground">
              {t("ui.pages.systemnoticeuxlab.replaces-current-pattern-where")}</p>

            <div className="mt-5 flex flex-wrap items-center gap-2">
              <Badge variant="outline" className="rounded-full px-3 py-1 text-(length:--text-nano) uppercase tracking-(--tracking-caps)">
                {t("ui.pages.systemnoticeuxlab.pap-3525-plan")}</Badge>
              <Badge variant="outline" className="rounded-full px-3 py-1 text-(length:--text-nano) uppercase tracking-(--tracking-caps)">
                {t("ui.pages.systemnoticeuxlab.phase-ux")}</Badge>
              <Badge variant="outline" className="rounded-full px-3 py-1 text-(length:--text-nano) uppercase tracking-(--tracking-caps)">
                {t("ui.pages.systemnoticeuxlab.tones-warning-danger-neutral")}</Badge>
            </div>
          </div>

          <aside className="border-t border-border/60 bg-background/70 p-6 lg:border-l lg:border-t-0">
            <div className="mb-4 flex items-center gap-2 text-(length:--text-micro) font-semibold uppercase tracking-(--tracking-caps) text-muted-foreground">
              <ListChecks className="h-4 w-4 text-amber-700 dark:text-amber-300" />
              {t("ui.pages.systemnoticeuxlab.what-lab-proves")}</div>
            <div className="space-y-3">
              {checklist.map((line) => (
                <div
                  key={line}
                  className="rounded-2xl border border-border/70 bg-background/85 px-4 py-3 text-sm text-muted-foreground"
                >
                  {line}
                </div>
              ))}
            </div>
          </aside>
        </div>
      </div>

      <LabSection
        id="tones"
        eyebrow={t("pages.systemNoticeUxLab.toneMatrix", { defaultValue: "Tone matrix" })}
        title={t("pages.systemNoticeUxLab.threeTones", { defaultValue: "Three tones, two states" })}
        description={t("ui.pages.systemnoticeuxlab.each-tone-pairs-unique")}
        accentClassName="bg-[linear-gradient(180deg,rgba(245,158,11,0.05),transparent_28%),var(--background)]"
      >
        <div className="space-y-5">
          <FixtureFrame caption={warningCollapsed.caption}>
            <SystemNotice {...warningCollapsed} />
          </FixtureFrame>
          <FixtureFrame caption={warningExpanded.caption}>
            <SystemNotice {...warningExpanded} />
          </FixtureFrame>
          <FixtureFrame caption={dangerCollapsed.caption}>
            <SystemNotice {...dangerCollapsed} />
          </FixtureFrame>
          <FixtureFrame caption={dangerExpanded.caption}>
            <SystemNotice {...dangerExpanded} />
          </FixtureFrame>
          <FixtureFrame caption={neutralCollapsed.caption}>
            <SystemNotice {...neutralCollapsed} />
          </FixtureFrame>
          <FixtureFrame caption={neutralExpanded.caption}>
            <SystemNotice {...neutralExpanded} />
          </FixtureFrame>
          <FixtureFrame caption={warningNoDetails.caption}>
            <SystemNotice {...warningNoDetails} />
          </FixtureFrame>
        </div>
      </LabSection>

      <LabSection
        id="hierarchy"
        eyebrow={t("pages.systemNoticeUxLab.hierarchyInThread", { defaultValue: "Hierarchy in thread" })}
        title={t("pages.systemNoticeUxLab.distinctComments", { defaultValue: "Distinct from user and agent comments" })}
        description={t("ui.pages.systemnoticeuxlab.side-side-adjacent-comment")}
        accentClassName="bg-[linear-gradient(180deg,rgba(8,145,178,0.05),transparent_28%),var(--background)]"
      >
        <div className="space-y-4 rounded-2xl border border-border/70 bg-background/70 p-4">
          <MockUserBubble
            authorName="Riley Board"
            body="Why does this issue keep waking back up without a clear next step?"
            alignEnd
          />
          <MockAgentBubble
            agentName="CodexCoder"
            body="The previous run completed without picking a disposition. I'll wait for the new system notice to surface so the recovery owner is unambiguous."
          />
          <SystemNotice
            tone="danger"
            label={t("pages.systemNoticeUxLab.systemAlert", { defaultValue: "System alert" })}
            source={{ label: "Paperclip", href: "/PAP/agents" }}
            timestamp="2026-05-04T16:48:00.000Z"
            body="Paperclip could not resolve this issue's missing disposition automatically. The issue is blocked on a recovery owner."
            metadata={[
              {
                title: t("pages.systemNoticeUxLab.recoveryOwner", { defaultValue: "Recovery owner" }),
                rows: [
                  {
                    kind: "issue",
                    label: t("pages.systemNoticeUxLab.recoveryIssue", { defaultValue: "Recovery issue" }),
                    identifier: "PAP-3440",
                    href: "/PAP/issues/PAP-3440",
                    title: t("pages.systemNoticeUxLab.handoffMissing", { defaultValue: "Successful run handoff missing disposition" }),
                  },
                  {
                    kind: "agent",
                    label: t("pages.systemNoticeUxLab2.owner", { defaultValue: "Owner" }),
                    name: t("pages.systemNoticeUxLab2.cto", { defaultValue: "CTO" }),
                    href: "/PAP/agents/cto",
                  },
                ],
              },
              {
                title: t("pages.systemNoticeUxLab.runEvidence", { defaultValue: "Run evidence" }),
                rows: [
                  {
                    kind: "run",
                    label: t("pages.systemNoticeUxLab.sourceRun", { defaultValue: "Source run" }),
                    runId: "9cdba892-c7ca-4d93-8604-4843873b127c",
                    href: "/PAP/agents/codexcoder/runs/9cdba892-c7ca-4d93-8604-4843873b127c",
                    status: "succeeded",
                  },
                ],
              },
            ]}
          />
          <MockUserBubble
            authorName="Riley Board"
            body={t("pages.systemNoticeUxLab.thanksAssigning", { defaultValue: "Thanks — assigning the recovery owner now." })}
            alignEnd
          />
        </div>
      </LabSection>

      <div className="grid gap-5 xl:grid-cols-2">
        <LabSection
          eyebrow={t("pages.systemNoticeUxLab.before", { defaultValue: "Before" })}
          title={t("pages.systemNoticeUxLab.nestedTreatment", { defaultValue: "Today's nested treatment" })}
          description={t("ui.pages.systemnoticeuxlab.same-content-rendered-through")}
          accentClassName="bg-[linear-gradient(180deg,rgba(244,63,94,0.05),transparent_28%),var(--background)]"
        >
          <div className="space-y-3 rounded-2xl border border-border/70 bg-background/70 p-4">
            <div className="flex items-start gap-2.5">
              <Avatar size="sm" className="shrink-0">
                <AvatarFallback>{t("ui.pages.systemnoticeuxlab.yo")}</AvatarFallback>
              </Avatar>
              <div className="flex min-w-0 max-w-(--pct-85) flex-col">
                <div className="mb-1 px-1 text-sm font-medium text-foreground">{t("pages.systemNoticeUxLab.you", { defaultValue: "You" })}</div>
                <div className="min-w-0 max-w-full rounded-2xl bg-muted px-4 py-2.5 text-sm leading-6 text-foreground">
                  <div className="rounded-md border border-red-500/35 bg-red-500/10 px-3 py-2.5 text-sm text-red-950 dark:text-red-100">
                    <div className="flex items-start gap-2">
                      <Sparkles className="mt-1 h-4 w-4 shrink-0 text-red-600 dark:text-red-300" />
                      <div className="min-w-0">
                        <p className="m-0 font-semibold">{t("pages.systemNoticeUxLab.handoffMissing2", { defaultValue: "Successful run handoff missing" })}</p>
                        <ul className="mt-1.5 list-disc space-y-0.5 pl-4 text-(length:--text-compact) leading-5">
                          <li>{t("pages.systemNoticeUxLab.sourceIssue", { defaultValue: "Source issue: PAP-3440" })}</li>
                          <li>{t("pages.systemNoticeUxLab2.sourceRun", { defaultValue: "Source run: 9cdba892-c7ca-4d93-8604-4843873b127c" })}</li>
                          <li>{t("pages.systemNoticeUxLab2.recoveryRun", { defaultValue: "Recovery run: 61fdb79b-8012-4676-ac71-2971830e126a" })}</li>
                          <li>{t("pages.systemNoticeUxLab.statusBefore", { defaultValue: "Status before: in_progress" })}</li>
                          <li>{t("pages.systemNoticeUxLab.normalizedCause", { defaultValue: "Normalized cause: Run completed without disposition" })}</li>
                          <li>{t("pages.systemNoticeUxLab.recoveryOwnerCto", { defaultValue: "Recovery owner: CTO" })}</li>
                          <li>{t("pages.systemNoticeUxLab.suggestedAction", { defaultValue: "Suggested action: Reassign to recovery agent" })}</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <p className="px-1 text-xs text-muted-foreground">
              {t("ui.pages.systemnoticeuxlab.author-reads")}<span className="font-medium text-foreground">{t("pages.systemNoticeUxLab.you", { defaultValue: "You" })}</span> {t("ui.pages.systemnoticeuxlab.even-though-author-paperclip")}</p>
          </div>
        </LabSection>

        <LabSection
          eyebrow={t("pages.systemNoticeUxLab.after", { defaultValue: "After" })}
          title={t("pages.systemNoticeUxLab.noticeReplacement", { defaultValue: "System notice replacement" })}
          description={t("ui.pages.systemnoticeuxlab.one-container-system-authored")}
          accentClassName="bg-[linear-gradient(180deg,rgba(16,185,129,0.05),transparent_28%),var(--background)]"
        >
          <div className="space-y-3 rounded-2xl border border-border/70 bg-background/70 p-4">
            <SystemNotice {...dangerCollapsed} />
            <p className="px-1 text-xs text-muted-foreground">
              {t("ui.pages.systemnoticeuxlab.same-content-visible-body")}{" "}
              <span className="font-medium text-foreground">{t("pages.systemNoticeUxLab.details", { defaultValue: "Details" })}</span> {t("ui.pages.systemnoticeuxlab.only-when-they-need")}</p>
          </div>
        </LabSection>
      </div>

      <Card className="gap-4 border-border/70 bg-background/85 py-0">
        <CardHeader className="px-5 pt-5 pb-0">
          <div className="flex items-center gap-2 text-(length:--text-micro) font-semibold uppercase tracking-(--tracking-caps) text-muted-foreground">
            <Layers className="h-4 w-4 text-amber-700 dark:text-amber-300" />
            {t("ui.pages.systemnoticeuxlab.implementation-notes")}</div>
          <CardTitle className="text-lg">{t("pages.systemNoticeUxLab.handoffToEng", { defaultValue: "Handoff to engineering" })}</CardTitle>
          <CardDescription>
            {t("ui.pages.systemnoticeuxlab.what-phase-ui-implementation")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 px-5 pb-5 pt-0 text-sm text-muted-foreground">
          <div className="rounded-2xl border border-border/70 bg-background/80 px-4 py-3">
            <div className="mb-1 font-medium text-foreground">{t("pages.systemNoticeUxLab.component", { defaultValue: "Component" })}</div>
            {t("ui.pages.systemnoticeuxlab.use")}<code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{`<SystemNotice />`}</code>{" "}
            from <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">@/components/SystemNotice</code>.
            It accepts <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">tone</code>,{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">label</code>,{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">body</code>,{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">metadata</code>{t("ui.pages.companyimport.text")}{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{t("ui.pages.systemnoticeuxlab.detailsdefaultopen")}</code>.
          </div>
          <div className="rounded-2xl border border-border/70 bg-background/80 px-4 py-3">
            <div className="mb-1 font-medium text-foreground">{t("pages.systemNoticeUxLab.routingInThread", { defaultValue: "Routing in IssueChatThread" })}</div>
            {t("ui.pages.systemnoticeuxlab.comments-where")}{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{t("ui.pages.systemnoticeuxlab.authortype-quot-system-quot")}</code>{" "}
            or{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{t("ui.pages.systemnoticeuxlab.presentation-kind-quot-system")}</code>{" "}
            {t("ui.pages.systemnoticeuxlab.should-render-systemnotice-row")}{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{t("ui.pages.systemnoticeuxlab.issuechatusermessage")}</code>{" "}
            {t("ui.pages.systemnoticeuxlab.assistant-bubble")}</div>
          <div className="rounded-2xl border border-border/70 bg-background/80 px-4 py-3">
            <div className="mb-1 font-medium text-foreground">{t("pages.systemNoticeUxLab.accessibility", { defaultValue: "Accessibility" })}</div>
            {t("ui.pages.systemnoticeuxlab.details-button-has")}{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">aria-expanded</code>{" "}
            and{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">aria-controls</code>{" "}
            {t("ui.pages.systemnoticeuxlab.wired-panel-id-container")}{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{t("ui.pages.systemnoticeuxlab.role-quot-status-quot")}</code>{" "}
            {t("ui.pages.systemnoticeuxlab.text")}{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">aria-label</code>{" "}
            {t("ui.pages.systemnoticeuxlab.equal-visible-tone-label")}</div>
          <div className="rounded-2xl border border-border/70 bg-background/80 px-4 py-3">
            <div className="mb-1 font-medium text-foreground">{t("pages.systemNoticeUxLab.legacyFallback", { defaultValue: "Legacy fallback" })}</div>
            {t("ui.pages.systemnoticeuxlab.existing-comments-without")}{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">presentation</code>{" "}
            {t("ui.pages.systemnoticeuxlab.keep-rendering-through-current")}{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{t("ui.pages.systemnoticeuxlab.successfulrunhandoffcommentcallout")}</code>{" "}
            {t("ui.pages.systemnoticeuxlab.string-detector-new-contract")}</div>
        </CardContent>
      </Card>
    </div>
  );
}

export default SystemNoticeUxLab;
