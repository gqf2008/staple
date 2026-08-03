import { t } from "../i18n";
import { cn } from "../lib/utils";

interface OpenCodeLogoIconProps {
  className?: string;
}

export function OpenCodeLogoIcon({ className }: OpenCodeLogoIconProps) {
  return (
    <>
      <img
        src="/brands/opencode-logo-light-square.svg"
        alt={t("ui.components.opencodelogoicon.opencode")}
        className={cn("dark:hidden", className)}
      />
      <img
        src="/brands/opencode-logo-dark-square.svg"
        alt={t("ui.components.opencodelogoicon.opencode")}
        className={cn("hidden dark:block", className)}
      />
    </>
  );
}
