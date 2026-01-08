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

//! Comprehensive data generation module for HEDL benchmarks.
//!
//! Provides modular, DRY-compliant generators organized by structure type:
//!
//! - **simple**: Flat and basic structures
//! - **hierarchical**: Nested and tree structures
//! - **graph**: Cross-references and graph structures
//! - **specialized**: Domain-specific generators (tensors, rows, etc.)
//! - **config**: Configuration types and complexity levels
//! - **validation**: Validation utilities for generated data

pub mod config;
pub mod graph;
pub mod hierarchical;
pub mod simple;
pub mod specialized;
pub mod validation;

// Re-export commonly used types
pub use config::{ComplexityLevel, GeneratorConfig, SizeDistribution};
pub use validation::{validate_generated, verify_complexity};

// Re-export generator functions for convenience
pub use simple::{
    generate_analytics_simple, generate_events_simple, generate_flat_struct, generate_list_simple,
    generate_nested_simple, generate_products_simple, generate_users_simple,
};

pub use hierarchical::{
    generate_balanced_tree, generate_blog_hierarchy, generate_custom_nested, generate_deep_nesting,
    generate_order_hierarchy, generate_org_hierarchy, generate_wide_tree,
};

pub use graph::{
    generate_bidirectional_graph, generate_complex_graph, generate_dag, generate_linked_list,
    generate_reference_graph, generate_tree_graph,
};

pub use specialized::{
    generate_csv_like, generate_ditto_data, generate_key_value, generate_row_data,
    generate_sparse_matrix, generate_tensor_data, generate_time_series, generate_wide_rows,
};
