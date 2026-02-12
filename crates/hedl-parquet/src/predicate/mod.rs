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

//! Predicate pushdown support for efficient Parquet filtering.
//!
//! This module provides predicate expressions that enable row group pruning
//! by leveraging Parquet column statistics (min/max values, null counts).
//!
//! # Performance
//!
//! Predicate pushdown can provide 10-100x performance improvement for selective
//! queries by eliminating unnecessary I/O and decompression.
//!
//! # Example
//!
//! ```
//! use hedl_parquet::predicate::{Predicate, PredicateValue};
//!
//! // Filter users where age = 25
//! let pred = Predicate::equal("age", PredicateValue::Int(25));
//!
//! // Filter users where age BETWEEN 30 AND 40
//! let pred = Predicate::between("age", PredicateValue::Int(30), PredicateValue::Int(40));
//!
//! // Combine predicates with AND
//! let pred = Predicate::and(vec![
//!     Predicate::equal("status", PredicateValue::String("active".into())),
//!     Predicate::greater_than("age", PredicateValue::Int(18)),
//! ]);
//! ```

mod evaluation;
mod pruning;
mod types;

// Re-export public types and functions
pub use evaluation::{evaluate_predicate, filter_batch};
pub use pruning::{
    extract_row_group_statistics, select_row_groups, ColumnStatistics, RowGroupStatistics,
};
pub use types::{Predicate, PredicateValue};

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_predicate_value_comparison() {
        // Int comparisons
        assert!(PredicateValue::Int(5).lt(&PredicateValue::Int(10)).unwrap());
        assert!(!PredicateValue::Int(10).lt(&PredicateValue::Int(5)).unwrap());
        assert!(PredicateValue::Int(5).le(&PredicateValue::Int(5)).unwrap());

        // String comparisons
        assert!(PredicateValue::String("a".into())
            .lt(&PredicateValue::String("b".into()))
            .unwrap());

        // Float comparisons
        assert!(PredicateValue::Float(1.5)
            .lt(&PredicateValue::Float(2.5))
            .unwrap());

        // Cross-type int/float
        assert!(PredicateValue::Int(5)
            .lt(&PredicateValue::Float(5.5))
            .unwrap());
    }

    #[test]
    fn test_predicate_constructors() {
        let p = Predicate::equal("age", PredicateValue::Int(25));
        assert!(matches!(p, Predicate::Equal(col, PredicateValue::Int(25)) if col == "age"));

        let p = Predicate::between("age", PredicateValue::Int(18), PredicateValue::Int(65));
        assert!(matches!(p, Predicate::Between(col, _, _) if col == "age"));

        let p = Predicate::and(vec![
            Predicate::equal("a", PredicateValue::Int(1)),
            Predicate::equal("b", PredicateValue::Int(2)),
        ]);
        assert!(matches!(p, Predicate::And(preds) if preds.len() == 2));
    }

    #[test]
    fn test_row_group_pruning_equal() {
        let mut stats = RowGroupStatistics {
            row_group_id: 0,
            total_rows: 1000,
            columns: HashMap::new(),
        };

        stats.columns.insert(
            "age".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::Int(20)),
                max_value: Some(PredicateValue::Int(40)),
                null_count: 0,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // Value in range - cannot skip
        let pred = Predicate::equal("age", PredicateValue::Int(25));
        assert!(!pred.can_skip_row_group(&stats));

        // Value below range - can skip
        let pred = Predicate::equal("age", PredicateValue::Int(10));
        assert!(pred.can_skip_row_group(&stats));

        // Value above range - can skip
        let pred = Predicate::equal("age", PredicateValue::Int(50));
        assert!(pred.can_skip_row_group(&stats));
    }

    #[test]
    fn test_row_group_pruning_range() {
        let mut stats = RowGroupStatistics {
            row_group_id: 0,
            total_rows: 1000,
            columns: HashMap::new(),
        };

        stats.columns.insert(
            "age".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::Int(30)),
                max_value: Some(PredicateValue::Int(50)),
                null_count: 0,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // Overlapping range - cannot skip
        let pred = Predicate::between("age", PredicateValue::Int(25), PredicateValue::Int(35));
        assert!(!pred.can_skip_row_group(&stats));

        // Range entirely below - can skip
        let pred = Predicate::between("age", PredicateValue::Int(10), PredicateValue::Int(20));
        assert!(pred.can_skip_row_group(&stats));

        // Range entirely above - can skip
        let pred = Predicate::between("age", PredicateValue::Int(60), PredicateValue::Int(70));
        assert!(pred.can_skip_row_group(&stats));
    }

    #[test]
    fn test_row_group_pruning_null_checks() {
        let mut stats = RowGroupStatistics {
            row_group_id: 0,
            total_rows: 1000,
            columns: HashMap::new(),
        };

        // Column with some nulls
        stats.columns.insert(
            "email".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::String("a@b.com".into())),
                max_value: Some(PredicateValue::String("z@b.com".into())),
                null_count: 100,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // IS NULL - cannot skip (there are nulls)
        let pred = Predicate::is_null("email");
        assert!(!pred.can_skip_row_group(&stats));

        // IS NOT NULL - cannot skip (not all are null)
        let pred = Predicate::is_not_null("email");
        assert!(!pred.can_skip_row_group(&stats));

        // Column with no nulls
        stats.columns.insert(
            "id".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::Int(1)),
                max_value: Some(PredicateValue::Int(1000)),
                null_count: 0,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // IS NULL on no-null column - can skip
        let pred = Predicate::is_null("id");
        assert!(pred.can_skip_row_group(&stats));

        // Column with all nulls
        stats.columns.insert(
            "optional".into(),
            ColumnStatistics {
                min_value: None,
                max_value: None,
                null_count: 1000,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // IS NOT NULL on all-null column - can skip
        let pred = Predicate::is_not_null("optional");
        assert!(pred.can_skip_row_group(&stats));
    }

    #[test]
    fn test_row_group_pruning_and() {
        let mut stats = RowGroupStatistics {
            row_group_id: 0,
            total_rows: 1000,
            columns: HashMap::new(),
        };

        stats.columns.insert(
            "age".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::Int(30)),
                max_value: Some(PredicateValue::Int(50)),
                null_count: 0,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        stats.columns.insert(
            "status".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::String("active".into())),
                max_value: Some(PredicateValue::String("pending".into())),
                null_count: 0,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // Both predicates could match - cannot skip
        let pred = Predicate::and(vec![
            Predicate::equal("age", PredicateValue::Int(35)),
            Predicate::equal("status", PredicateValue::String("active".into())),
        ]);
        assert!(!pred.can_skip_row_group(&stats));

        // One predicate fails (age outside range) - can skip
        let pred = Predicate::and(vec![
            Predicate::equal("age", PredicateValue::Int(10)), // Outside range
            Predicate::equal("status", PredicateValue::String("active".into())),
        ]);
        assert!(pred.can_skip_row_group(&stats));
    }

    #[test]
    fn test_row_group_pruning_or() {
        let mut stats = RowGroupStatistics {
            row_group_id: 0,
            total_rows: 1000,
            columns: HashMap::new(),
        };

        stats.columns.insert(
            "age".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::Int(30)),
                max_value: Some(PredicateValue::Int(50)),
                null_count: 0,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // One predicate matches - cannot skip
        let pred = Predicate::or(vec![
            Predicate::equal("age", PredicateValue::Int(10)), // Outside range
            Predicate::equal("age", PredicateValue::Int(35)), // Inside range
        ]);
        assert!(!pred.can_skip_row_group(&stats));

        // Both predicates fail - can skip
        let pred = Predicate::or(vec![
            Predicate::equal("age", PredicateValue::Int(10)), // Outside range
            Predicate::equal("age", PredicateValue::Int(60)), // Also outside range
        ]);
        assert!(pred.can_skip_row_group(&stats));
    }

    #[test]
    fn test_evaluate_predicate_int() {
        let schema = Arc::new(Schema::new(vec![Field::new("age", DataType::Int64, false)]));

        let age_array = Int64Array::from(vec![20, 25, 30, 35, 40]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(age_array)]).unwrap();

        // Test equality
        let pred = Predicate::equal("age", PredicateValue::Int(25));
        let mask = evaluate_predicate(&batch, &pred).unwrap();
        assert!(!mask.value(0));
        assert!(mask.value(1));
        assert!(!mask.value(2));

        // Test greater than
        let pred = Predicate::greater_than("age", PredicateValue::Int(30));
        let mask = evaluate_predicate(&batch, &pred).unwrap();
        assert!(!mask.value(0));
        assert!(!mask.value(1));
        assert!(!mask.value(2));
        assert!(mask.value(3));
        assert!(mask.value(4));

        // Test between
        let pred = Predicate::between("age", PredicateValue::Int(25), PredicateValue::Int(35));
        let mask = evaluate_predicate(&batch, &pred).unwrap();
        assert!(!mask.value(0));
        assert!(mask.value(1));
        assert!(mask.value(2));
        assert!(mask.value(3));
        assert!(!mask.value(4));
    }

    #[test]
    fn test_evaluate_predicate_string() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "status",
            DataType::Utf8,
            false,
        )]));

        let status_array = StringArray::from(vec!["active", "inactive", "pending", "active"]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(status_array)]).unwrap();

        let pred = Predicate::equal("status", PredicateValue::String("active".into()));
        let mask = evaluate_predicate(&batch, &pred).unwrap();
        assert!(mask.value(0));
        assert!(!mask.value(1));
        assert!(!mask.value(2));
        assert!(mask.value(3));
    }

    #[test]
    fn test_evaluate_predicate_in() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "status",
            DataType::Utf8,
            false,
        )]));

        let status_array = StringArray::from(vec!["active", "inactive", "pending", "suspended"]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(status_array)]).unwrap();

        let pred = Predicate::in_set(
            "status",
            vec![
                PredicateValue::String("active".into()),
                PredicateValue::String("pending".into()),
            ],
        );
        let mask = evaluate_predicate(&batch, &pred).unwrap();
        assert!(mask.value(0)); // active
        assert!(!mask.value(1)); // inactive
        assert!(mask.value(2)); // pending
        assert!(!mask.value(3)); // suspended
    }

    #[test]
    fn test_filter_batch() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let id_array = Int64Array::from(vec![1, 2, 3, 4, 5]);
        let name_array = StringArray::from(vec!["alice", "bob", "charlie", "diana", "eve"]);
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(id_array), Arc::new(name_array)]).unwrap();

        let pred = Predicate::greater_than("id", PredicateValue::Int(2));
        let filtered = filter_batch(&batch, &pred).unwrap();

        assert_eq!(filtered.num_rows(), 3);

        let id_col = filtered
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        assert_eq!(id_col.value(0), 3);
        assert_eq!(id_col.value(1), 4);
        assert_eq!(id_col.value(2), 5);
    }

    #[test]
    fn test_predicate_display() {
        let pred = Predicate::equal("age", PredicateValue::Int(25));
        assert_eq!(pred.to_string(), "age = 25");

        let pred = Predicate::between("age", PredicateValue::Int(18), PredicateValue::Int(65));
        assert_eq!(pred.to_string(), "age BETWEEN 18 AND 65");

        let pred = Predicate::and(vec![
            Predicate::equal("status", PredicateValue::String("active".into())),
            Predicate::greater_than("age", PredicateValue::Int(18)),
        ]);
        assert_eq!(pred.to_string(), "(status = 'active') AND (age > 18)");
    }
}
