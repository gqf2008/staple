//! Structured tool-call parsing for agent transcripts.
//!
//! Agent CLIs (Codex, Claude Code, Cursor, Gemini, OpenCode, Pi, Hermes,
//! ...) stream tool invocations in several transcript formats: JSON lines
//! (stream-json event shapes), prefixed human-readable lines (Hermes `┊`),
//! and XML-ish tags. This module normalizes the reliably parseable shapes
//! into a single [`ToolCall`] event for the Board Chat tool accordions.
//!
//! Parsing is defensive: the transcript is LLM-produced, untrusted input.
//! We never execute anything from it, and unrecognized lines fall back to
//! `None` so callers can keep rendering them as plain text. Formats that
//! cannot be parsed reliably (XML tool tags, Grok/OpenClaw plain text) are
//! intentionally degraded to plain deltas — see
//! `doc/plans/2026-08-07-adapter-tool-call-parse.md`.

use serde_json::{Map, Value};

/// One structured tool invocation parsed from an agent transcript line.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    /// Tool-call id from the transcript. Empty when the format carries no
    /// id; callers may synthesize a stable display id.
    pub id: String,
    /// Tool/function name.
    pub name: String,
    /// Arguments/input for the call.
    pub arguments: Value,
}

/// Parses one transcript line into a structured tool call.
///
/// Returns `None` when the line is not a reliably recognizable tool-call
/// event (plain text, unknown JSON shapes, XML tags, ...). Callers then
/// render the line as a plain delta.
pub fn parse_tool_call_line(line: &str) -> Option<ToolCall> {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(Value::Object(obj)) = serde_json::from_str(trimmed) {
        return parse_json_tool_call(&obj);
    }
    parse_hermes_line(trimmed)
}

/// Removes ANSI escape sequences (OSC + CSI) from terminal text.
fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            // OSC: ESC ] ... (BEL | ESC \)
            Some(']') => {
                let _ = chars.next();
                for next in chars.by_ref() {
                    if next == '\u{7}' {
                        break;
                    }
                    if next == '\u{1b}' {
                        if chars.peek() == Some(&'\\') {
                            let _ = chars.next();
                        }
                        break;
                    }
                }
            }
            // CSI: ESC [ params* intermediate* final
            Some('[') => {
                let _ = chars.next();
                for next in chars.by_ref() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                    if next == '\u{1b}' {
                        break;
                    }
                }
            }
            // Charset/control intermediate sequences (ESC ( B, ESC ) 0, ESC # 8,
            // ESC % G, ...): consume the intermediate and the final byte.
            Some(&('(' | ')' | '#' | '%' | '*' | '+')) => {
                let _ = chars.next();
                if let Some(&final_byte) = chars.peek()
                    && final_byte.is_ascii()
                    && !final_byte.is_ascii_control()
                {
                    let _ = chars.next();
                }
            }
            // Two-character control sequences (ESC 7 / ESC 8 / ...): consume
            // the single following printable ASCII byte.
            Some(&next) if next.is_ascii() && !next.is_ascii_control() => {
                let _ = chars.next();
            }
            // Bare ESC: drop it.
            _ => {}
        }
    }
    out
}

fn str_field<'a>(obj: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    obj.get(key).and_then(Value::as_str)
}

fn obj_field<'a>(obj: &'a Map<String, Value>, key: &str) -> Option<&'a Map<String, Value>> {
    obj.get(key).and_then(Value::as_object)
}

fn first_str<'a>(obj: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| str_field(obj, key))
}

fn first_value(obj: &Map<String, Value>, keys: &[&str]) -> Option<Value> {
    keys.iter().find_map(|key| obj.get(*key).cloned())
}

fn empty_args() -> Value {
    Value::Object(Map::new())
}

fn parse_json_tool_call(obj: &Map<String, Value>) -> Option<ToolCall> {
    match str_field(obj, "type")? {
        "acpx.tool_call" => parse_acpx_tool_call(obj),
        "item.started" => parse_codex_item_started(obj),
        "assistant" => parse_assistant_tool_call(obj),
        "tool_call" => parse_top_level_tool_call(obj),
        "tool_use" => parse_tool_use_event(obj),
        "tool_execution_start" => parse_tool_execution_start(obj),
        "cursor_cloud.message" => parse_cursor_cloud_message(obj),
        "message" => parse_gemini_message_tool_call(obj),
        _ => None,
    }
}

/// ACPX protocol: `{"type":"acpx.tool_call","name":...,"toolCallId"|"toolUseId"|"id":...,"input":{...}}`.
fn parse_acpx_tool_call(obj: &Map<String, Value>) -> Option<ToolCall> {
    let name = str_field(obj, "name").unwrap_or("acp_tool").to_owned();
    let id = first_str(obj, &["toolCallId", "toolUseId", "id"])
        .unwrap_or("")
        .to_owned();
    let arguments = match obj.get("input") {
        Some(Value::Object(input)) => {
            let mut input = input.clone();
            for (key, source) in [("status", "status"), ("text", "text")] {
                if let Some(value) = str_field(obj, source) {
                    input
                        .entry(key)
                        .or_insert_with(|| Value::String(value.to_owned()));
                }
            }
            Value::Object(input)
        }
        Some(other) => other.clone(),
        None => {
            let mut args = Map::new();
            if let Some(status) = str_field(obj, "status") {
                args.insert("status".to_owned(), Value::String(status.to_owned()));
            }
            if let Some(text) = str_field(obj, "text") {
                args.insert("text".to_owned(), Value::String(text.to_owned()));
            }
            Value::Object(args)
        }
    };
    Some(ToolCall {
        id,
        name,
        arguments,
    })
}

/// Codex stream-json: `{"type":"item.started","item":{"type":"command_execution"|"tool_use",...}}`.
fn parse_codex_item_started(obj: &Map<String, Value>) -> Option<ToolCall> {
    let item = obj_field(obj, "item")?;
    match str_field(item, "type")? {
        "command_execution" => {
            let id = str_field(item, "id").unwrap_or("").to_owned();
            let command = str_field(item, "command").unwrap_or("").to_owned();
            let mut args = Map::new();
            if !id.is_empty() {
                args.insert("id".to_owned(), Value::String(id.clone()));
            }
            if !command.is_empty() {
                args.insert("command".to_owned(), Value::String(command.clone()));
            }
            Some(ToolCall {
                id: if id.is_empty() { command } else { id },
                name: "command_execution".to_owned(),
                arguments: Value::Object(args),
            })
        }
        "tool_use" => {
            let name = str_field(item, "name").unwrap_or("tool").to_owned();
            let id = str_field(item, "id").unwrap_or("").to_owned();
            let arguments = item.get("input").cloned().unwrap_or_else(empty_args);
            Some(ToolCall {
                id,
                name,
                arguments,
            })
        }
        _ => None,
    }
}

/// Claude Code stream-json: assistant message with a `tool_use` content block.
fn parse_assistant_tool_call(obj: &Map<String, Value>) -> Option<ToolCall> {
    let message = obj_field(obj, "message")?;
    let content = message.get("content")?.as_array()?;
    for block in content {
        let Some(block) = block.as_object() else {
            continue;
        };
        let block_type = str_field(block, "type");
        if block_type != Some("tool_use") && block_type != Some("tool_call") {
            continue;
        }
        let name = str_field(block, "name").unwrap_or("tool").to_owned();
        let id = first_str(block, &["id", "tool_use_id"])
            .unwrap_or("")
            .to_owned();
        let arguments = block.get("input").cloned().unwrap_or_else(empty_args);
        return Some(ToolCall {
            id,
            name,
            arguments,
        });
    }
    None
}

/// OpenAI-style function call: `{"type":"tool_call","function":{"name":...,"arguments":"{...}"}}`.
fn parse_function_call(obj: &Map<String, Value>) -> Option<ToolCall> {
    let function = obj_field(obj, "function")?;
    let name = str_field(function, "name").unwrap_or("tool").to_owned();
    let id = first_str(obj, &["id", "call_id", "callId"])
        .unwrap_or("")
        .to_owned();
    let raw = function.get("arguments").cloned();
    let arguments = match raw {
        Some(Value::String(text)) => serde_json::from_str(&text).unwrap_or(Value::String(text)),
        Some(other) => other,
        None => empty_args(),
    };
    Some(ToolCall {
        id,
        name,
        arguments,
    })
}

/// Cursor local / Gemini top-level `tool_call` events.
fn parse_top_level_tool_call(obj: &Map<String, Value>) -> Option<ToolCall> {
    if let Some(call) = parse_function_call(obj) {
        return Some(call);
    }
    // Direct shape: {"type":"tool_call","name":...,"args"|"arguments"|"input":...}.
    if str_field(obj, "name").is_some() {
        let name = str_field(obj, "name").unwrap_or("tool").to_owned();
        let id = first_str(obj, &["call_id", "callId", "tool_call_id", "id"])
            .unwrap_or("")
            .to_owned();
        let arguments =
            first_value(obj, &["args", "arguments", "input"]).unwrap_or_else(empty_args);
        return Some(ToolCall {
            id,
            name,
            arguments,
        });
    }
    // Map shape: {"type":"tool_call","call_id":...,"tool_call":{"shell":{"args":...}}}.
    let tool_call = obj_field(obj, "tool_call").or_else(|| obj_field(obj, "toolCall"))?;
    let (tool_name, payload) = tool_call.iter().next()?;
    let id = first_str(obj, &["call_id", "callId", "id"])
        .unwrap_or("")
        .to_owned();
    let (name, arguments) = match payload.as_object() {
        Some(payload) => match payload.get("function").and_then(Value::as_object) {
            Some(function) => {
                let name = str_field(function, "name").unwrap_or(tool_name).to_owned();
                let raw = function.get("arguments").cloned();
                let arguments = raw
                    .map(|value| match value {
                        Value::String(text) => {
                            serde_json::from_str(&text).unwrap_or(Value::String(text))
                        }
                        other => other,
                    })
                    .unwrap_or_else(empty_args);
                (name, arguments)
            }
            None => (
                tool_name.clone(),
                first_value(payload, &["args", "arguments", "input"])
                    .unwrap_or_else(|| payload.clone().into()),
            ),
        },
        None => (tool_name.clone(), payload.clone()),
    };
    Some(ToolCall {
        id,
        name,
        arguments,
    })
}

/// OpenCode / Cursor stream-json: `{"type":"tool_use","part":{"tool":...,"callID":...,"input":...}}`.
fn parse_tool_use_event(obj: &Map<String, Value>) -> Option<ToolCall> {
    let part = obj_field(obj, "part").unwrap_or(obj);
    let name = str_field(part, "tool")
        .or_else(|| str_field(obj, "name"))
        .unwrap_or("tool")
        .to_owned();
    let id = first_str(part, &["callID", "id"])
        .or_else(|| first_str(obj, &["callID", "id"]))
        .unwrap_or("")
        .to_owned();
    let state_input = obj_field(part, "state")
        .and_then(|state| state.get("input"))
        .cloned();
    let arguments = first_value(part, &["input", "args"])
        .or(state_input)
        .unwrap_or_else(empty_args);
    Some(ToolCall {
        id,
        name,
        arguments,
    })
}

/// Pi local: `{"type":"tool_execution_start","toolCallId":...,"toolName":...,"args":...}`.
fn parse_tool_execution_start(obj: &Map<String, Value>) -> Option<ToolCall> {
    let name = str_field(obj, "toolName").unwrap_or("tool").to_owned();
    let id = str_field(obj, "toolCallId").unwrap_or("").to_owned();
    let arguments = obj.get("args").cloned().unwrap_or_else(empty_args);
    Some(ToolCall {
        id,
        name,
        arguments,
    })
}

/// Cursor Cloud SDK: `{"type":"cursor_cloud.message","message":{"type":"tool_call",...}}`.
fn parse_cursor_cloud_message(obj: &Map<String, Value>) -> Option<ToolCall> {
    let message = obj_field(obj, "message")?;
    if str_field(message, "type") != Some("tool_call") {
        return None;
    }
    let name = str_field(message, "name").unwrap_or("tool").to_owned();
    let id = first_str(message, &["call_id", "id"])
        .unwrap_or("")
        .to_owned();
    let arguments = first_value(message, &["args", "input"]).unwrap_or_else(empty_args);
    Some(ToolCall {
        id,
        name,
        arguments,
    })
}

/// Gemini v0.38+ stream-json: assistant `message` with a `tool_call` content block.
fn parse_gemini_message_tool_call(obj: &Map<String, Value>) -> Option<ToolCall> {
    if str_field(obj, "role") != Some("assistant") {
        return None;
    }
    let content = obj.get("content")?.as_array()?;
    for block in content {
        let Some(block) = block.as_object() else {
            continue;
        };
        if str_field(block, "type") != Some("tool_call") {
            continue;
        }
        let name = str_field(block, "name")
            .or_else(|| str_field(block, "tool"))
            .unwrap_or("tool")
            .to_owned();
        let id = first_str(block, &["tool_use_id", "toolUseId", "call_id", "id"])
            .unwrap_or("")
            .to_owned();
        let arguments =
            first_value(block, &["input", "args", "arguments"]).unwrap_or_else(empty_args);
        return Some(ToolCall {
            id,
            name,
            arguments,
        });
    }
    None
}

/// Hermes CLI quiet-mode line: `[done]? ┊ {emoji} {verb} {detail} {duration}`.
fn parse_hermes_line(trimmed: &str) -> Option<ToolCall> {
    // The `┊` prefix must be at the start of the line, after an optional
    // `[done]` marker. A `┊` appearing in the middle of ordinary prose is
    // not a Hermes tool line.
    let after_marker = trimmed
        .strip_prefix("[done]")
        .map(str::trim)
        .unwrap_or(trimmed);
    let rest = after_marker.strip_prefix('┊')?;
    let mut rest = rest.trim();
    // Assistant lines start with a speech bubble; `[tool]`/`[hermes]` lines
    // are start/system markers without a usable payload.
    if rest.starts_with('💬') || rest.starts_with("[tool]") || rest.starts_with("[hermes]") {
        return None;
    }
    // Strip leading kaomoji faces, emoji, and whitespace before the verb.
    loop {
        let before = rest.len();
        rest = rest.trim_start_matches(|c: char| !c.is_ascii() || c.is_whitespace());
        if let Some(after_open) = rest.strip_prefix('(')
            && let Some(close) = after_open.find(')')
        {
            rest = after_open[close + 1..].trim_start();
            continue;
        }
        if rest.len() == before {
            break;
        }
    }
    let (duration, verb_and_detail) = extract_duration(rest);
    if duration.is_empty() {
        return None;
    }
    let mut parts = verb_and_detail.splitn(2, char::is_whitespace);
    let verb = parts.next().unwrap_or("tool");
    let detail = parts.next().unwrap_or("").trim();
    // Only known Hermes tool verbs produce a structured call; ordinary prose
    // such as "it took 2.5s" after a `┊` must not be misclassified.
    let name = hermes_tool_name(verb)?;
    Some(ToolCall {
        id: String::new(),
        name: name.to_owned(),
        arguments: serde_json::json!({ "detail": detail, "duration": duration }),
    })
}

/// Maps a Hermes quiet-mode verb to a tool name (upstream `nameMap`).
fn hermes_tool_name(verb: &str) -> Option<&'static str> {
    // Upstream `packages/adapters/hermes/src/ui/parse-stdout.ts` resolves the
    // verb case-insensitively through its `nameMap`; mirror it exactly so the
    // Rust parser and the reference mirror agree on which lines are tools.
    match verb.to_ascii_lowercase().as_str() {
        "$" | "exec" | "terminal" => Some("shell"),
        "search" | "grep" | "find" | "web_search" | "search_files" => Some("search"),
        "fetch" | "web_extract" => Some("fetch"),
        "crawl" => Some("crawl"),
        "navigate" | "snapshot" | "click" | "type" | "scroll" | "back" | "press" | "close"
        | "images" | "vision" | "browser_navigate" | "browser_click" | "browser_type"
        | "browser_snapshot" | "browser_vision" | "browser_scroll" | "browser_press"
        | "browser_back" | "browser_close" | "browser_get_images" => Some("browser"),
        "read" | "read_file" => Some("read"),
        "write" => Some("write"),
        "write_file" => Some("write_file"),
        "patch" | "patch_file" => Some("patch"),
        "plan" => Some("plan"),
        "recall" | "session_search" => Some("recall"),
        "proc" => Some("process"),
        "delegate" => Some("delegate"),
        "todo" => Some("todo"),
        "memory" => Some("memory"),
        "clarify" => Some("clarify"),
        "code" | "execute" | "execute_code" => Some("execute"),
        _ => None,
    }
}

/// Extracts a trailing `0.2s` (optionally followed by `(0.5s)`) duration.
fn extract_duration(text: &str) -> (String, &str) {
    let trimmed = text.trim_end();
    let mut end = trimmed.len();
    if let Some(open) = trimmed.rfind('(') {
        let tail = &trimmed[open + 1..trimmed.len() - 1];
        if is_duration(tail) {
            end = open;
        }
    }
    let head = trimmed[..end].trim_end();
    if let Some((head2, tail)) = head.rsplit_once(' ')
        && is_duration(tail)
    {
        return (tail.to_owned(), head2);
    }
    (String::new(), trimmed)
}

fn is_duration(text: &str) -> bool {
    text.strip_suffix('s')
        .and_then(|number| number.parse::<f64>().ok())
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(line: &str) -> ToolCall {
        parse_tool_call_line(line).unwrap_or_else(|| panic!("expected tool call in: {line}"))
    }

    fn none(line: &str) {
        assert_eq!(
            parse_tool_call_line(line),
            None,
            "expected no tool call for: {line}"
        );
    }

    #[test]
    fn codex_command_execution_item_started() {
        let c = call(
            r#"{"type":"item.started","item":{"type":"command_execution","id":"cmd-1","command":"cargo test"}}"#,
        );
        assert_eq!(c.id, "cmd-1");
        assert_eq!(c.name, "command_execution");
        assert_eq!(c.arguments["command"], "cargo test");
        assert_eq!(c.arguments["id"], "cmd-1");
    }

    #[test]
    fn codex_command_execution_without_id_uses_command() {
        let c =
            call(r#"{"type":"item.started","item":{"type":"command_execution","command":"ls"}}"#);
        assert_eq!(c.id, "ls");
        assert_eq!(c.name, "command_execution");
    }

    #[test]
    fn codex_tool_use_item_started() {
        let c = call(
            r#"{"type":"item.started","item":{"type":"tool_use","id":"tu-1","name":"read_file","input":{"path":"README.md"}}}"#,
        );
        assert_eq!(c.id, "tu-1");
        assert_eq!(c.name, "read_file");
        assert_eq!(c.arguments["path"], "README.md");
    }

    #[test]
    fn codex_item_completed_is_not_a_tool_call() {
        none(
            r#"{"type":"item.completed","item":{"type":"command_execution","id":"cmd-1","command":"ls","status":"completed"}}"#,
        );
    }

    #[test]
    fn claude_assistant_tool_use_block() {
        let c = call(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"thinking"},{"type":"tool_use","id":"toolu_1","name":"Bash","input":{"command":"git status"}}]}}"#,
        );
        assert_eq!(c.id, "toolu_1");
        assert_eq!(c.name, "Bash");
        assert_eq!(c.arguments["command"], "git status");
    }

    #[test]
    fn acpx_tool_call_with_input() {
        let c = call(
            r#"{"type":"acpx.tool_call","name":"shell","toolCallId":"acp-1","input":{"command":"ls"},"status":"running"}"#,
        );
        assert_eq!(c.id, "acp-1");
        assert_eq!(c.name, "shell");
        assert_eq!(c.arguments["command"], "ls");
        assert_eq!(c.arguments["status"], "running");
    }

    #[test]
    fn acpx_tool_call_without_input_falls_back_to_text_and_status() {
        let c = call(
            r#"{"type":"acpx.tool_call","name":"shell","toolUseId":"acp-2","text":"running npm test"}"#,
        );
        assert_eq!(c.id, "acp-2");
        assert_eq!(c.name, "shell");
        assert_eq!(c.arguments["text"], "running npm test");
    }

    #[test]
    fn cursor_cloud_message_tool_call() {
        let c = call(
            r#"{"type":"cursor_cloud.message","message":{"type":"tool_call","call_id":"cc-1","name":"shellToolCall","args":{"command":"npm run build"},"status":"running"}}"#,
        );
        assert_eq!(c.id, "cc-1");
        assert_eq!(c.name, "shellToolCall");
        assert_eq!(c.arguments["command"], "npm run build");
    }

    #[test]
    fn cursor_local_tool_call_map_shape() {
        let c = call(
            r#"{"type":"tool_call","subtype":"started","call_id":"cl-1","tool_call":{"shell":{"args":{"command":"git log --oneline"}}}}"#,
        );
        assert_eq!(c.id, "cl-1");
        assert_eq!(c.name, "shell");
        assert_eq!(c.arguments["command"], "git log --oneline");
    }

    #[test]
    fn cursor_local_assistant_tool_call_block() {
        let c = call(
            r#"{"type":"assistant","message":{"content":[{"type":"tool_call","name":"shellToolCall","tool_use_id":"cl-2","input":{"command":"pwd"}}]}}"#,
        );
        assert_eq!(c.id, "cl-2");
        assert_eq!(c.name, "shellToolCall");
        assert_eq!(c.arguments["command"], "pwd");
    }

    #[test]
    fn gemini_tool_call_map_with_function_arguments() {
        let c = call(
            r#"{"type":"tool_call","call_id":"gm-1","tool_call":{"search":{"function":{"name":"search","arguments":"{\"query\":\"paperclip\"}"}}}}"#,
        );
        assert_eq!(c.id, "gm-1");
        assert_eq!(c.name, "search");
        assert_eq!(c.arguments["query"], "paperclip");
    }

    #[test]
    fn gemini_message_tool_call_block() {
        let c = call(
            r#"{"type":"message","role":"assistant","content":[{"type":"tool_call","name":"search","toolUseId":"gm-2","input":{"query":"rust"}}]}"#,
        );
        assert_eq!(c.id, "gm-2");
        assert_eq!(c.name, "search");
        assert_eq!(c.arguments["query"], "rust");
    }

    #[test]
    fn opencode_tool_use_with_part() {
        let c = call(
            r#"{"type":"tool_use","part":{"tool":"bash","callID":"oc-1","input":{"command":"cargo fmt --check"},"state":{"status":"completed"}}}"#,
        );
        assert_eq!(c.id, "oc-1");
        assert_eq!(c.name, "bash");
        assert_eq!(c.arguments["command"], "cargo fmt --check");
    }

    #[test]
    fn pi_tool_execution_start() {
        let c = call(
            r#"{"type":"tool_execution_start","toolCallId":"pi-1","toolName":"shell","args":{"command":"echo hi"}}"#,
        );
        assert_eq!(c.id, "pi-1");
        assert_eq!(c.name, "shell");
        assert_eq!(c.arguments["command"], "echo hi");
    }

    #[test]
    fn openai_function_style() {
        let c = call(
            r#"{"type":"tool_call","id":"fn-1","function":{"name":"read_file","arguments":"{\"path\":\"Cargo.toml\"}"}}"#,
        );
        assert_eq!(c.id, "fn-1");
        assert_eq!(c.name, "read_file");
        assert_eq!(c.arguments["path"], "Cargo.toml");
    }

    #[test]
    fn hermes_prefix_marker_line() {
        let c = call("  ┊ 💻 $         curl -s https://example.com  0.2s");
        assert_eq!(c.name, "shell");
        assert_eq!(c.arguments["detail"], "curl -s https://example.com");
        assert_eq!(c.arguments["duration"], "0.2s");
    }

    #[test]
    fn hermes_pipe_marker_line_with_total() {
        let c = call("  [done] ┊ 🔍 search    pattern  0.1s (0.5s)");
        assert_eq!(c.name, "search");
        assert_eq!(c.arguments["detail"], "pattern");
        assert_eq!(c.arguments["duration"], "0.1s");
    }

    #[test]
    fn hermes_kaomoji_and_emoji_are_stripped() {
        let c = call("  ┊ (｡◕‿◕｡) 💻 exec    make test  1.2s");
        assert_eq!(c.name, "shell");
        assert_eq!(c.arguments["detail"], "make test");
        assert_eq!(c.arguments["duration"], "1.2s");
    }

    #[test]
    fn hermes_assistant_line_is_not_a_tool_call() {
        none("  ┊ 💬 Hello! How can I help?");
        none("  ┊ 💬 took 2.5s to answer");
    }

    #[test]
    fn hermes_prefix_in_middle_of_prose_is_not_a_tool_call() {
        none("I waited ┊ then it took 2.5s");
        none("progress: ┊ 0.5s remaining");
    }

    #[test]
    fn hermes_unknown_verb_is_not_a_tool_call() {
        none("  ┊ thinking about the plan  2.5s");
        none("  ┊ done with the task  1.0s");
        none("  [done] ┊ noting the result  0.3s");
    }

    #[test]
    fn strips_ansi_before_parsing() {
        let c = call(
            "\u{1b}[32m{\"type\":\"acpx.tool_call\",\"name\":\"shell\",\"id\":\"a1\",\"input\":{\"command\":\"ls\"}}\u{1b}[0m",
        );
        assert_eq!(c.id, "a1");
        assert_eq!(c.name, "shell");
        assert_eq!(c.arguments["command"], "ls");
    }

    #[test]
    fn plain_text_degrades_to_none() {
        none("just some assistant text");
        none("   ");
        none("");
    }

    #[test]
    fn unknown_json_degrades_to_none() {
        none(r#"{"type":"thought","data":"let me think"}"#);
        none(r#"{"type":"acpx.text_delta","text":"hi"}"#);
        none(r#"{"type":"grok.text","data":"hi"}"#);
    }

    #[test]
    fn openclaw_gateway_events_degrade_to_none() {
        none(r#"[openclaw-gateway:event] run=r1 stream=assistant data={"delta":"hi"}"#);
        none(r#"[openclaw-gateway:event] run=r1 stream=lifecycle data={"phase":"completed"}"#);
    }

    #[test]
    fn ansi_wrapped_non_json_text_degrades_to_none() {
        none("\u{1b}[31mplain assistant text\u{1b}[0m");
        none(
            "\u{1b}[36m[openclaw-gateway:event] run=r1 stream=assistant data={\"delta\":\"hi\"}\u{1b}[0m",
        );
    }

    #[test]
    fn strips_common_escape_sequences_before_parsing() {
        // ESC 7 / ESC 8 / ESC ( B are cursor/charset sequences, not CSI/OSC.
        let c = call(
            "\u{1b}7\u{1b}8\u{1b}(B{\"type\":\"acpx.tool_call\",\"name\":\"shell\",\"id\":\"a2\",\"input\":{\"command\":\"ls\"}}",
        );
        assert_eq!(c.id, "a2");
        assert_eq!(c.name, "shell");
        assert_eq!(c.arguments["command"], "ls");
    }

    #[test]
    fn xml_tool_tags_degrade_to_none() {
        none("<tool_call><tool_name>shell</tool_name></tool_call>");
        none("<invoke name=\"shell\"><parameter name=\"command\">ls</parameter></invoke>");
    }

    #[test]
    fn multi_line_json_pretty_printed_degrades_to_none() {
        none("{");
        none(r#"  "type": "tool_call","#);
    }

    #[test]
    fn hermes_name_map_covers_upstream_verbs() {
        // Every verb in the reference mirror's Hermes nameMap must resolve,
        // case-insensitively, to the same tool name the upstream does.
        let cases = [
            ("browser_scroll", "browser"),
            ("browser_press", "browser"),
            ("browser_back", "browser"),
            ("browser_close", "browser"),
            ("browser_get_images", "browser"),
            ("read_file", "read"),
            ("write_file", "write_file"),
            ("search_files", "search"),
            ("patch_file", "patch"),
            ("execute_code", "execute"),
            ("READ_FILE", "read"),
            ("$", "shell"),
            ("Web_Search", "search"),
        ];
        for (verb, expected) in cases {
            assert_eq!(hermes_tool_name(verb), Some(expected), "verb {verb}");
        }
        // Non-tool verbs stay non-tools even when they look like lines.
        assert_eq!(hermes_tool_name("thinking"), None);
        assert_eq!(hermes_tool_name("done"), None);
        assert_eq!(hermes_tool_name("bash"), None);
    }
}
