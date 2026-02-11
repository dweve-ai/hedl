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

//! Client identification for per-client resource tracking.

/// Client identifier for per-client resource tracking.
///
/// Used to enforce rate limits and concurrency limits independently per client.
/// Currently defaults to anonymous since authentication is not yet implemented.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ClientId(pub String);

impl ClientId {
    /// Create an anonymous client ID.
    ///
    /// Used when no client identification is available (e.g., no authentication).
    #[must_use]
    pub fn anonymous() -> Self {
        Self("anonymous".to_string())
    }

    /// Create a client ID from a string identifier.
    #[must_use]
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// Get the string value of this client ID.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self::anonymous()
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
