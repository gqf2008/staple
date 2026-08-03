import type { AdapterConfigFieldsProps } from "../types";
import { t } from "../../i18n";
import {
  DraftNumberInput,
  DraftInput,
  Field,
} from "../../components/agent-config-primitives";
import { ChoosePathButton } from "../../components/PathInstructionsModal";

const inputClass =
  "w-full rounded-md border border-border px-2.5 py-1.5 bg-transparent outline-none text-sm font-mono placeholder:text-muted-foreground/40";
const instructionsFileHint =
  "Absolute path to a markdown file (e.g. AGENTS.md) that defines this agent's behavior. Prepended to the Gemini prompt at runtime.";

export function GeminiLocalConfigFields({
  isCreate,
  values,
  set,
  config,
  eff,
  mark,
  hideInstructionsFile,
}: AdapterConfigFieldsProps) {
  const rawEngine = isCreate
    ? values!.geminiEngine ?? "auto"
    : eff("adapterConfig", "engine", String(config.engine ?? "auto"));
  const engine = rawEngine === "acp" || rawEngine === "cli" ? rawEngine : "auto";
  const acpSelected = engine === "acp";

  return (
    <>
      <Field label={t("components.geminiConfig.executionEngine", { defaultValue: "Execution engine" })} hint={t("components.geminiConfig.engineHint", { defaultValue: "Auto uses ACP when prerequisites pass and falls back to Gemini CLI with diagnostics." })}>
        <select
          className={inputClass}
          value={engine}
          onChange={(e) => {
            const value = e.target.value === "acp" ? "acp" : e.target.value === "cli" ? "cli" : "auto";
            isCreate
              ? set!({ geminiEngine: value })
              : mark("adapterConfig", "engine", value === "auto" ? undefined : value);
          }}
        >
          <option value="auto">{t("components.geminiConfig.auto", { defaultValue: "Auto (ACP preferred)" })}</option>
          <option value="cli">{t("components.geminiConfig.geminiCli", { defaultValue: "Gemini CLI" })}</option>
          <option value="acp">{t("components.geminiConfig.acp", { defaultValue: "ACP" })}</option>
        </select>
      </Field>
      {acpSelected && (
        <>
          <Field
            label={t("components.geminiConfig.acpCommand", { defaultValue: "ACP server command" })}
            hint={t("components.geminiConfig.acpCommandHint", { defaultValue: "Optional override for the Gemini ACP server command. Defaults to gemini --acp." })}
          >
            <DraftInput
              value={
                isCreate
                  ? values!.geminiAcpAgentCommand ?? ""
                  : eff("adapterConfig", "agentCommand", String(config.agentCommand ?? ""))
              }
              onCommit={(v) =>
                isCreate
                  ? set!({ geminiAcpAgentCommand: v })
                  : mark("adapterConfig", "agentCommand", v || undefined)
              }
              immediate
              className={inputClass}
              placeholder="gemini --acp"
            />
          </Field>
          <Field label={t("components.geminiConfig.acpSessionMode", { defaultValue: "ACP session mode" })} hint={t("components.geminiConfig.sessionModeHint", { defaultValue: "Persistent keeps ACP session state between runs. One-shot starts fresh each run." })}>
            <select
              className={inputClass}
              value={
                isCreate
                  ? values!.geminiAcpMode ?? "persistent"
                  : eff("adapterConfig", "mode", String(config.mode ?? "persistent"))
              }
              onChange={(e) => {
                const value = e.target.value === "oneshot" ? "oneshot" : "persistent";
                isCreate
                  ? set!({ geminiAcpMode: value })
                  : mark("adapterConfig", "mode", value);
              }}
            >
              <option value="persistent">{t("components.geminiConfig.persistent", { defaultValue: "Persistent" })}</option>
              <option value="oneshot">{t("components.geminiConfig.oneShot", { defaultValue: "One-shot" })}</option>
            </select>
          </Field>
          <Field
            label={t("components.geminiConfig.acpPermissions", { defaultValue: "ACP non-interactive permissions" })}
            hint={t("components.geminiConfig.permissionsHint", { defaultValue: "Fallback if the ACP agent asks for input outside an interactive session." })}
          >
            <select
              className={inputClass}
              value={
                isCreate
                  ? values!.geminiAcpNonInteractivePermissions ?? "deny"
                  : eff("adapterConfig", "nonInteractivePermissions", String(config.nonInteractivePermissions ?? "deny"))
              }
              onChange={(e) => {
                const value = e.target.value === "fail" ? "fail" : "deny";
                isCreate
                  ? set!({ geminiAcpNonInteractivePermissions: value })
                  : mark("adapterConfig", "nonInteractivePermissions", value);
              }}
            >
              <option value="deny">{t("components.geminiConfig.deny", { defaultValue: "Deny" })}</option>
              <option value="fail">{t("components.geminiConfig.fail", { defaultValue: "Fail" })}</option>
            </select>
          </Field>
          <Field
            label={t("components.geminiConfig.acpStateDir", { defaultValue: "ACP state directory" })}
            hint="Optional ACP session state directory. Defaults to Paperclip-managed company/agent scoped storage."
          >
            <div className="flex items-center gap-2">
              <DraftInput
                value={
                  isCreate
                    ? values!.geminiAcpStateDir ?? ""
                    : eff("adapterConfig", "stateDir", String(config.stateDir ?? ""))
                }
                onCommit={(v) =>
                  isCreate
                    ? set!({ geminiAcpStateDir: v })
                    : mark("adapterConfig", "stateDir", v || undefined)
                }
                immediate
                className={inputClass}
                placeholder="/path/to/acp-state"
              />
              <ChoosePathButton />
            </div>
          </Field>
          <Field
            label={t("components.geminiConfig.warmIdleMs", { defaultValue: "ACP warm process idle ms" })}
            hint="Defaults to 0, which closes the ACP process after each run while retaining persistent session state."
          >
            {isCreate ? (
              <input
                type="number"
                className={inputClass}
                value={values!.geminiAcpWarmHandleIdleMs ?? 0}
                onChange={(e) => set!({ geminiAcpWarmHandleIdleMs: Number(e.target.value) })}
              />
            ) : (
              <DraftNumberInput
                value={eff(
                  "adapterConfig",
                  "warmHandleIdleMs",
                  Number(config.warmHandleIdleMs ?? 0),
                )}
                onCommit={(v) => mark("adapterConfig", "warmHandleIdleMs", v || 0)}
                immediate
                className={inputClass}
              />
            )}
          </Field>
        </>
      )}
      {!hideInstructionsFile && (
        <Field label={t("components.geminiConfig.instructionsFile", { defaultValue: "Agent instructions file" })} hint={instructionsFileHint}>
          <div className="flex items-center gap-2">
            <DraftInput
              value={
                isCreate
                  ? values!.instructionsFilePath ?? ""
                  : eff(
                      "adapterConfig",
                      "instructionsFilePath",
                      String(config.instructionsFilePath ?? ""),
                    )
              }
              onCommit={(v) =>
                isCreate
                  ? set!({ instructionsFilePath: v })
                  : mark("adapterConfig", "instructionsFilePath", v || undefined)
              }
              immediate
              className={inputClass}
              placeholder="/absolute/path/to/AGENTS.md"
            />
            <ChoosePathButton />
          </div>
        </Field>
      )}
    </>
  );
}
