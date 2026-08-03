import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { t } from "../i18n";

interface ShortcutEntry {
  keys: string[];
  label: string;
  /** Render keys as a simultaneous chord (joined with "+") rather than a
   *  "then" sequence. */
  combo?: boolean;
}

// Platform-appropriate label for the Cmd/Ctrl modifier so the cheatsheet shows
// the same key the user actually presses (re-pointed in the collapsible sidebar
// work — Cmd/Ctrl+B toggles the rail).
function getPlatformLabel() {
  if (typeof navigator === "undefined") return "";
  const nav = navigator as Navigator & { userAgentData?: { platform?: string } };
  return nav.userAgentData?.platform || navigator.userAgent || "";
}

const META_KEY = /Mac|iPhone|iPad|iPod/.test(getPlatformLabel()) ? "⌘" : t("components.keyboardShortcuts.ctrl", { defaultValue: "Ctrl" });

interface ShortcutSection {
  title: string;
  shortcuts: ShortcutEntry[];
}

const sections: ShortcutSection[] = [
  {
    title: t("components.keyboardShortcuts.inbox", { defaultValue: "Inbox" }),
    shortcuts: [
      { keys: ["j"], label: t("components.keyboardShortcuts.moveDown", { defaultValue: "Move down" }) },
      { keys: ["↓"], label: t("components.keyboardShortcuts.moveDown", { defaultValue: "Move down" }) },
      { keys: ["k"], label: t("components.keyboardShortcuts.moveUp", { defaultValue: "Move up" }) },
      { keys: ["↑"], label: t("components.keyboardShortcuts.moveUp", { defaultValue: "Move up" }) },
      { keys: ["←"], label: t("components.keyboardShortcuts.collapseGroup", { defaultValue: "Collapse selected group" }) },
      { keys: ["→"], label: t("components.keyboardShortcuts.expandGroup", { defaultValue: "Expand selected group" }) },
      { keys: ["Enter"], label: t("components.keyboardShortcuts.openItem", { defaultValue: "Open selected item" }) },
      { keys: ["a"], label: t("components.keyboardShortcuts.archiveItem", { defaultValue: "Archive item" }) },
      { keys: ["y"], label: t("components.keyboardShortcuts.archiveItem", { defaultValue: "Archive item" }) },
      { keys: ["r"], label: t("components.keyboardShortcuts.markAsRead", { defaultValue: "Mark as read" }) },
      { keys: ["U"], label: t("components.keyboardShortcuts.markAsUnread", { defaultValue: "Mark as unread" }) },
    ],
  },
  {
    title: t("components.keyboardShortcuts.taskDetail", { defaultValue: "Task detail" }),
    shortcuts: [
      { keys: ["y"], label: t("components.keyboardShortcuts.quickArchive", { defaultValue: "Quick-archive back to inbox" }) },
      { keys: ["g", "i"], label: t("components.keyboardShortcuts.goToInbox", { defaultValue: "Go to inbox" }) },
      { keys: ["g", "c"], label: t("components.keyboardShortcuts.focusComposer", { defaultValue: "Focus comment composer" }) },
    ],
  },
  {
    title: t("components.keyboardShortcuts.decisions", { defaultValue: "Decisions" }),
    shortcuts: [
      { keys: ["j"], label: t("components.keyboardShortcuts.moveDown", { defaultValue: "Move down" }) },
      { keys: ["↓"], label: t("components.keyboardShortcuts.moveDown", { defaultValue: "Move down" }) },
      { keys: ["k"], label: t("components.keyboardShortcuts.moveUp", { defaultValue: "Move up" }) },
      { keys: ["↑"], label: t("components.keyboardShortcuts.moveUp", { defaultValue: "Move up" }) },
      { keys: ["Enter"], label: t("components.keyboardShortcuts.toggleDecision", { defaultValue: "Open or close selected decision" }) },
      { keys: ["x"], label: t("components.keyboardShortcuts.dismissDecision", { defaultValue: "Dismiss selected decision" }) },
    ],
  },
  {
    title: t("components.keyboardShortcuts.global", { defaultValue: "Global" }),
    shortcuts: [
      { keys: ["/"], label: t("components.keyboardShortcuts.search", { defaultValue: "Search current page or quick search" }) },
      { keys: ["c"], label: t("components.keyboardShortcuts.newTask", { defaultValue: "New task" }) },
      { keys: ["["], label: t("components.keyboardShortcuts.toggleSidebar", { defaultValue: "Toggle sidebar" }) },
      { keys: [META_KEY, "B"], label: t("components.keyboardShortcuts.collapseSidebar", { defaultValue: "Collapse or expand sidebar" }), combo: true },
      { keys: ["]"], label: t("components.keyboardShortcuts.togglePanel", { defaultValue: "Toggle panel" }) },
      { keys: ["?"], label: t("components.keyboardShortcuts.showShortcuts", { defaultValue: "Show keyboard shortcuts" }) },
    ],
  },
];

function KeyCap({ children }: { children: string }) {
  return (
    <kbd className="inline-flex h-6 min-w-6 items-center justify-center rounded border border-border bg-muted px-1.5 font-mono text-xs font-medium text-foreground shadow-(--shadow-extract-10)">
      {children}
    </kbd>
  );
}

export function KeyboardShortcutsCheatsheetContent() {
  return (
    <>
      <div className="divide-y divide-border border-t border-border">
        {sections.map((section) => (
          <div key={section.title} className="px-5 py-3">
            <h3 className="mb-2 text-(length:--text-micro) font-semibold uppercase tracking-wider text-muted-foreground">
              {section.title}
            </h3>
            <div className="space-y-1.5">
              {section.shortcuts.map((shortcut) => (
                <div
                  key={shortcut.label + shortcut.keys.join()}
                  className="flex items-center justify-between gap-4"
                >
                  <span className="text-sm text-foreground/90">{shortcut.label}</span>
                  <div className="flex items-center gap-1">
                    {shortcut.keys.map((key, i) => (
                      <span key={key} className="flex items-center gap-1">
                        {i > 0 && (
                          <span className="text-xs text-muted-foreground">
                            {shortcut.combo ? "+" : "then"}
                          </span>
                        )}
                        <KeyCap>{key}</KeyCap>
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
      <div className="border-t border-border px-5 py-3">
        <p className="text-xs text-muted-foreground">
          Press <KeyCap>{t("components.keyboardShortcuts.esc", { defaultValue: "Esc" })}</KeyCap> to close &middot; Shortcuts are disabled in text fields
        </p>
      </div>
    </>
  );
}

export function KeyboardShortcutsCheatsheet({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md gap-0 p-0 overflow-hidden" showCloseButton={false}>
        <DialogHeader className="px-5 pt-5 pb-3">
          <DialogTitle className="text-base">{t("components.keyboardShortcuts.title", { defaultValue: "Keyboard shortcuts" })}</DialogTitle>
        </DialogHeader>
        <KeyboardShortcutsCheatsheetContent />
      </DialogContent>
    </Dialog>
  );
}
