import { Languages } from "lucide-react";
import { useCallback, useState } from "react";

import { cn } from "@/lib/utils";
import { i18n, setLocale, t } from "@/i18n";

type LanguageToggleVariant = "icon" | "menu-action";

interface LanguageToggleProps {
  className?: string;
  /**
   * `icon` (default): compact icon button — suitable for headers and
   * floating chrome.
   *
   * `menu-action`: full-width row with label + description + icon —
   * matches the surrounding `MenuAction` rows in `SidebarAccountMenu`.
   */
  variant?: LanguageToggleVariant;
  /**
   * Called after the locale changes. Surfaces like a popover menu use
   * this to dismiss the menu once the user has acted.
   */
  onAfterToggle?: () => void;
}

const LOCALE_CYCLE = ["en", "zh-CN", "zh-TW"] as const;

const LOCALE_NAMES: Record<(typeof LOCALE_CYCLE)[number], string> = {
  en: "English",
  "zh-CN": "简体中文",
  "zh-TW": "繁體中文",
};

function currentLocale(): string {
  const active = i18n.language?.split("-")[0] === "zh" ? i18n.language : "en";
  return LOCALE_CYCLE.includes(active as (typeof LOCALE_CYCLE)[number]) ? active : "en";
}

export function LanguageToggle({ className, variant = "icon", onAfterToggle }: LanguageToggleProps) {
  const [locale, setLocaleState] = useState<string>(currentLocale());

  const handleClick = useCallback(() => {
    const index = LOCALE_CYCLE.indexOf(locale as (typeof LOCALE_CYCLE)[number]);
    const next = LOCALE_CYCLE[(index + 1) % LOCALE_CYCLE.length];
    setLocale(next);
    setLocaleState(next);
    onAfterToggle?.();
  }, [locale, onAfterToggle]);

  const label = t("common.language", { defaultValue: "Language" });
  const Icon = Languages;

  if (variant === "menu-action") {
    return (
      <button
        type="button"
        className={cn(
          "flex w-full items-start gap-3 rounded-xl px-3 py-3 text-left transition-colors hover:bg-accent/60",
          className,
        )}
        onClick={handleClick}
        aria-label={label}
      >
        <span className="mt-0.5 rounded-lg border border-border bg-background/70 p-2 text-muted-foreground">
          <Icon className="size-4" />
        </span>
        <span className="min-w-0 flex-1">
          <span className="block text-sm font-medium text-foreground">{label}</span>
          <span className="block text-xs text-muted-foreground">
            {t("common.languageDescription", { defaultValue: "Switch the interface language." })}
            {" · "}
            {LOCALE_NAMES[locale as (typeof LOCALE_CYCLE)[number]] ?? "English"}
          </span>
        </span>
      </button>
    );
  }

  return (
    <button
      type="button"
      className={cn(
        "inline-flex size-9 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-accent/60 hover:text-foreground",
        className,
      )}
      onClick={handleClick}
      aria-label={label}
      title={label}
    >
      <Icon className="size-4" />
    </button>
  );
}
