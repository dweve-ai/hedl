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

//! Configuration for XML to HEDL conversion

/// Policy for handling XML entities and DTDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntityPolicy {
    /// Reject XML with DOCTYPE declarations (strictest, recommended)
    RejectDtd,
    /// Allow DOCTYPE but never resolve external entities (default)
    #[default]
    AllowDtdNoExternal,
    /// Log warnings when DTDs or entity references detected
    WarnOnEntities,
}

/// Configuration for XML import
#[derive(Debug, Clone)]
pub struct FromXmlConfig {
    /// Default type name for list items without metadata
    pub default_type_name: String,
    /// HEDL version to use
    pub version: (u32, u32),
    /// Try to infer list structures from repeated elements
    pub infer_lists: bool,

    /// Entity handling policy (XXE prevention)
    pub entity_policy: EntityPolicy,

    /// Enable security event logging
    pub log_security_events: bool,
}

impl Default for FromXmlConfig {
    fn default() -> Self {
        Self {
            default_type_name: "Item".to_string(),
            version: (2, 0),
            infer_lists: true,
            entity_policy: EntityPolicy::default(),
            log_security_events: false,
        }
    }
}

impl FromXmlConfig {
    /// Create a config with strict security (reject DTDs entirely)
    pub fn strict_security() -> Self {
        Self {
            entity_policy: EntityPolicy::RejectDtd,
            log_security_events: true,
            ..Default::default()
        }
    }
}

impl hedl_core::convert::ImportConfig for FromXmlConfig {
    fn default_type_name(&self) -> &str {
        &self.default_type_name
    }

    fn version(&self) -> (u32, u32) {
        self.version
    }
}
