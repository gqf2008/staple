import i18n, { type InitOptions, type TOptions } from "i18next";
import { initReactI18next, useTranslation as useReactI18nextTranslation } from "react-i18next";

import { DEFAULT_LOCALE, i18nextResources, supportedLocales } from "./locales";

const LOCALE_STORAGE_KEY = "paperclip.locale";

/**
 * Normalize a raw locale candidate (browser string, stored value, user input)
 * into one of the supported locale codes, or null when it cannot be matched.
 */
export function normalizeLocale(candidate: string | null | undefined): string | null {
  if (!candidate) return null;
  const normalized = candidate.replace("_", "-").toLowerCase();
  if (normalized.startsWith("zh")) {
    if (normalized === "zh-tw" || normalized === "zh-hant") return "zh-TW";
    return "zh-CN";
  }
  const exact = supportedLocales.find((locale) => locale.toLowerCase() === normalized);
  return exact ?? null;
}

/**
 * Resolve the initial UI locale: an explicitly stored choice wins, then the
 * browser language, then the repository default (en).
 */
export function detectInitialLocale(): string {
  if (typeof window !== "undefined") {
    try {
      const stored = window.localStorage.getItem(LOCALE_STORAGE_KEY);
      const fromStorage = normalizeLocale(stored);
      if (fromStorage) return fromStorage;
    } catch {
      // Storage unavailable (private mode, tests, etc.) — fall through.
    }
    const fromBrowser = normalizeLocale(
      typeof navigator !== "undefined" ? navigator.language : undefined,
    );
    if (fromBrowser) return fromBrowser;
  }
  return DEFAULT_LOCALE;
}

/**
 * Switch the active UI locale and persist the choice for the next visit.
 */
export function setLocale(locale: string): void {
  const normalized = normalizeLocale(locale) ?? DEFAULT_LOCALE;
  void i18n.changeLanguage(normalized);
  if (typeof window !== "undefined") {
    try {
      window.localStorage.setItem(LOCALE_STORAGE_KEY, normalized);
    } catch {
      // Storage unavailable — the in-memory switch still applies.
    }
  }
}

const i18nextOptions: InitOptions = {
  resources: i18nextResources,
  lng: detectInitialLocale(),
  fallbackLng: DEFAULT_LOCALE,
  supportedLngs: supportedLocales,
  defaultNS: "translation",
  interpolation: { escapeValue: false },
  returnObjects: false,
  initAsync: false,
};

void i18n.use(initReactI18next).init(i18nextOptions).catch((error: unknown) => {
  console.error("Failed to initialize i18next", error);
});

export function t(key: string, options: TOptions = {}) {
  return i18n.t(key, options);
}

export const useTranslation = useReactI18nextTranslation;
export { i18n };
