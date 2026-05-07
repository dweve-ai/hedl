// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Hook installation and configuration for AI agents.

use std::path::PathBuf;

/// Install hooks for the specified agent.
pub fn install(agent: &str) -> Result<(), String> {
    match agent {
        "claude" => install_claude(),
        "cursor" => install_cursor(),
        "copilot" => install_copilot(),
        "gemini" => install_gemini(),
        "opencode" => install_opencode(),
        "cline" => install_cline(),
        "codex" => install_codex(),
        "all" => {
            install_claude()?;
            install_cursor()?;
            install_copilot()?;
            install_gemini()?;
            install_opencode()?;
            install_cline()?;
            install_codex()?;
            Ok(())
        }
        _ => Err(format!(
            "Unknown agent: {}. Supported: claude, cursor, copilot, gemini, opencode, cline, codex, all",
            agent
        )),
    }
}

fn install_claude() -> Result<(), String> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    let claude_dir = config_dir.join("claude");
    std::fs::create_dir_all(&claude_dir).map_err(|e| format!("Failed to create Claude dir: {}", e))?;

    let settings_path = claude_dir.join("settings.json");
    let mut settings = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings.json: {}", e))?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Add or update hooks
    let hooks = settings
        .as_object_mut()
        .unwrap()
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let pre_tool = hooks
        .as_object_mut()
        .unwrap()
        .entry("preToolUse")
        .or_insert_with(|| serde_json::json!([]));

    let hook_entry = serde_json::json!({
        "command": "hedl",
        "args": ["hook", "claude"],
        "pattern": "^bash$"
    });

    let hooks_array = pre_tool.as_array_mut().unwrap();
    // Remove existing hedl hooks
    hooks_array.retain(|h| {
        h.get("command")
            .and_then(|c| c.as_str())
            .map(|c| c != "hedl")
            .unwrap_or(true)
    });
    hooks_array.push(hook_entry);

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;

    println!("✓ Claude Code hooks installed");
    println!("  Config: {}", settings_path.display());
    Ok(())
}

fn install_cursor() -> Result<(), String> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    let cursor_dir = config_dir.join("cursor");
    std::fs::create_dir_all(&cursor_dir).map_err(|e| format!("Failed to create Cursor dir: {}", e))?;

    let hooks_path = cursor_dir.join("hooks.json");
    let mut hooks = if hooks_path.exists() {
        let content = std::fs::read_to_string(&hooks_path)
            .map_err(|e| format!("Failed to read hooks.json: {}", e))?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let pre_tool = hooks
        .as_object_mut()
        .unwrap()
        .entry("preToolUse")
        .or_insert_with(|| serde_json::json!([]));

    let hook_entry = serde_json::json!({
        "command": "hedl",
        "args": ["hook", "cursor"],
        "pattern": "^bash$"
    });

    let hooks_array = pre_tool.as_array_mut().unwrap();
    hooks_array.retain(|h| {
        h.get("command")
            .and_then(|c| c.as_str())
            .map(|c| c != "hedl")
            .unwrap_or(true)
    });
    hooks_array.push(hook_entry);

    let content = serde_json::to_string_pretty(&hooks)
        .map_err(|e| format!("Failed to serialize hooks: {}", e))?;
    std::fs::write(&hooks_path, content)
        .map_err(|e| format!("Failed to write hooks.json: {}", e))?;

    println!("✓ Cursor hooks installed");
    println!("  Config: {}", hooks_path.display());
    Ok(())
}

fn install_copilot() -> Result<(), String> {
    println!("✓ Copilot hooks");
    println!("  Add to VS Code settings.json:");
    println!();
    println!("  \"github.copilot.chat.codeGeneration.instructions\": [");
    println!("    {{");
    println!("      \"text\": \"Use 'hedl run -- <command>' for all shell commands to optimize token usage\"");
    println!("    }}");
    println!("  ]");
    println!();
    Ok(())
}

fn install_gemini() -> Result<(), String> {
    println!("✓ Gemini hooks");
    println!("  Add to your Gemini CLI configuration:");
    println!();
    println!("  export GEMINI_HOOKS=~/.config/gemini/hooks.json");
    println!();
    Ok(())
}

fn install_opencode() -> Result<(), String> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    let opencode_dir = config_dir.join("opencode");
    std::fs::create_dir_all(&opencode_dir).map_err(|e| format!("Failed to create OpenCode dir: {}", e))?;

    let plugins_path = opencode_dir.join("plugins.json");
    let mut plugins = if plugins_path.exists() {
        let content = std::fs::read_to_string(&plugins_path)
            .map_err(|e| format!("Failed to read plugins.json: {}", e))?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let plugin_list = plugins
        .as_object_mut()
        .unwrap()
        .entry("plugins")
        .or_insert_with(|| serde_json::json!([]));

    let plugin_entry = serde_json::json!({
        "name": "hedl-hook",
        "command": "hedl",
        "args": ["hook", "run", "opencode"]
    });

    let plugins_array = plugin_list.as_array_mut().unwrap();
    plugins_array.retain(|p| {
        p.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n != "hedl-hook")
            .unwrap_or(true)
    });
    plugins_array.push(plugin_entry);

    let content = serde_json::to_string_pretty(&plugins)
        .map_err(|e| format!("Failed to serialize plugins: {}", e))?;
    std::fs::write(&plugins_path, content)
        .map_err(|e| format!("Failed to write plugins.json: {}", e))?;

    println!("✓ OpenCode hooks installed");
    println!("  Config: {}", plugins_path.display());
    Ok(())
}

fn install_cline() -> Result<(), String> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    
    // Cline hooks are stored in ~/Documents/Cline/Hooks (macOS) or ~/.config/Cline/Hooks (Linux)
    #[cfg(target_os = "macos")]
    let cline_hooks_dir = dirs::document_dir()
        .ok_or("Could not find documents directory")?
        .join("Cline")
        .join("Hooks");
    
    #[cfg(not(target_os = "macos"))]
    let cline_hooks_dir = config_dir.join("Cline").join("Hooks");

    std::fs::create_dir_all(&cline_hooks_dir).map_err(|e| format!("Failed to create Cline hooks dir: {}", e))?;

    // Create the PreToolUse hook script
    let hook_script = r#"#!/bin/bash
# HEDL PreToolUse hook for Cline
# Automatically rewrites commands for optimal token usage

read -r input
if [ -z "$input" ]; then
    echo "{}"
    exit 0
fi

echo "$input" | hedl hook run cline
"#;

    let hook_path = cline_hooks_dir.join("pretooluse-hedl");
    std::fs::write(&hook_path, hook_script)
        .map_err(|e| format!("Failed to write hook script: {}", e))?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_path, perms).unwrap();
    }

    println!("✓ Cline hooks installed");
    println!("  Hooks dir: {}", cline_hooks_dir.display());
    println!("  PreToolUse hook: {}", hook_path.display());
    Ok(())
}

fn install_codex() -> Result<(), String> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    let codex_dir = config_dir.join("codex");
    std::fs::create_dir_all(&codex_dir).map_err(|e| format!("Failed to create Codex dir: {}", e))?;

    let hooks_path = codex_dir.join("hooks.json");
    let mut hooks = if hooks_path.exists() {
        let content = std::fs::read_to_string(&hooks_path)
            .map_err(|e| format!("Failed to read hooks.json: {}", e))?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let pre_tool = hooks
        .as_object_mut()
        .unwrap()
        .entry("preToolUse")
        .or_insert_with(|| serde_json::json!([]));

    let hook_entry = serde_json::json!({
        "command": "hedl",
        "args": ["hook", "run", "codex"],
        "pattern": "bash"
    });

    let hooks_array = pre_tool.as_array_mut().unwrap();
    hooks_array.retain(|h| {
        h.get("command")
            .and_then(|c| c.as_str())
            .map(|c| c != "hedl")
            .unwrap_or(true)
    });
    hooks_array.push(hook_entry);

    let content = serde_json::to_string_pretty(&hooks)
        .map_err(|e| format!("Failed to serialize hooks: {}", e))?;
    std::fs::write(&hooks_path, content)
        .map_err(|e| format!("Failed to write hooks.json: {}", e))?;

    println!("✓ Codex hooks installed");
    println!("  Config: {}", hooks_path.display());
    Ok(())
}

/// Uninstall hooks for the specified agent.
pub fn uninstall(agent: &str) -> Result<(), String> {
    match agent {
        "claude" => uninstall_claude(),
        "cursor" => uninstall_cursor(),
        "copilot" => {
            println!("✓ Copilot hooks removed (manual configuration)");
            Ok(())
        }
        "gemini" => {
            println!("✓ Gemini hooks removed (manual configuration)");
            Ok(())
        }
        "opencode" => uninstall_opencode(),
        "cline" => uninstall_cline(),
        "codex" => uninstall_codex(),
        "all" => {
            uninstall_claude()?;
            uninstall_cursor()?;
            uninstall_opencode()?;
            uninstall_cline()?;
            uninstall_codex()?;
            println!("✓ All hooks uninstalled");
            Ok(())
        }
        _ => Err(format!("Unknown agent: {}", agent)),
    }
}

fn uninstall_claude() -> Result<(), String> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    let settings_path = config_dir.join("claude").join("settings.json");

    if !settings_path.exists() {
        println!("✓ Claude hooks not installed");
        return Ok(());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;
    let mut settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))?;

    if let Some(hooks) = settings.get_mut("hooks") {
        if let Some(pre_tool) = hooks.get_mut("preToolUse") {
            if let Some(array) = pre_tool.as_array_mut() {
                array.retain(|h| {
                    h.get("command")
                        .and_then(|c| c.as_str())
                        .map(|c| c != "hedl")
                        .unwrap_or(true)
                });
            }
        }
    }

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;

    println!("✓ Claude hooks uninstalled");
    Ok(())
}

fn uninstall_cursor() -> Result<(), String> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    let hooks_path = config_dir.join("cursor").join("hooks.json");

    if !hooks_path.exists() {
        println!("✓ Cursor hooks not installed");
        return Ok(());
    }

    let content = std::fs::read_to_string(&hooks_path)
        .map_err(|e| format!("Failed to read hooks.json: {}", e))?;
    let mut hooks: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse hooks.json: {}", e))?;

    if let Some(pre_tool) = hooks.get_mut("preToolUse") {
        if let Some(array) = pre_tool.as_array_mut() {
            array.retain(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .map(|c| c != "hedl")
                    .unwrap_or(true)
            });
        }
    }

    let content = serde_json::to_string_pretty(&hooks)
        .map_err(|e| format!("Failed to serialize hooks: {}", e))?;
    std::fs::write(&hooks_path, content)
        .map_err(|e| format!("Failed to write hooks.json: {}", e))?;

    println!("✓ Cursor hooks uninstalled");
    Ok(())
}

fn uninstall_opencode() -> Result<(), String> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    let plugins_path = config_dir.join("opencode").join("plugins.json");

    if !plugins_path.exists() {
        println!("✓ OpenCode hooks not installed");
        return Ok(());
    }

    let content = std::fs::read_to_string(&plugins_path)
        .map_err(|e| format!("Failed to read plugins.json: {}", e))?;
    let mut plugins: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse plugins.json: {}", e))?;

    if let Some(plugin_list) = plugins.get_mut("plugins") {
        if let Some(array) = plugin_list.as_array_mut() {
            array.retain(|p| {
                p.get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n != "hedl-hook")
                    .unwrap_or(true)
            });
        }
    }

    let content = serde_json::to_string_pretty(&plugins)
        .map_err(|e| format!("Failed to serialize plugins: {}", e))?;
    std::fs::write(&plugins_path, content)
        .map_err(|e| format!("Failed to write plugins.json: {}", e))?;

    println!("✓ OpenCode hooks uninstalled");
    Ok(())
}

fn uninstall_cline() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let cline_hooks_dir = dirs::document_dir()
        .ok_or("Could not find documents directory")?
        .join("Cline")
        .join("Hooks");
    
    #[cfg(not(target_os = "macos"))]
    let cline_hooks_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("Cline")
        .join("Hooks");

    let hook_path = cline_hooks_dir.join("pretooluse-hedl");

    if hook_path.exists() {
        std::fs::remove_file(&hook_path)
            .map_err(|e| format!("Failed to remove hook: {}", e))?;
        println!("✓ Cline hooks uninstalled");
    } else {
        println!("✓ Cline hooks not installed");
    }
    Ok(())
}

fn uninstall_codex() -> Result<(), String> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    let hooks_path = config_dir.join("codex").join("hooks.json");

    if !hooks_path.exists() {
        println!("✓ Codex hooks not installed");
        return Ok(());
    }

    let content = std::fs::read_to_string(&hooks_path)
        .map_err(|e| format!("Failed to read hooks.json: {}", e))?;
    let mut hooks: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse hooks.json: {}", e))?;

    if let Some(pre_tool) = hooks.get_mut("preToolUse") {
        if let Some(array) = pre_tool.as_array_mut() {
            array.retain(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .map(|c| c != "hedl")
                    .unwrap_or(true)
            });
        }
    }

    let content = serde_json::to_string_pretty(&hooks)
        .map_err(|e| format!("Failed to serialize hooks: {}", e))?;
    std::fs::write(&hooks_path, content)
        .map_err(|e| format!("Failed to write hooks.json: {}", e))?;

    println!("✓ Codex hooks uninstalled");
    Ok(())
}
