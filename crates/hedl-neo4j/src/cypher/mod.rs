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

//! Cypher query building utilities.
//!
//! This module provides types and functions for building and manipulating
//! Cypher queries safely.

pub mod escape;
pub mod statements;

pub use escape::{
    escape_identifier, escape_label, escape_relationship_type, escape_string, is_valid_identifier,
    normalize_unicode, quote_string, to_identifier, to_relationship_type, validate_identifier,
    validate_string_length,
};

pub use statements::{CypherScript, CypherStatement, CypherValue, StatementType};
