// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Configuration management for HEDL Filter.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// HEDL Filter configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Global settings
    #[serde(default)]
    pub global: GlobalConfig,
    /// Hook settings
    #[serde(default)]
    pub hooks: HookConfig,
    /// Tee settings
    #[serde(default)]
    pub tee: TeeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    /// Default output mode
    #[serde(default)]
    pub mode: String,
    /// Enable HEDL output by default
    #[serde(default = "default_true")]
    pub hedl_output: bool,
    /// Maximum output size in bytes
    #[serde(default = "default_max_size")]
    pub max_output_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookConfig {
    /// Commands to exclude from rewriting
    #[serde(default)]
    pub exclude_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeeConfig {
    /// Enable tee mode
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Tee mode: always, failures, never
    #[serde(default = "default_tee_mode")]
    pub mode: String,
    /// Custom tee directory
    pub directory: Option<PathBuf>,
}

fn default_true() -> bool { true }
fn default_max_size() -> usize { 10 * 1024 * 1024 }
fn default_tee_mode() -> String { "failures".to_string() }

impl Config {
    /// Load configuration from file or create default.
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("hedl").join("config.toml");
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        Config::default()
    }

    /// Save configuration to file.
    pub fn save(&self) -> Result<(), String> {
        if let Some(config_dir) = dirs::config_dir() {
            let dir = config_dir.join("hedl");
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let path = dir.join("config.toml");
            let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
            std::fs::write(&path, content).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            global: GlobalConfig::default(),
            hooks: HookConfig::default(),
            tee: TeeConfig::default(),
        }
    }
}
