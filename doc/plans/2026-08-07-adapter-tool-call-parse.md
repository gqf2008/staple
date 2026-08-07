# adapter tool-call 结构化解析（issue #230）

> 状态：实施中 → 完成。日期：2026-08-07。
> 关联：`crates/adapters`（契约 + CLI adapter）、`crates/app`（SSE / Board Chat UI）、
> `doc/plans/parity-checklist.md`（B6.3 登记）。

## 目标

各 agent adapter（CLI 类）输出 transcript 中的工具调用（tool-call）目前只作为
普通 Delta 文本流入 Board Chat；B6.2 已把 stdout/stderr 提升为结构化输出事件
（`Delta` / `Stderr`），但 tool-call 尚未统一解析。本 issue 将可可靠识别的
transcript tool-call 格式统一解析为结构化事件（id / function name / arguments），
供 `crates/app/src/ui/board_chat.js` 的工具折叠（accordion）展示；无法可靠解析的
格式登记降级策略（不阻塞显示）。

## 盘点：现网 adapter transcript 的 tool-call 格式

以下清单以参考镜像 `gqf2008/paperclip`（只读）的 adapter 输出解析器为准
（`packages/adapters/*/src/ui/parse-stdout.ts`、`adapter-utils/src/acpx-engine/ui.ts`）。
Staple Rust 侧 `crates/adapters` 的 CLI adapter 是通用 `sh -c` 宿主，可驱动上述任意
CLI，因此解析层必须格式无关（按行自识别），而不是为每个 CLI 写一个 adapter。

### A. JSON 行（stream-json / NDJSON）——可靠解析 ✅

| # | 来源 CLI | 样例（一行一个事件） | 识别字段 |
|---|---|---|---|
| A1 | Codex CLI | `{"type":"item.started","item":{"type":"command_execution","id":"c1","command":"cargo test"}}` | `item.type=command_execution` → name=`command_execution`，id=`item.id`（缺省用 command），arguments=`{id,command}` |
| A2 | Codex CLI | `{"type":"item.started","item":{"type":"tool_use","id":"tu1","name":"read_file","input":{"path":"x"}}}` | name=`item.name`，id=`item.id`，arguments=`item.input` |
| A3 | Claude Code CLI | `{"type":"assistant","message":{"content":[{"type":"tool_use","id":"toolu_1","name":"Bash","input":{...}}]}}` | content 块 `type=tool_use`（首个），name/id/input |
| A4 | ACPX（Claude/Codex/Gemini 共用协议） | `{"type":"acpx.tool_call","name":"shell","toolCallId":"a1","input":{...},"status":"running"}` | id=`toolCallId`/`toolUseId`/`id`，arguments=`input`（可补 status/text） |
| A5 | Cursor Cloud SDK | `{"type":"cursor_cloud.message","message":{"type":"tool_call","call_id":"cc1","name":"shellToolCall","args":{...}}}` | `message.type=tool_call`，name/id/args |
| A6 | Cursor Local / Gemini CLI | `{"type":"tool_call","subtype":"started","call_id":"cl1","tool_call":{"shell":{"args":{...}}}}` | tool 名 = `tool_call` 对象首键，id=`call_id`/`callId`/`id`，arguments=`args`/`arguments`/`input`/`function.arguments` |
| A7 | Gemini CLI v0.38+ | `{"type":"message","role":"assistant","content":[{"type":"tool_call","name":"search","input":{...}}]}` | content 块 `type=tool_call` |
| A8 | OpenCode CLI | `{"type":"tool_use","part":{"tool":"bash","callID":"oc1","input":{...}}}` | `part.tool`/`part.callID`/`part.input` |
| A9 | Pi local | `{"type":"tool_execution_start","toolCallId":"pi1","toolName":"shell","args":{...}}` | `toolName`/`toolCallId`/`args` |
| A10 | OpenAI 风格 function-call（常见兼容形态） | `{"type":"tool_call","id":"fn1","function":{"name":"read_file","arguments":"{\"path\":\"x\"}"}}` | `function.name`，arguments 为 JSON 字符串时解析为对象 |

### B. 前缀标记（非 JSON）——可靠解析 ✅

| # | 来源 CLI | 样例 | 识别 |
|---|---|---|---|
| B1 | Hermes CLI quiet-mode（TTY/pipe） | `  ┊ 💻 $         curl -s https://example.com  0.2s`、`  [done] ┊ 🔍 search    pattern  0.1s (0.5s)` | 含 `┊`、非 `💬` 助手行、结尾带 `N.Ns` 时长；动词映射（`$`/`exec`/`terminal` → `shell`）；id 由调用方合成 |

### C. 无法可靠解析——登记降级（不阻塞显示）⏳

| # | 来源 CLI / 形态 | 说明 | 降级策略 |
|---|---|---|---|
| C1 | Grok local | 输出仅 `thought`/`text`/`error`/`end` JSON，无工具调用结构 | 全部按 Delta 显示，无工具折叠 |
| C2 | OpenClaw gateway | `[openclaw-gateway:event] ... stream=assistant/error/lifecycle`，无工具调用结构 | 全部按 Delta/Stderr 显示，无工具折叠 |
| C3 | XML 工具标签（`<tool_call>`/`<invoke>` 等） | 无统一 schema，参考镜像 adapter 未使用 | 按 Delta 显示；不尝试启发式解析，避免误判 |
| C4 | 多行 pretty-printed JSON | 单事件跨多行，行级解析无法重组 | 各片段按 Delta 显示 |
| C5 | ANSI 包装的非 JSON 行 | 剥离 ANSI 后仍无法识别 | 按 Delta 显示（保留原始文本） |
| C6 | HTTP adapter | 当前无 stream（`stream()` 返回不支持），transcript 仅在 `observe` 终态里 | 暂不解析；待 HTTP 流式落地后再接入 |

> 降级总原则：解析层只做“识别成功 → 结构化事件；其余 → 原样 Delta”，任何情况下
> 都不吞掉 transcript 文本、不阻塞 SSE/UI 展示。

## 统一事件结构

`crates/adapters/src/tool_call.rs` 新增纯解析模块；`OutputEvent` 增加变体：

```text
OutputEvent::ToolCall {
  id: String,        // 格式自带 id；Hermes 等无 id 格式由 CLI adapter 合成 tool-N
  name: String,      // 工具/函数名
  arguments: Value,  // JSON 参数（对象或字符串）
}
```

SSE 线上形态：`{"type":"toolCall","id":"t1","name":"shell","arguments":{...}}`。
`board_chat.js` 兼容 `toolCall` 与旧 `tool` 两种 type，标题为 `name · id`，正文为
arguments 的 pretty-print JSON。

## 接入点

- `crates/adapters/src/tool_call.rs` — `parse_tool_call_line(line) -> Option<ToolCall>`
- `crates/adapters/src/cli.rs` — stdout 按行缓冲后解析：识别为 tool-call 则发
  `OutputEvent::ToolCall`，否则原样 `OutputEvent::Delta`；stderr 行为不变
- `crates/adapters/src/contract.rs` — `OutputEvent::ToolCall(ToolCall)`
- `crates/app/src/ui/board_chat.js` — 工具折叠渲染 arguments
- `crates/app/tests/api.rs` — SSE 线上形态断言

## 测试

- `crates/adapters/src/tool_call.rs` 单元测试：A1–A10、B1、C1–C5 样例 transcript
- `crates/adapters/src/cli.rs` 集成测试：sh 输出 JSON 行 / Hermes 前缀行 / 跨 chunk
  长行（5000 字符 JSON 行）均能产出结构化事件；普通行仍为 Delta
- `crates/app/tests/api.rs`：Board Chat SSE 含 `"type":"toolCall"` 与工具名

验证命令：

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p staple-adapters
cargo test -p staple-app --test api board_chat_stream_validation_and_sse
```

## 后续（明确不在本 issue）

- tool_call 事件持久化到 `tool_call_events`（MCP gateway 用途不同，另行评估）
- 按 CLI 细分 adapter 配置（如只对 codex/hermes 启用解析）——当前格式无关自识别
  已覆盖主要格式，无需配置开关
