import { useState } from "react";
import { t } from "../i18n";
import {
  BookOpen,
  Bot,
  Check,
  ChevronDown,
  CircleDot,
  Command as CommandIcon,
  DollarSign,
  Hexagon,
  History,
  Inbox,
  LayoutDashboard,
  ListTodo,
  Mail,
  Plus,
  Search,
  Settings,
  Target,
  Trash2,
  Upload,
  User,
  Zap,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { InlineBanner } from "@/components/InlineBanner";
import { BuiltInLifecycleChip } from "@/components/BuiltInAgentBadges";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable-panels";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "@/components/ui/card";
import {
  Dialog,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui/select";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuCheckboxItem,
  DropdownMenuShortcut,
} from "@/components/ui/dropdown-menu";
import {
  Popover,
  PopoverTrigger,
  PopoverContent,
} from "@/components/ui/popover";
import {
  Sheet,
  SheetTrigger,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
  SheetFooter,
} from "@/components/ui/sheet";
import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "@/components/ui/collapsible";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Command,
  CommandInput,
  CommandList,
  CommandGroup,
  CommandItem,
  CommandEmpty,
  CommandSeparator,
} from "@/components/ui/command";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import {
  Avatar,
  AvatarFallback,
  AvatarGroup,
  AvatarGroupCount,
} from "@/components/ui/avatar";
import { AgentCapsule, AGENT_GRADIENT_COUNT } from "@/components/AgentCapsule";
import { StatusBadge, IssueStatusBadge } from "@/components/StatusBadge";
import { StatusIcon } from "@/components/StatusIcon";
import { EnforcementBanner } from "@/components/EnforcementBanner";
import { ActionCard, ActionCardMobile, BindingsTable } from "@/components/actions/ActionCard";
import { PriorityIcon } from "@/components/PriorityIcon";
import { agentStatusDot, agentStatusDotDefault } from "@/lib/status-colors";
import { EntityRow } from "@/components/EntityRow";
import { EmptyState } from "@/components/EmptyState";
import { MetricCard } from "@/components/MetricCard";
import { FilterBar, type FilterValue } from "@/components/FilterBar";
import { InlineEditor } from "@/components/InlineEditor";
import { PageSkeleton } from "@/components/PageSkeleton";
import { Identity } from "@/components/Identity";
import { IssueReferencePill } from "@/components/IssueReferencePill";
import { MembershipAction } from "@/components/MembershipAction";
import { IssueOutputSection } from "@/components/issue-output/IssueOutputSection";
import { EnvironmentVariablesEditor } from "@/components/environment-variables-editor";
import type { CompanySecret, EnvBinding } from "@paperclipai/shared";
import {
  EnvInputsList,
  ExternalSourcesList,
  RequiredSkillsList,
  StepSkillPlan,
  StepSourcePolicy,
  TeamCard,
  TeamHierarchyPreview,
  TeamRow,
} from "@/pages/TeamCatalog";
import {
  currentInstalledState,
  onboardingTeams,
  optionalTeam,
  outOfDateInstalledState,
  sampleSkillPreparations,
  sampleTeam,
  warnTeam,
} from "@/pages/TeamCatalog.fixtures";
import type { IssueWorkProduct } from "@paperclipai/shared";

/* ------------------------------------------------------------------ */
/*  Sample data for the Issue Output surface showcase                  */
/* ------------------------------------------------------------------ */

function sampleOutput(
  id: string,
  attachmentId: string,
  contentType: string,
  filename: string,
  opts: { byteSize: number; isPrimary?: boolean; createdAt: string },
): IssueWorkProduct {
  const contentPath = `/api/attachments/${attachmentId}/content`;
  return {
    id,
    companyId: "demo-company",
    projectId: null,
    issueId: "demo-issue",
    executionWorkspaceId: null,
    runtimeServiceId: null,
    type: "artifact",
    provider: "paperclip",
    externalId: null,
    title: filename,
    url: null,
    status: "active",
    reviewState: "none",
    isPrimary: Boolean(opts.isPrimary),
    healthStatus: "unknown",
    summary: null,
    createdByRunId: null,
    createdAt: new Date(opts.createdAt),
    updatedAt: new Date(opts.createdAt),
    metadata: {
      attachmentId,
      contentType,
      byteSize: opts.byteSize,
      contentPath,
      openPath: contentPath,
      downloadPath: `${contentPath}?download=1`,
      originalFilename: filename,
    },
  } as IssueWorkProduct;
}

const DESIGN_GUIDE_OUTPUTS: IssueWorkProduct[] = [
  sampleOutput("wp-vid", "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", "video/mp4", "q3-summary.mp4", {
    byteSize: 19_293_798,
    isPrimary: true,
    createdAt: "2026-05-30T12:00:00Z",
  }),
  sampleOutput("wp-pdf", "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", "application/pdf", "talking-points.pdf", {
    byteSize: 421_888,
    createdAt: "2026-05-30T11:52:00Z",
  }),
];

const DESIGN_GUIDE_DEGRADED_OUTPUTS: IssueWorkProduct[] = [
  {
    ...sampleOutput("wp-broken", "cccccccc-cccc-4ccc-8ccc-cccccccccccc", "video/mp4", "corrupt-output.mp4", {
      byteSize: 0,
      isPrimary: true,
      createdAt: "2026-05-30T12:01:00Z",
    }),
    // Strip the path metadata so it fails the shared artifact schema.
    metadata: { attachmentId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc", contentType: "video/mp4" },
  } as IssueWorkProduct,
];

/* ------------------------------------------------------------------ */
/*  Section wrapper                                                    */
/* ------------------------------------------------------------------ */

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="space-y-4">
      <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wide">
        {title}
      </h3>
      <Separator />
      {children}
    </section>
  );
}

function SubSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-3">
      <h4 className="text-sm font-medium">{title}</h4>
      {children}
    </div>
  );
}

// Onboarding seam (design §6 + §12.5): the TeamCard tile in its "Pick a starter
// team" 3-col grid, with the first defaultInstall tile selected.
function TeamCardShowcase() {
  const [selectedId, setSelectedId] = useState(onboardingTeams[0]?.id ?? null);
  return (
    <div className="grid max-w-2xl gap-4 md:grid-cols-2 lg:grid-cols-3">
      {onboardingTeams.map((team) => (
        <TeamCard
          key={team.id}
          team={team}
          selected={team.id === selectedId}
          onSelect={() => setSelectedId(team.id)}
        />
      ))}
    </div>
  );
}

// Reusable environment-variables editor: one shared grid, in-field source
// switch, fuzzy secret picker, sensitive-value detection, inline health.
const DESIGN_GUIDE_SECRETS: CompanySecret[] = [
  {
    id: "dg-github",
    companyId: "dg",
    scope: "company",
    ownerUserId: null,
    userSecretDefinitionId: null,
    key: "github_token",
    name: "GITHUB_TOKEN",
    provider: "local_encrypted",
    status: "active",
    managedMode: "paperclip_managed",
    externalRef: null,
    providerConfigId: null,
    providerMetadata: null,
    latestVersion: 3,
    description: null,
    lastResolvedAt: null,
    lastRotatedAt: null,
    deletedAt: null,
    createdByAgentId: null,
    createdByUserId: null,
    createdAt: new Date("2026-03-01T10:00:00.000Z"),
    updatedAt: new Date("2026-03-01T10:00:00.000Z"),
  },
  {
    id: "dg-db",
    companyId: "dg",
    scope: "company",
    ownerUserId: null,
    userSecretDefinitionId: null,
    key: "db_connection",
    name: "DB_CONNECTION",
    provider: "local_encrypted",
    status: "active",
    managedMode: "paperclip_managed",
    externalRef: null,
    providerConfigId: null,
    providerMetadata: null,
    latestVersion: 3,
    description: null,
    lastResolvedAt: null,
    lastRotatedAt: null,
    deletedAt: null,
    createdByAgentId: null,
    createdByUserId: null,
    createdAt: new Date("2026-03-01T10:00:00.000Z"),
    updatedAt: new Date("2026-03-01T10:00:00.000Z"),
  },
];

function EnvironmentVariablesEditorShowcase() {
  const [env, setEnv] = useState<Record<string, EnvBinding>>({
    NODE_ENV: { type: "plain", value: "production" },
    GH_TOKEN: { type: "secret_ref", secretId: "dg-github", version: "latest" },
    DB_URL: { type: "secret_ref", secretId: "dg-db", version: 3 },
    STRIPE_API_KEY: { type: "plain", value: "sk-live-51H8xL0aBcDeFgHiJkLmNoPq" },
  });
  return (
    <div className="max-w-(--sz-640px) rounded-md border border-border p-4">
      <EnvironmentVariablesEditor
        value={env}
        secrets={DESIGN_GUIDE_SECRETS}
        onChange={(next) => setEnv(next ?? {})}
        onCreateSecret={async (name) => ({
          ...DESIGN_GUIDE_SECRETS[0]!,
          id: `dg-${name}`,
          key: name,
          name: name.toUpperCase(),
          latestVersion: 1,
        })}
      />
    </div>
  );
}

/* ------------------------------------------------------------------ */
/*  Color swatch                                                       */
/* ------------------------------------------------------------------ */

function Swatch({ name, cssVar }: { name: string; cssVar: string }) {
  return (
    <div className="flex items-center gap-3">
      <div
        className="h-8 w-8 rounded-md border border-border shrink-0"
        style={{ backgroundColor: `var(${cssVar})` }}
      />
      <div>
        <p className="text-xs font-mono">{cssVar}</p>
        <p className="text-xs text-muted-foreground">{name}</p>
      </div>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/*  Page                                                               */
/* ------------------------------------------------------------------ */

export function DesignGuide() {
  const [status, setStatus] = useState("todo");
  const [priority, setPriority] = useState("medium");
  const [selectValue, setSelectValue] = useState("in_progress");
  const [menuChecked, setMenuChecked] = useState(true);
  const [collapsibleOpen, setCollapsibleOpen] = useState(false);
  const [inlineText, setInlineText] = useState(t("pages.designGuide.clickToEdit", { defaultValue: "Click to edit this text" }));
  const [inlineTitle, setInlineTitle] = useState(t("pages.designGuide.editableTitle", { defaultValue: "Editable Title" }));
  const [inlineDesc, setInlineDesc] = useState(
    "This is an editable description. Click to edit it — the textarea auto-sizes to fit the content without layout shift."
  );
  const [filters, setFilters] = useState<FilterValue[]>([
    { key: "status", label: t("pages.designGuide.status", { defaultValue: "Status" }), value: t("pages.designGuide.active", { defaultValue: "Active" }) },
    { key: "priority", label: t("pages.designGuide.priority", { defaultValue: "Priority" }), value: t("pages.designGuide.high", { defaultValue: "High" }) },
  ]);
  const [allowExternal, setAllowExternal] = useState(false);
  const [allowUnpinned, setAllowUnpinned] = useState(false);
  const [allowLocalPath, setAllowLocalPath] = useState(false);

  return (
    <div className="space-y-10 max-w-4xl">
      {/* Page header */}
      <div>
        <h2 className="text-xl font-bold">{t("pages.designGuide.title", { defaultValue: "Design Guide" })}</h2>
        <p className="text-sm text-muted-foreground mt-1">
          {t("ui.pages.designguide.every-component-style-pattern")}</p>
      </div>

      {/* ============================================================ */}
      {/*  COVERAGE                                                     */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.componentCoverage", { defaultValue: "Component Coverage" })}>
        <p className="text-sm text-muted-foreground">
          {t("ui.pages.designguide.page-should-updated-when")}</p>
        <div className="grid gap-6 md:grid-cols-2">
          <SubSection title={t("pages.designGuide.uiPrimitives", { defaultValue: "UI primitives" })}>
            <div className="flex flex-wrap gap-2">
              {[
                "avatar", "badge", "breadcrumb", "button", "card", "checkbox", "collapsible",
                "command", "dialog", "dropdown-menu", "input", "label", "popover", "resizable-panels",
                "scroll-area", "select", "separator", "sheet", "skeleton", "tabs", "textarea", "tooltip",
              ].map((name) => (
                <Badge key={name} variant="outline" className="font-mono text-(length:--text-nano)">
                  {name}
                </Badge>
              ))}
            </div>
          </SubSection>
          <SubSection title={t("pages.designGuide.appComponents", { defaultValue: "App components" })}>
            <div className="flex flex-wrap gap-2">
              {[
                "StatusBadge", "StatusIcon", "PriorityIcon", "EntityRow", "EmptyState", "MetricCard",
                "FilterBar", "InlineEditor", "PageSkeleton", "Identity", "CommentThread", "MarkdownEditor",
                "PropertiesPanel", "Sidebar", "CommandPalette", "EnvironmentVariablesEditor",
                "InlineBanner", "BuiltInAgentGate", "BuiltInLifecycleChip",
              ].map((name) => (
                <Badge key={name} variant="ghost" className="font-mono text-(length:--text-nano)">
                  {name}
                </Badge>
              ))}
            </div>
          </SubSection>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  COLORS                                                       */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.colors", { defaultValue: "Colors" })}>
        <SubSection title={t("pages.designGuide.core", { defaultValue: "Core" })}>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            <Swatch name={t("pages.designGuide.background", { defaultValue: "Background" })} cssVar="--background" />
            <Swatch name={t("pages.designGuide.foreground", { defaultValue: "Foreground" })} cssVar="--foreground" />
            <Swatch name={t("pages.designGuide.card", { defaultValue: "Card" })} cssVar="--card" />
            <Swatch name={t("pages.designGuide.primary", { defaultValue: "Primary" })} cssVar="--primary" />
            <Swatch name={t("pages.designGuide.primaryForeground", { defaultValue: "Primary foreground" })} cssVar="--primary-foreground" />
            <Swatch name={t("pages.designGuide.secondary", { defaultValue: "Secondary" })} cssVar="--secondary" />
            <Swatch name={t("pages.designGuide.muted", { defaultValue: "Muted" })} cssVar="--muted" />
            <Swatch name={t("pages.designGuide.mutedForeground", { defaultValue: "Muted foreground" })} cssVar="--muted-foreground" />
            <Swatch name={t("pages.designGuide.accent", { defaultValue: "Accent" })} cssVar="--accent" />
            <Swatch name={t("pages.designGuide.destructive", { defaultValue: "Destructive" })} cssVar="--destructive" />
            <Swatch name="Border" cssVar="--border" />
            <Swatch name={t("pages.designGuide.ring", { defaultValue: "Ring" })} cssVar="--ring" />
          </div>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.sidebar")}>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            <Swatch name="Sidebar" cssVar="--sidebar" />
            <Swatch name={t("pages.designGuide.sidebarBorder", { defaultValue: "Sidebar border" })} cssVar="--sidebar-border" />
          </div>
        </SubSection>

        <SubSection title={t("pages.designGuide.chart", { defaultValue: "Chart" })}>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            <Swatch name={t("pages.designGuide.chart1", { defaultValue: "Chart 1" })} cssVar="--chart-1" />
            <Swatch name={t("pages.designGuide.chart2", { defaultValue: "Chart 2" })} cssVar="--chart-2" />
            <Swatch name={t("pages.designGuide.chart3", { defaultValue: "Chart 3" })} cssVar="--chart-3" />
            <Swatch name={t("pages.designGuide.chart4", { defaultValue: "Chart 4" })} cssVar="--chart-4" />
            <Swatch name={t("pages.designGuide.chart5", { defaultValue: "Chart 5" })} cssVar="--chart-5" />
          </div>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  TYPOGRAPHY                                                   */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.typography", { defaultValue: "Typography" })}>
        <div className="space-y-3">
          <h2 className="text-xl font-bold">{t("ui.pages.designguide.page-title-text-xl")}</h2>
          <h2 className="text-lg font-semibold">{t("ui.pages.designguide.section-title-text-lg")}</h2>
          <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wide">
            {t("ui.pages.designguide.section-heading-text-sm")}</h3>
          <p className="text-sm font-medium">{t("ui.pages.designguide.card-title-text-sm")}</p>
          <p className="text-sm font-semibold">{t("ui.pages.designguide.card-title-alt-text")}</p>
          <p className="text-sm">{t("ui.pages.designguide.body-text-text-sm")}</p>
          <p className="text-sm text-muted-foreground">
            {t("ui.pages.designguide.muted-description-text-sm")}</p>
          <p className="text-xs text-muted-foreground">
            {t("ui.pages.designguide.tiny-label-text-xs")}</p>
          <p className="text-sm font-mono text-muted-foreground">
            {t("ui.pages.designguide.mono-identifier-text-sm")}</p>
          <p className="text-2xl font-bold">{t("ui.pages.designguide.large-stat-text-2xl")}</p>
          <p className="font-mono text-xs">{t("ui.pages.designguide.log-code-text-font")}</p>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  SPACING & RADIUS                                             */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.radius", { defaultValue: "Radius" })}>
        <div className="flex items-end gap-4 flex-wrap">
          {[
            ["sm", "var(--radius-sm)"],
            ["md", "var(--radius-md)"],
            ["lg", "var(--radius-lg)"],
            ["xl", "var(--radius-xl)"],
            ["full", "9999px"],
          ].map(([label, radius]) => (
            <div key={label} className="flex flex-col items-center gap-1">
              <div
                className="h-12 w-12 bg-primary"
                style={{ borderRadius: radius }}
              />
              <span className="text-xs text-muted-foreground">{label}</span>
            </div>
          ))}
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  BUTTONS                                                      */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.buttons", { defaultValue: "Buttons" })}>
        <SubSection title={t("pages.designGuide.variants", { defaultValue: "Variants" })}>
          <div className="flex items-center gap-2 flex-wrap">
            <Button variant="default">{t("pages.designGuide.default", { defaultValue: "Default" })}</Button>
            <Button variant="secondary">{t("pages.designGuide.secondary", { defaultValue: "Secondary" })}</Button>
            <Button variant="outline">{t("pages.designGuide.outline", { defaultValue: "Outline" })}</Button>
            <Button variant="ghost">{t("pages.designGuide.ghost", { defaultValue: "Ghost" })}</Button>
            <Button variant="destructive">{t("pages.designGuide.destructive", { defaultValue: "Destructive" })}</Button>
            <Button variant="link">{t("pages.designGuide.link", { defaultValue: "Link" })}</Button>
          </div>
        </SubSection>

        <SubSection title={t("pages.designGuide.sizes", { defaultValue: "Sizes" })}>
          <div className="flex items-center gap-2 flex-wrap">
            <Button size="xs">{t("pages.designGuide.extraSmall", { defaultValue: "Extra Small" })}</Button>
            <Button size="sm">{t("pages.designGuide.small", { defaultValue: "Small" })}</Button>
            <Button size="default">{t("pages.designGuide.default", { defaultValue: "Default" })}</Button>
            <Button size="lg">{t("pages.designGuide.large", { defaultValue: "Large" })}</Button>
          </div>
        </SubSection>

        <SubSection title={t("pages.designGuide.iconButtons", { defaultValue: "Icon buttons" })}>
          <div className="flex items-center gap-2 flex-wrap">
            <Button variant="ghost" size="icon-xs"><Search /></Button>
            <Button variant="ghost" size="icon-sm"><Search /></Button>
            <Button variant="outline" size="icon"><Search /></Button>
            <Button variant="outline" size="icon-lg"><Search /></Button>
          </div>
        </SubSection>

        <SubSection title={t("pages.designGuide.withIcons", { defaultValue: "With icons" })}>
          <div className="flex items-center gap-2 flex-wrap">
            <Button><Plus /> {t("ui.pages.designguide.new-issue")}</Button>
            <Button variant="outline"><Upload /> {t("pages.issueDetail.upload")}</Button>
            <Button variant="destructive"><Trash2 /> {t("common.delete")}</Button>
            <Button size="sm"><Plus /> {t("components.jsonSchemaForm.add")}</Button>
          </div>
        </SubSection>

        <SubSection title={t("pages.designGuide.states", { defaultValue: "States" })}>
          <div className="flex items-center gap-2 flex-wrap">
            <Button disabled>{t("pages.designGuide.disabled", { defaultValue: "Disabled" })}</Button>
            <Button variant="outline" disabled>{t("pages.designGuide.disabledOutline", { defaultValue: "Disabled Outline" })}</Button>
          </div>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  BADGES                                                       */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.badges", { defaultValue: "Badges" })}>
        <SubSection title={t("pages.designGuide.variants", { defaultValue: "Variants" })}>
          <div className="flex items-center gap-2 flex-wrap">
            <Badge variant="default">{t("pages.designGuide.default", { defaultValue: "Default" })}</Badge>
            <Badge variant="secondary">{t("pages.designGuide.secondary", { defaultValue: "Secondary" })}</Badge>
            <Badge variant="outline">{t("pages.designGuide.outline", { defaultValue: "Outline" })}</Badge>
            <Badge variant="destructive">{t("pages.designGuide.destructive", { defaultValue: "Destructive" })}</Badge>
            <Badge variant="ghost">{t("pages.designGuide.ghost", { defaultValue: "Ghost" })}</Badge>
          </div>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  STATUS BADGES & ICONS                                        */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.statusSystem", { defaultValue: "Status System" })}>
        <SubSection title={t("ui.pages.designguide.statusbadge-all-statuses")}>
          <div className="flex items-center gap-2 flex-wrap">
            {[
              "active", "running", "paused", "idle", "archived", "planned",
              "achieved", "completed", "failed", "timed_out", "succeeded", "error",
              "pending_approval", "backlog", "todo", "in_progress", "in_review", "blocked",
              "done", "terminated", "cancelled", "pending", "revision_requested",
              "approved", "rejected",
            ].map((s) => (
              <StatusBadge key={s} status={s} />
            ))}
          </div>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.issuestatusbadge-brand-chip-glyph")}>
          <div className="flex items-center gap-2 flex-wrap">
            {["backlog", "todo", "in_progress", "in_review", "done", "blocked", "cancelled"].map(
              (s) => (
                <IssueStatusBadge key={s} status={s} />
              )
            )}
          </div>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.statusicon-interactive")}>
          <div className="flex items-center gap-3 flex-wrap">
            {["backlog", "todo", "in_progress", "in_review", "done", "cancelled", "blocked"].map(
              (s) => (
                <div key={s} className="flex items-center gap-1.5">
                  <StatusIcon status={s} />
                  <span className="text-xs text-muted-foreground">{s}</span>
                </div>
              )
            )}
          </div>
          <div className="flex items-center gap-2 mt-2">
            <StatusIcon status={status} onChange={setStatus} />
            <span className="text-sm">{t("ui.pages.designguide.click-icon-change-status")}{status})</span>
          </div>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.priorityicon-interactive")}>
          <div className="flex items-center gap-3 flex-wrap">
            {["critical", "high", "medium", "low"].map((p) => (
              <div key={p} className="flex items-center gap-1.5">
                <PriorityIcon priority={p} />
                <span className="text-xs text-muted-foreground">{p}</span>
              </div>
            ))}
          </div>
          <div className="flex items-center gap-2 mt-2">
            <PriorityIcon priority={priority} onChange={setPriority} />
            <span className="text-sm">{t("ui.pages.designguide.click-icon-change-current")}{priority})</span>
          </div>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.agent-status-dots")}>
          <div className="flex items-center gap-4 flex-wrap">
            {(["running", "active", "paused", "error", "archived"] as const).map((label) => (
              <div key={label} className="flex items-center gap-2">
                <span className="relative flex h-2.5 w-2.5">
                  <span className={`inline-flex h-full w-full rounded-full ${agentStatusDot[label] ?? agentStatusDotDefault}`} />
                </span>
                <span className="text-xs text-muted-foreground">{label}</span>
              </div>
            ))}
          </div>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.run-invocation-badges")}>
          <div className="flex items-center gap-2 flex-wrap">
            {[
              ["timer", "bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-300"],
              ["assignment", "bg-violet-100 text-violet-700 dark:bg-violet-900/50 dark:text-violet-300"],
              ["on_demand", "bg-cyan-100 text-cyan-700 dark:bg-cyan-900/50 dark:text-cyan-300"],
              ["automation", "bg-muted text-muted-foreground"],
            ].map(([label, cls]) => (
              <Badge variant="ghost" key={label} className={`px-1.5 text-(length:--text-nano) ${cls}`}>
                {label}
              </Badge>
            ))}
          </div>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.issuereferencepill")}>
          <p className="text-xs text-muted-foreground">
            {t("ui.pages.designguide.used-wherever-task-referenced")}<code className="font-mono">status</code> {t("ui.pages.designguide.show-target-issue-apos")}<code className="font-mono">strikethrough</code> {t("ui.pages.designguide.quot-removed-quot-contexts")}</p>
          <div className="flex items-center gap-2 flex-wrap">
            <IssueReferencePill issue={{ id: "demo-1", identifier: "PAP-123", title: "Identifier only — no status yet" }} />
            <IssueReferencePill issue={{ id: "demo-2", identifier: "PAP-456", title: "With in_progress status", status: "in_progress" }} />
            <IssueReferencePill issue={{ id: "demo-3", identifier: "PAP-789", title: "Done status", status: "done" }} />
            <IssueReferencePill issue={{ id: "demo-4", identifier: "PAP-101", title: "Blocked status", status: "blocked" }} />
            <IssueReferencePill strikethrough issue={{ id: "demo-5", identifier: "PAP-202", title: "Removed (strikethrough)", status: "todo" }} />
          </div>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  AGENT CAPSULE                                                */}
      {/* ============================================================ */}
      <Section title={t("ui.pages.designguide.agent-capsule")}>
        <p className="text-sm text-muted-foreground max-w-prose">
          {t("ui.pages.designguide.brand-quot-capsule-agent")}<code className="font-mono">{t("ui.pages.designguide.agent-na")}</code> →{" "}
          <code className="font-mono">{t("ui.pages.designguide.agent-nb")}</code>); <code className="font-mono">prefers-reduced-motion</code>{" "}
          {t("ui.pages.designguide.skips-liquid-rise-pulses")}</p>
        <SubSection title={t("pages.designGuide.states", { defaultValue: "States" })}>
          <div className="flex items-end gap-10">
            <div className="flex flex-col items-center gap-2">
              <AgentCapsule state="slot" />
              <span className="text-xs text-muted-foreground">slot</span>
            </div>
            <div className="flex flex-col items-center gap-2">
              <AgentCapsule state="configured" />
              <span className="text-xs text-muted-foreground">configured</span>
            </div>
            <div className="flex flex-col items-center gap-2">
              <AgentCapsule state="online" gradient={5} />
              <span className="text-xs text-muted-foreground">online</span>
            </div>
            <div className="flex flex-col items-center gap-2">
              <AgentCapsule state="online" gradient={5} glow="blue" />
              <span className="text-xs text-muted-foreground">{t("ui.pages.designguide.online-blue-glow")}</span>
            </div>
          </div>
        </SubSection>
        <SubSection title={t("pages.designGuide.sizes", { defaultValue: "Sizes" })}>
          <div className="flex items-end gap-8">
            <div className="flex flex-col items-center gap-2">
              <AgentCapsule state="online" size="sm" gradient={1} />
              <span className="text-xs text-muted-foreground">sm</span>
            </div>
            <div className="flex flex-col items-center gap-2">
              <AgentCapsule state="online" size="md" gradient={4} />
              <span className="text-xs text-muted-foreground">md</span>
            </div>
            <div className="flex flex-col items-center gap-2">
              <AgentCapsule state="online" size="lg" gradient={8} />
              <span className="text-xs text-muted-foreground">lg</span>
            </div>
            <div className="flex flex-col items-center gap-2">
              <AgentCapsule state="online" size={{ width: 28, height: 96 }} gradient={6} />
              <span className="text-xs text-muted-foreground">{t("ui.pages.designguide.custom-px")}</span>
            </div>
          </div>
        </SubSection>
        <SubSection title={t("pages.designGuide.gradients", { defaultValue: "Gradients" })}>
          <div className="flex items-end gap-3 flex-wrap">
            {Array.from({ length: AGENT_GRADIENT_COUNT }, (_, i) => (
              <div key={i} className="flex flex-col items-center gap-1.5">
                <AgentCapsule state="online" size="sm" gradient={i + 1} />
                <span className="text-(length:--text-nano) font-mono text-muted-foreground">{i + 1}</span>
              </div>
            ))}
          </div>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  FORM ELEMENTS                                                */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.formElements", { defaultValue: "Form Elements" })}>
        <div className="grid gap-6 md:grid-cols-2">
          <SubSection title={t("pages.designGuide.input", { defaultValue: "Input" })}>
            <Input placeholder={t("pages.designGuide.defaultInput", { defaultValue: "Default input" })} />
            <Input placeholder={t("pages.designGuide.disabledInput", { defaultValue: "Disabled input" })} disabled className="mt-2" />
          </SubSection>

          <SubSection title={t("pages.designGuide.textarea", { defaultValue: "Textarea" })}>
            <Textarea placeholder={t("pages.designGuide.writeSomething", { defaultValue: "Write something..." })} />
          </SubSection>

          <SubSection title={t("pages.designGuide.checkboxLabel", { defaultValue: "Checkbox & Label" })}>
            <div className="space-y-3">
              <div className="flex items-center gap-2">
                <Checkbox id="check1" defaultChecked />
                <Label htmlFor="check1">{t("pages.designGuide.checkedItem", { defaultValue: "Checked item" })}</Label>
              </div>
              <div className="flex items-center gap-2">
                <Checkbox id="check2" />
                <Label htmlFor="check2">{t("pages.designGuide.uncheckedItem", { defaultValue: "Unchecked item" })}</Label>
              </div>
              <div className="flex items-center gap-2">
                <Checkbox id="check3" disabled />
                <Label htmlFor="check3">{t("pages.designGuide.disabledItem", { defaultValue: "Disabled item" })}</Label>
              </div>
            </div>
          </SubSection>

          <SubSection title={t("ui.pages.designguide.inline-editor")}>
            <div className="space-y-4">
              <div>
                <p className="text-xs text-muted-foreground mb-1">{t("pages.designGuide.titleSingle", { defaultValue: "Title (single-line)" })}</p>
                <InlineEditor
                  value={inlineTitle}
                  onSave={setInlineTitle}
                  as="h2"
                  className="text-xl font-bold"
                />
              </div>
              <div>
                <p className="text-xs text-muted-foreground mb-1">{t("pages.designGuide.bodySingle", { defaultValue: "Body text (single-line)" })}</p>
                <InlineEditor
                  value={inlineText}
                  onSave={setInlineText}
                  as="p"
                  className="text-sm"
                />
              </div>
              <div>
                <p className="text-xs text-muted-foreground mb-1">{t("pages.designGuide.descMultiline", { defaultValue: "Description (multiline, auto-sizing)" })}</p>
                <InlineEditor
                  value={inlineDesc}
                  onSave={setInlineDesc}
                  as="p"
                  className="text-sm text-muted-foreground"
                  placeholder={t("pages.designGuide.addDescription", { defaultValue: "Add a description..." })}
                  multiline
                />
              </div>
            </div>
          </SubSection>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  SELECT                                                       */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.select", { defaultValue: "Select" })}>
        <div className="grid gap-6 md:grid-cols-2">
          <SubSection title={t("pages.designGuide.defaultSize", { defaultValue: "Default size" })}>
            <Select value={selectValue} onValueChange={setSelectValue}>
              <SelectTrigger className="w-full">
                <SelectValue placeholder={t("pages.designGuide.selectStatus", { defaultValue: "Select status" })} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="backlog">{t("pages.designGuide.backlog", { defaultValue: "Backlog" })}</SelectItem>
                <SelectItem value="todo">{t("pages.designGuide.todo", { defaultValue: "Todo" })}</SelectItem>
                <SelectItem value="in_progress">{t("pages.designGuide.inProgress", { defaultValue: "In Progress" })}</SelectItem>
                <SelectItem value="in_review">{t("pages.designGuide.inReview", { defaultValue: "In Review" })}</SelectItem>
                <SelectItem value="done">{t("pages.designGuide.done", { defaultValue: "Done" })}</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">{t("ui.pages.designguide.current-value")}{selectValue}</p>
          </SubSection>
          <SubSection title={t("pages.designGuide.smallTrigger", { defaultValue: "Small trigger" })}>
            <Select defaultValue="high">
              <SelectTrigger size="sm" className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="critical">{t("pages.designGuide.critical", { defaultValue: "Critical" })}</SelectItem>
                <SelectItem value="high">{t("pages.designGuide.high", { defaultValue: "High" })}</SelectItem>
                <SelectItem value="medium">{t("pages.designGuide.medium", { defaultValue: "Medium" })}</SelectItem>
                <SelectItem value="low">{t("pages.designGuide.low", { defaultValue: "Low" })}</SelectItem>
              </SelectContent>
            </Select>
          </SubSection>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  DROPDOWN MENU                                                */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.dropdownMenu", { defaultValue: "Dropdown Menu" })}>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              {t("ui.pages.designguide.quick-actions")}<ChevronDown className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start" className="w-56">
            <DropdownMenuItem>
              <Check className="h-4 w-4" />
              {t("ui.pages.designguide.mark-done")}<DropdownMenuShortcut>⌘D</DropdownMenuShortcut>
            </DropdownMenuItem>
            <DropdownMenuItem>
              <BookOpen className="h-4 w-4" />
              {t("ui.pages.designguide.open-docs")}</DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuCheckboxItem
              checked={menuChecked}
              onCheckedChange={(value) => setMenuChecked(value === true)}
            >
              {t("ui.pages.designguide.watch-issue")}</DropdownMenuCheckboxItem>
            <DropdownMenuItem variant="destructive">
              <Trash2 className="h-4 w-4" />
              {t("ui.pages.designguide.delete-issue")}</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </Section>

      {/* ============================================================ */}
      {/*  POPOVER                                                      */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.popover", { defaultValue: "Popover" })}>
        <Popover>
          <PopoverTrigger asChild>
            <Button variant="outline" size="sm">{t("pages.designGuide.openPopover", { defaultValue: "Open Popover" })}</Button>
          </PopoverTrigger>
          <PopoverContent className="space-y-2">
            <p className="text-sm font-medium">{t("pages.designGuide.agentHeartbeat", { defaultValue: "Agent heartbeat" })}</p>
            <p className="text-xs text-muted-foreground">
              {t("ui.pages.designguide.last-run-succeeded-24s")}</p>
            <Button size="xs">{t("pages.designGuide.wakeNow", { defaultValue: "Wake now" })}</Button>
          </PopoverContent>
        </Popover>
      </Section>

      {/* ============================================================ */}
      {/*  COLLAPSIBLE                                                  */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.collapsible", { defaultValue: "Collapsible" })}>
        <Collapsible open={collapsibleOpen} onOpenChange={setCollapsibleOpen} className="space-y-2">
          <CollapsibleTrigger asChild>
            <Button variant="outline" size="sm">
              {collapsibleOpen ? t("pages.designGuide.hide", { defaultValue: "Hide" }) : t("pages.designGuide.show", { defaultValue: "Show" })} {t("ui.pages.designguide.advanced-filters")}</Button>
          </CollapsibleTrigger>
          <CollapsibleContent className="rounded-md border border-border p-3">
            <div className="space-y-2">
              <Label htmlFor="owner-filter">{t("pages.designGuide.owner", { defaultValue: "Owner" })}</Label>
              <Input id="owner-filter" placeholder={t("pages.designGuide.filterByAgent", { defaultValue: "Filter by agent name" })} />
            </div>
          </CollapsibleContent>
        </Collapsible>
      </Section>

      {/* ============================================================ */}
      {/*  SHEET                                                        */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.sheet", { defaultValue: "Sheet" })}>
        <Sheet>
          <SheetTrigger asChild>
            <Button variant="outline" size="sm">{t("pages.designGuide.openSidePanel", { defaultValue: "Open Side Panel" })}</Button>
          </SheetTrigger>
          <SheetContent side="right">
            <SheetHeader>
              <SheetTitle>{t("pages.designGuide.issueProperties", { defaultValue: "Issue Properties" })}</SheetTitle>
              <SheetDescription>{t("pages.designGuide.editMetadataHint", { defaultValue: "Edit metadata without leaving the current page." })}</SheetDescription>
            </SheetHeader>
            <div className="space-y-4 px-4">
              <div className="space-y-1">
                <Label htmlFor="sheet-title">{t("pages.designGuide.title2", { defaultValue: "Title" })}</Label>
                <Input id="sheet-title" defaultValue={t("pages.designGuide.onboardingDoc", { defaultValue: "Improve onboarding docs" })} />
              </div>
              <div className="space-y-1">
                <Label htmlFor="sheet-description">{t("pages.designGuide.description", { defaultValue: "Description" })}</Label>
                <Textarea id="sheet-description" defaultValue={t("pages.designGuide.onboardingDocHint", { defaultValue: "Capture setup pitfalls and screenshots." })} />
              </div>
            </div>
            <SheetFooter>
              <Button variant="outline">{t("pages.designGuide.cancel", { defaultValue: "Cancel" })}</Button>
              <Button>{t("pages.designGuide.save", { defaultValue: "Save" })}</Button>
            </SheetFooter>
          </SheetContent>
        </Sheet>
      </Section>

      {/* ============================================================ */}
      {/*  SCROLL AREA                                                  */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.scrollArea", { defaultValue: "Scroll Area" })}>
        <ScrollArea className="h-36 rounded-md border border-border">
          <div className="space-y-2 p-3">
            {Array.from({ length: 12 }).map((_, i) => (
              <div key={i} className="rounded-md border border-border p-2 text-sm">
                {t("ui.pages.designguide.heartbeat-run")}{i + 1}{t("ui.pages.designguide.completed-successfully")}</div>
            ))}
          </div>
        </ScrollArea>
      </Section>

      {/* ============================================================ */}
      {/*  COMMAND                                                      */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.commandPalette", { defaultValue: "Command (CMDK)" })}>
        <div className="rounded-md border border-border">
          <Command>
            <CommandInput placeholder={t("pages.designGuide.commandPlaceholder", { defaultValue: "Type a command or search..." })} />
            <CommandList>
              <CommandEmpty>{t("pages.designGuide.noResults", { defaultValue: "No results found." })}</CommandEmpty>
              <CommandGroup heading={t("pages.designGuide.pages", { defaultValue: "Pages" })}>
                <CommandItem>
                  <LayoutDashboard className="h-4 w-4" />
                  {t("nav.dashboard")}</CommandItem>
                <CommandItem>
                  <CircleDot className="h-4 w-4" />
                  {t("ui.pages.designguide.issues")}</CommandItem>
              </CommandGroup>
              <CommandSeparator />
              <CommandGroup heading={t("pages.designGuide.actions", { defaultValue: "Actions" })}>
                <CommandItem>
                  <CommandIcon className="h-4 w-4" />
                  {t("pages.designGuide.openCommandPalette")}</CommandItem>
                <CommandItem>
                  <Plus className="h-4 w-4" />
                  {t("ui.pages.designguide.create-new-issue")}</CommandItem>
              </CommandGroup>
            </CommandList>
          </Command>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  BREADCRUMB                                                   */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.breadcrumb", { defaultValue: "Breadcrumb" })}>
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem>
              <BreadcrumbLink href="#">{t("pages.designGuide.projects", { defaultValue: "Projects" })}</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
              <BreadcrumbLink href="#">{t("pages.designGuide.paperclipApp", { defaultValue: "Paperclip App" })}</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
              <BreadcrumbPage>{t("pages.designGuide.issueList", { defaultValue: "Issue List" })}</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </Section>

      {/* ============================================================ */}
      {/*  CARDS                                                        */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.cards", { defaultValue: "Cards" })}>
        <SubSection title={t("pages.designGuide.standardCard", { defaultValue: "Standard Card" })}>
          <Card>
            <CardHeader>
              <CardTitle>{t("ui.pages.designguide.card-title")}</CardTitle>
              <CardDescription>{t("ui.pages.designguide.card-description-supporting-text")}</CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-sm">{t("ui.pages.designguide.card-content-goes-here")}</p>
            </CardContent>
            <CardFooter className="gap-2">
              <Button size="sm">{t("pages.apps.testPanel.action")}</Button>
              <Button variant="outline" size="sm">{t("pages.designGuide.cancel", { defaultValue: "Cancel" })}</Button>
            </CardFooter>
          </Card>
        </SubSection>

        <SubSection title={t("pages.designGuide.metricCards", { defaultValue: "Metric Cards" })}>
          <div className="grid md:grid-cols-2 xl:grid-cols-4 gap-4">
            <MetricCard icon={Bot} value={12} label={t("pages.designGuide.activeAgents", { defaultValue: "Active Agents" })} description={t("ui.pages.designguide.week")} />
            <MetricCard icon={CircleDot} value={48} label={t("pages.designGuide.openIssues", { defaultValue: "Open Issues" })} />
            <MetricCard icon={DollarSign} value="$1,234" label={t("pages.designGuide.monthlyCost", { defaultValue: "Monthly Cost" })} description={t("pages.designGuide.underBudget", { defaultValue: "Under budget" })} />
            <MetricCard icon={Zap} value="99.9%" label={t("pages.designGuide.uptime", { defaultValue: "Uptime" })} />
          </div>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  TABS                                                         */}
      {/* ============================================================ */}
      <Section title={t("ui.pages.designguide.tabs")}>
        <SubSection title={t("pages.designGuide.pillVariant", { defaultValue: "Default (pill) variant" })}>
          <Tabs defaultValue="overview">
            <TabsList>
              <TabsTrigger value="overview">{t("components.routineSubSidebar.overview")}</TabsTrigger>
              <TabsTrigger value="runs">{t("components.decisionTraining.runs")}</TabsTrigger>
              <TabsTrigger value="config">{t("ui.pages.designguide.config")}</TabsTrigger>
              <TabsTrigger value="costs">{t("nav.costs")}</TabsTrigger>
            </TabsList>
            <TabsContent value="overview">
              <p className="text-sm text-muted-foreground py-4">{t("ui.pages.designguide.overview-tab-content")}</p>
            </TabsContent>
            <TabsContent value="runs">
              <p className="text-sm text-muted-foreground py-4">{t("ui.pages.designguide.runs-tab-content")}</p>
            </TabsContent>
            <TabsContent value="config">
              <p className="text-sm text-muted-foreground py-4">{t("ui.pages.designguide.config-tab-content")}</p>
            </TabsContent>
            <TabsContent value="costs">
              <p className="text-sm text-muted-foreground py-4">{t("ui.pages.designguide.costs-tab-content")}</p>
            </TabsContent>
          </Tabs>
        </SubSection>

        <SubSection title={t("pages.designGuide.lineVariant", { defaultValue: "Line variant" })}>
          <Tabs defaultValue="summary">
            <TabsList variant="line">
              <TabsTrigger value="summary">{t("components.approvalPayload.summary")}</TabsTrigger>
              <TabsTrigger value="details">{t("components.systemNotice.details")}</TabsTrigger>
              <TabsTrigger value="comments">{t("components.decisionTraining.comments")}</TabsTrigger>
            </TabsList>
            <TabsContent value="summary">
              <p className="text-sm text-muted-foreground py-4">{t("ui.pages.designguide.summary-content-underline-tabs")}</p>
            </TabsContent>
            <TabsContent value="details">
              <p className="text-sm text-muted-foreground py-4">{t("ui.pages.designguide.details-content")}</p>
            </TabsContent>
            <TabsContent value="comments">
              <p className="text-sm text-muted-foreground py-4">{t("ui.pages.designguide.comments-content")}</p>
            </TabsContent>
          </Tabs>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  ENTITY ROWS                                                  */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.entityRows", { defaultValue: "Entity Rows" })}>
        <div className="border border-border rounded-md">
          <EntityRow
            leading={
              <>
                <StatusIcon status="in_progress" />
                <PriorityIcon priority="high" />
              </>
            }
            identifier="PAP-001"
            title={t("pages.designGuide.entityTitle1", { defaultValue: "Implement authentication flow" })}
            subtitle="Responsible: Agent Alpha"
            trailing={<IssueStatusBadge status="in_progress" />}
            onClick={() => {}}
          />
          <EntityRow
            leading={
              <>
                <StatusIcon status="done" />
                <PriorityIcon priority="medium" />
              </>
            }
            identifier="PAP-002"
            title={t("pages.designGuide.entityTitle2", { defaultValue: "Set up CI/CD pipeline" })}
            subtitle="Completed 2 days ago"
            trailing={<IssueStatusBadge status="done" />}
            onClick={() => {}}
          />
          <EntityRow
            leading={
              <>
                <StatusIcon status="todo" />
                <PriorityIcon priority="low" />
              </>
            }
            identifier="PAP-003"
            title={t("pages.designGuide.entityTitle3", { defaultValue: "Write API documentation" })}
            trailing={<IssueStatusBadge status="todo" />}
            onClick={() => {}}
          />
          <EntityRow
            leading={
              <>
                <StatusIcon status="blocked" />
                <PriorityIcon priority="critical" />
              </>
            }
            identifier="PAP-004"
            title={t("pages.designGuide.entityTitle4", { defaultValue: "Deploy to production" })}
            subtitle="Blocked by PAP-001"
            trailing={<IssueStatusBadge status="blocked" />}
            selected
          />
        </div>
        <SubSection title={t("pages.designGuide.membershipAction", { defaultValue: "Membership action" })}>
          <div className="border border-border rounded-md">
            <EntityRow
              title={t("pages.designGuide.joinedResource", { defaultValue: "Joined resource" })}
              subtitle="Hover or focus the row to reveal the reserved action slot."
              className="group"
              trailing={
                <MembershipAction
                  state="joined"
                  resourceName={t("pages.designGuide.joinedResource", { defaultValue: "Joined resource" })}
                  onJoin={() => {}}
                  onLeave={() => {}}
                />
              }
            />
            <EntityRow
              title={t("pages.designGuide.leftResource", { defaultValue: "Left resource" })}
              subtitle={t("pages.designGuide.dimmedRowHint", { defaultValue: "Persistent action with dimmed row content." })}
              className="group text-foreground/55"
              trailing={
                <MembershipAction
                  state="left"
                  resourceName={t("pages.designGuide.leftResource", { defaultValue: "Left resource" })}
                  onJoin={() => {}}
                  onLeave={() => {}}
                />
              }
            />
            <EntityRow
              title={t("pages.designGuide.leavingResource", { defaultValue: "Leaving resource" })}
              subtitle={t("pages.designGuide.disabledPendingHint", { defaultValue: "Disabled while the optimistic mutation is pending." })}
              className="group text-foreground/55"
              trailing={
                <MembershipAction
                  state="left"
                  pending
                  pendingState="left"
                  resourceName={t("pages.designGuide.leavingResource", { defaultValue: "Leaving resource" })}
                  onJoin={() => {}}
                  onLeave={() => {}}
                />
              }
            />
            <EntityRow
              title={t("pages.designGuide.joiningResource", { defaultValue: "Joining resource" })}
              subtitle={t("pages.designGuide.optimisticHint", { defaultValue: "The target state is visible immediately while the server confirms." })}
              className="group"
              trailing={
                <MembershipAction
                  state="joined"
                  pending
                  pendingState="joined"
                  resourceName={t("pages.designGuide.joiningResource", { defaultValue: "Joining resource" })}
                  onJoin={() => {}}
                  onLeave={() => {}}
                />
              }
            />
          </div>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  FILTER BAR                                                   */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.filterBar", { defaultValue: "Filter Bar" })}>
        <FilterBar
          filters={filters}
          onRemove={(key) => setFilters((f) => f.filter((x) => x.key !== key))}
          onClear={() => setFilters([])}
        />
        {filters.length === 0 && (
          <Button
            variant="outline"
            size="sm"
            onClick={() =>
              setFilters([
                { key: "status", label: t("pages.designGuide.status", { defaultValue: "Status" }), value: t("pages.designGuide.active", { defaultValue: "Active" }) },
                { key: "priority", label: t("pages.designGuide.priority", { defaultValue: "Priority" }), value: t("pages.designGuide.high", { defaultValue: "High" }) },
              ])
            }
          >
            {t("pages.teamCatalog.resetFilters")}</Button>
        )}
      </Section>

      {/* ============================================================ */}
      {/*  AVATARS                                                      */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.avatars", { defaultValue: "Avatars" })}>
        <SubSection title={t("pages.designGuide.sizes", { defaultValue: "Sizes" })}>
          <div className="flex items-center gap-3">
            <Avatar size="sm"><AvatarFallback>{t("ui.pages.designguide.sm")}</AvatarFallback></Avatar>
            <Avatar><AvatarFallback>{t("ui.pages.designguide.df")}</AvatarFallback></Avatar>
            <Avatar size="lg"><AvatarFallback>{t("ui.pages.designguide.lg")}</AvatarFallback></Avatar>
          </div>
        </SubSection>

        <SubSection title={t("pages.designGuide.group", { defaultValue: "Group" })}>
          <AvatarGroup>
            <Avatar><AvatarFallback>A1</AvatarFallback></Avatar>
            <Avatar><AvatarFallback>A2</AvatarFallback></Avatar>
            <Avatar><AvatarFallback>A3</AvatarFallback></Avatar>
            <AvatarGroupCount>+5</AvatarGroupCount>
          </AvatarGroup>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  IDENTITY                                                     */}
      {/* ============================================================ */}
      <Section title={t("components.agentConfigForm.identity")}>
        <SubSection title={t("pages.designGuide.sizes", { defaultValue: "Sizes" })}>
          <div className="flex items-center gap-6">
            <Identity name="Agent Alpha" size="sm" />
            <Identity name="Agent Alpha" />
            <Identity name="Agent Alpha" size="lg" />
          </div>
        </SubSection>

        <SubSection title={t("pages.designGuide.initialsDerivation", { defaultValue: "Initials derivation" })}>
          <div className="flex flex-col gap-2">
            <Identity name="CEO Agent" size="sm" />
            <Identity name="Alpha" size="sm" />
            <Identity name="Quality Assurance Lead" size="sm" />
          </div>
        </SubSection>

        <SubSection title={t("pages.designGuide.customInitials", { defaultValue: "Custom initials" })}>
          <Identity name="Backend Service" initials="BS" size="sm" />
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  TOOLTIPS                                                     */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.tooltips", { defaultValue: "Tooltips" })}>
        <div className="flex items-center gap-4">
          <Tooltip>
            <TooltipTrigger asChild>
              <Button variant="outline" size="sm">{t("ui.pages.designguide.hover-me")}</Button>
            </TooltipTrigger>
            <TooltipContent>{t("ui.pages.designguide.tooltip")}</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button variant="ghost" size="icon-sm"><Settings /></Button>
            </TooltipTrigger>
            <TooltipContent>{t("nav.settings")}</TooltipContent>
          </Tooltip>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  DIALOG                                                       */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.dialog", { defaultValue: "Dialog" })}>
        <Dialog>
          <DialogTrigger asChild>
            <Button variant="outline">{t("ui.pages.designguide.open-dialog")}</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>{t("ui.pages.designguide.dialog-title")}</DialogTitle>
              <DialogDescription>
                {t("ui.pages.designguide.sample-dialog-showing-standard")}</DialogDescription>
            </DialogHeader>
            <div className="space-y-3">
              <div>
                <Label>{t("components.agentConfigForm.name")}</Label>
                <Input placeholder={t("ui.pages.designguide.enter-name")} className="mt-1.5" />
              </div>
              <div>
                <Label>{t("pages.designGuide.description", { defaultValue: "Description" })}</Label>
                <Textarea placeholder={t("pages.designGuide.describe", { defaultValue: "Describe..." })} className="mt-1.5" />
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline">{t("pages.designGuide.cancel", { defaultValue: "Cancel" })}</Button>
              <Button>{t("pages.designGuide.save", { defaultValue: "Save" })}</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </Section>

      {/* ============================================================ */}
      {/*  EMPTY STATE                                                  */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.emptyState", { defaultValue: "Empty State" })}>
        <div className="border border-border rounded-md">
          <EmptyState
            icon={Inbox}
            message={t("pages.designGuide.emptyStateHint", { defaultValue: "No items to show. Create your first one to get started." })}
            action={t("pages.designGuide.createItem", { defaultValue: "Create Item" })}
            onAction={() => {}}
          />
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  PROGRESS BARS                                                */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.progressBars", { defaultValue: "Progress Bars (Budget)" })}>
        <div className="space-y-3">
          {[
            { label: "Under budget (40%)", pct: 40, color: "bg-green-400" },
            { label: "Warning (75%)", pct: 75, color: "bg-yellow-400" },
            { label: "Over budget (95%)", pct: 95, color: "bg-red-400" },
          ].map(({ label, pct, color }) => (
            <div key={label} className="space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-xs text-muted-foreground">{label}</span>
                <span className="text-xs font-mono">{pct}%</span>
              </div>
              <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
                <div
                  className={`h-full rounded-full transition-(--tp-width-background-color) duration-150 ${color}`}
                  style={{ width: `${pct}%` }}
                />
              </div>
            </div>
          ))}
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  LOG VIEWER                                                   */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.logViewer", { defaultValue: "Log Viewer" })}>
        <div className="bg-neutral-950 rounded-lg p-3 font-mono text-xs max-h-80 overflow-y-auto">
          <div className="text-foreground">{t("ui.pages.designguide.12-00-01-info")}</div>
          <div className="text-foreground">{t("ui.pages.designguide.12-00-02-info")}</div>
          <div className="text-yellow-400">{t("ui.pages.designguide.12-00-05-warn")}</div>
          <div className="text-foreground">{t("ui.pages.designguide.12-00-08-info")}</div>
          <div className="text-red-400">{t("ui.pages.designguide.12-00-12-error")}</div>
          <div className="text-blue-300">{t("ui.pages.designguide.12-00-12-sys")}</div>
          <div className="text-foreground">{t("ui.pages.designguide.12-00-17-info")}</div>
          <div className="flex items-center gap-1.5">
            <span className="relative flex h-1.5 w-1.5">
              <span className="absolute inline-flex h-full w-full rounded-full bg-blue-400 animate-pulse" />
              <span className="inline-flex h-full w-full rounded-full bg-blue-500" />
            </span>
            <span className="text-blue-600 dark:text-blue-400">{t("components.issueChatThread.live")}</span>
          </div>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  PROPERTY ROW PATTERN                                         */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.propertyRow", { defaultValue: "Property Row Pattern" })}>
        <div className="border border-border rounded-md p-4 space-y-1 max-w-sm">
          <div className="flex items-center justify-between py-1.5">
            <span className="text-xs text-muted-foreground">{t("pages.designGuide.status", { defaultValue: "Status" })}</span>
            <StatusBadge status="active" />
          </div>
          <div className="flex items-center justify-between py-1.5">
            <span className="text-xs text-muted-foreground">{t("pages.designGuide.priority", { defaultValue: "Priority" })}</span>
            <PriorityIcon priority="high" />
          </div>
          <div className="flex items-center justify-between py-1.5">
            <span className="text-xs text-muted-foreground">{t("components.commentThread.responsible")}</span>
            <div className="flex items-center gap-1.5">
              <Avatar size="sm"><AvatarFallback>A</AvatarFallback></Avatar>
              <span className="text-xs">{t("ui.pages.designguide.agent-alpha")}</span>
            </div>
          </div>
          <div className="flex items-center justify-between py-1.5">
            <span className="text-xs text-muted-foreground">{t("components.agentProperties.created")}</span>
            <span className="text-xs">Jan 15, 2025</span>
          </div>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  NAVIGATION PATTERNS                                          */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.navigationPatterns", { defaultValue: "Navigation Patterns" })}>
        <SubSection title={t("pages.designGuide.sidebarNav", { defaultValue: "Sidebar nav items" })}>
          <Card className="block w-60 p-3 space-y-0.5">
            <div className="flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium bg-accent text-accent-foreground">
              <LayoutDashboard className="h-4 w-4" />
              {t("nav.dashboard")}</div>
            <div className="flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium text-muted-foreground hover:bg-accent/50 hover:text-accent-foreground cursor-pointer">
              <CircleDot className="h-4 w-4" />
              {t("ui.pages.designguide.issues")}<Badge variant="ghost" className="ml-auto bg-primary text-primary-foreground px-1.5">
                12
              </Badge>
            </div>
            <div className="flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium text-muted-foreground hover:bg-accent/50 hover:text-accent-foreground cursor-pointer">
              <Bot className="h-4 w-4" />
              {t("common.agents")}</div>
            <div className="flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium text-muted-foreground hover:bg-accent/50 hover:text-accent-foreground cursor-pointer">
              <Hexagon className="h-4 w-4" />
              {t("nav.projects")}</div>
          </Card>
        </SubSection>

        <SubSection title={t("pages.designGuide.viewToggle", { defaultValue: "View toggle" })}>
          <div className="flex items-center border border-border rounded-md w-fit">
            <button className="px-3 py-1.5 text-xs font-medium bg-accent text-foreground rounded-l-md">
              <ListTodo className="h-3.5 w-3.5 inline mr-1" />
              {t("ui.pages.designguide.list")}</button>
            <button className="px-3 py-1.5 text-xs font-medium text-muted-foreground hover:bg-accent/50 rounded-r-md">
              <Target className="h-3.5 w-3.5 inline mr-1" />
              {t("nav.org")}</button>
          </div>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  GROUPED LIST (Issues pattern)                                */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.groupedList", { defaultValue: "Grouped List (Issues pattern)" })}>
        <div>
          <div className="flex items-center gap-2 px-4 py-2 bg-muted/50 rounded-t-md">
            <StatusIcon status="in_progress" />
            <span className="text-sm font-medium">{t("pages.designGuide.inProgress", { defaultValue: "In Progress" })}</span>
            <span className="text-xs text-muted-foreground ml-1">2</span>
          </div>
          <div className="border border-border rounded-b-md">
            <EntityRow
              leading={<PriorityIcon priority="high" />}
              identifier="PAP-101"
              title={t("pages.designGuide.groupedTitle1", { defaultValue: "Build agent heartbeat system" })}
              onClick={() => {}}
            />
            <EntityRow
              leading={<PriorityIcon priority="medium" />}
              identifier="PAP-102"
              title={t("pages.designGuide.groupedTitle2", { defaultValue: "Add cost tracking dashboard" })}
              onClick={() => {}}
            />
          </div>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  COMMENT THREAD PATTERN                                       */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.commentThread", { defaultValue: "Comment Thread Pattern" })}>
        <div className="space-y-3 max-w-2xl">
          <h3 className="text-sm font-semibold">{t("ui.pages.designguide.comments")}</h3>
          <div className="space-y-3">
            <div className="rounded-md border border-border p-3">
              <div className="flex items-center justify-between mb-1">
                <span className="text-xs font-medium text-muted-foreground">{t("components.dialogs.newGoal.levelAgent")}</span>
                <span className="text-xs text-muted-foreground">Jan 15, 2025</span>
              </div>
              <p className="text-sm">{t("ui.pages.designguide.started-working-authentication-module")}</p>
            </div>
            <div className="rounded-md border border-border p-3">
              <div className="flex items-center justify-between mb-1">
                <span className="text-xs font-medium text-muted-foreground">{t("pages.companyInvites.human")}</span>
                <span className="text-xs text-muted-foreground">Jan 16, 2025</span>
              </div>
              <p className="text-sm">{t("ui.pages.designguide.api-keys-have-been")}</p>
            </div>
          </div>
          <div className="space-y-2">
            <Textarea placeholder={t("pages.designGuide.commentPlaceholder", { defaultValue: "Leave a comment..." })} rows={3} />
            <Button size="sm">{t("components.commentThread.comment")}</Button>
          </div>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  COST TABLE PATTERN                                           */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.costTable", { defaultValue: "Cost Table Pattern" })}>
        <div className="border border-border rounded-lg overflow-hidden">
          <table className="w-full text-xs">
            <thead className="border-b border-border bg-accent/20">
              <tr>
                <th className="text-left px-3 py-2 font-medium text-muted-foreground">{t("components.agentConfigForm.model")}</th>
                <th className="text-left px-3 py-2 font-medium text-muted-foreground">{t("pages.connectClientDialog.tokens")}</th>
                <th className="text-left px-3 py-2 font-medium text-muted-foreground">{t("pages.agentDetail.cost")}</th>
              </tr>
            </thead>
            <tbody>
              <tr className="border-b border-border">
                <td className="px-3 py-2">claude-sonnet-4-20250514</td>
                <td className="px-3 py-2 font-mono">1.2M</td>
                <td className="px-3 py-2 font-mono">$18.00</td>
              </tr>
              <tr className="border-b border-border">
                <td className="px-3 py-2">claude-haiku-4-20250506</td>
                <td className="px-3 py-2 font-mono">500k</td>
                <td className="px-3 py-2 font-mono">$1.25</td>
              </tr>
              <tr>
                <td className="px-3 py-2 font-medium">{t("ui.pages.designguide.total")}</td>
                <td className="px-3 py-2 font-mono">1.7M</td>
                <td className="px-3 py-2 font-mono font-medium">$19.25</td>
              </tr>
            </tbody>
          </table>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  SKELETONS                                                    */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.skeletons", { defaultValue: "Skeletons" })}>
        <SubSection title={t("pages.designGuide.individual", { defaultValue: "Individual" })}>
          <div className="space-y-2">
            <Skeleton className="h-4 w-48" />
            <Skeleton className="h-8 w-full max-w-sm" />
            <Skeleton className="h-20 w-full" />
          </div>
        </SubSection>

        <SubSection title={t("pages.designGuide.pageSkeletonList", { defaultValue: "Page Skeleton (list)" })}>
          <div className="border border-border rounded-md p-4">
            <PageSkeleton variant="list" />
          </div>
        </SubSection>

        <SubSection title={t("pages.designGuide.pageSkeletonDetail", { defaultValue: "Page Skeleton (detail)" })}>
          <div className="border border-border rounded-md p-4">
            <PageSkeleton variant="detail" />
          </div>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  SEPARATOR                                                    */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.separator", { defaultValue: "Separator" })}>
        <div className="space-y-4">
          <p className="text-sm text-muted-foreground">{t("ui.pages.designguide.horizontal")}</p>
          <Separator />
          <div className="flex items-center gap-4 h-8">
            <span className="text-sm">{t("ui.pages.designguide.left")}</span>
            <Separator orientation="vertical" />
            <span className="text-sm">{t("ui.pages.designguide.right")}</span>
          </div>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  ICON REFERENCE                                               */}
      {/* ============================================================ */}
      {/*  TEAM CATALOG                                                 */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.teamCatalog", { defaultValue: "Team Catalog" })}>
        <p className="text-sm text-muted-foreground">
          {t("ui.pages.designguide.components-from-team-catalog")}<code className="font-mono text-xs">/teams-catalog</code>{t("ui.pages.designguide.fixtures-shared-storybook-stories")}</p>

        <SubSection title={t("ui.pages.designguide.teamrow-browse-list")}>
          <div className="w-(--sz-28rem) rounded-md border border-border">
            <div className="px-3 py-2 text-(length:--text-micro) font-semibold uppercase tracking-wide text-muted-foreground">
              {t("ui.pages.designguide.bundled")}</div>
            <TeamRow team={sampleTeam} selected onSelect={() => {}} />
            <div className="px-3 py-2 text-(length:--text-micro) font-semibold uppercase tracking-wide text-muted-foreground">
              {t("ui.pages.designguide.optional")}</div>
            <TeamRow team={optionalTeam} selected={false} onSelect={() => {}} />
            <div className="px-3 py-2 text-(length:--text-micro) font-semibold uppercase tracking-wide text-muted-foreground">
              {t("ui.pages.designguide.installed")}</div>
            <TeamRow team={sampleTeam} selected={false} onSelect={() => {}} installed={outOfDateInstalledState} />
            <TeamRow team={warnTeam} selected={false} onSelect={() => {}} installed={currentInstalledState} />
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            {t("ui.pages.designguide.installed-teams-collapse-under")}<code className="font-mono">{t("ui.pages.designguide.installed-alt")}</code>{t("ui.pages.designguide.date-install-server")}<code className="font-mono">{t("ui.pages.designguide.originhash")}</code> {t("ui.pages.designguide.catalog")}<code className="font-mono">{t("ui.pages.designguide.contenthash")}</code>)
            shows the amber <code className="font-mono">↑</code> badge (PAP-10256).
          </p>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.teamcard-onboarding-grid")}>
          <p className="text-xs text-muted-foreground">
            {t("ui.pages.designguide.square-tile-onboarding-ldquo")}{" "}
            <code className="font-mono">{t("ui.pages.designguide.ring-ring-ring")}</code>{t("ui.pages.designguide.drives")}{" "}
            <code className="font-mono">{t("ui.pages.designguide.useinstallteamcatalogentry")}</code> {t("ui.pages.designguide.simplified-flow")}</p>
          <TeamCardShowcase />
        </SubSection>

        <SubSection title={t("ui.pages.designguide.teamhierarchypreview")}>
          <div className="max-w-md">
            <TeamHierarchyPreview team={sampleTeam} />
          </div>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.requiredskillslist")}>
          <div className="max-w-xl">
            <RequiredSkillsList skills={sampleTeam.requiredSkills} />
          </div>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.envinputslist")}>
          <div className="max-w-xl">
            <EnvInputsList inputs={sampleTeam.envInputs} />
          </div>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.externalsourceslist")}>
          <div className="max-w-xl">
            <ExternalSourcesList sources={sampleTeam.sourceRefs} />
          </div>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.source-policy-step-stepsourcepolicy")}>
          <div className="max-w-xl rounded-md border border-border p-4">
            <StepSourcePolicy
              team={warnTeam}
              allowExternalSources={allowExternal}
              allowUnpinnedOptionalSources={allowUnpinned}
              allowLocalPathSources={allowLocalPath}
              onChange={(key, value) => {
                if (key === "external") setAllowExternal(value);
                if (key === "unpinned") setAllowUnpinned(value);
                if (key === "localPath") setAllowLocalPath(value);
              }}
            />
          </div>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.skill-plan-step-stepskillplan")}>
          <div className="max-w-xl rounded-md border border-border p-4">
            <StepSkillPlan team={sampleTeam} preparations={sampleSkillPreparations} />
          </div>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      <Section title={t("ui.pages.designguide.common-icons-lucide")}>
        <div className="grid grid-cols-4 md:grid-cols-6 gap-4">
          {[
            ["Inbox", Inbox],
            ["ListTodo", ListTodo],
            ["CircleDot", CircleDot],
            ["Hexagon", Hexagon],
            ["Target", Target],
            ["LayoutDashboard", LayoutDashboard],
            ["Bot", Bot],
            ["DollarSign", DollarSign],
            ["History", History],
            ["Search", Search],
            ["Plus", Plus],
            ["Trash2", Trash2],
            ["Settings", Settings],
            ["User", User],
            ["Mail", Mail],
            ["Upload", Upload],
            ["Zap", Zap],
          ].map(([name, Icon]) => {
            const LucideIcon = Icon as React.FC<{ className?: string }>;
            return (
              <div key={name as string} className="flex flex-col items-center gap-1.5 p-2">
                <LucideIcon className="h-4 w-4 text-muted-foreground" />
                <span className="text-(length:--text-nano) text-muted-foreground font-mono">{name as string}</span>
              </div>
            );
          })}
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  KEYBOARD SHORTCUTS                                           */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.keyboardShortcuts", { defaultValue: "Keyboard Shortcuts" })}>
        <div className="border border-border rounded-md divide-y divide-border text-sm">
          {[
            [t("pages.designGuide.cmdK", { defaultValue: "Cmd+K / Ctrl+K" }), t("pages.designGuide.openCommandPalette", { defaultValue: "Open Command Palette" })],
            ["C", t("pages.designGuide.newIssueShortcut", { defaultValue: "New Issue (outside inputs)" })],
            ["[", t("pages.designGuide.toggleSidebar", { defaultValue: "Toggle Sidebar" })],
            ["]", t("pages.designGuide.toggleProperties", { defaultValue: "Toggle Properties Panel" })],

            [t("pages.designGuide.cmdEnter", { defaultValue: "Cmd+Enter / Ctrl+Enter" }), t("pages.designGuide.submitComment", { defaultValue: "Submit markdown comment" })],
          ].map(([key, desc]) => (
            <div key={key} className="flex items-center justify-between px-4 py-2">
              <span className="text-muted-foreground">{desc}</span>
              <kbd className="px-2 py-0.5 text-xs font-mono bg-muted rounded border border-border">
                {key}
              </kbd>
            </div>
          ))}
        </div>
      </Section>

      <Section title={t("pages.designGuide.issueOutput", { defaultValue: "Issue Output Surface" })}>
        <SubSection title={t("pages.designGuide.multipleOutputs", { defaultValue: "Multiple outputs (primary video + 'Also produced')" })}>
          <IssueOutputSection workProducts={DESIGN_GUIDE_OUTPUTS} />
        </SubSection>
        <SubSection title={t("pages.designGuide.degradedOutput", { defaultValue: "Degraded output (invalid / failed attachment metadata)" })}>
          <IssueOutputSection workProducts={DESIGN_GUIDE_DEGRADED_OUTPUTS} />
        </SubSection>
        <SubSection title={t("pages.designGuide.emptyState2", { defaultValue: "Empty state" })}>
          <p className="text-xs text-muted-foreground">
            {t("ui.pages.designguide.when-issue-has-produced")}</p>
        </SubSection>
      </Section>

      {/* ============================================================ */}
      {/*  TOOLS & ACCESS (PAP-10389)                                   */}
      {/* ============================================================ */}
      <Section title={t("pages.designGuide.toolsAccess", { defaultValue: "Tools & Access" })}>
        <SubSection title={t("ui.pages.designguide.enforcementbanner-default-denied-detected")}>
          <div className="space-y-3">
            <EnforcementBanner companyId="" forceVariant="default" recentDenialCount={0} />
            <EnforcementBanner companyId="" forceVariant="denied-detected" recentDenialCount={3} />
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            {t("ui.pages.designguide.persistent-top-tools-amp")}<code>denied-detected</code> {t("ui.pages.designguide.when-governed-tool-calls")}</p>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.enforcementbanner-presentational-tones-info")}>
          <div className="space-y-3">
            <EnforcementBanner
              tone="info"
              title={t("ui.pages.designguide.effective-access-server-resolved")}
              body="This is exactly what the tool gateway will accept. Profile and policy edits reflect within ~5s; the prompt cannot expand it."
            />
            <EnforcementBanner
              tone="warning"
              title={t("ui.pages.designguide.local-stdio-local-code")}
              body="A local-stdio slot runs with the orchestrator's privileges. Only bind trusted commands; quarantine anything you would not run yourself."
            />
            <EnforcementBanner
              tone="error"
              title={t("ui.pages.designguide.runtime-failed-closed")}
              body="The supervisor is restarting (attempt 2/3). The gateway returns runtime-error and the agent does not see partial output."
            />
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            {t("ui.pages.designguide.static-governance-copy-tone")}<code>title</code>/<code>body</code> {t("ui.pages.designguide.optional-alt")}{" "}
            <code>icon</code>.
          </p>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.action-approval-card-pending")}>
          <div className="grid gap-4 lg:grid-cols-2">
            <ActionCard
              toolName="slack.post_message"
              risk="medium"
              isWrite
              binding={{
                application: t("pages.designGuide.slack", { defaultValue: "Slack" }),
                manifestVersion: "2.4.1",
                connection: "https://slack.com/api · acme-workspace",
                catalogSha256: "sha256:9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
                payloadSha256: "sha256:2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
              }}
              input={{ channel: "#launch", text: "Deploy v2 is live 🎉", unfurl_links: false }}
              reason={t("pages.designGuide.toolApprovalHint", { defaultValue: "This tool can write to your workspace, so a human signs off before the agent posts." })}
              policyNumber={7}
              expiresInLabel="expires in 23h 51m"
            />
            <ActionCard
              variant="stale"
              toolName="slack.post_message"
              risk="medium"
              isWrite
              binding={{
                application: t("pages.designGuide.slack", { defaultValue: "Slack" }),
                manifestVersion: "2.4.1",
                connection: "https://slack.com/api · acme-workspace",
                catalogSha256: "sha256:7d793037a0760186574b0282f2f435e7a4b1b2b0b822cd15d6c15b0f00a0e3f1",
                previousCatalogSha256: "sha256:9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
                payloadSha256: "sha256:2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
              }}
              input={{ channel: "#launch", text: "Deploy v2 is live 🎉", unfurl_links: false }}
              reason={t("pages.designGuide.toolApprovalHint", { defaultValue: "This tool can write to your workspace, so a human signs off before the agent posts." })}
              policyNumber={7}
              expiresInLabel="expires in 18h 02m"
            />
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            {t("ui.pages.designguide.signed-payload-sha256-expiry")}{" "}
            <code>stale</code> {t("ui.pages.designguide.variant-tints-border-amber")}<code>{t("common.approve")}</code> {t("ui.pages.designguide.disabled-until-request-re")}</p>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.action-approval-card-mobile")}>
          <div className="w-(--sz-390px) max-w-full rounded-xl border border-border bg-background p-3">
            <ActionCardMobile
              toolName="slack.post_message"
              risk="medium"
              isWrite
              binding={{
                application: t("pages.designGuide.slack", { defaultValue: "Slack" }),
                manifestVersion: "2.4.1",
                connection: "https://slack.com/api · acme-workspace",
                catalogSha256: "sha256:9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
                payloadSha256: "sha256:2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
              }}
              input={{ channel: "#launch", text: "Deploy v2 is live 🎉" }}
              reason={t("pages.designGuide.toolApprovalHint", { defaultValue: "This tool can write to your workspace, so a human signs off before the agent posts." })}
              policyNumber={7}
              expiresInLabel="expires in 23h 51m"
            />
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            {t("ui.pages.designguide.identical-content-three-buttons")}</p>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.bindingstable-reused-audit-row")}>
          <BindingsTable
            rows={[
              { label: "Application", value: "Slack · manifest v2.4.1" },
              { label: "Connection", value: "https://slack.com/api · acme-workspace", mono: true },
              { label: "Catalog", value: "sha256:9f86d081…f00a08", mono: true },
              { label: "Payload", value: "sha256:2c26b46b…66e7ae", mono: true },
            ]}
          />
          <p className="mt-2 text-xs text-muted-foreground">
            {t("ui.pages.designguide.two-column-key-value")}<code>{t("ui.pages.designguide.actioncard")}</code> {t("ui.pages.designguide.reused-standalone-audit-row")}</p>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.tool-access-status-keys")}>
          <div className="flex flex-wrap items-center gap-2">
            {[
              "allowed", "denied", "block", "require-approval", "redacted", "rate-limit",
              "deferred", "hidden", "quarantined", "healthy", "degraded", "runtime-error", "unchecked",
            ].map((s) => (
              <StatusBadge key={s} status={s} />
            ))}
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            {t("ui.pages.designguide.policy-decisions-connection-runtime")}{" "}
            <code>{t("ui.pages.designguide.statusbadge")}</code> {t("ui.pages.designguide.keys-defined")}<code>{t("ui.pages.designguide.lib-status-colors")}</code>.
          </p>
        </SubSection>

        <SubSection title={t("ui.pages.designguide.emptystate-canonical-description-action")}>
          <EmptyState
            icon={Inbox}
            message={t("pages.designGuide.noConnections", { defaultValue: "No connections yet" })}
            description={t("pages.designGuide.noConnectionsHint", { defaultValue: "Add a connection to an application to configure credentials and discover its tools." })}
            action={t("pages.designGuide.newConnection", { defaultValue: "New connection" })}
            onAction={() => {}}
          />
        </SubSection>
      </Section>

      <Section title={t("pages.designGuide.envVarsEditor", { defaultValue: "Environment Variables Editor" })}>
        <p className="text-sm text-muted-foreground">
          {t("ui.pages.designguide.reusable-env-var-editor")}<span className="font-mono">{t("ui.pages.designguide.product-environment-variables-editor")}</span> {t("ui.pages.designguide.stories-all-10-states")}</p>
        <EnvironmentVariablesEditorShowcase />
      </Section>

      <Section title={t("pages.designGuide.resizablePanels", { defaultValue: "Resizable Panels" })}>
        <p className="text-sm text-muted-foreground">
          {t("ui.pages.designguide.design-system-wrapper-over")}<span className="font-mono">react-resizable-panels</span>{" "}
          {t("ui.pages.designguide.skill-studio-d2-drag")}<span className="font-mono">{t("ui.pages.designguide.minsize-240px")}</span>{t("ui.pages.designguide.constraints-middle-panel-collapsible")}</p>
        <div className="h-48 max-w-2xl overflow-hidden rounded-md border border-border">
          <ResizablePanelGroup>
            <ResizablePanel id="a" minSize="120px" className="bg-muted/30">
              <div className="flex h-full items-center justify-center text-xs text-muted-foreground">
                {t("ui.pages.designguide.panel")}</div>
            </ResizablePanel>
            <ResizableHandle />
            <ResizablePanel id="b" minSize="120px" collapsible collapsedSize="40px" className="bg-muted/10">
              <div className="flex h-full items-center justify-center text-xs text-muted-foreground">
                {t("ui.pages.designguide.panel-collapsible")}</div>
            </ResizablePanel>
            <ResizableHandle />
            <ResizablePanel id="c" minSize="120px" className="bg-muted/30">
              <div className="flex h-full items-center justify-center text-xs text-muted-foreground">
                {t("ui.pages.designguide.panel-alt")}</div>
            </ResizablePanel>
          </ResizablePanelGroup>
        </div>
      </Section>

      {/* ============================================================ */}
      {/*  INLINE BANNER + BUILT-IN AGENTS                              */}
      {/* ============================================================ */}
      <Section title={t("ui.pages.designguide.inline-banner")}>
        <p className="text-sm text-muted-foreground">
          {t("ui.pages.designguide.token-backed-full-width")}<span className="font-mono">{t("ui.pages.designguide.brandbanner")}</span> {t("ui.pages.designguide.tones-use")}{" "}
          <span className="font-mono">info</span> {t("ui.pages.designguide.provenance-context")}{" "}
          <span className="font-mono">warning</span> {t("ui.pages.designguide.paused-attention-supports-optional")}{" "}
          <span className="font-mono">{t("ui.pages.designguide.bg-yellow")}</span>/<span className="font-mono">{t("ui.pages.designguide.bg-blue")}</span>{" "}
          banners.
        </p>
        <div className="space-y-3">
          <InlineBanner
            tone="info"
            title={t("pages.designGuide.builtInAgent", { defaultValue: "Built-in agent" })}
            actions={<Button variant="outline" size="sm">{t("pages.agentDetail.resetToDefaults")}</Button>}
          >
            {t("ui.pages.agentdetail.ships-paperclip-powers")}<strong>{t("ui.pages.designguide.briefs")}</strong>{t("ui.pages.designguide.can-paused-but-not")}</InlineBanner>
          <InlineBanner
            tone="warning"
            title={t("pages.designGuide.briefsPaused", { defaultValue: "Briefs is paused." })}
            actions={
              <>
                <Button variant="ghost" size="sm">{t("components.liveUpdates.viewAgent")}</Button>
                <Button size="sm">{t("components.sidebarAgents.resumeAgent")}</Button>
              </>
            }
          >
            {t("ui.pages.designguide.built-agent-was-paused")}</InlineBanner>
          <InlineBanner
            tone="danger"
            title={t("pages.designGuide.summaryFailed", { defaultValue: "Summary generation failed." })}
            actions={<Button size="sm">{t("components.issueProperties.retry")}</Button>}
          >
            {t("ui.pages.designguide.linked-issue-reached-terminal")}</InlineBanner>
          <InlineBanner tone="info" compact>
            {t("ui.pages.designguide.compact-variant-embedding-inside")}</InlineBanner>
        </div>
      </Section>

      <Section title={t("pages.designGuide.builtInLifecycle", { defaultValue: "Built-in Agent Lifecycle Chips" })}>
        <p className="text-sm text-muted-foreground">
          {t("ui.pages.designguide.derived-lifecycle-chip-amber")}{" "}
          <span className="font-mono">needs_setup</span> / <span className="font-mono">pending_approval</span>.
        </p>
        <div className="flex flex-wrap items-center gap-4">
          <BuiltInLifecycleChip status="needs_setup" />
          <BuiltInLifecycleChip status="pending_approval" />
          <BuiltInLifecycleChip status="needs_setup" compact />
        </div>
        <p className="mt-3 text-sm text-muted-foreground">
          <span className="font-mono">{t("ui.pages.designguide.lt-builtinagentgate-agentkey-gt")}</span> composes{" "}
          <span className="font-mono">{t("ui.pages.designguide.pageskeleton")}</span> + <span className="font-mono">{t("ui.pages.designguide.emptystate")}</span>{" "}
          + <span className="font-mono">{t("ui.pages.designguide.inlinebanner")}</span> {t("ui.pages.designguide.render-loading-setup-pending")}</p>
      </Section>
    </div>
  );
}
