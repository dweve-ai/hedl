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

//! Resource limit enforcement metrics.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Resource limit enforcement metrics.
///
/// Tracks statistics about limit violations for monitoring and alerting.
#[derive(Debug, Default)]
pub struct ResourceMetrics {
    /// Number of rate limit violations.
    pub rate_limit_exceeded: AtomicUsize,

    /// Number of request size violations.
    pub request_size_exceeded: AtomicUsize,

    /// Number of response size violations.
    pub response_size_exceeded: AtomicUsize,

    /// Number of concurrency limit violations.
    pub concurrency_exceeded: AtomicUsize,

    /// Number of operation timeouts.
    pub timeouts: AtomicUsize,

    /// Number of successful requests.
    pub requests_succeeded: AtomicUsize,
}

impl ResourceMetrics {
    /// Create new metrics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all metrics.
    pub fn reset(&self) {
        self.rate_limit_exceeded.store(0, Ordering::Relaxed);
        self.request_size_exceeded.store(0, Ordering::Relaxed);
        self.response_size_exceeded.store(0, Ordering::Relaxed);
        self.concurrency_exceeded.store(0, Ordering::Relaxed);
        self.timeouts.store(0, Ordering::Relaxed);
        self.requests_succeeded.store(0, Ordering::Relaxed);
    }

    /// Get all metrics as a tuple.
    pub fn get_all(&self) -> (usize, usize, usize, usize, usize, usize) {
        (
            self.rate_limit_exceeded.load(Ordering::Relaxed),
            self.request_size_exceeded.load(Ordering::Relaxed),
            self.response_size_exceeded.load(Ordering::Relaxed),
            self.concurrency_exceeded.load(Ordering::Relaxed),
            self.timeouts.load(Ordering::Relaxed),
            self.requests_succeeded.load(Ordering::Relaxed),
        )
    }
}
