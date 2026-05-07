// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Hook commands for AI agent integration.

use clap::Subcommand;

/// Hook commands for AI agent integration.
#[derive(Subcommand)]
pub enum HookCommands {
    /// Run a hook for a specific AI agent
    ///
    /// Reads JSON from stdin and writes the hook response to stdout.
    /// This is called automatically by AI agents when configured.
    Run {
        /// Agent to run hook for
        #[arg(value_enum)]
        agent: Agent,
    },

    /// Install hooks for AI agents
    ///
    /// Configures Claude Code, Cursor, Copilot, or Gemini to use HEDL
    /// for automatic command optimization.
    Init {
        /// Agent to install hooks for
        #[arg(value_enum)]
        agent: Agent,
    },

    /// Uninstall hooks for AI agents
    Uninstall {
        /// Agent to uninstall hooks for
        #[arg(value_enum)]
        agent: Agent,
    },

    /// Test hook rewrite without an agent
    ///
    /// Shows how a command would be rewritten by the hook system.
    Rewrite {
        /// Command to test
        #[arg(required = true)]
        command: String,
    },
}

/// Supported AI agents.
#[derive(Clone, Debug, clap::ValueEnum)]
pub enum Agent {
    /// Claude Code (Anthropic)
    Claude,
    /// Cursor (Anysphere)
    Cursor,
    /// GitHub Copilot
    Copilot,
    /// Gemini CLI (Google)
    Gemini,
    /// OpenCode (anomalyco/opencode)
    Opencode,
    /// Cline (cline.bot)
    Cline,
    /// Codex (OpenAI)
    Codex,
    /// All agents
    All,
}

impl Agent {
    /// Convert to string slice.
    pub fn as_str(&self) -> &str {
        match self {
            Agent::Claude => "claude",
            Agent::Cursor => "cursor",
            Agent::Copilot => "copilot",
            Agent::Gemini => "gemini",
            Agent::Opencode => "opencode",
            Agent::Cline => "cline",
            Agent::Codex => "codex",
            Agent::All => "all",
        }
    }
}

impl HookCommands {
    /// Execute the hook command.
    pub fn execute(self) -> Result<(), crate::error::CliError> {
        match self {
            HookCommands::Run { agent } => {
                match agent {
                    Agent::Claude => crate::hooks::agents::run_claude()
                        .map_err(|e| crate::error::CliError::InvalidInput(e))?,
                    Agent::Cursor => crate::hooks::agents::run_cursor()
                        .map_err(|e| crate::error::CliError::InvalidInput(e))?,
                    Agent::Copilot => crate::hooks::agents::run_copilot()
                        .map_err(|e| crate::error::CliError::InvalidInput(e))?,
                    Agent::Gemini => crate::hooks::agents::run_gemini()
                        .map_err(|e| crate::error::CliError::InvalidInput(e))?,
                    Agent::Opencode => crate::hooks::agents::run_opencode()
                        .map_err(|e| crate::error::CliError::InvalidInput(e))?,
                    Agent::Cline => crate::hooks::agents::run_cline()
                        .map_err(|e| crate::error::CliError::InvalidInput(e))?,
                    Agent::Codex => crate::hooks::agents::run_codex()
                        .map_err(|e| crate::error::CliError::InvalidInput(e))?,
                    Agent::All => {
                        return Err(crate::error::CliError::InvalidInput(
                            "Cannot run hook for 'all' agents. Specify one agent.".to_string()
                        ));
                    }
                }
                Ok(())
            }
            HookCommands::Init { agent } => {
                crate::hooks::init::install(agent.as_str())
                    .map_err(|e| crate::error::CliError::InvalidInput(e))
            }
            HookCommands::Uninstall { agent } => {
                crate::hooks::init::uninstall(agent.as_str())
                    .map_err(|e| crate::error::CliError::InvalidInput(e))
            }
            HookCommands::Rewrite { command } => {
                match crate::hooks::rewrite::get_rewritten(&command) {
                    Some(rewritten) => {
                        println!("{}", rewritten);
                    }
                    None => {
                        println!("{}", command);
                    }
                }
                Ok(())
            }
        }
    }
}
