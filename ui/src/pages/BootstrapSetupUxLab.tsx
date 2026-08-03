import { t } from "../i18n";
import type { ReactElement, ReactNode } from "react";
import { Loader2, ShieldCheck, Terminal, TriangleAlert } from "lucide-react";
import { BOOTSTRAP_FALLBACK_COMMAND } from "@/bootstrapSetup";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";

type LabFixtureKey =
  | "signed-out-private"
  | "signed-in-private"
  | "claiming"
  | "claim-error"
  | "claim-success"
  | "public-invite-only";

const FIXTURE_LABELS: Record<LabFixtureKey, string> = {
  "signed-out-private": "1 · authenticated/private — signed out (browser claim available)",
  "signed-in-private": "2 · authenticated/private — signed in (claim CTA primary)",
  claiming: "3 · authenticated/private — claim in flight",
  "claim-error": "4 · authenticated/private — claim error (e.g. 409 already claimed)",
  "claim-success": "5 · authenticated/private — claim succeeded, redirect pending",
  "public-invite-only": "6 · authenticated/public — invite-only (no browser claim)",
};

const FIXTURE_ORDER: LabFixtureKey[] = [
  "signed-out-private",
  "signed-in-private",
  "claiming",
  "claim-error",
  "claim-success",
  "public-invite-only",
];

function CliFallback({ hasActiveInvite }: { hasActiveInvite: boolean }) {
  return (
    <div className="mt-6 border-t border-border pt-5">
      <div className="flex items-center gap-2 text-sm font-medium">
        <Terminal className="size-4 text-muted-foreground" aria-hidden />
        <span>{t("ui.components.bootstrappendingpage.prefer-finish-setup-from")}</span>
      </div>
      <p className="mt-2 text-sm text-muted-foreground">
        {hasActiveInvite
          ? "A bootstrap invite is already active. Check your Paperclip startup logs for the first‑admin URL, or run this command on the host to rotate it:"
          : "Run this command on the host that runs Paperclip to print a one‑time first‑admin invite URL:"}
      </p>
      <pre className="mt-3 overflow-x-auto rounded-md border border-border bg-muted/30 p-3 font-mono text-xs">
{BOOTSTRAP_FALLBACK_COMMAND}
      </pre>
    </div>
  );
}

function StateChrome({ children }: { children: ReactNode }) {
  return (
    <div className="mx-auto max-w-xl py-10">
      <Card className="block p-6">{children}</Card>
    </div>
  );
}

function SignedOutPrivate() {
  return (
    <StateChrome>
      <h1 className="text-xl font-semibold">{t("ui.components.bootstrappendingpage.finish-setting-up-paperclip")}</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        {t("ui.pages.bootstrapsetupuxlab.no-admin-has-claimed")}</p>
      <div className="mt-5">
        <Button asChild>
          <a href="/auth?next=/">{t("pages.cliAuth.signInCreate")}</a>
        </Button>
      </div>
      <CliFallback hasActiveInvite={false} />
    </StateChrome>
  );
}

function SignedInPrivate() {
  return (
    <StateChrome>
      <h1 className="text-xl font-semibold">{t("ui.components.bootstrappendingpage.finish-setting-up-paperclip")}</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        {t("ui.components.bootstrappendingpage.no-admin-has-claimed-alt")}</p>
      <div className="mt-5 flex flex-wrap items-center gap-3">
        <Button>{t("ui.pages.bootstrapsetupuxlab.claim-instance")}</Button>
        <span className="text-sm text-muted-foreground">
          {t("ui.components.bootstrappendingpage.signed")}<span className="font-medium text-foreground">jane@appliance.local</span>
        </span>
      </div>
      <p className="mt-3 text-xs text-muted-foreground">
        {t("ui.components.bootstrappendingpage.wrong-account")}{" "}
        <a href="/auth?next=/" className="underline underline-offset-2">
          {t("ui.components.bootstrappendingpage.switch-account")}</a>
        .
      </p>
      <CliFallback hasActiveInvite={false} />
    </StateChrome>
  );
}

function ClaimingPrivate() {
  return (
    <StateChrome>
      <h1 className="text-xl font-semibold">{t("ui.components.bootstrappendingpage.finish-setting-up-paperclip")}</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        {t("ui.components.bootstrappendingpage.no-admin-has-claimed-alt")}</p>
      <div className="mt-5 flex flex-wrap items-center gap-3">
        <Button disabled>
          <Loader2 className="mr-2 size-4 animate-spin" aria-hidden />
          {t("ui.pages.bootstrapsetupuxlab.claiming")}</Button>
        <span className="text-sm text-muted-foreground">
          {t("ui.components.bootstrappendingpage.signed")}<span className="font-medium text-foreground">jane@appliance.local</span>
        </span>
      </div>
      <CliFallback hasActiveInvite={false} />
    </StateChrome>
  );
}

function ClaimErrorPrivate() {
  return (
    <StateChrome>
      <h1 className="text-xl font-semibold">{t("ui.components.bootstrappendingpage.finish-setting-up-paperclip")}</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        {t("ui.components.bootstrappendingpage.no-admin-has-claimed-alt")}</p>
      <div className="mt-5 flex flex-wrap items-center gap-3">
        <Button>{t("ui.pages.bootstrapsetupuxlab.claim-instance")}</Button>
        <span className="text-sm text-muted-foreground">
          {t("ui.components.bootstrappendingpage.signed")}<span className="font-medium text-foreground">jane@appliance.local</span>
        </span>
      </div>
      <div
        role="alert"
        className="mt-4 flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive"
      >
        <TriangleAlert className="mt-0.5 size-4 flex-shrink-0" aria-hidden />
        <div>
          <p className="font-medium">{t("ui.pages.bootstrapsetupuxlab.someone-else-has-already")}</p>
          <p className="mt-1 text-destructive/90">
            {t("ui.pages.bootstrapsetupuxlab.refresh-sign-ask-existing")}{" "}
            <span className="font-mono">{t("ui.pages.bootstrapsetupuxlab.instance-settings-access")}</span>.
          </p>
        </div>
      </div>
      <CliFallback hasActiveInvite={false} />
    </StateChrome>
  );
}

function ClaimSuccess() {
  return (
    <StateChrome>
      <div className="flex items-start gap-3">
        <div className="mt-0.5 flex size-9 flex-shrink-0 items-center justify-center rounded-full bg-emerald-500/15 text-emerald-600 dark:text-emerald-400">
          <ShieldCheck className="size-5" aria-hidden />
        </div>
        <div>
          <h1 className="text-xl font-semibold">{t("ui.pages.bootstrapsetupuxlab.you-rsquo-re-instance")}</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            {t("ui.pages.bootstrapsetupuxlab.setup-complete-taking-you")}</p>
        </div>
      </div>
      <div className="mt-5 flex items-center gap-3">
        <Loader2 className="size-4 animate-spin text-muted-foreground" aria-hidden />
        <span className="text-sm text-muted-foreground">{t("ui.pages.bootstrapsetupuxlab.redirecting-hellip")}</span>
      </div>
      <div className="mt-5">
        <Button asChild variant="outline">
          <a href="/">{t("ui.components.bootstrappendingpage.continue-dashboard")}</a>
        </Button>
      </div>
    </StateChrome>
  );
}

function PublicInviteOnly() {
  return (
    <StateChrome>
      <h1 className="text-xl font-semibold">{t("ui.components.bootstrappendingpage.paperclip-waiting-first-admin")}</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        {t("ui.pages.bootstrapsetupuxlab.instance-runs-invite-only")}</p>
      <CliFallback hasActiveInvite />
      <p className="mt-4 text-xs text-muted-foreground">
        {t("ui.pages.bootstrapsetupuxlab.browser-based-claim-intentionally")}</p>
    </StateChrome>
  );
}

const FIXTURE_BODIES: Record<LabFixtureKey, ReactElement> = {
  "signed-out-private": <SignedOutPrivate />,
  "signed-in-private": <SignedInPrivate />,
  claiming: <ClaimingPrivate />,
  "claim-error": <ClaimErrorPrivate />,
  "claim-success": <ClaimSuccess />,
  "public-invite-only": <PublicInviteOnly />,
};

export function BootstrapSetupUxLab() {
  return (
    <div className="bg-background min-h-screen pb-16">
      <header className="border-b border-border bg-muted/20">
        <div className="mx-auto max-w-3xl px-6 py-6">
          <p className="text-xs font-medium uppercase tracking-wider text-muted-foreground">{t("ui.pages.bootstrapsetupuxlab.ux-lab")}</p>
          <h1 className="mt-1 text-2xl font-semibold">{t("ui.pages.bootstrapsetupuxlab.bootstrap-pending-setup-states")}</h1>
          <p className="mt-2 max-w-2xl text-sm text-muted-foreground">
            {t("ui.pages.bootstrapsetupuxlab.fixtures-bootstrap-pending-screen")}<span className="font-mono">{t("ui.pages.bootstrapsetupuxlab.cloudaccessgate")}</span>{t("ui.pages.bootstrapsetupuxlab.used-ux-spec")}{" "}
            <a className="underline underline-offset-2" href="/PAP/issues/PAP-10113">
              PAP-10113
            </a>{" "}
            {t("ui.pages.bootstrapsetupuxlab.implementation-reference")}{" "}
            <a className="underline underline-offset-2" href="/PAP/issues/PAP-10114">
              PAP-10114
            </a>
            {t("ui.pages.bootstrapsetupuxlab.browser-claim-cta-only")}{" "}
            <span className="font-mono">{t("ui.pages.bootstrapsetupuxlab.deploymentmode-quot-authenticated-quot")}</span> and{" "}
            <span className="font-mono">{t("ui.pages.bootstrapsetupuxlab.deploymentexposure-quot-private-quot")}</span>.
          </p>
        </div>
      </header>
      <main className="mx-auto max-w-3xl space-y-12 px-6 pt-10">
        {FIXTURE_ORDER.map((key) => (
          <section key={key} aria-labelledby={`lab-${key}`}>
            <h2
              id={`lab-${key}`}
              className="mb-3 text-xs font-medium uppercase tracking-wider text-muted-foreground"
            >
              {FIXTURE_LABELS[key]}
            </h2>
            <div className="rounded-lg border border-dashed border-border/70 bg-muted/10 p-2">
              {FIXTURE_BODIES[key]}
            </div>
          </section>
        ))}
      </main>
    </div>
  );
}
