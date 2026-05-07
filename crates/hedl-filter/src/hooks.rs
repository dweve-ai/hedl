// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Hooks for AI agent integration.
//!
//! Provides command rewrite hooks for Claude Code, Cursor, and other AI tools.

use crate::registry;

/// Rewrite a command for AI agent hooks.
///
/// Returns the HEDL-filter equivalent if one exists.
pub fn rewrite_for_hook(cmd: &str) -> Option<String> {
    registry::rewrite_command(cmd)
}

/// Check if a command should be intercepted.
pub fn should_intercept(cmd: &str) -> bool {
    registry::rewrite_command(cmd).is_some()
}

/// Generate hook installation instructions.
pub fn install_instructions(agent: &str) -> String {
    match agent {
        "claude" => {
            r#"# Claude Code Hook Installation

Add to your Claude Code hooks configuration:

```json
{
  "hooks": {
    "preToolUse": [
      {
        "command": "hedl",
        "args": ["hook", "rewrite"],
        "pattern": "^bash$"
      }
    ]
  }
}
```
"#.to_string()
        }
        "cursor" => {
            "# Cursor Hook Installation\n\nAdd to your Cursor settings.\n".to_string()
        }
        _ => format!("# Hook Installation for {}\n\nAgent not yet supported.\n", agent),
    }
}
