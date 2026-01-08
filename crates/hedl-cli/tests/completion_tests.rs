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

//! Integration tests for shell completion generation

use assert_cmd::Command;
use predicates::prelude::*;

fn hedl_bin() -> String {
    std::env::var("CARGO_BIN_EXE_hedl")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_hedl").to_string())
}

#[test]
fn test_completion_bash() {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("bash");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("_hedl()"))
        .stdout(predicate::str::contains("COMPREPLY"));
}

#[test]
fn test_completion_zsh() {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("zsh");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("#compdef hedl"))
        .stdout(predicate::str::contains("_hedl()"));
}

#[test]
fn test_completion_fish() {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("fish");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("complete -c hedl"));
}

#[test]
fn test_completion_powershell() {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("powershell");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Register-ArgumentCompleter"))
        .stdout(predicate::str::contains("'hedl'"));
}

#[test]
fn test_completion_elvish() {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("elvish");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("edit:completion:arg-completer"));
}

#[test]
fn test_completion_case_insensitive() {
    // Test that shell names are case-insensitive
    let mut cmd1 = Command::new(hedl_bin());
    cmd1.arg("completion").arg("BASH");
    cmd1.assert().success();

    let mut cmd2 = Command::new(hedl_bin());
    cmd2.arg("completion").arg("Zsh");
    cmd2.assert().success();

    let mut cmd3 = Command::new(hedl_bin());
    cmd3.arg("completion").arg("FiSh");
    cmd3.assert().success();
}

#[test]
fn test_completion_invalid_shell() {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("invalid");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported shell"))
        .stderr(predicate::str::contains("bash, zsh, fish, powershell, elvish"));
}

#[test]
fn test_completion_install_bash() {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("bash").arg("--install");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Bash completion installation"))
        .stdout(predicate::str::contains("~/.bashrc"))
        .stdout(predicate::str::contains("eval"));
}

#[test]
fn test_completion_install_zsh() {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("zsh").arg("--install");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Zsh completion installation"))
        .stdout(predicate::str::contains("~/.zshrc"));
}

#[test]
fn test_completion_install_fish() {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("fish").arg("--install");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Fish completion installation"))
        .stdout(predicate::str::contains(".config/fish/completions"));
}

#[test]
fn test_completion_install_powershell() {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("powershell").arg("--install");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PowerShell completion installation"))
        .stdout(predicate::str::contains("$PROFILE"));
}

#[test]
fn test_completion_install_elvish() {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("elvish").arg("--install");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Elvish completion installation"))
        .stdout(predicate::str::contains("~/.elvish/rc.elv"));
}

#[test]
fn test_completion_help() {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Generate shell completion scripts"))
        .stdout(predicate::str::contains("bash, zsh, fish, powershell, elvish"));
}

#[test]
fn test_completion_validates_all_commands() {
    // Verify that the completion script includes all major commands
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("completion").arg("bash");

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Check that all major commands are present in the completion
    assert!(stdout.contains("validate"));
    assert!(stdout.contains("format"));
    assert!(stdout.contains("lint"));
    assert!(stdout.contains("to-json"));
    assert!(stdout.contains("from-json"));
    assert!(stdout.contains("inspect"));
    assert!(stdout.contains("stats"));
    assert!(stdout.contains("completion"));
}
