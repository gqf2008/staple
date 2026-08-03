import { t } from "../i18n";
/**
 * Prosumer copy for the Apps surface (PAP-10856).
 *
 * The P1a gallery manifest carries developer-flavoured taglines and credential
 * labels (e.g. "Connect Zapier-hosted MCP actions", "Zapier MCP token"). Those
 * strings would fail the vocabulary gate from PAP-10827 — no "MCP", "server",
 * "profile", "policy", "gateway", or "transport" anywhere on this surface. So
 * the UI never renders the raw manifest copy directly: it looks up plain copy
 * here, and `sanitizeProsumerCopy` is a final backstop for any free-text we do
 * surface (app names, fallback taglines).
 */

/** Words that must never appear in prosumer-facing copy on the Apps surface. */
const BANNED_WORDS = [
  "mcp",
  "server",
  "profile",
  "policy",
  "gateway",
  "transport",
  "stdio",
  "endpoint",
];

const BANNED_RE = new RegExp(`\\b(${BANNED_WORDS.join("|")})s?\\b`, "gi");

/**
 * Strip banned vocabulary from a free-text string as a last-resort backstop.
 * Prefer curated copy below; this only protects against manifest text we can't
 * fully control (e.g. a newly added gallery app with no curated entry yet).
 */
export function sanitizeProsumerCopy(text: string): string {
  return text
    .replace(BANNED_RE, "")
    .replace(/\s{2,}/g, " ")
    .replace(/\s+([.,])/g, "$1")
    .trim();
}

export interface AppCopy {
  /** Two short lines for the gallery card (M2). */
  tagline: string;
  /** Single line for the connect step header (M3b). */
  short: string;
}

/**
 * Curated prosumer copy keyed by gallery key. Taken from the M-series wires
 * (https://happy-grove-jzyc.here.now/). Apps without an entry fall back to a
 * generic, gate-safe line.
 */
const APP_COPY: Record<string, AppCopy> = {
  zapier: {
    tagline: t("ui.lib.app-gallery-copy.reach-000-apps-your"),
    short: t("ui.lib.app-gallery-copy.reach-000-apps-from"),
  },
  github: {
    tagline: t("ui.lib.app-gallery-copy.read-code-pull-requests"),
    short: t("ui.lib.app-gallery-copy.read-code-pull-requests"),
  },
  slack: {
    tagline: t("ui.lib.app-gallery-copy.send-read-messages-your"),
    short: t("ui.lib.app-gallery-copy.send-read-messages-your.2"),
  },
  notion: {
    tagline: t("ui.lib.app-gallery-copy.read-update-pages-your"),
    short: t("ui.lib.app-gallery-copy.read-update-pages-your"),
  },
  linear: {
    tagline: t("ui.lib.app-gallery-copy.create-update-read-tickets"),
    short: t("ui.lib.app-gallery-copy.create-update-read-tickets"),
  },
  "google-sheets": {
    tagline: t("ui.lib.app-gallery-copy.read-update-selected-spreadsheets"),
    short: t("ui.lib.app-gallery-copy.share-each-sheet-robot"),
  },
  gmail: {
    tagline: t("ui.lib.app-gallery-copy.read-mail-send-drafts"),
    short: t("ui.lib.app-gallery-copy.read-mail-send-drafts"),
  },
  hubspot: {
    tagline: t("ui.lib.app-gallery-copy.look-up-contacts-update"),
    short: t("ui.lib.app-gallery-copy.look-up-contacts-update"),
  },
  intercom: {
    tagline: t("ui.lib.app-gallery-copy.read-reply-customer-conversations"),
    short: t("ui.lib.app-gallery-copy.read-reply-customer-conversations"),
  },
  figma: {
    tagline: t("ui.lib.app-gallery-copy.read-files-post-comments"),
    short: t("ui.lib.app-gallery-copy.read-files-post-comments"),
  },
  stripe: {
    tagline: t("ui.lib.app-gallery-copy.read-customers-invoices-payouts"),
    short: t("ui.lib.app-gallery-copy.read-customers-invoices-payouts"),
  },
  context7: {
    tagline: t("ui.lib.app-gallery-copy.look-up-up-date"),
    short: t("ui.lib.app-gallery-copy.look-up-up-date"),
  },
};

const GENERIC: AppCopy = {
  tagline: t("ui.lib.app-gallery-copy.give-your-agents-access"),
  short: t("ui.lib.app-gallery-copy.give-your-agents-access"),
};

/** Curated, gate-safe copy for a gallery app. */
export function appCopyFor(key: string, fallbackTagline?: string | null): AppCopy {
  const curated = APP_COPY[key];
  if (curated) return curated;
  if (fallbackTagline) {
    const cleaned = sanitizeProsumerCopy(fallbackTagline);
    if (cleaned) return { tagline: cleaned, short: cleaned };
  }
  return GENERIC;
}

/**
 * Label for a single credential field on the key-paste step (M3b). The raw
 * manifest label can contain banned vocab ("Zapier MCP token"), so for the
 * common single-field case we present "Your {App} key" per the wires; multi-
 * field apps fall back to a sanitized version of the manifest label.
 */
export function credentialFieldLabel(
  appName: string,
  rawLabel: string,
  fieldCount: number,
): string {
  if (fieldCount <= 1) return `Your ${appName} key`;
  const cleaned = sanitizeProsumerCopy(rawLabel);
  return cleaned || `Your ${appName} key`;
}
