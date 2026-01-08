// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Shell completion generation - Tab completion for various shells

use clap::Command;
use clap_complete::{generate, Generator};
use std::io;

/// Generate shell completion script to stdout for a given command.
///
/// Generates shell-specific completion scripts that enable tab completion
/// for HEDL CLI commands, arguments, and file paths.
///
/// # Arguments
///
/// * `generator` - The shell generator (Bash, Zsh, Fish, PowerShell, or Elvish)
/// * `cmd` - The clap Command to generate completions for
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// This function does not typically return errors, but uses `Result` for
/// consistency with other command functions.
///
/// # Examples
///
/// ```no_run
/// use clap::Command;
/// use clap_complete::shells::Bash;
/// use hedl_cli::commands::generate_completion_for_command;
///
/// # fn main() -> Result<(), String> {
/// let mut cmd = Command::new("hedl");
/// generate_completion_for_command(Bash, &mut cmd)?;
/// # Ok(())
/// # }
/// ```
///
/// # Output
///
/// Writes the completion script to stdout. Users typically redirect this to
/// a file or evaluate it in their shell configuration.
pub fn generate_completion_for_command<G: Generator>(
    generator: G,
    cmd: &mut Command,
) -> Result<(), String> {
    generate(generator, cmd, cmd.get_name().to_string(), &mut io::stdout());
    Ok(())
}

/// Print installation instructions for shell completions.
///
/// Returns detailed, shell-specific instructions for installing and enabling
/// HEDL CLI tab completions. Instructions cover both temporary (current session)
/// and persistent (profile-based) installation methods.
///
/// # Arguments
///
/// * `shell` - The target shell name (bash, zsh, fish, powershell/pwsh, elvish)
///
/// # Returns
///
/// Returns a formatted string with installation instructions.
///
/// # Examples
///
/// ```
/// use hedl_cli::commands::print_installation_instructions;
///
/// // Get bash installation instructions
/// let instructions = print_installation_instructions("bash");
/// assert!(instructions.contains("bash"));
///
/// // Get zsh installation instructions
/// let instructions = print_installation_instructions("zsh");
/// assert!(instructions.contains("zsh"));
///
/// // Unsupported shells return a generic message
/// let instructions = print_installation_instructions("unknown");
/// assert_eq!(instructions, "Unsupported shell");
/// ```
///
/// # Supported Shells
///
/// - **bash**: Instructions for ~/.bashrc and bash-completion directories
/// - **zsh**: Instructions for ~/.zshrc and $fpath completion directories
/// - **fish**: Instructions for ~/.config/fish/completions/
/// - **powershell/pwsh**: Instructions for PowerShell profile
/// - **elvish**: Instructions for ~/.elvish/rc.elv
///
/// # Case Sensitivity
///
/// Shell names are case-insensitive (e.g., "BASH", "Bash", "bash" all work).
pub fn print_installation_instructions(shell: &str) -> String {
    match shell.to_lowercase().as_str() {
        "bash" => {
            r#"# Bash completion installation:

# For current session only:
eval "$(hedl completion bash)"

# For persistent installation, add to your ~/.bashrc:
echo 'eval "$(hedl completion bash)"' >> ~/.bashrc

# Or save to completions directory:
hedl completion bash > ~/.local/share/bash-completion/completions/hedl
"#
        }
        "zsh" => {
            r#"# Zsh completion installation:

# For current session only:
eval "$(hedl completion zsh)"

# For persistent installation, add to your ~/.zshrc:
echo 'eval "$(hedl completion zsh)"' >> ~/.zshrc

# Or save to completions directory (ensure directory is in $fpath):
hedl completion zsh > ~/.zsh/completions/_hedl
"#
        }
        "fish" => {
            r#"# Fish completion installation:

# Save to fish completions directory:
hedl completion fish > ~/.config/fish/completions/hedl.fish

# Completions will be available in new fish sessions
"#
        }
        "powershell" | "pwsh" => {
            r#"# PowerShell completion installation:

# For current session only:
hedl completion powershell | Out-String | Invoke-Expression

# For persistent installation, add to your PowerShell profile:
# Find profile location with: $PROFILE
# Then add this line:
hedl completion powershell | Out-String | Invoke-Expression
"#
        }
        "elvish" => {
            r#"# Elvish completion installation:

# For current session only:
eval (hedl completion elvish)

# For persistent installation, add to your ~/.elvish/rc.elv:
eval (hedl completion elvish)
"#
        }
        _ => "Unsupported shell",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installation_instructions_bash() {
        let instructions = print_installation_instructions("bash");
        assert!(!instructions.is_empty());
        assert!(instructions.to_lowercase().contains("bash"));
    }

    #[test]
    fn test_installation_instructions_zsh() {
        let instructions = print_installation_instructions("zsh");
        assert!(!instructions.is_empty());
        assert!(instructions.contains("zsh"));
    }

    #[test]
    fn test_installation_instructions_fish() {
        let instructions = print_installation_instructions("fish");
        assert!(!instructions.is_empty());
        assert!(instructions.contains("fish"));
    }

    #[test]
    fn test_installation_instructions_powershell() {
        let instructions = print_installation_instructions("powershell");
        assert!(!instructions.is_empty());
        assert!(instructions.to_lowercase().contains("powershell"));
    }

    #[test]
    fn test_installation_instructions_elvish() {
        let instructions = print_installation_instructions("elvish");
        assert!(!instructions.is_empty());
        assert!(instructions.contains("elvish"));
    }

    #[test]
    fn test_installation_instructions_case_insensitive() {
        let lower = print_installation_instructions("bash");
        let upper = print_installation_instructions("BASH");
        let mixed = print_installation_instructions("Bash");
        assert_eq!(lower, upper);
        assert_eq!(lower, mixed);
    }

    #[test]
    fn test_installation_instructions_unsupported() {
        let instructions = print_installation_instructions("invalid");
        assert_eq!(instructions, "Unsupported shell");
    }
}
