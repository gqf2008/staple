import { AlertTriangle } from "lucide-react";
import { t } from "../../../i18n";
import type { ToolProfileWithDetails } from "@paperclipai/shared";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

export type ProfileActionDialogKind = "archive" | "delete" | "restore";

export function ProfileActionDialog({
  kind,
  profile,
  pending,
  onClose,
  onArchive,
  onRestore,
  onDelete,
}: {
  kind: ProfileActionDialogKind | null;
  profile: ToolProfileWithDetails | null;
  pending: boolean;
  onClose: () => void;
  onArchive: () => void;
  onRestore: () => void;
  onDelete: () => void;
}) {
  if (!kind || !profile) return null;

  const defaultDeleteBlocked = kind === "delete" && profile.summary.isCompanyDefault;
  const copy = {
    archive: {
      title: t("pages.tools.profileActionDialog.archiveProfile", { defaultValue: "Archive profile" }),
      body: `This profile stops applying to ${profile.summary.appliesToAgentCount} ${profile.summary.appliesToAgentCount === 1 ? "agent" : "agents"}. You can restore it later.`,
      confirm: t("pages.tools.profileActionDialog.archive", { defaultValue: "Archive" }),
      action: onArchive,
    },
    restore: {
      title: t("pages.tools.profileActionDialog.restoreProfile", { defaultValue: "Restore profile" }),
      body: t("pages.tools.profileActionDialog.restoreHint", { defaultValue: "This profile will be active again and can be assigned to agents." }),
      confirm: t("pages.tools.profileActionDialog.restore", { defaultValue: "Restore" }),
      action: onRestore,
    },
    delete: {
      title: t("pages.tools.profileActionDialog.deleteProfile", { defaultValue: "Delete profile" }),
      body: defaultDeleteBlocked
        ? "This profile is the company default. Reassign the company default to another profile before deleting it."
        : `This permanently deletes the profile and removes ${profile.summary.assignmentCount} ${profile.summary.assignmentCount === 1 ? "assignment" : "assignments"}.`,
      confirm: t("pages.tools.profileActionDialog.delete", { defaultValue: "Delete" }),
      action: onDelete,
    },
  }[kind];

  return (
    <Dialog open onOpenChange={(next) => !next && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{copy.title}</DialogTitle>
          <DialogDescription>{copy.body}</DialogDescription>
        </DialogHeader>
        {defaultDeleteBlocked ? (
          <div className="flex gap-3 rounded-lg border border-destructive/40 bg-destructive/5 px-3 py-2 text-sm text-destructive">
            <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" />
            <span>{t("pages.tools.profileActionDialog.companyDefaultHint", { defaultValue: "Choose another access profile and make it the company default first." })}</span>
          </div>
        ) : null}
        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>{t("pages.tools.profileActionDialog.cancel", { defaultValue: "Cancel" })}</Button>
          <Button
            variant={kind === "delete" ? "destructive" : "default"}
            disabled={pending || defaultDeleteBlocked}
            onClick={copy.action}
          >
            {copy.confirm}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
