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

//! Value parsing utilities for XML to HEDL conversion

use super::config::{EntityPolicy, FromXmlConfig};
use hedl_core::lex::parse_expression_token;
use hedl_core::Value;

pub(crate) fn parse_value_with_config(s: &str, config: &FromXmlConfig) -> Result<Value, String> {
    let trimmed = s.trim();

    // Detect entity references (&entity;)
    if trimmed.contains('&') && trimmed.contains(';') {
        if config.log_security_events {
            eprintln!("[SECURITY] Entity reference detected in value: {}", trimmed);
        }

        // Check for potentially malicious entity patterns
        if (trimmed.contains("&xxe;")
            || trimmed.contains("&file;")
            || trimmed.contains("&passwd;")
            || trimmed.contains("&secret;"))
            && config.entity_policy == EntityPolicy::WarnOnEntities
        {
            eprintln!(
                "[WARNING] Suspicious entity reference detected: {}",
                trimmed
            );
        }
    }

    if trimmed.is_empty() {
        return Ok(Value::Null);
    }

    // Note: References are NOT auto-detected from @... pattern.
    // They must be explicitly marked with __hedl_type__="ref" attribute.
    // This prevents strings like "@not-a-ref" from being incorrectly parsed as references.

    // Check for expression pattern $(...)
    if trimmed.starts_with("$(") && trimmed.ends_with(')') {
        let expr =
            parse_expression_token(trimmed).map_err(|e| format!("Invalid expression: {}", e))?;
        return Ok(Value::Expression(Box::new(expr)));
    }

    // Try parsing as boolean
    if trimmed == "true" {
        return Ok(Value::Bool(true));
    }
    if trimmed == "false" {
        return Ok(Value::Bool(false));
    }

    // Try parsing as number
    if let Ok(i) = trimmed.parse::<i64>() {
        return Ok(Value::Int(i));
    }
    if let Ok(f) = trimmed.parse::<f64>() {
        return Ok(Value::Float(f));
    }

    // Default to string
    Ok(Value::String(trimmed.to_string().into()))
}

#[cfg(test)]
pub(crate) fn parse_value(s: &str) -> Result<Value, String> {
    // Legacy function for tests - uses default config
    let config = FromXmlConfig::default();
    parse_value_with_config(s, &config)
}

pub(crate) fn parse_version(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        Some((major, minor))
    } else {
        None
    }
}
