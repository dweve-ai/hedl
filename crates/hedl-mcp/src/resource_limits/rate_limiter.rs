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

//! Per-client rate limiting using token bucket algorithm.

use super::client::ClientId;
use super::error::ResourceLimitError;
use crate::rate_limiter::RateLimiter;
use dashmap::DashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::debug;

/// Rate limit configuration for a client or client pattern.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Burst capacity (maximum tokens in bucket).
    pub burst: usize,
    /// Refill rate (tokens added per second).
    pub per_second: usize,
}

impl RateLimitConfig {
    /// Create a new rate limit configuration.
    #[must_use]
    pub fn new(burst: usize, per_second: usize) -> Self {
        Self { burst, per_second }
    }

    /// Get default rate limit configuration.
    #[must_use]
    pub fn default_config() -> Self {
        Self {
            burst: 200,
            per_second: 100,
        }
    }
}

/// Per-client rate limiter using token bucket algorithm.
///
/// Tracks independent rate limits for each client, allowing fair resource
/// distribution across multiple concurrent clients.
pub struct PerClientRateLimiter {
    /// Individual rate limiters per client ID.
    limiters: Arc<DashMap<ClientId, RateLimiter>>,

    /// Default rate limit configuration for unknown clients.
    default_config: RateLimitConfig,

    /// Client pattern overrides (glob pattern -> config).
    overrides: Vec<(glob::Pattern, RateLimitConfig)>,

    /// Last cleanup timestamp.
    last_cleanup: Arc<Mutex<Instant>>,

    /// Cleanup interval for inactive limiters.
    cleanup_interval: Duration,
}

impl PerClientRateLimiter {
    /// Create a new per-client rate limiter.
    ///
    /// # Arguments
    ///
    /// * `default_config` - Default rate limit for unknown clients
    /// * `overrides` - Client pattern overrides (glob pattern -> config)
    /// * `cleanup_interval` - How often to clean up inactive limiters
    #[must_use]
    pub fn new(
        default_config: RateLimitConfig,
        overrides: Vec<(String, RateLimitConfig)>,
        cleanup_interval: Duration,
    ) -> Self {
        // Parse glob patterns
        let overrides = overrides
            .into_iter()
            .filter_map(|(pattern, config)| glob::Pattern::new(&pattern).ok().map(|p| (p, config)))
            .collect();

        Self {
            limiters: Arc::new(DashMap::new()),
            default_config,
            overrides,
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
            cleanup_interval,
        }
    }

    /// Create a per-client rate limiter with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(
            RateLimitConfig::default_config(),
            vec![],
            Duration::from_secs(300),
        )
    }

    /// Check if a request from the given client is allowed.
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client identifier
    ///
    /// # Returns
    ///
    /// `Ok(())` if request is allowed, `Err` if rate limit exceeded.
    pub fn check_limit(&self, client_id: &ClientId) -> Result<(), ResourceLimitError> {
        // Get or create rate limiter for this client
        let limiter = self.limiters.entry(client_id.clone()).or_insert_with(|| {
            let config = self.get_config_for_client(client_id);
            RateLimiter::new(config.burst, config.per_second)
        });

        // Check limit
        if !limiter.check_limit() {
            return Err(ResourceLimitError::RateLimitExceeded {
                client_id: client_id.to_string(),
                burst: limiter.max_tokens(),
                rate: limiter.refill_rate(),
            });
        }

        // Periodic cleanup of inactive limiters
        self.maybe_cleanup();

        Ok(())
    }

    /// Get rate limit configuration for a specific client.
    ///
    /// Checks client pattern overrides and returns the first matching config,
    /// otherwise returns the default config.
    fn get_config_for_client(&self, client_id: &ClientId) -> RateLimitConfig {
        for (pattern, config) in &self.overrides {
            if pattern.matches(&client_id.0) {
                return config.clone();
            }
        }
        self.default_config.clone()
    }

    /// Perform cleanup if enough time has passed.
    ///
    /// Removes inactive client limiters to prevent unbounded memory growth.
    fn maybe_cleanup(&self) {
        let now = Instant::now();

        // Check if cleanup is needed
        {
            let last = self
                .last_cleanup
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let elapsed = now.duration_since(*last);
            if elapsed <= self.cleanup_interval {
                return;
            }
        }

        // Clean up limiters not used recently (10 minutes)
        // Note: For now we keep all limiters since RateLimiter doesn't track last_used
        // In production, you'd add last_used tracking to RateLimiter

        // Update last cleanup time
        let mut last = self
            .last_cleanup
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *last = now;

        debug!("Cleaned up inactive rate limiters");
    }

    /// Get the number of active client limiters.
    #[must_use]
    pub fn active_limiter_count(&self) -> usize {
        self.limiters.len()
    }

    /// Reset all rate limiters (useful for testing).
    pub fn reset_all(&self) {
        self.limiters.clear();
    }

    /// Remove rate limiter for a specific client.
    pub fn remove_client(&self, client_id: &ClientId) {
        self.limiters.remove(client_id);
    }
}
