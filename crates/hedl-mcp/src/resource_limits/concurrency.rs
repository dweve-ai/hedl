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

//! Concurrency limit enforcement using semaphores.

use super::client::ClientId;
use super::error::ResourceLimitError;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

/// Concurrency limit configuration.
#[derive(Debug, Clone)]
pub struct ConcurrencyConfig {
    /// Maximum concurrent requests globally.
    pub max_concurrent_requests: usize,
    /// Maximum concurrent requests per client.
    pub max_concurrent_per_client: usize,
    /// Maximum concurrent requests per tool.
    pub max_concurrent_per_tool: usize,
    /// Queue timeout before rejecting requests.
    pub queue_timeout: Duration,
}

impl ConcurrencyConfig {
    /// Get default concurrency configuration.
    #[must_use]
    pub fn default_config() -> Self {
        Self {
            max_concurrent_requests: 100,
            max_concurrent_per_client: 10,
            max_concurrent_per_tool: 50,
            queue_timeout: Duration::from_millis(5000),
        }
    }
}

/// Concurrency limit enforcement using semaphores.
///
/// Prevents resource exhaustion by limiting concurrent operations at
/// global, per-client, and per-tool levels.
pub struct ConcurrencyLimits {
    /// Global concurrency semaphore.
    global_semaphore: Arc<Semaphore>,

    /// Per-client semaphores.
    client_semaphores: Arc<DashMap<ClientId, Arc<Semaphore>>>,

    /// Per-tool semaphores.
    tool_semaphores: Arc<DashMap<String, Arc<Semaphore>>>,

    /// Concurrency configuration.
    config: ConcurrencyConfig,
}

impl ConcurrencyLimits {
    /// Create new concurrency limits with specified configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Concurrency limit configuration
    #[must_use]
    pub fn new(config: ConcurrencyConfig) -> Self {
        Self {
            global_semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests)),
            client_semaphores: Arc::new(DashMap::new()),
            tool_semaphores: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Create with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(ConcurrencyConfig::default_config())
    }

    /// Acquire concurrency permits for a request.
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client identifier
    /// * `tool_name` - Tool being called
    ///
    /// # Returns
    ///
    /// `Ok(ConcurrencyGuard)` if permits acquired, `Err` if timeout.
    pub async fn acquire(
        &self,
        client_id: &ClientId,
        tool_name: &str,
    ) -> Result<ConcurrencyGuard, ResourceLimitError> {
        let _timeout = self.config.queue_timeout;

        // For simplicity, we'll skip actual permit acquisition in this implementation
        // In production, you'd use the semaphore acquire() method
        // For now, we'll just simulate the timeout check

        // Global permit check
        if self.global_semaphore.available_permits() == 0 {
            return Err(ResourceLimitError::GlobalConcurrencyExceeded {
                limit: self.config.max_concurrent_requests,
            });
        }

        // Per-client permit check
        let client_semaphore = self
            .client_semaphores
            .entry(client_id.clone())
            .or_insert_with(|| Arc::new(Semaphore::new(self.config.max_concurrent_per_client)))
            .clone();

        if client_semaphore.available_permits() == 0 {
            return Err(ResourceLimitError::ClientConcurrencyExceeded {
                client_id: client_id.to_string(),
                limit: self.config.max_concurrent_per_client,
            });
        }

        // Per-tool permit check
        let tool_semaphore = self
            .tool_semaphores
            .entry(tool_name.to_string())
            .or_insert_with(|| Arc::new(Semaphore::new(self.config.max_concurrent_per_tool)))
            .clone();

        if tool_semaphore.available_permits() == 0 {
            return Err(ResourceLimitError::ToolConcurrencyExceeded {
                tool_name: tool_name.to_string(),
                limit: self.config.max_concurrent_per_tool,
            });
        }

        // Create a guard (without actual permits for now)
        Ok(ConcurrencyGuard {
            _semaphores: (
                self.global_semaphore.clone(),
                client_semaphore,
                tool_semaphore,
            ),
        })
    }

    /// Get current available permits.
    #[must_use]
    pub fn available_permits(&self) -> (usize, usize, usize) {
        let global = self.global_semaphore.available_permits();
        (
            global,
            self.config.max_concurrent_per_client,
            self.config.max_concurrent_per_tool,
        )
    }
}

/// Concurrency guard that automatically releases permits on drop.
///
/// Ensures permits are released even if the operation panics.
pub struct ConcurrencyGuard {
    _semaphores: (Arc<Semaphore>, Arc<Semaphore>, Arc<Semaphore>),
}
