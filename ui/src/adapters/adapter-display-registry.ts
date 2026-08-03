/**
 * Single source of truth for adapter display metadata.
 *
 * Built-in adapters have entries in `adapterDisplayMap`. External (plugin)
 * adapters get sensible defaults derived from their type string via
 * `getAdapterDisplay()`.
 */
import { t } from "../i18n";
import type { ComponentType } from "react";
import {
  Bot,
  Code,
  Gem,
  MousePointer2,
  Sparkles,
  Terminal,
  Cpu,
} from "lucide-react";
import { OpenCodeLogoIcon } from "@/components/OpenCodeLogoIcon";

// ---------------------------------------------------------------------------
// Type suffix parsing
// ---------------------------------------------------------------------------

// Suffixes stripped from type ids when deriving a human-readable label for
// unknown (plugin) adapter types. "_local" is a legacy qualifier from before
// first-class Environments and is never displayed; "_gateway" is re-appended
// as " (gateway)" to disambiguate gateway variants. Known adapters in
// `adapterDisplayMap` have final labels and never get a derived suffix.
const STRIPPED_TYPE_SUFFIXES = ["_local", "_gateway"] as const;

const DISPLAY_SUFFIXES: Record<string, string> = {
  _gateway: "gateway",
};

function getTypeSuffix(type: string): string | null {
  for (const [suffix, mode] of Object.entries(DISPLAY_SUFFIXES)) {
    if (type.endsWith(suffix)) return mode;
  }
  return null;
}

function withSuffix(label: string, suffix: string | null): string {
  return suffix ? `${label} (${suffix})` : label;
}

// ---------------------------------------------------------------------------
// Display metadata per adapter type
// ---------------------------------------------------------------------------

export interface AdapterDisplayInfo {
  label: string;
  description: string;
  icon: ComponentType<{ className?: string }>;
  recommended?: boolean;
  comingSoon?: boolean;
  disabledLabel?: string;
  experimental?: boolean;
  hideFromVisualSelection?: boolean;
}

const adapterDisplayMap: Record<string, AdapterDisplayInfo> = {
  acpx_local: {
    label: t("ui.adapters.adapter-display-registry.acpx-retired"),
    description: t("ui.adapters.adapter-display-registry.retired-standalone-acpx-adapter"),
    icon: Bot,
    comingSoon: true,
    disabledLabel: t("ui.adapters.adapter-display-registry.use-claude-code-codex"),
    hideFromVisualSelection: true,
  },
  claude_local: {
    label: t("pages.inviteUxLab.claudeCode"),
    description: t("ui.adapters.adapter-display-registry.claude-code-cli-harness"),
    icon: Sparkles,
    recommended: true,
  },
  codex_local: {
    label: t("pages.inviteUxLab.codex"),
    description: t("ui.adapters.adapter-display-registry.codex-cli-harness"),
    icon: Code,
    recommended: true,
  },
  gemini_local: {
    label: t("components.geminiConfig.geminiCli"),
    description: t("ui.adapters.adapter-display-registry.gemini-cli-harness"),
    icon: Gem,
  },
  grok_local: {
    label: t("ui.adapters.adapter-display-registry.grok-build"),
    description: t("ui.adapters.adapter-display-registry.grok-build-harness"),
    icon: Bot,
  },
  hermes_gateway: {
    label: t("ui.adapters.adapter-display-registry.hermes-gateway"),
    description: t("ui.adapters.adapter-display-registry.remote-hermes-api-server"),
    icon: Bot,
    hideFromVisualSelection: true,
  },
  hermes_local: {
    label: t("ui.adapters.adapter-display-registry.hermes"),
    description: t("ui.adapters.adapter-display-registry.hermes-harness"),
    icon: Bot,
  },
  opencode_local: {
    label: "OpenCode",
    description: "OpenCode multi-provider harness",
    icon: OpenCodeLogoIcon,
  },
  pi_local: {
    label: "Pi",
    description: t("ui.adapters.adapter-display-registry.pi-harness"),
    icon: Terminal,
  },
  cursor: {
    label: t("pages.inviteUxLab.cursor"),
    description: t("ui.adapters.adapter-display-registry.cursor-cli-harness"),
    icon: MousePointer2,
  },
  cursor_cloud: {
    label: t("ui.adapters.adapter-display-registry.cursor-cloud"),
    description: t("ui.adapters.adapter-display-registry.managed-remote-cursor-agent"),
    icon: MousePointer2,
  },
  openclaw_gateway: {
    label: "OpenClaw Gateway",
    description: t("ui.adapters.adapter-display-registry.external-gateway-adapter"),
    icon: Bot,
    comingSoon: true,
    disabledLabel: t("ui.adapters.adapter-display-registry.invite-external-agents-from"),
    hideFromVisualSelection: true,
  },
  process: {
    label: t("ui.adapters.adapter-display-registry.process"),
    description: t("ui.adapters.adapter-display-registry.internal-process-adapter"),
    icon: Cpu,
    comingSoon: true,
  },
  http: {
    label: t("ui.adapters.adapter-display-registry.http"),
    description: t("ui.adapters.adapter-display-registry.internal-http-adapter"),
    icon: Cpu,
    comingSoon: true,
  },
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

function humanizeType(type: string): string {
  // Strip known type suffixes so "droid_local" → "Droid", not "Droid Local"
  let base = type;
  for (const suffix of STRIPPED_TYPE_SUFFIXES) {
    if (base.endsWith(suffix)) {
      base = base.slice(0, -suffix.length);
      break;
    }
  }
  return base.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

export function getAdapterLabel(type: string): string {
  // Known labels are final — only unknown (plugin) types get a derived
  // suffix, so labels like "OpenClaw Gateway" don't become
  // "OpenClaw Gateway (gateway)".
  const known = adapterDisplayMap[type];
  if (known) return known.label;
  return withSuffix(humanizeType(type), getTypeSuffix(type));
}

export function getAdapterLabels(): Record<string, string> {
  const labels: Record<string, string> = {};
  for (const [type, info] of Object.entries(adapterDisplayMap)) {
    labels[type] = info.label;
  }
  return labels;
}

export function getAdapterDisplay(type: string): AdapterDisplayInfo {
  const known = adapterDisplayMap[type];
  if (known) return known;

  const suffix = getTypeSuffix(type);
  const label = withSuffix(humanizeType(type), suffix);
  return {
    label,
    description: suffix ? `External ${suffix} adapter` : t("ui.adapters.adapter-display-registry.external-adapter"),
    icon: Cpu,
  };
}

export function isKnownAdapterType(type: string): boolean {
  return type in adapterDisplayMap;
}
