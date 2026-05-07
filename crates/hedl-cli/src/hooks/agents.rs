// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! AI agent protocol handlers.
//!
//! Supports Claude Code, Cursor, Copilot (VS Code + CLI), Gemini,
//! OpenCode, Cline, and Codex.
//!
//! Based on actual research of agent hook APIs:
//! - OpenCode: Plugin system with lifecycle hooks (opencode.ai/docs/plugins)
//! - Cline: 8 hook types including PreToolUse, PostToolUse, TaskStart, etc. (docs.cline.bot/customization/hooks)
//! - Codex: PreToolUse and PostToolUse hooks (developers.openai.com/codex/hooks)

use super::{read_stdin_limited, write_json};
use serde_json::{json, Value};
use std::io::Write;

// ── Claude Code ───────────────────────────────────────────────

/// Run the Claude Code PreToolUse hook.
pub fn run_claude() -> Result<(), String> {
    let input = read_stdin_limited()?;
    let input = input.trim();
    if input.is_empty() {
        return Ok(());
    }

    let v: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => {
            let _ = std::io::stderr().write_all(format!("[hedl hook] Failed to parse JSON: {}\n", e).as_bytes());
            return Ok(());
        }
    };

    match process_claude_payload(&v) {
        PayloadAction::Rewrite { output, .. } => {
            let _ = write_json(&output);
        }
        PayloadAction::Skip { .. } => {}
        PayloadAction::Ignore => {}
    }
    Ok(())
}

enum PayloadAction {
    Rewrite {
        cmd: String,
        rewritten: String,
        output: Value,
    },
    Skip {
        reason: &'static str,
        cmd: String,
    },
    Ignore,
}

fn process_claude_payload(v: &Value) -> PayloadAction {
    let cmd = match v
        .pointer("/tool_input/command")
        .and_then(|c| c.as_str())
        .filter(|c| !c.is_empty())
    {
        Some(c) => c,
        None => return PayloadAction::Ignore,
    };

    let rewritten = match super::rewrite::get_rewritten(cmd) {
        Some(r) => r,
        None => {
            return PayloadAction::Skip {
                reason: "skip:no_match",
                cmd: cmd.to_string(),
            }
        }
    };

    let updated_input = {
        let mut ti = v.get("tool_input").cloned().unwrap_or_else(|| json!({}));
        if let Some(obj) = ti.as_object_mut() {
            obj.insert("command".into(), Value::String(rewritten.clone()));
        }
        ti
    };

    let hook_output = json!({
        "hookEventName": "PreToolUse",
        "permissionDecisionReason": "HEDL auto-rewrite",
        "updatedInput": updated_input
    });

    PayloadAction::Rewrite {
        cmd: cmd.to_string(),
        rewritten,
        output: json!({ "hookSpecificOutput": hook_output }),
    }
}

// ── Cursor ────────────────────────────────────────────────────

/// Run the Cursor Agent hook.
pub fn run_cursor() -> Result<(), String> {
    let input = read_stdin_limited()?;
    let input = input.trim();
    if input.is_empty() {
        let _ = writeln!(std::io::stdout(), "{{}}");
        return Ok(());
    }

    let v: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(_) => {
            let _ = writeln!(std::io::stdout(), "{{}}");
            return Ok(());
        }
    };

    let cmd = match v
        .pointer("/tool_input/command")
        .and_then(|c| c.as_str())
        .filter(|c| !c.is_empty())
    {
        Some(c) => c.to_string(),
        None => {
            let _ = writeln!(std::io::stdout(), "{{}}");
            return Ok(());
        }
    };

    let rewritten = match super::rewrite::get_rewritten(&cmd) {
        Some(r) => r,
        None => {
            let _ = writeln!(std::io::stdout(), "{{}}");
            return Ok(());
        }
    };

    let output = json!({
        "permission": "allow",
        "updated_input": { "command": rewritten }
    });
    let _ = writeln!(std::io::stdout(), "{}", output);
    Ok(())
}

// ── Copilot ───────────────────────────────────────────────────

enum CopilotFormat {
    VsCode { command: String },
    CopilotCli { command: String },
    PassThrough,
}

/// Run the Copilot preToolUse hook.
pub fn run_copilot() -> Result<(), String> {
    let input = read_stdin_limited()?;
    let input = input.trim();
    if input.is_empty() {
        return Ok(());
    }

    let v: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => {
            let _ = std::io::stderr().write_all(format!("[hedl hook] JSON parse error: {}\n", e).as_bytes());
            return Ok(());
        }
    };

    match detect_copilot_format(&v) {
        CopilotFormat::VsCode { command } => handle_copilot_vscode(&command),
        CopilotFormat::CopilotCli { command } => handle_copilot_cli(&command),
        CopilotFormat::PassThrough => Ok(()),
    }
}

fn detect_copilot_format(v: &Value) -> CopilotFormat {
    // VS Code Copilot Chat: snake_case keys
    if let Some(tool_name) = v.get("tool_name").and_then(|t| t.as_str()) {
        if matches!(tool_name, "runTerminalCommand" | "Bash" | "bash") {
            if let Some(cmd) = v
                .pointer("/tool_input/command")
                .and_then(|c| c.as_str())
                .filter(|c| !c.is_empty())
            {
                return CopilotFormat::VsCode {
                    command: cmd.to_string(),
                };
            }
        }
        return CopilotFormat::PassThrough;
    }

    // Copilot CLI: camelCase keys, toolArgs is a JSON-encoded string
    if let Some(tool_name) = v.get("toolName").and_then(|t| t.as_str()) {
        if tool_name == "bash" {
            if let Some(tool_args_str) = v.get("toolArgs").and_then(|t| t.as_str()) {
                if let Ok(tool_args) = serde_json::from_str::<Value>(tool_args_str) {
                    if let Some(cmd) = tool_args
                        .get("command")
                        .and_then(|c| c.as_str())
                        .filter(|c| !c.is_empty())
                    {
                        return CopilotFormat::CopilotCli {
                            command: cmd.to_string(),
                        };
                    }
                }
            }
        }
        return CopilotFormat::PassThrough;
    }

    CopilotFormat::PassThrough
}

fn handle_copilot_vscode(cmd: &str) -> Result<(), String> {
    let rewritten = match super::rewrite::get_rewritten(cmd) {
        Some(r) => r,
        None => return Ok(()),
    };

    let output = json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "permissionDecisionReason": "HEDL auto-rewrite",
            "updatedInput": { "command": rewritten }
        }
    });
    write_json(&output)
}

fn handle_copilot_cli(cmd: &str) -> Result<(), String> {
    let rewritten = match super::rewrite::get_rewritten(cmd) {
        Some(r) => r,
        None => return Ok(()),
    };

    let output = json!({
        "permissionDecision": "deny",
        "permissionDecisionReason": format!(
            "Token savings: use `{}` instead (HEDL saves 60-90% tokens)",
            rewritten
        )
    });
    write_json(&output)
}

// ── Gemini ────────────────────────────────────────────────────

/// Run the Gemini CLI BeforeTool hook.
pub fn run_gemini() -> Result<(), String> {
    let input = read_stdin_limited()?;
    let json: Value = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(_) => {
            print_allow();
            return Ok(());
        }
    };

    let tool_name = json.get("tool_name").and_then(|v| v.as_str()).unwrap_or("");
    if tool_name != "run_shell_command" {
        print_allow();
        return Ok(());
    }

    let cmd = json
        .pointer("/tool_input/command")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if cmd.is_empty() {
        print_allow();
        return Ok(());
    }

    match super::rewrite::get_rewritten(cmd) {
        Some(ref rewritten) => {
            let output = json!({
                "decision": "allow",
                "hookSpecificOutput": {
                    "tool_input": {
                        "command": rewritten
                    }
                }
            });
            write_json(&output)
        }
        None => {
            print_allow();
            Ok(())
        }
    }
}

fn print_allow() {
    let _ = writeln!(std::io::stdout(), r#"{{"decision":"allow"}}"#);
}

// ── OpenCode ──────────────────────────────────────────────────

/// Run the OpenCode plugin hook.
///
/// OpenCode plugins receive a context object and return hooks.
/// The hook system intercepts tool execution via PreToolUse.
pub fn run_opencode() -> Result<(), String> {
    let input = read_stdin_limited()?;
    let input = input.trim();
    if input.is_empty() {
        return Ok(());
    }

    let v: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => {
            let _ = std::io::stderr().write_all(format!("[hedl hook] JSON parse error: {}\n", e).as_bytes());
            return Ok(());
        }
    };

    // OpenCode plugin input structure:
    // { "client": {...}, "project": {...}, "directory": "...", "tool": "...", "args": {...} }
    let cmd = match v
        .pointer("/args/command")
        .and_then(|c| c.as_str())
        .filter(|c| !c.is_empty())
    {
        Some(c) => c,
        None => return Ok(()),
    };

    let rewritten = match super::rewrite::get_rewritten(cmd) {
        Some(r) => r,
        None => return Ok(()),
    };

    // OpenCode expects hooks to be returned as an object with event handlers
    let output = json!({
        "hooks": {
            "pretooluse": {
                "action": "rewrite",
                "args": {
                    "command": rewritten
                },
                "reason": "HEDL auto-rewrite"
            }
        }
    });
    write_json(&output)
}

// ── Cline ─────────────────────────────────────────────────────

/// Run the Cline hook.
///
/// Cline supports 8 hook types:
/// - TaskStart, TaskResume, TaskCancel, TaskComplete
/// - PreToolUse, PostToolUse
/// - UserPromptSubmit, PreCompact
///
/// Hooks are scripts that receive JSON on stdin and output JSON on stdout.
pub fn run_cline() -> Result<(), String> {
    let input = read_stdin_limited()?;
    let input = input.trim();
    if input.is_empty() {
        return Ok(());
    }

    let v: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => {
            let _ = std::io::stderr().write_all(format!("[hedl hook] JSON parse error: {}\n", e).as_bytes());
            return Ok(());
        }
    };

    // Cline input structure:
    // { "taskId": "...", "hookName": "PreToolUse", "toolName": "...", "toolInput": {...} }
    let hook_name = v.get("hookName").and_then(|h| h.as_str()).unwrap_or("");
    
    // Only process PreToolUse for command rewriting
    if hook_name != "PreToolUse" {
        // For other hooks, just pass through
        let _ = writeln!(std::io::stdout(), "{{}}");
        return Ok(());
    }

    let cmd = match v
        .pointer("/toolInput/command")
        .and_then(|c| c.as_str())
        .filter(|c| !c.is_empty())
    {
        Some(c) => c,
        None => {
            let _ = writeln!(std::io::stdout(), "{{}}");
            return Ok(());
        }
    };

    let rewritten = match super::rewrite::get_rewritten(cmd) {
        Some(r) => r,
        None => {
            let _ = writeln!(std::io::stdout(), "{{}}");
            return Ok(());
        }
    };

    // Cline expects output with optional context modifications
    let output = json!({
        "context": [{
            "type": "text",
            "content": format!("Command auto-rewritten by HEDL: {}", rewritten)
        }],
        "toolInput": {
            "command": rewritten
        }
    });
    write_json(&output)
}

// ── Codex ─────────────────────────────────────────────────────

/// Run the Codex hook.
///
/// Codex supports PreToolUse and PostToolUse hooks.
/// See: https://developers.openai.com/codex/hooks
pub fn run_codex() -> Result<(), String> {
    let input = read_stdin_limited()?;
    let input = input.trim();
    if input.is_empty() {
        return Ok(());
    }

    let v: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => {
            let _ = std::io::stderr().write_all(format!("[hedl hook] JSON parse error: {}\n", e).as_bytes());
            return Ok(());
        }
    };

    // Codex input structure:
    // { "event": "PreToolUse", "tool": "bash", "input": { "command": "..." } }
    let event = v.get("event").and_then(|e| e.as_str()).unwrap_or("");
    
    if event != "PreToolUse" {
        return Ok(());
    }

    let cmd = match v
        .pointer("/input/command")
        .and_then(|c| c.as_str())
        .filter(|c| !c.is_empty())
    {
        Some(c) => c,
        None => return Ok(()),
    };

    let rewritten = match super::rewrite::get_rewritten(cmd) {
        Some(r) => r,
        None => return Ok(()),
    };

    // Codex expects output with updated input
    let output = json!({
        "input": {
            "command": rewritten
        },
        "reason": "HEDL auto-rewrite"
    });
    write_json(&output)
}
