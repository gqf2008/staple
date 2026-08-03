import { useState } from "react";
import { t } from "../i18n";
import { Apple, Monitor, Terminal } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { cn } from "@/lib/utils";

type Platform = "mac" | "windows" | "linux";

const platforms: { id: Platform; label: string; icon: typeof Apple }[] = [
  { id: "mac", label: t("ui.components.pathinstructionsmodal.macos"), icon: Apple },
  { id: "windows", label: t("components.pathInstructions.windows", { defaultValue: "Windows" }), icon: Monitor },
  { id: "linux", label: t("components.pathInstructions.linux", { defaultValue: "Linux" }), icon: Terminal },
];

const instructions: Record<Platform, { steps: string[]; tip?: string }> = {
  mac: {
    steps: [
      t("components.pathInstructions.finder1", { defaultValue: "Open Finder and navigate to the folder." }),
      t("components.pathInstructions.finder2", { defaultValue: "Right-click (or Control-click) the folder." }),
      t("ui.components.pathinstructionsmodal.hold-option-key-copy"),
      t("ui.components.pathinstructionsmodal.click-copy-pathname-then"),
    ],
    tip: t("ui.components.pathinstructionsmodal.you-can-also-open"),
  },
  windows: {
    steps: [
      t("components.pathInstructions.explorer1", { defaultValue: "Open File Explorer and navigate to the folder." }),
      t("components.pathInstructions.explorer2", { defaultValue: "Click in the address bar at the top — the full path will appear." }),
      t("components.pathInstructions.copyPaste", { defaultValue: "Copy the path, then paste here." }),
    ],
    tip: "Alternatively, hold Shift and right-click the folder, then select \"Copy as path\".",
  },
  linux: {
    steps: [
      t("components.pathInstructions.terminal1", { defaultValue: "Open a terminal and navigate to the directory with cd." }),
      t("components.pathInstructions.terminal2", { defaultValue: "Run pwd to print the full path." }),
      t("components.pathInstructions.terminal3", { defaultValue: "Copy the output and paste here." }),
    ],
    tip: t("components.pathInstructions.ctrlLHint", { defaultValue: "In most file managers, Ctrl+L reveals the full path in the address bar." }),
  },
};

function detectPlatform(): Platform {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac")) return "mac";
  if (ua.includes("win")) return "windows";
  return "linux";
}

interface PathInstructionsModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function PathInstructionsModal({
  open,
  onOpenChange,
}: PathInstructionsModalProps) {
  const [platform, setPlatform] = useState<Platform>(detectPlatform);

  const current = instructions[platform];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="text-base">{t("components.pathInstructions.title", { defaultValue: "How to get a full path" })}</DialogTitle>
          <DialogDescription>
            {t("ui.components.pathinstructionsmodal.paste-absolute-path")}{" "}
            <code className="text-xs bg-muted px-1 py-0.5 rounded">/Users/you/project</code>
            {t("ui.components.pathinstructionsmodal.into-input-field")}</DialogDescription>
        </DialogHeader>

        {/* Platform tabs */}
        <div className="flex gap-1 rounded-md border border-border p-0.5">
          {platforms.map((p) => (
            <button
              key={p.id}
              type="button"
              className={cn(
                "flex flex-1 items-center justify-center gap-1.5 rounded px-2 py-1 text-xs transition-colors",
                platform === p.id
                  ? "bg-accent text-foreground"
                  : "text-muted-foreground hover:text-foreground hover:bg-accent/50",
              )}
              onClick={() => setPlatform(p.id)}
            >
              <p.icon className="h-3.5 w-3.5" />
              {p.label}
            </button>
          ))}
        </div>

        {/* Steps */}
        <ol className="space-y-2 text-sm">
          {current.steps.map((step, i) => (
            <li key={i} className="flex gap-2">
              <span className="text-muted-foreground font-mono text-xs mt-0.5 shrink-0">
                {i + 1}.
              </span>
              <span>{step}</span>
            </li>
          ))}
        </ol>

        {current.tip && (
          <p className="text-xs text-muted-foreground border-l-2 border-border pl-3">
            {current.tip}
          </p>
        )}
      </DialogContent>
    </Dialog>
  );
}

/**
 * Small t("components.pathInstructions.choose", { defaultValue: "Choose" }) button that opens the PathInstructionsModal.
 * Drop-in replacement for the old showDirectoryPicker buttons.
 */
export function ChoosePathButton({ className }: { className?: string }) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <button
        type="button"
        className={cn(
          "inline-flex items-center rounded-md border border-border px-2 py-0.5 text-xs text-muted-foreground hover:bg-accent/50 transition-colors shrink-0",
          className,
        )}
        onClick={() => setOpen(true)}
      >
        {t("components.agentConfigPrimitives.choose")}</button>
      <PathInstructionsModal open={open} onOpenChange={setOpen} />
    </>
  );
}
