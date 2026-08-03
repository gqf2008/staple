import { useEffect, useMemo, useRef, useState } from "react";
import { t } from "../../i18n";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
  ArrowUpRight,
  Check,
  ChevronRight,
  Copy,
  ClipboardPaste,
  Link2,
  Loader2,
  Lock,
  Search,
  TerminalSquare,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import type {
  Agent,
  AppDefinition,
  ConnectToolAppResult,
  ToolAppConnectionActionSummary,
} from "@paperclipai/shared";
import { credentialConfigPath, getAppDefinitionForUrl, getAvailableConnectionMethod } from "@paperclipai/shared";
import { useNavigate, useParams, useSearchParams } from "@/lib/router";
import { useCompany } from "@/context/CompanyContext";
import { useBreadcrumbs } from "@/context/BreadcrumbContext";
import { useToast } from "@/context/ToastContext";
import { queryKeys } from "@/lib/queryKeys";
import { ApiError } from "@/api/client";
import { toolsApi } from "@/api/tools";
import { agentsApi } from "@/api/agents";
import { appCopyFor, credentialFieldLabel } from "@/lib/app-gallery-copy";
import { advancedTabHref } from "@/pages/tools/tool-tabs";
import { AgentIcon } from "@/components/AgentIconPicker";
import { AgentMultiSelect } from "@/components/AgentMultiSelect";
import { InlineBanner } from "@/components/InlineBanner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import { AppLogo } from "./AppLogo";
import { parseGoogleSheetIds } from "./google-sheets";
import { autoExtendNotice, INSTALL_ALL_WARNING, installInfoNotice, installPayload } from "@/lib/tool-installs";

type Step = "gallery" | "key" | "actions" | "who" | "install" | "success";

const ROUTE_STAGE_BY_STEP: Partial<Record<Step, string>> = {
  key: "setup",
  actions: "actions",
  who: "access",
  install: "install",
  success: "complete",
};

function appConnectHref(appKey: string, step: Step): string {
  const stage = ROUTE_STAGE_BY_STEP[step] ?? "setup";
  const params = new URLSearchParams({ byo: "1", appKey, stage });
  return `/apps/connect?${params.toString()}`;
}
type AppAccessSelection = "all_agents" | { agentIds: string[] };
type InstallMode = "none" | "specific" | "all";
const LINK_CREDENTIAL_CONFIG_PATH = "credentials.authorization";

const STEP_LABELS = [t("pages.appsConnect.pickApp", { defaultValue: "Pick app" }), t("pages.appsConnect.addKey", { defaultValue: "Add your key" }), t("pages.appsConnect.chooseActions", { defaultValue: "Choose actions" }), t("pages.appsConnect.chooseAccess", { defaultValue: "Choose access" }), t("pages.appsConnect.installTools", { defaultValue: "Install tools" })];
const STEP_INDEX: Record<Exclude<Step, "success">, number> = {
  gallery: 0,
  key: 1,
  actions: 2,
  who: 3,
  install: 4,
};
const ZAPIER_STEP_INDEX: Record<Exclude<Step, "gallery" | "success">, number> = {
  key: 0,
  actions: 1,
  who: 2,
  install: 3,
};
const ZAPIER_STEP_LABELS = [t("pages.appsConnect.addMcpUrl", { defaultValue: "Add MCP URL" }), t("pages.appsConnect.chooseActions", { defaultValue: "Choose actions" }), t("pages.appsConnect.chooseAccess", { defaultValue: "Choose access" }), t("pages.appsConnect.installTools", { defaultValue: "Install tools" })];

function askFirstLevelsFrom(result: ConnectToolAppResult): string[] {
  const raw = (result.suggestedDefaults as { askFirstRiskLevels?: unknown })?.askFirstRiskLevels;
  return Array.isArray(raw) ? raw.filter((x): x is string => typeof x === "string") : ["write", "destructive"];
}

function isGoogleSheetsEntry(entry: AppDefinition | null): boolean {
  return entry?.slug === "google-sheets";
}

export function AppsConnect() {
  const navigate = useNavigate();
  const routeParams = useParams<{ appKey?: string }>();
  const { selectedCompany, selectedCompanyId } = useCompany();
  const { setBreadcrumbs } = useBreadcrumbs();
  const { pushToast } = useToast();
  const [searchParams] = useSearchParams();
  const appKey = routeParams.appKey ?? searchParams.get("appKey") ?? undefined;
  const zapierSource = searchParams.get("source") === "zapier";

  // Prefill arrives from the app page for reconnects; read once so later
  // wizard navigation doesn't fight the URL.
  const [prefill] = useState(() => {
    const rawLink = searchParams.get("link")?.trim() ?? "";
    return {
      link: /^https?:\/\//i.test(rawLink) ? rawLink : "",
      name: searchParams.get("name")?.trim() ?? "",
      applicationId: searchParams.get("applicationId")?.trim() || undefined,
    };
  });

  const [step, setStep] = useState<Step>(appKey || prefill.link || zapierSource ? "key" : "gallery");
  const [entry, setEntry] = useState<AppDefinition | null>(null);
  const [galleryName, setGalleryName] = useState("");
  const [linkUrl, setLinkUrl] = useState(prefill.link);
  const [linkName, setLinkName] = useState(prefill.name || (zapierSource ? t("pages.appsConnect.zapier", { defaultValue: "Zapier" }) : ""));
  const [linkNeedsKey, setLinkNeedsKey] = useState(false);
  const [linkKey, setLinkKey] = useState("");
  const [credentials, setCredentials] = useState<Record<string, string>>({});
  const [googleSheetsLinks, setGoogleSheetsLinks] = useState("");
  const [googleSheetsError, setGoogleSheetsError] = useState<string | null>(null);
  const [connectResult, setConnectResult] = useState<ConnectToolAppResult | null>(null);
  const [enabled, setEnabled] = useState<Record<string, boolean>>({});
  const [access, setAccess] = useState<"all" | "specific">("all");
  const [agentIds, setAgentIds] = useState<Set<string>>(new Set());
  const [installMode, setInstallMode] = useState<InstallMode>("none");
  const [installAgentIds, setInstallAgentIds] = useState<Set<string>>(new Set());

  const openGallery = () => {
    setEntry(null);
    setGalleryName("");
    setLinkUrl("");
    setLinkName("");
    setLinkNeedsKey(false);
    setLinkKey("");
    setCredentials({});
    setGoogleSheetsLinks("");
    setGoogleSheetsError(null);
    setConnectResult(null);
    setInstallMode("none");
    setInstallAgentIds(new Set());
    setStep("gallery");
    navigate("/apps/connect?byo=1");
  };

  useEffect(() => {
    setBreadcrumbs([
      { label: selectedCompany?.name ?? t("pages.appsConnect.company", { defaultValue: "Company" }), href: "/dashboard" },
      { label: t("pages.appsConnect.apps", { defaultValue: "Apps" }), href: "/apps" },
      { label: t("pages.appsConnect.connectApp", { defaultValue: "Connect an app" }) },
    ]);
    return () => setBreadcrumbs([]);
  }, [setBreadcrumbs, selectedCompany?.name]);

  const galleryQuery = useQuery({
    queryKey: queryKeys.apps.gallery(selectedCompanyId ?? t("ui.components.appconnectionsidebar.fallback-none")),
    queryFn: () => toolsApi.listGallery(selectedCompanyId!),
    enabled: !!selectedCompanyId,
  });

  useEffect(() => {
    if (!appKey || galleryQuery.isLoading || !galleryQuery.data) return;

    const requestedEntry = galleryQuery.data.apps.find((candidate) => candidate.slug === appKey);
    if (!requestedEntry || getAvailableConnectionMethod(requestedEntry)?.auth === "oauth" || requestedEntry.availability?.available === false) {
      setEntry(null);
      setStep("gallery");
      navigate("/apps/connect", { replace: true });
      return;
    }

    if (entry?.slug !== requestedEntry.slug) {
      setEntry(requestedEntry);
      setGalleryName(requestedEntry.name);
      setLinkUrl("");
      setLinkName("");
      setLinkNeedsKey(false);
      setLinkKey("");
      setCredentials({});
      setGoogleSheetsLinks("");
      setGoogleSheetsError(null);
      setConnectResult(null);
    }
    setInstallMode("none");
    setInstallAgentIds(new Set());
    setStep("key");
  }, [appKey, entry?.slug, galleryQuery.data, galleryQuery.isLoading, navigate]);

  const setAppStep = (nextStep: Step) => {
    setStep(nextStep);
    if (entry) navigate(appConnectHref(entry.slug, nextStep));
  };

  const connectMutation = useMutation({
    mutationFn: () => {
      if (entry) {
        const sheetIds = isGoogleSheetsEntry(entry) ? parseGoogleSheetIds(googleSheetsLinks).ids : [];
        const trimmedGalleryName = galleryName.trim();
        return toolsApi.connectApp(selectedCompanyId!, {
          galleryKey: entry.slug,
          name: trimmedGalleryName || undefined,
          credentialValues: credentials,
          configValues: isGoogleSheetsEntry(entry) ? { allowedSpreadsheetIds: sheetIds } : undefined,
          applicationId: prefill.applicationId,
        });
      }
      const trimmedKey = linkNeedsKey ? linkKey.trim() : "";
      const trimmedName = linkName.trim();
      return toolsApi.connectApp(selectedCompanyId!, {
        link: linkUrl,
        name: trimmedName || undefined,
        credentialValues: trimmedKey ? { [LINK_CREDENTIAL_CONFIG_PATH]: trimmedKey } : undefined,
        applicationId: prefill.applicationId,
      });
    },
    onSuccess: (result) => {
      setConnectResult(result);
      const defaults: Record<string, boolean> = {};
      for (const a of result.actions.readOnly) defaults[a.catalogEntryId] = true;
      for (const a of result.actions.canMakeChanges) defaults[a.catalogEntryId] = false;
      setEnabled(defaults);
      setInstallMode("none");
      setInstallAgentIds(new Set());
      setAppStep("actions");
    },
    onError: (error) => {
      const details = error instanceof ApiError && error.body && typeof error.body === "object"
        ? (error.body as { details?: { code?: unknown } }).details
        : null;
      const oauthRequired = details?.code === "oauth_challenge";
      pushToast({
        title: oauthRequired ? t("pages.appsConnect.signInRequired", { defaultValue: "Sign-in required" }) : t("ui.pages.apps.appsconnect.couldn-connect"),
        body: oauthRequired
          ? t("pages.appsConnect.signInSoon", { defaultValue: "This app needs you to sign in - coming soon." })
          : error instanceof Error
            ? error.message
            : t("pages.appsConnect.keyInvalid", { defaultValue: "Please check your key and try again." }),
        tone: "error",
      });
    },
  });

  const finishMutation = useMutation({
    mutationFn: async () => {
      const askFirstLevels = connectResult ? askFirstLevelsFrom(connectResult) : [];
      const changeActions = connectResult?.actions.canMakeChanges ?? [];
      const enabledIds = Object.entries(enabled)
        .filter(([, on]) => on)
        .map(([id]) => id);
      const askFirstIds = changeActions
        .filter((a) => enabled[a.catalogEntryId] && askFirstLevels.includes(a.riskLevel))
        .map((a) => a.catalogEntryId);
      const selection: AppAccessSelection =
        access === "all" ? "all_agents" : { agentIds: Array.from(agentIds) };
      const result = await toolsApi.finishApp(selectedCompanyId!, connectResult!.connectionId, {
        enabledCatalogEntryIds: enabledIds,
        askFirstCatalogEntryIds: askFirstIds,
        access: selection,
      });
      const installState = installMode === "all"
        ? { onAll: true, agentIds: new Set<string>() }
        : { onAll: false, agentIds: installMode === "specific" ? installAgentIds : new Set<string>() };
      await toolsApi.putConnectionInstalls(
        connectResult!.connectionId,
        installPayload(selectedCompanyId!, installState),
      );
      return result;
    },
    onSuccess: () => setAppStep("success"),
    onError: (error) => {
      pushToast({
        title: t("ui.pages.apps.appsconnect.couldn-finish-setup"),
        body: error instanceof Error ? error.message : t("pages.appsConnect.tryAgain", { defaultValue: "Please try again." }),
        tone: "error",
      });
    },
  });

  if (!selectedCompanyId) {
    return <div className="p-6 text-sm text-muted-foreground">{t("pages.appsConnect.selectCompany", { defaultValue: "Select a company to connect apps." })}</div>;
  }

  const appName =
    connectResult?.application.name ??
    entry?.name ??
    (linkName.trim() || defaultLinkName(linkUrl) || t("ui.pages.apps.appsconnect.fallback-app"));
  const zapierEntry = zapierSource
    ? galleryQuery.data?.apps.find((app) => app.slug === "zapier") ?? null
    : null;
  const stepLabels = zapierSource
    ? ZAPIER_STEP_LABELS
    : isGoogleSheetsEntry(entry)
      ? [t("pages.appsConnect.pickApp", { defaultValue: "Pick app" }), t("pages.appsConnect.shareSheet", { defaultValue: "Share sheet" }), t("pages.appsConnect.chooseActions", { defaultValue: "Choose actions" }), t("pages.appsConnect.chooseAccess", { defaultValue: "Choose access" }), t("pages.appsConnect.installTools", { defaultValue: "Install tools" })]
      : STEP_LABELS;
  const stepIndex = zapierSource && step !== "gallery" && step !== "success"
    ? ZAPIER_STEP_INDEX[step]
    : step === "success"
      ? stepLabels.length
      : STEP_INDEX[step];

  return (
    <div className="max-w-5xl">
      {step !== "success" && (
        <StepHeader
          subtitle={
            step === "gallery"
              ? t("pages.appsConnect.pickHint", { defaultValue: "Pick the app you want your agents to use." })
              : `Step ${stepIndex + 1} of ${stepLabels.length}`
          }
          step={step}
          activeIndex={stepIndex}
          labels={stepLabels}
          appIdentity={
            zapierSource
              ? { name: t("pages.appsConnect.zapier", { defaultValue: "Zapier" }), logoUrl: zapierEntry?.branding.logoUrl ?? null }
              : undefined
          }
          onCancel={() => navigate(zapierSource ? "/apps/browse" : "/apps")}
        />
      )}

      {step === "gallery" && (
        <GalleryStep
          loading={galleryQuery.isLoading}
          apps={galleryQuery.data?.apps ?? []}
          byo={searchParams.get("byo") === "1"}
          source={searchParams.get("source")}
          onPick={(picked) => {
            setEntry(picked);
            setGalleryName(picked.name);
            setLinkUrl("");
            setLinkName("");
            setLinkNeedsKey(false);
            setLinkKey("");
            setCredentials({});
            setGoogleSheetsLinks("");
            setGoogleSheetsError(null);
            setConnectResult(null);
            setInstallMode("none");
            setInstallAgentIds(new Set());
            setStep("key");
            navigate(appConnectHref(picked.slug, "key"));
          }}
          onUseLink={(url) => {
            const matchedEntry = getAppDefinitionForUrl(url, galleryQuery.data?.apps ?? []);
            setEntry(null);
            setGalleryName("");
            setLinkUrl(url);
            setLinkName(matchedEntry?.name ?? defaultLinkName(url) ?? "");
            setLinkNeedsKey(false);
            setLinkKey("");
            setCredentials({});
            setGoogleSheetsLinks("");
            setGoogleSheetsError(null);
            setInstallMode("none");
            setInstallAgentIds(new Set());
            setStep("key");
          }}
          onRunYourOwn={() => navigate(advancedTabHref("run-your-own"))}
          onPasteConfig={() => navigate(advancedTabHref("paste-config"))}
        />
      )}

      {step === "key" && entry && (
        <KeyStep
          entry={entry}
          name={galleryName}
          onNameChange={setGalleryName}
          values={credentials}
          onChange={setCredentials}
          googleSheetsLinks={googleSheetsLinks}
          googleSheetsError={googleSheetsError}
          onGoogleSheetsLinksChange={(next) => {
            setGoogleSheetsLinks(next);
            setGoogleSheetsError(null);
          }}
          submitting={connectMutation.isPending}
          onBack={openGallery}
          onConnect={() => {
            if (isGoogleSheetsEntry(entry)) {
              const parsed = parseGoogleSheetIds(googleSheetsLinks);
              if (parsed.invalidCount > 0) {
                setGoogleSheetsError(t("pages.appsConnect.invalidSheetsLink", { defaultValue: "That doesn't look like a Google Sheets link." }));
                return;
              }
              if (parsed.ids.length === 0) {
                setGoogleSheetsError(t("pages.appsConnect.sheetsLinkRequired", { defaultValue: "Paste at least one Google Sheets link." }));
                return;
              }
            }
            connectMutation.mutate();
          }}
        />
      )}

      {step === "key" && !entry && linkUrl && !zapierSource && (
        <LinkConnectStep
          link={linkUrl}
          name={linkName}
          onNameChange={setLinkName}
          needsKey={linkNeedsKey}
          onNeedsKeyChange={(next) => {
            setLinkNeedsKey(next);
            if (!next) setLinkKey("");
          }}
          keyValue={linkKey}
          onKeyChange={setLinkKey}
          submitting={connectMutation.isPending}
          onBack={() => setStep("gallery")}
          onConnect={() => connectMutation.mutate()}
        />
      )}

      {step === "key" && !entry && zapierSource && (
        <ZapierConnectStep
          link={linkUrl}
          onLinkChange={setLinkUrl}
          submitting={connectMutation.isPending}
          onBack={() => navigate("/apps/browse")}
          onConnect={() => connectMutation.mutate()}
        />
      )}

      {step === "actions" && connectResult && (
        <ActionsStep
          appName={appName}
          result={connectResult}
          enabled={enabled}
          onToggle={(id, on) => setEnabled((prev) => ({ ...prev, [id]: on }))}
          onBulk={(ids, on) =>
            setEnabled((prev) => {
              const next = { ...prev };
              for (const id of ids) next[id] = on;
              return next;
            })
          }
          onBack={() => setAppStep("key")}
          onContinue={() => setAppStep("who")}
        />
      )}

      {step === "who" && connectResult && (
        <WhoStep
          appName={appName}
          companyId={selectedCompanyId}
          access={access}
          setAccess={setAccess}
          agentIds={agentIds}
          setAgentIds={setAgentIds}
          onBack={() => setAppStep("actions")}
          onContinue={() => setAppStep("install")}
        />
      )}

      {step === "install" && connectResult && (
        <InstallStep
          appName={appName}
          companyId={selectedCompanyId}
          access={access}
          accessAgentIds={agentIds}
          installMode={installMode}
          setInstallMode={setInstallMode}
          installAgentIds={installAgentIds}
          setInstallAgentIds={setInstallAgentIds}
          submitting={finishMutation.isPending}
          onBack={() => setAppStep("who")}
          onFinish={() => finishMutation.mutate()}
        />
      )}

      {step === "success" && (
        <SuccessStep
          appName={appName}
          logoUrl={entry?.branding.logoUrl}
          enabledCount={Object.values(enabled).filter(Boolean).length}
          access={access}
          installMode={installMode}
          installCount={installAgentIds.size}
          onDone={() => navigate("/apps")}
        />
      )}
    </div>
  );
}

function StepHeader({
  subtitle,
  step,
  activeIndex,
  labels,
  appIdentity,
  onCancel,
}: {
  subtitle: string;
  step: Step;
  activeIndex: number;
  labels: string[];
  appIdentity?: { name: string; logoUrl: string | null };
  onCancel: () => void;
}) {
  return (
    <div className="mb-6">
      <div className="flex items-start justify-between gap-4">
        <div className="flex items-center gap-3">
          {appIdentity ? (
            <AppLogo name={appIdentity.name} logoUrl={appIdentity.logoUrl} size={44} />
          ) : null}
          <div>
            <h1 className="text-2xl font-bold tracking-tight">
              {appIdentity ? `Connect ${appIdentity.name}` : t("pages.appsConnect.connectApp", { defaultValue: "Connect an app" })}
            </h1>
            <p className="mt-1 text-sm text-muted-foreground">{subtitle}</p>
          </div>
        </div>
        <Button variant="ghost" size="sm" onClick={onCancel}>
          {t("common.cancel")}</Button>
      </div>
      {step !== "gallery" && (
        <div className="mt-4">
          <div className="flex gap-2">
            {labels.map((label, i) => (
              <div
                key={label}
                className={cn("h-1 w-20 rounded-full", i <= activeIndex ? "bg-foreground" : "bg-border")}
              />
            ))}
          </div>
          <div className="mt-2 text-xs text-muted-foreground">{labels.join("   ·   ")}</div>
        </div>
      )}
    </div>
  );
}

function ZapierConnectStep({
  link,
  onLinkChange,
  submitting,
  onBack,
  onConnect,
}: {
  link: string;
  onLinkChange: (next: string) => void;
  submitting: boolean;
  onBack: () => void;
  onConnect: () => void;
}) {
  const normalizedLink = normalizeAppLink(link);
  const zapierHostname = normalizedLink ? new URL(normalizedLink).hostname : "";
  const isZapierLink = zapierHostname === "zapier.com" || zapierHostname.endsWith(".zapier.com");

  return (
    <div className="mx-auto max-w-xl rounded-2xl border border-border bg-card p-8">
      <div className="flex items-start gap-3">
        <span className="mt-1 inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-border bg-background">
          <Link2 className="h-5 w-5 text-muted-foreground" />
        </span>
        <div className="min-w-0">
          <h2 className="text-xl font-bold tracking-tight">{t("pages.appsConnect.connectZapier", { defaultValue: "Connect Zapier" })}</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            {t("pages.appsConnect.zapierUrlHint")}</p>
        </div>
      </div>

      <div className="mt-8">
        <label className="text-sm font-medium text-foreground">{t("pages.appsConnect.zapierMcpUrl", { defaultValue: "Zapier MCP URL" })}</label>
        <Input
          value={link}
          onChange={(event) => onLinkChange(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && isZapierLink && !submitting) onConnect();
          }}
          placeholder="https://mcp.zapier.com/api/v1/connect?token=…"
          className="mt-2 h-11"
          autoFocus
        />
        <p className="mt-2 text-xs text-muted-foreground">
          {t("ui.pages.apps.appsconnect.token-part-url-paperclip")}</p>
        {link.trim() && !isZapierLink && (
          <p className="mt-2 text-xs text-destructive">{t("pages.appsConnect.zapierUrlInvalid", { defaultValue: "Paste a valid Zapier URL to continue." })}</p>
        )}
      </div>

      <div className="mt-8 flex items-center justify-between">
        <Button variant="ghost" onClick={onBack} disabled={submitting}>
          {t("pages.teamCatalog.back")}</Button>
        <Button onClick={onConnect} disabled={submitting || !isZapierLink}>
          {submitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          {submitting ? t("pages.appsConnect.checking", { defaultValue: "Checking…" }) : t("pages.appsConnect.checkLink", { defaultValue: "Check link" })}
        </Button>
      </div>
    </div>
  );
}

function GalleryStep({
  loading,
  apps,
  byo = false,
  source = null,
  onPick,
  onUseLink,
  onRunYourOwn,
  onPasteConfig,
}: {
  loading: boolean;
  apps: AppDefinition[];
  /** Entered via the t("pages.appsConnect.connectOwnMcp", { defaultValue: "Connect your own MCP server" }) card (PAP-12371, Finding C): focus the link path. */
  byo?: boolean;
  source?: string | null;
  onPick: (entry: AppDefinition) => void;
  onUseLink: (link: string) => void;
  onRunYourOwn: () => void;
  onPasteConfig: () => void;
}) {
  const [search, setSearch] = useState("");
  const [linkInput, setLinkInput] = useState("");
  const [linkError, setLinkError] = useState<string | null>(null);
  const linkSectionRef = useRef<HTMLDivElement>(null);
  const linkInputRef = useRef<HTMLInputElement>(null);

  // Arriving from the BYO card: scroll the t("pages.appsConnect.connectWithLink", { defaultValue: "Connect with a link" }) section into
  // view and focus its input so the paste-URL path is the obvious next step.
  useEffect(() => {
    if (!byo || loading) return;
    linkSectionRef.current?.scrollIntoView({ block: "center" });
    linkInputRef.current?.focus();
  }, [byo, loading]);
  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return apps;
    return apps.filter((a) => a.name.toLowerCase().includes(q));
  }, [apps, search]);
  const normalizedLink = normalizeAppLink(linkInput);
  const matchedEntry = normalizedLink ? getAppDefinitionForUrl(normalizedLink, apps) : null;
  const zapierSource = source === "zapier";

  const continueWithLink = () => {
    const next = normalizeAppLink(linkInput);
    if (!next) {
      setLinkError(t("pages.appsConnect.pasteLinkHint", { defaultValue: "Paste a full http or https link." }));
      return;
    }
    setLinkError(null);
    onUseLink(next);
  };

  if (loading) {
    return (
      <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4">
        {Array.from({ length: 8 }).map((_, i) => (
          <Skeleton key={i} className="h-32 w-full rounded-xl" />
        ))}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="relative">
        <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder={t("pages.appsConnect.searchApps", { defaultValue: "Search apps…" })}
          className="h-11 pl-9"
        />
      </div>

      <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4">
        {filtered.map((app) => {
          const copy = appCopyFor(app.slug, app.description);
          const oauth = getAvailableConnectionMethod(app)?.auth === "oauth";
          const unavailable = app.availability?.available === false;
          return (
            <button
              key={app.slug}
              type="button"
              disabled={oauth || unavailable}
              title={
                unavailable
                  ? `${app.name} isn't configured on this instance yet. Ask your Paperclip admin.`
                  : undefined
              }
              onClick={() => onPick(app)}
              className={cn(
                "flex flex-col rounded-xl border border-border bg-card p-4 text-left transition-colors",
                oauth || unavailable ? "cursor-not-allowed opacity-60" : "hover:border-foreground/30 hover:bg-accent/40",
              )}
            >
              <AppLogo name={app.name} logoUrl={app.branding.logoUrl} size={36} />
              <div className="mt-3 text-sm font-bold text-foreground">{app.name}</div>
              <div className="mt-1 line-clamp-2 text-xs text-muted-foreground">{copy.tagline}</div>
              <div className="mt-3 text-xs font-semibold text-foreground">
                {unavailable ? (
                  <span className="text-muted-foreground">{t("pages.appsConnect.instanceNotAvailable", { defaultValue: "Not available on this instance - ask your admin." })}</span>
                ) : oauth ? (
                  <span className="text-muted-foreground">{t("pages.appsConnect.signInSoon2", { defaultValue: "Sign-in coming soon" })}</span>
                ) : (
                  <span>{t("pages.appsConnect.connectArrow", { defaultValue: "Connect →" })}</span>
                )}
              </div>
            </button>
          );
        })}
      </div>

      {filtered.length === 0 && (
        <div className="py-10 text-center text-sm text-muted-foreground">{t("ui.pages.apps.appsconnect.no-apps-match")}{search}”.</div>
      )}

      <div
        ref={linkSectionRef}
        className={cn(
          "grid gap-4 border-t border-border pt-5 md:grid-cols-(--gtc-13)",
          byo && "-mx-3 rounded-xl border border-primary/40 bg-primary/[0.04] px-3 pb-4 md:mx-0",
        )}
      >
        <div>
          <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
            <Link2 className="h-4 w-4 text-muted-foreground" />
            {zapierSource ? t("pages.appsConnect.connectZapier", { defaultValue: "Connect Zapier" }) : byo ? t("pages.appsConnect.connectOwnMcp", { defaultValue: "Connect your own MCP server" }) : t("pages.appsConnect.connectWithLink", { defaultValue: "Connect with a link" })}
          </div>
          <p className="mt-1 text-xs text-muted-foreground">
            {zapierSource
              ? t("pages.appsConnect.zapierUrlHint", { defaultValue: "Paste the complete MCP URL Zapier gives you, including its token." })
              : byo
              ? "Paste your MCP server’s URL and we’ll walk you through permissions and review."
              : t("pages.appsConnect.setupLinkHint", { defaultValue: "Paste a setup link from an app that is not listed here." })}
          </p>
          {!zapierSource && (
            <p className="mt-1 text-xs text-muted-foreground">
              {t("ui.pages.apps.appsconnect.any-remote-tool-url")}{" "}
              <code className="rounded bg-muted px-1 py-0.5 text-xs">http://127.0.0.1:8848/mcp</code>.
            </p>
          )}
          {matchedEntry && (
            <div className="mt-3 flex items-center justify-between gap-3 rounded-lg border border-border bg-muted/40 px-3 py-2">
              <div className="flex min-w-0 items-center gap-2 text-sm">
                <AppLogo name={matchedEntry.name} logoUrl={matchedEntry.branding.logoUrl} size={24} />
                <span className="truncate">{t("ui.pages.apps.appsconnect.looks-like")}{matchedEntry.name}.</span>
              </div>
              <Button
                type="button"
                size="sm"
                variant="outline"
                disabled={matchedEntry.availability?.available === false}
                onClick={() => {
                  setLinkError(null);
                  if (matchedEntry.slug === "zapier") {
                    continueWithLink();
                    return;
                  }
                  onPick(matchedEntry);
                }}
              >
                {matchedEntry.availability?.available === false
                  ? t("pages.appsConnect.notAvailable", { defaultValue: "Not available" })
                  : matchedEntry.slug === "zapier"
                    ? t("pages.appsConnect.continue", { defaultValue: "Continue" })
                    : `Use ${matchedEntry.name}`}
              </Button>
            </div>
          )}
        </div>
        <div className="flex min-w-0 flex-col gap-2 sm:min-w-(--sz-360px)">
          <div className="flex gap-2">
            <Input
              ref={linkInputRef}
              value={linkInput}
              onChange={(e) => {
                setLinkInput(e.target.value);
                setLinkError(null);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") continueWithLink();
              }}
              placeholder={zapierSource ? "https://mcp.zapier.com/api/v1/connect?token=…" : "https://example.com/actions"}
              className="h-10"
            />
            <Button type="button" variant="outline" onClick={continueWithLink}>
              {t("pages.appsConnect.continue")}</Button>
          </div>
          {linkError && <div className="text-xs text-destructive">{linkError}</div>}
        </div>
      </div>

      <div className="border-t border-border pt-5">
        <div className="text-sm font-semibold text-foreground">{t("pages.appsConnect.moreWays", { defaultValue: "More ways to connect" })}</div>
        <p className="mt-1 text-xs text-muted-foreground">
          {t("ui.pages.apps.appsconnect.tools-aren-gallery-you")}</p>
        <div className="mt-3 flex flex-col gap-2">
          <ConnectMethodRow
            icon={TerminalSquare}
            title={t("pages.appsConnect.runYourOwn", { defaultValue: "Run your own" })}
            description={t("ui.pages.apps.appsconnect.register-command-paperclip-runs")}
            onClick={onRunYourOwn}
          />
          <ConnectMethodRow
            icon={ClipboardPaste}
            title={t("pages.appsConnect.pasteConfig", { defaultValue: "Paste a config" })}
            description={t("ui.pages.apps.appsconnect.already-have-setup-snippet")}
            onClick={onPasteConfig}
          />
        </div>
      </div>
    </div>
  );
}

function ConnectMethodRow({
  icon: Icon,
  title,
  description,
  onClick,
}: {
  icon: LucideIcon;
  title: string;
  description: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="flex w-full items-center gap-3 rounded-lg border border-border bg-card px-4 py-3 text-left transition-colors hover:border-foreground/30 hover:bg-accent/40"
    >
      <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-border bg-background">
        <Icon className="h-4 w-4 text-muted-foreground" />
      </span>
      <div className="min-w-0 flex-1">
        <div className="text-sm font-semibold text-foreground">{title}</div>
        <div className="truncate text-xs text-muted-foreground">{description}</div>
      </div>
      <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
    </button>
  );
}

function normalizeAppLink(value: string): string | null {
  try {
    const parsed = new URL(value.trim());
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return null;
    return parsed.toString();
  } catch {
    return null;
  }
}

function defaultLinkName(link: string): string | null {
  try {
    return new URL(link).hostname.replace(/^www\./, "");
  } catch {
    return null;
  }
}

function LinkConnectStep({
  link,
  name,
  onNameChange,
  needsKey,
  onNeedsKeyChange,
  keyValue,
  onKeyChange,
  submitting,
  onBack,
  onConnect,
}: {
  link: string;
  name: string;
  onNameChange: (next: string) => void;
  needsKey: boolean;
  onNeedsKeyChange: (next: boolean) => void;
  keyValue: string;
  onKeyChange: (next: string) => void;
  submitting: boolean;
  onBack: () => void;
  onConnect: () => void;
}) {
  return (
    <div className="mx-auto max-w-xl rounded-2xl border border-border bg-card p-8">
      <div className="flex items-start gap-3">
        <span className="mt-1 inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-border bg-background">
          <Link2 className="h-5 w-5 text-muted-foreground" />
        </span>
        <div className="min-w-0">
          <h2 className="text-xl font-bold tracking-tight">{t("pages.appsConnect.connectWithLink", { defaultValue: "Connect with a link" })}</h2>
          <p className="mt-1 truncate text-sm text-muted-foreground">{link}</p>
        </div>
      </div>

      <div className="mt-8 space-y-6">
        <div>
          <label className="text-sm font-medium text-foreground">{t("pages.appsConnect.name", { defaultValue: "Name" })}</label>
          <Input
            value={name}
            onChange={(e) => onNameChange(e.target.value)}
            placeholder={t("pages.appsConnect.myApp", { defaultValue: "My app" })}
            className="mt-2 h-11"
          />
          <p className="mt-2 text-xs text-muted-foreground">
            {t("ui.pages.apps.appsconnect.we-filled-from-link")}</p>
        </div>

        <div>
          <label className="text-sm font-medium text-foreground">{t("pages.appsConnect.needKey", { defaultValue: "Does it need a key?" })}</label>
          <div className="mt-2 inline-flex rounded-lg border border-border bg-muted/50 p-1">
            <SegmentedOption
              label="No"
              selected={!needsKey}
              onClick={() => onNeedsKeyChange(false)}
            />
            <SegmentedOption
              label={t("pages.appsConnect.yes", { defaultValue: "Yes" })}
              selected={needsKey}
              onClick={() => onNeedsKeyChange(true)}
            />
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            {needsKey
              ? t("pages.appsConnect.keyHint", { defaultValue: "Paste the key this app gave you." })
              : t("pages.appsConnect.keyQuestionHint", { defaultValue: "Most apps just work from the link — pick Yes only if the app gave you a key." })}
          </p>
        </div>

        {needsKey && (
          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium text-foreground">{t("pages.appsConnect.appKey", { defaultValue: "App key" })}</label>
              <Input
                type="password"
                autoComplete="off"
                value={keyValue}
                onChange={(e) => onKeyChange(e.target.value)}
                placeholder="••••••••••••••••"
                className="mt-2 h-11 font-mono"
              />
            </div>

            <div className="flex items-start gap-3 rounded-lg bg-muted/50 p-4">
              <Lock className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
              <div>
                <div className="text-sm font-medium text-foreground">{t("pages.appsConnect.keySecure", { defaultValue: "Your key is stored securely." })}</div>
                <div className="text-xs text-muted-foreground">
                  {t("ui.pages.apps.appsconnect.you-can-replace-anytime")}</div>
              </div>
            </div>
          </div>
        )}
      </div>

      <div className="mt-8 flex items-center justify-between">
        <Button variant="ghost" onClick={onBack} disabled={submitting}>
          {t("pages.teamCatalog.back")}</Button>
        <div className="flex items-center gap-3">
          <span className="hidden text-xs text-muted-foreground sm:inline">
            {t("ui.pages.apps.appsconnect.we-ll-check-link")}</span>
          <Button onClick={onConnect} disabled={submitting || (needsKey && keyValue.trim().length === 0)}>
            {submitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {submitting ? t("pages.appsConnect.checking", { defaultValue: "Checking…" }) : t("pages.appsConnect.checkLink", { defaultValue: "Check link" })}
          </Button>
        </div>
      </div>
    </div>
  );
}

function SegmentedOption({
  label,
  selected,
  onClick,
}: {
  label: string;
  selected: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={selected}
      className={cn(
        "min-w-(--sz-64px) rounded-md px-4 py-1.5 text-sm font-medium transition-colors",
        selected
          ? "bg-background text-foreground shadow-sm"
          : "text-muted-foreground hover:text-foreground",
      )}
    >
      {label}
    </button>
  );
}

function ConnectionNameField({
  name,
  onNameChange,
}: {
  name: string;
  onNameChange: (next: string) => void;
}) {
  return (
    <div>
      <label className="text-sm font-medium text-foreground">{t("pages.appsConnect.name", { defaultValue: "Name" })}</label>
      <Input
        value={name}
        onChange={(e) => onNameChange(e.target.value)}
        placeholder={t("pages.appsConnect.myApp", { defaultValue: "My app" })}
        className="mt-2 h-11"
      />
      <p className="mt-2 text-xs text-muted-foreground">
        {t("ui.pages.apps.appsconnect.we-filled-from-app")}</p>
    </div>
  );
}

function KeyStep({
  entry,
  name,
  onNameChange,
  values,
  onChange,
  googleSheetsLinks,
  googleSheetsError,
  onGoogleSheetsLinksChange,
  submitting,
  onBack,
  onConnect,
}: {
  entry: AppDefinition;
  name: string;
  onNameChange: (next: string) => void;
  values: Record<string, string>;
  onChange: (next: Record<string, string>) => void;
  googleSheetsLinks: string;
  googleSheetsError: string | null;
  onGoogleSheetsLinksChange: (next: string) => void;
  submitting: boolean;
  onBack: () => void;
  onConnect: () => void;
}) {
  const copy = appCopyFor(entry.slug, entry.description);
  const method = getAvailableConnectionMethod(entry);
  const fields = (method?.credentialFields ?? []).map((field) => ({
    ...field,
    configPath: credentialConfigPath(field),
    helpUrl: method?.consoleLinks?.keys ?? method?.consoleLinks?.docs ?? "",
  }));
  const allFilled = fields.every(
    (f) => f.required === false || (values[f.configPath]?.trim().length ?? 0) > 0,
  );
  const robotEmail = entry.availability?.robotEmail ?? null;
  const unavailable = entry.availability?.available === false;

  if (isGoogleSheetsEntry(entry)) {
    const parsed = parseGoogleSheetIds(googleSheetsLinks);
    const canConnect = !unavailable && Boolean(robotEmail) && googleSheetsLinks.trim().length > 0;
    return (
      <div className="mx-auto max-w-xl rounded-2xl border border-border bg-card p-8">
        <div className="flex items-center gap-3">
          <AppLogo name={entry.name} logoUrl={entry.branding.logoUrl} size={48} />
          <div>
            <h2 className="text-lg font-bold tracking-tight sm:text-xl">{t("pages.appsConnect.connectSheets", { defaultValue: "Connect Google Sheets" })}</h2>
            <p className="text-sm text-muted-foreground">{copy.short}</p>
          </div>
        </div>

        <div className="mt-8 space-y-6">
          <ConnectionNameField name={name} onNameChange={onNameChange} />

          {robotEmail ? (
            <div>
              <label className="text-sm font-medium text-foreground">{t("pages.appsConnect.shareEmail", { defaultValue: "Share each sheet with this email" })}</label>
              <div className="mt-2 flex min-w-0 flex-col gap-2 sm:flex-row">
                <div
                  title={robotEmail}
                  className="min-h-11 min-w-0 flex-1 rounded-md border border-input bg-muted/40 px-3 py-2.5 font-mono text-xs leading-tight text-foreground break-all"
                >
                  {robotEmail}
                </div>
                <Button
                  type="button"
                  variant="outline"
                  className="shrink-0"
                  onClick={() => void navigator.clipboard?.writeText(robotEmail)}
                >
                  <Copy className="mr-2 h-4 w-4" />
                  {t("components.commentThread.copy")}</Button>
              </div>
              <p className="mt-2 text-xs text-muted-foreground">
                {t("ui.pages.apps.appsconnect.google-sheets-click-share")}</p>
            </div>
          ) : (
            <div className="rounded-lg bg-muted/50 p-4 text-sm text-muted-foreground">
              {t("ui.pages.apps.appsconnect.google-sheets-not-available")}</div>
          )}

          <div>
            <label className="text-sm font-medium text-foreground">{t("pages.appsConnect.pasteSheetLinks", { defaultValue: "Paste links to the sheets you shared" })}</label>
            <Textarea
              value={googleSheetsLinks}
              onChange={(e) => onGoogleSheetsLinksChange(e.target.value)}
              placeholder="https://docs.google.com/spreadsheets/d/..."
              className="mt-2 min-h-28"
            />
            <div className="mt-2 text-xs text-muted-foreground">
              {parsed.ids.length > 0
                ? `${parsed.ids.length} ${parsed.ids.length === 1 ? "sheet" : "sheets"} ready to connect.`
                : "Paste one link per line. Both .../edit and .../edit#gid=... links work."}
            </div>
            {googleSheetsError && <div className="mt-2 text-xs text-destructive">{googleSheetsError}</div>}
          </div>
        </div>

        <div className="mt-8 flex items-center justify-between">
          <Button variant="ghost" onClick={onBack} disabled={submitting}>
            {t("pages.teamCatalog.back")}</Button>
          <Button onClick={onConnect} disabled={submitting || !canConnect}>
            {submitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {submitting ? t("pages.appsConnect.checking", { defaultValue: "Checking…" }) : t("pages.appsConnect.connect", { defaultValue: "Connect" })}
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-xl rounded-2xl border border-border bg-card p-8">
      <div className="flex items-center gap-3">
        <AppLogo name={entry.name} logoUrl={entry.branding.logoUrl} size={48} />
        <div>
          <h2 className="text-xl font-bold tracking-tight">{t("pages.appNotConnected.connect")}{entry.name}</h2>
          <p className="text-sm text-muted-foreground">{copy.short}</p>
        </div>
      </div>

      <div className="mt-8 space-y-6">
        <ConnectionNameField name={name} onNameChange={onNameChange} />

        {fields.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            {t("ui.pages.apps.appsconnect.app-doesn-need-key")}</p>
        ) : (
          fields.map((field) => (
            <div key={field.configPath}>
              <label className="text-sm font-medium text-foreground">
                {credentialFieldLabel(entry.name, field.label, fields.length)}
              </label>
              <Input
                type="password"
                autoComplete="off"
                value={values[field.configPath] ?? ""}
                onChange={(e) => onChange({ ...values, [field.configPath]: e.target.value })}
                placeholder="••••••••••••••••"
                className="mt-2 h-11 font-mono"
              />
              {field.helpUrl && (
                <a
                  href={field.helpUrl}
                  target="_blank"
                  rel="noreferrer"
                  className="mt-2 inline-flex items-center gap-1 text-xs font-semibold text-foreground underline underline-offset-2"
                >
                  {t("ui.pages.apps.appsconnect.where-do-find")}<ArrowUpRight className="h-3 w-3" />
                </a>
              )}
            </div>
          ))
        )}

        <div className="flex items-start gap-3 rounded-lg bg-muted/50 p-4">
          <Lock className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
          <div>
            <div className="text-sm font-medium text-foreground">{t("pages.appsConnect.keySecure", { defaultValue: "Your key is stored securely." })}</div>
            <div className="text-xs text-muted-foreground">
              {t("ui.pages.apps.appsconnect.you-can-replace-anytime")}</div>
          </div>
        </div>
      </div>

      <div className="mt-8 flex items-center justify-between">
        <Button variant="ghost" onClick={onBack} disabled={submitting}>
          {t("pages.teamCatalog.back")}</Button>
        <div className="flex items-center gap-3">
          <span className="hidden text-xs text-muted-foreground sm:inline">
            {t("ui.pages.apps.appsconnect.we-ll-check-key")}</span>
          <Button onClick={onConnect} disabled={submitting || !allFilled}>
            {submitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {submitting ? t("pages.appsConnect.checking", { defaultValue: "Checking…" }) : t("pages.appsConnect.connect", { defaultValue: "Connect" })}
          </Button>
        </div>
      </div>
    </div>
  );
}

function ActionGroup({
  title,
  hint,
  actions,
  enabled,
  onToggle,
  bulkLabel,
  onBulk,
  askFirstLevels,
}: {
  title: string;
  hint: string;
  actions: ToolAppConnectionActionSummary[];
  enabled: Record<string, boolean>;
  onToggle: (id: string, on: boolean) => void;
  bulkLabel: string;
  onBulk: () => void;
  askFirstLevels: string[];
}) {
  if (actions.length === 0) return null;
  return (
    <div className="rounded-xl border border-border bg-card">
      <div className="flex items-center justify-between border-b border-border px-5 py-3">
        <div className="text-sm">
          <span className="font-bold text-foreground">{title}</span>
          <span className="ml-2 text-muted-foreground">· {hint}</span>
        </div>
        <button
          type="button"
          onClick={onBulk}
          className="text-xs text-muted-foreground hover:text-foreground"
        >
          {bulkLabel}
        </button>
      </div>
      <div className="divide-y divide-border">
        {actions.map((action) => {
          const on = enabled[action.catalogEntryId] ?? false;
          const showAskFirst = on && askFirstLevels.includes(action.riskLevel);
          return (
            <div key={action.catalogEntryId} className="flex items-center gap-4 px-5 py-3">
              <div className="min-w-0 flex-1">
                <div className="text-sm font-medium text-foreground">
                  {action.title ?? action.toolName}
                </div>
                {action.description && (
                  <div className="truncate text-xs text-muted-foreground">{action.description}</div>
                )}
              </div>
              {showAskFirst && (
                <span className="inline-flex items-center rounded-full border border-amber-500/40 bg-amber-500/10 px-2 py-0.5 text-xs font-semibold text-amber-700 dark:text-amber-300">
                  {t("pages.apps.testPanel.askFirst")}</span>
              )}
              <ToggleSwitch checked={on} onCheckedChange={(next) => onToggle(action.catalogEntryId, next)} />
            </div>
          );
        })}
      </div>
    </div>
  );
}

function ActionsStep({
  appName,
  result,
  enabled,
  onToggle,
  onBulk,
  onBack,
  onContinue,
}: {
  appName: string;
  result: ConnectToolAppResult;
  enabled: Record<string, boolean>;
  onToggle: (id: string, on: boolean) => void;
  onBulk: (ids: string[], on: boolean) => void;
  onBack: () => void;
  onContinue: () => void;
}) {
  const askFirstLevels = askFirstLevelsFrom(result);
  const { readOnly, canMakeChanges } = result.actions;
  const total = readOnly.length + canMakeChanges.length;
  const enabledCount = Object.values(enabled).filter(Boolean).length;

  return (
    <div className="space-y-5">
      <div className="flex items-start gap-3">
        <span className="mt-0.5 inline-flex h-6 w-6 items-center justify-center rounded-full border-2 border-emerald-500 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400">
          <Check className="h-3.5 w-3.5" />
        </span>
        <div>
          <div className="text-lg font-bold text-foreground">
            {t("ui.pages.apps.appsconnect.connected")}{appName} {t("ui.pages.apps.appsconnect.offers")}{total} {total === 1 ? "action" : "actions"}.
          </div>
          <div className="text-sm text-muted-foreground">
            {t("ui.pages.apps.appsconnect.read-only-actions-anything")}</div>
        </div>
      </div>

      <ActionGroup
        title={t("pages.appsConnect.readOnly", { defaultValue: "Read only" })}
        hint={t("ui.pages.apps.appsconnect.these-can-look-but")}
        actions={readOnly}
        enabled={enabled}
        onToggle={onToggle}
        bulkLabel={t("pages.appsConnect.turnAllOff", { defaultValue: "Turn all off" })}
        onBulk={() => onBulk(readOnly.map((a) => a.catalogEntryId), false)}
        askFirstLevels={askFirstLevels}
      />

      <ActionGroup
        title={t("pages.appsConnect.canMakeChanges", { defaultValue: "Can make changes" })}
        hint={t("ui.pages.apps.appsconnect.these-change-something-another")}
        actions={canMakeChanges}
        enabled={enabled}
        onToggle={onToggle}
        bulkLabel={t("pages.appsConnect.turnAllOn", { defaultValue: "Turn all on" })}
        onBulk={() => onBulk(canMakeChanges.map((a) => a.catalogEntryId), true)}
        askFirstLevels={askFirstLevels}
      />

      <div className="flex items-center justify-between pt-1">
        <Button variant="ghost" onClick={onBack}>
          {t("pages.teamCatalog.back")}</Button>
        <div className="flex items-center gap-3">
          <span className="hidden text-xs text-muted-foreground sm:inline">
            {t("ui.pages.apps.appsconnect.if")}{appName} {t("ui.pages.apps.appsconnect.adds-new-actions-later")}</span>
          <Button onClick={onContinue} disabled={enabledCount === 0}>
            {t("ui.pages.apps.appsconnect.continue")}{enabledCount} {enabledCount === 1 ? "action" : "actions"} on
          </Button>
        </div>
      </div>
    </div>
  );
}

function WhoStep({
  appName,
  companyId,
  access,
  setAccess,
  agentIds,
  setAgentIds,
  onBack,
  onContinue,
}: {
  appName: string;
  companyId: string;
  access: "all" | "specific";
  setAccess: (a: "all" | "specific") => void;
  agentIds: Set<string>;
  setAgentIds: (s: Set<string>) => void;
  onBack: () => void;
  onContinue: () => void;
}) {
  const agentsQuery = useQuery({
    queryKey: queryKeys.agents.list(companyId),
    queryFn: () => agentsApi.list(companyId),
    enabled: access === "specific",
  });
  const agents: Agent[] = (agentsQuery.data ?? []).filter((a) => a.status !== "terminated");
  const canFinish = access === "all" || agentIds.size > 0;

  return (
    <div className="mx-auto max-w-xl">
      <div className="rounded-2xl border border-border bg-card p-8">
        <h2 className="text-xl font-bold tracking-tight">{t("ui.pages.apps.appsconnect.who-can-use")}{appName}?</h2>
        <p className="mt-1 text-sm text-muted-foreground">{t("ui.pages.apps.appsconnect.you-can-change-later")}</p>

        <div className="mt-6 space-y-3">
          <button
            type="button"
            onClick={() => setAccess("all")}
            className={cn(
              "flex w-full items-start gap-3 rounded-xl border-2 p-4 text-left transition-colors",
              access === "all" ? "border-foreground bg-muted/40" : "border-border hover:border-foreground/30",
            )}
          >
            <Radio selected={access === "all"} />
            <div>
              <div className="flex items-center gap-2">
                <span className="font-bold text-foreground">{t("pages.appsConnect.allAgents2", { defaultValue: "All agents" })}</span>
                <span className="rounded-full bg-foreground px-2 py-0.5 text-(length:--text-nano) font-bold text-background">
                  {t("components.issueRecoveryAction.recommended")}</span>
              </div>
              <p className="mt-1 text-xs text-muted-foreground">
                {t("ui.pages.apps.appsconnect.anyone-you-ve-added")}{appName} {t("ui.pages.apps.appsconnect.their-tasks-what-most")}</p>
            </div>
          </button>

          <button
            type="button"
            onClick={() => setAccess("specific")}
            className={cn(
              "flex w-full items-start gap-3 rounded-xl border-2 p-4 text-left transition-colors",
              access === "specific" ? "border-foreground bg-muted/40" : "border-border hover:border-foreground/30",
            )}
          >
            <Radio selected={access === "specific"} />
            <div className="flex-1">
              <span className="font-semibold text-foreground">{t("pages.appsConnect.onlySpecific", { defaultValue: "Only specific agents" })}</span>
              <p className="mt-1 text-xs text-muted-foreground">{t("ui.pages.apps.appsconnect.tick-agents-who-can")}{appName}.</p>
            </div>
          </button>

          {access === "specific" && (
            <AgentMultiSelect
              agents={agents}
              selectedAgentIds={agentIds}
              onChange={setAgentIds}
              loading={agentsQuery.isLoading}
              showSelectionPreview={false}
            />
          )}
        </div>
      </div>

      <div className="mt-6 flex items-center justify-between">
        <Button variant="ghost" onClick={onBack}>
          {t("pages.teamCatalog.back")}</Button>
        <Button onClick={onContinue} disabled={!canFinish}>
          {t("ui.pages.apps.appsconnect.continue-install")}</Button>
      </div>
    </div>
  );
}

export function InstallStep({
  appName,
  companyId,
  access,
  accessAgentIds,
  installMode,
  setInstallMode,
  installAgentIds,
  setInstallAgentIds,
  submitting,
  onBack,
  onFinish,
}: {
  appName: string;
  companyId: string;
  access: "all" | "specific";
  accessAgentIds: Set<string>;
  installMode: InstallMode;
  setInstallMode: (mode: InstallMode) => void;
  installAgentIds: Set<string>;
  setInstallAgentIds: (ids: Set<string>) => void;
  submitting: boolean;
  onBack: () => void;
  onFinish: () => void;
}) {
  const agentsQuery = useQuery({
    queryKey: queryKeys.agents.list(companyId),
    queryFn: () => agentsApi.list(companyId),
  });
  const agents: Agent[] = (agentsQuery.data ?? []).filter((a) => a.status !== "terminated");
  const installSpecific = () => {
    setInstallMode("specific");
    if (installAgentIds.size === 0 && access === "specific") setInstallAgentIds(new Set(accessAgentIds));
  };
  const extendingAgentIds = access === "all"
    ? []
    : installMode === "all"
      ? agents.filter((agent) => !accessAgentIds.has(agent.id)).map((agent) => agent.id)
      : [...installAgentIds].filter((id) => !accessAgentIds.has(id));
  const canFinish = installMode !== "specific" || installAgentIds.size > 0;
  const extendingLabel = extendingAgentIds.length === 1
    ? agents.find((agent) => agent.id === extendingAgentIds[0])?.name ?? t("ui.pages.apps.appsconnect.fallback-agent")
    : `${extendingAgentIds.length} agents`;

  return (
    <div className="mx-auto max-w-xl">
      <div className="rounded-2xl border border-border bg-card p-8">
        <h2 className="text-xl font-bold tracking-tight">{t("pages.adapterManager.install")}{appName} {t("ui.pages.apps.appsconnect.tools")}</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          {t("ui.pages.apps.appsconnect.access-permission-install-decides")}</p>

        <div className="mt-5">
          <InlineBanner tone="info" compact>
            {installInfoNotice(appName)}
          </InlineBanner>
        </div>

        <div className="mt-6 space-y-3">
          <button
            type="button"
            onClick={() => setInstallMode("none")}
            className={cn(
              "flex w-full items-start gap-3 rounded-xl border-2 p-4 text-left transition-colors",
              installMode === "none" ? "border-foreground bg-muted/40" : "border-border hover:border-foreground/30",
            )}
          >
            <Radio selected={installMode === "none"} />
            <div>
              <span className="font-semibold text-foreground">{t("pages.appsConnect.notYet", { defaultValue: "Not yet" })}</span>
              <p className="mt-1 text-xs text-muted-foreground">
                {t("ui.pages.apps.appsconnect.keep")}{appName} {t("ui.pages.apps.appsconnect.permitted-only-you-can")}</p>
            </div>
          </button>

          <button
            type="button"
            onClick={installSpecific}
            className={cn(
              "flex w-full items-start gap-3 rounded-xl border-2 p-4 text-left transition-colors",
              installMode === "specific" ? "border-foreground bg-muted/40" : "border-border hover:border-foreground/30",
            )}
          >
            <Radio selected={installMode === "specific"} />
            <div className="flex-1">
              <span className="font-semibold text-foreground">{t("pages.appsConnect.specificAgents2", { defaultValue: "Specific agents" })}</span>
              <p className="mt-1 text-xs text-muted-foreground">{t("ui.pages.apps.appsconnect.tick-agents-should-load")}{appName} {t("ui.pages.apps.appsconnect.every-run")}</p>
            </div>
          </button>

          {installMode === "specific" ? (
            <div className="ml-7 border-l border-border pl-4">
              <AgentMultiSelect
                agents={agents}
                selectedAgentIds={installAgentIds}
                onChange={setInstallAgentIds}
                loading={agentsQuery.isLoading}
                showSelectionPreview={false}
              />
            </div>
          ) : null}

          <button
            type="button"
            onClick={() => setInstallMode("all")}
            className={cn(
              "flex w-full items-start gap-3 rounded-xl border-2 p-4 text-left transition-colors",
              installMode === "all" ? "border-foreground bg-muted/40" : "border-border hover:border-foreground/30",
            )}
          >
            <Radio selected={installMode === "all"} />
            <div>
              <span className="font-semibold text-foreground">{t("pages.appsConnect.allAgents2", { defaultValue: "All agents" })}</span>
              <p className="mt-1 text-xs text-muted-foreground">{INSTALL_ALL_WARNING}</p>
            </div>
          </button>

          {extendingAgentIds.length > 0 ? (
            <InlineBanner tone="warning" compact>
              {autoExtendNotice(extendingLabel)}
            </InlineBanner>
          ) : null}
        </div>
      </div>

      <div className="mt-6 flex items-center justify-between">
        <Button variant="ghost" onClick={onBack} disabled={submitting}>
          {t("pages.teamCatalog.back")}</Button>
        <Button onClick={onFinish} disabled={submitting || !canFinish}>
          {submitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          {submitting ? t("pages.appsConnect.finishing", { defaultValue: "Finishing..." }) : t("pages.appsConnect.finishSetup", { defaultValue: "Finish setup" })}
        </Button>
      </div>
    </div>
  );
}

function Radio({ selected }: { selected: boolean }) {
  return (
    <span
      className={cn(
        "mt-0.5 inline-flex h-4 w-4 shrink-0 items-center justify-center rounded-full border-2",
        selected ? "border-foreground" : "border-muted-foreground/40",
      )}
    >
      {selected && <span className="h-2 w-2 rounded-full bg-foreground" />}
    </span>
  );
}

function SuccessStep({
  appName,
  logoUrl,
  enabledCount,
  access,
  installMode,
  installCount,
  onDone,
}: {
  appName: string;
  logoUrl?: string | null;
  enabledCount: number;
  access: "all" | "specific";
  installMode: InstallMode;
  installCount: number;
  onDone: () => void;
}) {
  const installSummary = installMode === "all"
    ? t("pages.appsConnect.installedAll", { defaultValue: "Installed on all agents" })
    : installMode === "specific"
      ? `${installCount} ${installCount === 1 ? "agent" : "agents"} installed`
      : t("pages.appsConnect.permittedOnly", { defaultValue: "Permitted only" });
  return (
    <div className="mx-auto max-w-md py-10 text-center">
      <div className="mx-auto flex h-20 w-20 items-center justify-center rounded-full border-2 border-emerald-500 bg-emerald-500/10">
        <Check className="h-9 w-9 text-emerald-600 dark:text-emerald-400" />
      </div>
      <div className="mt-6 flex items-center justify-center gap-2">
        <AppLogo name={appName} logoUrl={logoUrl} size={28} />
        <h2 className="text-2xl font-bold tracking-tight">{appName} {t("ui.pages.apps.appsconnect.ready")}</h2>
      </div>
      <p className="mt-2 text-sm text-muted-foreground">
        {installMode === "none"
          ? t("pages.appsConnect.permittedHint", { defaultValue: "Agents can use it after you install it on their Tools tab." })
          : t("pages.appsConnect.installedHint", { defaultValue: "Installed agents will load it on their next run." })}
      </p>
      <p className="mt-1 text-xs text-muted-foreground">
        {enabledCount} {enabledCount === 1 ? "action" : "actions"} {t("ui.pages.apps.appsconnect.text")}{" "}
        {access === "all" ? t("pages.appsConnect.allAgents", { defaultValue: "All agents can use it" }) : t("pages.appsConnect.specificAgents", { defaultValue: "Specific agents can use it" })} · {installSummary}
      </p>
      <div className="mt-8">
        <Button size="lg" className="px-10" onClick={onDone}>
          {t("common.done")}</Button>
      </div>
    </div>
  );
}
