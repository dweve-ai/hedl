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

//! Type definitions and constants for Neo4j configuration.

use serde::{Deserialize, Serialize};

/// Default maximum string length for property values: 100 MB.
///
/// This limit is set high to accommodate large text content commonly found in
/// graph databases (descriptions, articles, documentation, etc.), while still
/// providing protection against resource exhaustion attacks.
///
/// Cypher queries can contain large text properties including:
/// - Long-form content (articles, documentation, descriptions)
/// - Serialized JSON or XML data
/// - Large text fields from data imports
///
/// For stricter security requirements, use `ToCypherConfig::for_untrusted_input()`
/// which enforces a conservative 1MB limit.
pub const DEFAULT_MAX_STRING_LENGTH: usize = 100 * 1024 * 1024; // 100 MB

/// Default transaction batch size: 100 statements per transaction.
pub const DEFAULT_TRANSACTION_BATCH_SIZE: usize = 100;

/// Default transaction row limit: 10,000 rows per transaction.
pub const DEFAULT_TRANSACTION_ROW_LIMIT: usize = 10_000;

/// Default batch size for streaming record processing: 1000 records.
pub const DEFAULT_FROM_NEO4J_BATCH_SIZE: usize = 1000;

/// Strategy for batching multiple statements into transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TransactionStrategy {
    /// Batch by statement count (default).
    #[default]
    StatementCount,
    /// Batch by estimated row count.
    RowCount,
    /// Batch by statement type (nodes, relationships, indexes).
    StatementType,
    /// Adaptive batching based on execution time.
    Adaptive,
}

/// Transaction isolation level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// Default isolation level (database default).
    #[default]
    Default,
    /// Serializable isolation for strict consistency.
    Serializable,
}

/// How to name relationships generated from references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RelationshipNaming {
    /// Use the property name as relationship type (e.g., `author` -> `:AUTHOR`).
    #[default]
    PropertyName,
    /// Use a generic relationship type (e.g., `:REFERENCES`).
    Generic,
    /// Use the target type name (e.g., `@User:alice` -> `:USER`).
    TargetType,
}

/// How to handle nested objects in node properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ObjectHandling {
    /// Flatten nested objects into dot-notation properties (e.g., `address.city`).
    #[default]
    Flatten,
    /// Serialize nested objects as JSON strings.
    JsonString,
}

/// Batch size strategy for determining optimal batch sizes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchSizeStrategy {
    /// Fixed batch size (legacy behavior).
    Fixed(usize),
    /// Adaptive based on node size estimation.
    ///
    /// Dynamically calculates optimal batch size based on:
    /// - Average node size (properties and their values)
    /// - Target batch memory footprint
    /// - Min/max batch size bounds
    Adaptive {
        /// Target batch size in bytes (default: 512KB).
        target_batch_bytes: usize,
        /// Minimum batch size regardless of node size (default: 100).
        min_batch_size: usize,
        /// Maximum batch size regardless of node size (default: 5000).
        max_batch_size: usize,
    },
}

impl Default for BatchSizeStrategy {
    fn default() -> Self {
        BatchSizeStrategy::Fixed(1000)
    }
}
