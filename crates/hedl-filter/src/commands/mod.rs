// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Native command filter implementations.

pub mod git;
pub mod cargo;
pub mod docker;
pub mod kubernetes;
pub mod system;
pub mod js;
pub mod python;
pub mod ruby;
pub mod go;
pub mod cloud;
pub mod dotnet;

/// Native filter function signature.
/// Parameters: (output, has_errors, use_hedl)
pub type NativeFilterFn = fn(&str, bool, bool) -> String;

/// Get a native filter for a command if available.
pub fn get_native_filter(cmd: &str, args: &[ String ]) -> Option<NativeFilterFn> {
    match cmd {
        "git" => {
            if let Some(subcmd) = args.first() {
                match subcmd.as_str() {
                    "status" => Some(git::filter_status),
                    "log" => Some(git::filter_log),
                    "diff" => Some(git::filter_diff),
                    "show" => Some(git::filter_show),
                    "branch" => Some(git::filter_branch),
                    "add" => Some(git::filter_add),
                    "commit" => Some(git::filter_commit),
                    "push" => Some(git::filter_push),
                    "pull" => Some(git::filter_pull),
                    "fetch" => Some(git::filter_fetch),
                    "stash" => Some(git::filter_stash),
                    "worktree" => Some(git::filter_worktree),
                    _ => None,
                }
            } else { None }
        }
        "cargo" => {
            if let Some(subcmd) = args.first() {
                match subcmd.as_str() {
                    "test" => Some(cargo::filter_test),
                    "build" | "check" => Some(cargo::filter_build),
                    "clippy" => Some(cargo::filter_clippy),
                    "install" => Some(cargo::filter_install),
                    "nextest" => Some(cargo::filter_nextest),
                    _ => None,
                }
            } else { None }
        }
        "docker" => {
            if let Some(subcmd) = args.first() {
                match subcmd.as_str() {
                    "ps" => Some(docker::filter_ps),
                    "images" => Some(docker::filter_images),
                    "logs" => Some(docker::filter_logs),
                    "compose" => {
                        if args.len() >= 2 {
                            match args[1].as_str() {
                                "ps" => Some(docker::filter_compose_ps),
                                "logs" => Some(docker::filter_compose_logs),
                                _ => None,
                            }
                        } else { None }
                    }
                    _ => None,
                }
            } else { None }
        }
        "kubectl" | "k" => {
            if args.len() >= 2 {
                match (args[0].as_str(), args[1].as_str()) {
                    ("get", "pods") => Some(kubernetes::filter_pods),
                    ("get", "services") => Some(kubernetes::filter_services),
                    ("logs", _) => Some(kubernetes::filter_logs),
                    _ => None,
                }
            } else { None }
        }
        "ls" => Some(system::filter_ls),
        "find" => Some(system::filter_find),
        "grep" | "rg" => Some(system::filter_grep),
        "env" => Some(system::filter_env),
        "ps" => Some(system::filter_ps),
        "df" => Some(system::filter_df),
        "du" => Some(system::filter_du),
        "ping" => Some(system::filter_ping),
        "npm" => Some(js::filter_npm),
        "npx" => Some(js::filter_npx),
        "pnpm" => Some(js::filter_pnpm),
        "vitest" => Some(js::filter_vitest),
        "jest" => Some(js::filter_jest),
        "tsc" => Some(js::filter_tsc),
        "next" => Some(js::filter_next),
        "eslint" => Some(js::filter_eslint),
        "prettier" => Some(js::filter_prettier),
        "playwright" => Some(js::filter_playwright),
        "prisma" => Some(js::filter_prisma),
        "pytest" => Some(python::filter_pytest),
        "ruff" => Some(python::filter_ruff),
        "mypy" => Some(python::filter_mypy),
        "pip" => Some(python::filter_pip),
        "rake" => Some(ruby::filter_rake),
        "rspec" => Some(ruby::filter_rspec),
        "rubocop" => Some(ruby::filter_rubocop),
        "go" => Some(go::filter_go),
        "golangci-lint" => Some(go::filter_golangci),
        "aws" => Some(cloud::filter_aws),
        "psql" => Some(cloud::filter_psql),
        "dotnet" => Some(dotnet::filter_dotnet),
        _ => None,
    }
}
