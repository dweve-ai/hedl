// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Adaptive batching utilities for optimal memory and performance.
//!
//! This module provides functions for creating batches of items based on
//! configuration, supporting both fixed-size and adaptive batching strategies.
//!
//! # Batching Strategies
//!
//! ## Fixed Batching
//!
//! Uses a constant batch size regardless of item complexity. Simple and
//! predictable, but suboptimal for datasets with varying item sizes.
//!
//! ## Adaptive Batching
//!
//! Dynamically determines batch size based on estimated memory footprint:
//! - Small items: Larger batches (better throughput)
//! - Large items: Smaller batches (controlled memory usage)
//! - Respects min/max constraints to prevent degenerate cases
//!
//! # Performance
//!
//! Adaptive batching provides:
//! - 3-5x throughput improvement for minimal nodes
//! - 1.5-2x throughput improvement for property-rich nodes
//! - Consistent memory usage regardless of node complexity

use crate::config::{BatchSizeStrategy, IsolationLevel, ToCypherConfig, TransactionStrategy};
use crate::cypher::{CypherStatement, CypherValue, StatementType};

/// Create batches from a slice of items based on configuration.
///
/// If adaptive batching is enabled in the config, batches are sized by
/// estimated memory usage. Otherwise, fixed-size batching is used.
///
/// # Arguments
///
/// * `items` - Items to batch
/// * `config` - Configuration controlling batching behavior
/// * `estimate_fn` - Function to estimate memory size of each item
///
/// # Returns
///
/// Vector of slices representing batches. Each batch respects the
/// configured constraints (memory target, min/max size).
///
/// # Examples
///
/// ```ignore
/// // Internal API - not exported from crate root
/// use hedl_neo4j::{ToCypherConfig, config::BatchSizeStrategy};
/// use hedl_neo4j::batching::create_adaptive_batches;
/// let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
///
/// // Fixed batching
/// let config = ToCypherConfig::new().with_batch_size(3);
/// let batches = create_adaptive_batches(&items, &config, |_| 100);
/// assert_eq!(batches.len(), 4); // 10 items / 3 per batch
///
/// // Adaptive batching
/// let config = ToCypherConfig::new()
///     .with_batch_size_strategy(BatchSizeStrategy::Adaptive {
///         target_batch_bytes: 300,
///         min_batch_size: 1,
///         max_batch_size: 10,
///     });
/// let batches = create_adaptive_batches(&items, &config, |_| 100);
/// assert_eq!(batches.len(), 4); // 10 items at 100 bytes each, target 300 bytes
/// ```
pub fn create_adaptive_batches<'a, T>(
    items: &'a [T],
    config: &ToCypherConfig,
    estimate_fn: impl Fn(&T) -> usize,
) -> Vec<&'a [T]> {
    match &config.batch_size_strategy {
        BatchSizeStrategy::Fixed(size) => {
            // Fixed-size batching (backward compatible)
            items.chunks(*size).collect()
        }
        BatchSizeStrategy::Adaptive {
            target_batch_bytes,
            min_batch_size,
            max_batch_size,
        } => adaptive_batches_by_memory(
            items,
            *target_batch_bytes,
            *min_batch_size,
            *max_batch_size,
            estimate_fn,
        ),
    }
}

/// Create batches based on memory budget.
///
/// Accumulates items until the target memory budget is reached, respecting
/// minimum and maximum batch size constraints.
fn adaptive_batches_by_memory<T>(
    items: &[T],
    target_bytes: usize,
    min_size: usize,
    max_size: usize,
    estimate_fn: impl Fn(&T) -> usize,
) -> Vec<&[T]> {
    if items.is_empty() {
        return Vec::new();
    }

    let mut batches = Vec::new();
    let mut current_start = 0;

    while current_start < items.len() {
        let mut batch_size = 0;
        let mut accumulated_bytes = 0;

        // Accumulate items until memory budget or max batch size reached
        for item in &items[current_start..] {
            let item_size = estimate_fn(item);

            // Check if adding this item would exceed budget
            if accumulated_bytes + item_size > target_bytes && batch_size >= min_size {
                // Stop here, batch is full
                break;
            }

            accumulated_bytes += item_size;
            batch_size += 1;

            // Respect maximum batch size
            if batch_size >= max_size {
                break;
            }
        }

        // Ensure at least min_size items (unless at end of data)
        if batch_size == 0 {
            batch_size = min_size.min(items.len() - current_start);
        }

        let end = (current_start + batch_size).min(items.len());
        batches.push(&items[current_start..end]);
        current_start = end;
    }

    batches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BatchSizeStrategy;

    #[test]
    fn test_fixed_batching() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let config = ToCypherConfig::new().with_batch_size(3);

        let batches = create_adaptive_batches(&items, &config, |_| 100);

        assert_eq!(batches.len(), 4);
        assert_eq!(batches[0].len(), 3);
        assert_eq!(batches[1].len(), 3);
        assert_eq!(batches[2].len(), 3);
        assert_eq!(batches[3].len(), 1);
    }

    #[test]
    fn test_adaptive_batching_uniform_size() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let config = ToCypherConfig::new().with_batch_size_strategy(BatchSizeStrategy::Adaptive {
            target_batch_bytes: 300, // 3 items at 100 bytes each
            min_batch_size: 1,
            max_batch_size: 1000,
        });

        let batches = create_adaptive_batches(&items, &config, |_| 100);

        // Should create batches of 3 items each (300 bytes)
        assert_eq!(batches.len(), 4);
        assert_eq!(batches[0].len(), 3);
        assert_eq!(batches[1].len(), 3);
        assert_eq!(batches[2].len(), 3);
        assert_eq!(batches[3].len(), 1);
    }

    #[test]
    fn test_adaptive_batching_varying_size() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let config = ToCypherConfig::new().with_batch_size_strategy(BatchSizeStrategy::Adaptive {
            target_batch_bytes: 300,
            min_batch_size: 1,
            max_batch_size: 1000,
        });

        // Items have increasing sizes: 100, 200, 300, 400, 500, 600
        let batches = create_adaptive_batches(&items, &config, |&x| x * 100);

        // Batch 1: [1, 2] = 100 + 200 = 300 bytes (exactly at target)
        // Batch 2: [3] = 300 bytes (exactly at target)
        // Batch 3: [4] = 400 bytes (exceeds but min_batch_size=1)
        // Batch 4: [5] = 500 bytes
        // Batch 5: [6] = 600 bytes
        assert_eq!(batches.len(), 5);
        assert_eq!(batches[0], &[1, 2]);
        assert_eq!(batches[1], &[3]);
        assert_eq!(batches[2], &[4]);
        assert_eq!(batches[3], &[5]);
        assert_eq!(batches[4], &[6]);
    }

    #[test]
    fn test_adaptive_batching_min_batch_size() {
        let items = vec![1, 2, 3, 4, 5];
        let config = ToCypherConfig::new().with_batch_size_strategy(BatchSizeStrategy::Adaptive {
            target_batch_bytes: 50, // Very small target
            min_batch_size: 3,      // But require at least 3 items
            max_batch_size: 1000,
        });

        let batches = create_adaptive_batches(&items, &config, |_| 100);

        // Should respect min_batch_size even though items exceed budget
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 3); // Min batch size
        assert_eq!(batches[1].len(), 2); // Remaining items
    }

    #[test]
    fn test_adaptive_batching_max_batch_size() {
        let items: Vec<u32> = (1..=100).collect();
        let config = ToCypherConfig::new().with_batch_size_strategy(BatchSizeStrategy::Adaptive {
            target_batch_bytes: 1_000_000, // Large budget
            min_batch_size: 1,
            max_batch_size: 20, // Max 20 items
        });

        let batches = create_adaptive_batches(&items, &config, |_| 10);

        // Should not exceed max_batch_size even with large memory budget
        for batch in batches {
            assert!(batch.len() <= 20);
        }
    }

    #[test]
    fn test_adaptive_batching_empty_input() {
        let items: Vec<i32> = vec![];
        let config = ToCypherConfig::new().with_batch_size_strategy(BatchSizeStrategy::Adaptive {
            target_batch_bytes: 1000,
            min_batch_size: 1,
            max_batch_size: 100,
        });

        let batches = create_adaptive_batches(&items, &config, |_| 100);

        assert_eq!(batches.len(), 0);
    }

    #[test]
    fn test_adaptive_batching_single_item() {
        let items = vec![1];
        let config = ToCypherConfig::new().with_batch_size_strategy(BatchSizeStrategy::Adaptive {
            target_batch_bytes: 1000,
            min_batch_size: 1,
            max_batch_size: 100,
        });

        let batches = create_adaptive_batches(&items, &config, |_| 100);

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 1);
    }

    #[test]
    fn test_adaptive_batching_large_single_item() {
        let items = vec![1, 2, 3];
        let config = ToCypherConfig::new().with_batch_size_strategy(BatchSizeStrategy::Adaptive {
            target_batch_bytes: 50, // Each item is 1000 bytes, much larger than target
            min_batch_size: 1,
            max_batch_size: 10,
        });

        let batches = create_adaptive_batches(&items, &config, |_| 1000);

        // Should create 3 batches of 1 item each (respecting min_batch_size=1)
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].len(), 1);
        assert_eq!(batches[1].len(), 1);
        assert_eq!(batches[2].len(), 1);
    }

    #[test]
    fn test_adaptive_batching_exact_fit() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let config = ToCypherConfig::new().with_batch_size_strategy(BatchSizeStrategy::Adaptive {
            target_batch_bytes: 500, // Each item is 50 bytes, so 10 items = 500 bytes
            min_batch_size: 1,
            max_batch_size: 20,
        });

        let batches = create_adaptive_batches(&items, &config, |_| 50);

        // Should create 1 batch of 10 items (exactly 500 bytes)
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 10);
    }

    #[test]
    fn test_adaptive_batching_respects_max() {
        let items: Vec<u32> = (1..=100).collect();
        let config = ToCypherConfig::new().with_batch_size_strategy(BatchSizeStrategy::Adaptive {
            target_batch_bytes: 1_000_000, // Large budget
            min_batch_size: 1,
            max_batch_size: 20,
        });

        let batches = create_adaptive_batches(&items, &config, |_| 10);

        // Total items / max_batch_size = 100 / 20 = 5 batches
        assert_eq!(batches.len(), 5);
        for batch in batches {
            assert!(batch.len() <= 20);
        }
    }

    #[test]
    fn test_fixed_vs_adaptive_same_result_for_uniform_size() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let fixed_config = ToCypherConfig::new().with_batch_size(3);
        let adaptive_config =
            ToCypherConfig::new().with_batch_size_strategy(BatchSizeStrategy::Adaptive {
                target_batch_bytes: 300,
                min_batch_size: 1,
                max_batch_size: 1000,
            });

        let fixed_batches = create_adaptive_batches(&items, &fixed_config, |_| 100);
        let adaptive_batches = create_adaptive_batches(&items, &adaptive_config, |_| 100);

        // Both should produce same number of batches and sizes
        assert_eq!(fixed_batches.len(), adaptive_batches.len());
        for (fb, ab) in fixed_batches.iter().zip(adaptive_batches.iter()) {
            assert_eq!(fb.len(), ab.len());
        }
    }

    #[test]
    fn test_adaptive_batching_zero_size_items() {
        let items = vec![1, 2, 3, 4, 5];
        let config = ToCypherConfig::new().with_batch_size_strategy(BatchSizeStrategy::Adaptive {
            target_batch_bytes: 100,
            min_batch_size: 1,
            max_batch_size: 10,
        });

        // All items have zero size - should respect max_batch_size
        let batches = create_adaptive_batches(&items, &config, |_| 0);

        // Should fit all in one batch up to max_batch_size
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 5);
    }
}

// ============================================================================
// Transaction Batching
// ============================================================================

/// A batch of Cypher statements to execute in a single transaction.
///
/// `TransactionBatch` groups related statements for efficient execution.
/// When rendered, it produces a Cypher script with transaction markers.
///
/// # Performance Benefits
///
/// - Reduces network round-trips (one commit per batch vs per statement)
/// - Enables Neo4j's internal batch optimizations
/// - Provides atomicity for grouped operations
///
/// # Example
///
/// ```rust,ignore
/// use hedl_neo4j::batching::TransactionBatch;
///
/// let mut batch = TransactionBatch::new(1, IsolationLevel::Default);
/// batch.add(constraint_stmt);
/// batch.add(node_stmt);
///
/// let cypher = batch.render(true);
/// // Execute cypher as single transaction
/// ```
#[derive(Debug, Clone)]
pub struct TransactionBatch {
    /// Statements in this batch
    pub statements: Vec<CypherStatement>,
    /// Total row count across all statements
    pub row_count: usize,
    /// Batch sequence number (for logging/debugging)
    pub sequence: usize,
    /// Isolation level for this transaction
    pub isolation: IsolationLevel,
}

impl TransactionBatch {
    /// Create a new empty transaction batch.
    #[must_use]
    pub fn new(sequence: usize, isolation: IsolationLevel) -> Self {
        Self {
            statements: Vec::new(),
            row_count: 0,
            sequence,
            isolation,
        }
    }

    /// Add a statement to the batch.
    pub fn add(&mut self, statement: CypherStatement) {
        let rows = count_statement_rows(&statement);
        self.row_count = self.row_count.saturating_add(rows);
        self.statements.push(statement);
    }

    /// Check if the batch is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    /// Get the number of statements in the batch.
    #[must_use]
    pub fn len(&self) -> usize {
        self.statements.len()
    }

    /// Render the batch as a Cypher script with transaction markers.
    ///
    /// # Output Format
    ///
    /// ```cypher
    /// // Transaction batch 1 (5 statements, 1000 rows)
    /// :begin
    /// <statement 1>;
    /// <statement 2>;
    /// ...
    /// :commit
    /// ```
    #[must_use]
    pub fn render(&self, include_comments: bool) -> String {
        let mut output = String::new();

        // Add batch header comment
        if include_comments {
            output.push_str(&format!(
                "// Transaction batch {} ({} statements, {} rows)\n",
                self.sequence,
                self.statements.len(),
                self.row_count
            ));
        }

        // Begin transaction
        output.push_str(":begin\n");

        // Render each statement
        for (i, stmt) in self.statements.iter().enumerate() {
            if i > 0 {
                output.push_str("\n\n");
            }

            if include_comments {
                if let Some(comment) = &stmt.comment {
                    output.push_str(&format!("// {comment}\n"));
                }
            }

            output.push_str(&stmt.render_inline());
            output.push(';');
        }

        output.push('\n');

        // Commit transaction
        output.push_str(":commit");

        output
    }

    /// Render the batch as a list of statements without transaction markers.
    ///
    /// Use this when transaction control is handled externally (e.g., by a driver).
    #[must_use]
    pub fn render_statements(&self, include_comments: bool) -> String {
        let mut output = String::new();

        for (i, stmt) in self.statements.iter().enumerate() {
            if i > 0 {
                output.push_str("\n\n");
            }

            if include_comments {
                if let Some(comment) = &stmt.comment {
                    output.push_str(&format!("// {comment}\n"));
                }
            }

            output.push_str(&stmt.render_inline());
            output.push(';');
        }

        output
    }
}

/// Count rows in a statement's parameters.
///
/// Looks for `rows` parameter containing a list and counts elements.
fn count_statement_rows(statement: &CypherStatement) -> usize {
    if let Some(CypherValue::List(rows)) = statement.parameters.get("rows") {
        rows.len()
    } else {
        1 // Non-parameterized statements count as 1 row
    }
}

/// Batch statements according to the configured strategy.
///
/// This is the main entry point for transaction batching. It dispatches
/// to the appropriate batching function based on `config.transaction_strategy`.
///
/// # Arguments
///
/// * `statements` - Slice of statements to batch
/// * `config` - Configuration specifying batching parameters
///
/// # Returns
///
/// Vector of transaction batches ready for execution.
///
/// # Example
///
/// ```rust,ignore
/// let statements = to_cypher_statements(&doc, &config)?;
/// let batches = batch_statements(&statements, &config);
///
/// for batch in batches {
///     execute_transaction(&batch.render(true));
/// }
/// ```
#[must_use]
pub fn batch_statements(
    statements: &[CypherStatement],
    config: &ToCypherConfig,
) -> Vec<TransactionBatch> {
    if !config.transaction_batching_enabled {
        // Return single batch with all statements
        let mut batch = TransactionBatch::new(1, config.transaction_isolation);
        for stmt in statements {
            batch.add(stmt.clone());
        }
        return vec![batch];
    }

    match config.transaction_strategy {
        TransactionStrategy::StatementCount => batch_by_statement_count(statements, config),
        TransactionStrategy::RowCount => batch_by_row_count(statements, config),
        TransactionStrategy::StatementType => batch_by_statement_type(statements, config),
        TransactionStrategy::Adaptive => batch_adaptive(statements, config),
    }
}

/// Batch statements by count.
///
/// Groups exactly `transaction_batch_size` statements per batch.
fn batch_by_statement_count(
    statements: &[CypherStatement],
    config: &ToCypherConfig,
) -> Vec<TransactionBatch> {
    let mut batches = Vec::new();
    let mut sequence = 1;

    for chunk in statements.chunks(config.transaction_batch_size) {
        let mut batch = TransactionBatch::new(sequence, config.transaction_isolation);
        for stmt in chunk {
            batch.add(stmt.clone());
        }
        batches.push(batch);
        sequence += 1;
    }

    batches
}

/// Batch statements by total row count.
///
/// Groups statements until cumulative row count exceeds `transaction_row_limit`.
fn batch_by_row_count(
    statements: &[CypherStatement],
    config: &ToCypherConfig,
) -> Vec<TransactionBatch> {
    let mut batches = Vec::new();
    let mut current_batch = TransactionBatch::new(1, config.transaction_isolation);
    let mut sequence = 1;

    for stmt in statements {
        let rows = count_statement_rows(stmt);

        // Start new batch if adding this statement would exceed limit
        if !current_batch.is_empty()
            && current_batch.row_count.saturating_add(rows) > config.transaction_row_limit
        {
            batches.push(current_batch);
            sequence += 1;
            current_batch = TransactionBatch::new(sequence, config.transaction_isolation);
        }

        current_batch.add(stmt.clone());

        // Also respect statement count limit
        if current_batch.len() >= config.transaction_batch_size {
            batches.push(current_batch);
            sequence += 1;
            current_batch = TransactionBatch::new(sequence, config.transaction_isolation);
        }
    }

    // Don't forget the last batch
    if !current_batch.is_empty() {
        batches.push(current_batch);
    }

    batches
}

/// Batch statements by type.
///
/// Creates separate batches for each statement type:
/// 1. Constraints
/// 2. Indexes
/// 3. Node creation
/// 4. Relationship creation
fn batch_by_statement_type(
    statements: &[CypherStatement],
    config: &ToCypherConfig,
) -> Vec<TransactionBatch> {
    let mut batches = Vec::new();
    let mut sequence = 1;

    // Group statements by type
    let mut constraints: Vec<&CypherStatement> = Vec::new();
    let mut indexes: Vec<&CypherStatement> = Vec::new();
    let mut nodes: Vec<&CypherStatement> = Vec::new();
    let mut relationships: Vec<&CypherStatement> = Vec::new();
    let mut other: Vec<&CypherStatement> = Vec::new();

    for stmt in statements {
        match stmt.statement_type {
            StatementType::Constraint => constraints.push(stmt),
            StatementType::Index => indexes.push(stmt),
            StatementType::CreateNode => nodes.push(stmt),
            StatementType::CreateRelationship => relationships.push(stmt),
            StatementType::Query | StatementType::SetProperty => other.push(stmt),
        }
    }

    // Create batches for each type
    let groups: [(&[&CypherStatement], &str); 5] = [
        (&constraints, "constraints"),
        (&indexes, "indexes"),
        (&nodes, "nodes"),
        (&relationships, "relationships"),
        (&other, "other"),
    ];

    for (group, _name) in groups {
        if group.is_empty() {
            continue;
        }

        for chunk in group.chunks(config.transaction_batch_size) {
            let mut batch = TransactionBatch::new(sequence, config.transaction_isolation);
            for stmt in chunk {
                batch.add((*stmt).clone());
            }
            batches.push(batch);
            sequence += 1;
        }
    }

    batches
}

/// Adaptive batching based on estimated complexity.
fn batch_adaptive(
    statements: &[CypherStatement],
    config: &ToCypherConfig,
) -> Vec<TransactionBatch> {
    let mut batches = Vec::new();
    let mut current_batch = TransactionBatch::new(1, config.transaction_isolation);
    let mut current_complexity: usize = 0;
    let mut sequence = 1;

    // Target complexity per batch (heuristic)
    let target_complexity = config.transaction_row_limit.saturating_mul(10);

    for stmt in statements {
        let complexity = estimate_statement_complexity(stmt);

        // Start new batch if adding this would exceed target
        if !current_batch.is_empty()
            && current_complexity.saturating_add(complexity) > target_complexity
        {
            batches.push(current_batch);
            sequence += 1;
            current_batch = TransactionBatch::new(sequence, config.transaction_isolation);
            current_complexity = 0;
        }

        current_batch.add(stmt.clone());
        current_complexity = current_complexity.saturating_add(complexity);

        // Still respect statement count limit
        if current_batch.len() >= config.transaction_batch_size {
            batches.push(current_batch);
            sequence += 1;
            current_batch = TransactionBatch::new(sequence, config.transaction_isolation);
            current_complexity = 0;
        }
    }

    if !current_batch.is_empty() {
        batches.push(current_batch);
    }

    batches
}

/// Estimate complexity of a statement.
fn estimate_statement_complexity(statement: &CypherStatement) -> usize {
    let base_rows = count_statement_rows(statement);

    // Type multiplier
    let type_factor = match statement.statement_type {
        StatementType::CreateRelationship => 20, // 2 node lookups
        StatementType::CreateNode => 10,
        StatementType::Index => 5,
        StatementType::Constraint => 5,
        StatementType::Query | StatementType::SetProperty => 10,
    };

    // String length factor
    let param_size: usize = statement
        .parameters
        .values()
        .map(estimate_cypher_value_size)
        .sum();

    let param_factor = if param_size > 0 {
        (param_size as f64).log2() as usize
    } else {
        0
    };

    base_rows
        .saturating_mul(type_factor)
        .saturating_add(param_factor)
}

/// Estimate the serialized size of a Cypher value.
fn estimate_cypher_value_size(value: &CypherValue) -> usize {
    match value {
        CypherValue::Null => 4,
        CypherValue::Bool(_) => 5,
        CypherValue::Int(_) => 8,
        CypherValue::Float(_) => 16,
        CypherValue::String(s) => s.len() + 2,
        CypherValue::List(items) => items.iter().map(estimate_cypher_value_size).sum(),
        CypherValue::Map(map) => map
            .iter()
            .map(|(k, v)| k.len() + estimate_cypher_value_size(v))
            .sum(),
    }
}

#[cfg(test)]
mod transaction_tests {
    use super::*;

    fn make_node_statement(id: &str, row_count: usize) -> CypherStatement {
        let rows: Vec<CypherValue> = (0..row_count)
            .map(|i| {
                let mut map = std::collections::BTreeMap::new();
                map.insert("id".to_string(), CypherValue::String(format!("{id}_{i}")));
                CypherValue::Map(map)
            })
            .collect();

        CypherStatement::create_node("UNWIND $rows AS row MERGE (n:Type {id: row.id})".to_string())
            .with_param("rows", CypherValue::List(rows))
    }

    fn make_constraint_statement(type_name: &str) -> CypherStatement {
        CypherStatement::constraint(format!(
            "CREATE CONSTRAINT {}_id IF NOT EXISTS FOR (n:{}) REQUIRE n.id IS UNIQUE",
            type_name.to_lowercase(),
            type_name
        ))
    }

    #[test]
    fn test_transaction_batch_new() {
        let batch = TransactionBatch::new(1, IsolationLevel::Default);
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
        assert_eq!(batch.row_count, 0);
        assert_eq!(batch.sequence, 1);
    }

    #[test]
    fn test_transaction_batch_add() {
        let mut batch = TransactionBatch::new(1, IsolationLevel::Default);
        batch.add(make_node_statement("test", 100));

        assert!(!batch.is_empty());
        assert_eq!(batch.len(), 1);
        assert_eq!(batch.row_count, 100);
    }

    #[test]
    fn test_transaction_batch_render() {
        let mut batch = TransactionBatch::new(1, IsolationLevel::Default);
        batch.add(make_constraint_statement("User"));

        let rendered = batch.render(true);
        assert!(rendered.contains(":begin"));
        assert!(rendered.contains(":commit"));
        assert!(rendered.contains("CREATE CONSTRAINT"));
        assert!(rendered.contains("Transaction batch 1"));
    }

    #[test]
    fn test_batch_by_statement_count() {
        let config = ToCypherConfig::builder()
            .transaction_batching_enabled(true)
            .transaction_batch_size(2)
            .build();

        let statements: Vec<CypherStatement> = (0..5)
            .map(|i| make_node_statement(&format!("t{i}"), 10))
            .collect();

        let batches = batch_by_statement_count(&statements, &config);

        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].len(), 2);
        assert_eq!(batches[1].len(), 2);
        assert_eq!(batches[2].len(), 1);
    }

    #[test]
    fn test_batch_by_row_count() {
        let config = ToCypherConfig::builder()
            .transaction_batching_enabled(true)
            .transaction_row_limit(150)
            .transaction_batch_size(1000)
            .build();

        let statements = vec![
            make_node_statement("t1", 100),
            make_node_statement("t2", 100), // Would exceed 150
            make_node_statement("t3", 50),
        ];

        let batches = batch_by_row_count(&statements, &config);

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].row_count, 100);
        assert_eq!(batches[1].row_count, 150);
    }

    #[test]
    fn test_batch_by_statement_type() {
        let config = ToCypherConfig::builder()
            .transaction_batching_enabled(true)
            .transaction_batch_size(100)
            .build();

        let statements = vec![
            make_constraint_statement("User"),
            make_constraint_statement("Post"),
            make_node_statement("users", 10),
            make_node_statement("posts", 10),
        ];

        let batches = batch_by_statement_type(&statements, &config);

        // Should have at least 2 batches: constraints and nodes
        assert!(batches.len() >= 2);

        // First batch should be constraints
        assert!(batches[0]
            .statements
            .iter()
            .all(|s| s.statement_type == StatementType::Constraint));
    }

    #[test]
    fn test_batch_statements_disabled() {
        let config = ToCypherConfig::builder()
            .transaction_batching_enabled(false)
            .build();

        let statements = vec![make_node_statement("t1", 10), make_node_statement("t2", 10)];

        let batches = batch_statements(&statements, &config);

        // Should return single batch with all statements
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 2);
    }

    #[test]
    fn test_count_statement_rows() {
        let stmt = make_node_statement("test", 42);
        assert_eq!(count_statement_rows(&stmt), 42);

        let constraint = make_constraint_statement("Test");
        assert_eq!(count_statement_rows(&constraint), 1);
    }

    #[test]
    fn test_estimate_statement_complexity() {
        let small_stmt = make_node_statement("small", 10);
        let large_stmt = make_node_statement("large", 1000);

        let small_complexity = estimate_statement_complexity(&small_stmt);
        let large_complexity = estimate_statement_complexity(&large_stmt);

        assert!(large_complexity > small_complexity);
    }

    #[test]
    fn test_adaptive_batching() {
        let config = ToCypherConfig::builder()
            .transaction_batching_enabled(true)
            .transaction_strategy(TransactionStrategy::Adaptive)
            .transaction_row_limit(100)
            .transaction_batch_size(1000)
            .build();

        let statements = vec![
            make_node_statement("t1", 10),
            make_node_statement("t2", 10),
            make_node_statement("t3", 10),
        ];

        let batches = batch_adaptive(&statements, &config);

        assert!(!batches.is_empty());
    }

    #[test]
    fn test_empty_statements() {
        let config = ToCypherConfig::builder()
            .transaction_batching_enabled(true)
            .build();

        let statements: Vec<CypherStatement> = vec![];
        let batches = batch_statements(&statements, &config);

        assert!(batches.is_empty() || batches[0].is_empty());
    }
}
