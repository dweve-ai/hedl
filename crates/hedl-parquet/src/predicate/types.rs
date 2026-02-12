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

//! Predicate types and constructors.

/// A value used in predicate comparisons.
///
/// This is separate from `hedl_core::Value` to support efficient comparison
/// with Parquet column statistics without requiring full HEDL value semantics.
#[derive(Debug, Clone, PartialEq)]
pub enum PredicateValue {
    /// Null value for IS NULL / IS NOT NULL comparisons.
    Null,
    /// Boolean value.
    Bool(bool),
    /// 64-bit signed integer.
    Int(i64),
    /// 64-bit floating point.
    Float(f64),
    /// String value.
    String(String),
}

impl PredicateValue {
    /// Compare two predicate values.
    ///
    /// Returns None if types are incompatible.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => Some(a.cmp(b)),
            (Self::Float(a), Self::Float(b)) => a.partial_cmp(b),
            (Self::String(a), Self::String(b)) => Some(a.cmp(b)),
            (Self::Bool(a), Self::Bool(b)) => Some(a.cmp(b)),
            // Cross-type: int can be compared to float
            (Self::Int(a), Self::Float(b)) => (*a as f64).partial_cmp(b),
            (Self::Float(a), Self::Int(b)) => a.partial_cmp(&(*b as f64)),
            _ => None,
        }
    }

    /// Check if this value is less than another.
    #[must_use]
    pub fn lt(&self, other: &Self) -> Option<bool> {
        self.partial_cmp(other)
            .map(|o| o == std::cmp::Ordering::Less)
    }

    /// Check if this value is less than or equal to another.
    #[must_use]
    pub fn le(&self, other: &Self) -> Option<bool> {
        self.partial_cmp(other)
            .map(|o| o != std::cmp::Ordering::Greater)
    }

    /// Check if this value is greater than another.
    #[must_use]
    pub fn gt(&self, other: &Self) -> Option<bool> {
        self.partial_cmp(other)
            .map(|o| o == std::cmp::Ordering::Greater)
    }

    /// Check if this value is greater than or equal to another.
    #[must_use]
    pub fn ge(&self, other: &Self) -> Option<bool> {
        self.partial_cmp(other)
            .map(|o| o != std::cmp::Ordering::Less)
    }
}

impl std::fmt::Display for PredicateValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "NULL"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "'{s}'"),
        }
    }
}

/// Predicate expression for filtering Parquet data.
///
/// Predicates enable efficient row group pruning by leveraging Parquet column
/// statistics. Only row groups that might contain matching rows are read.
#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    /// Equality comparison: column = value
    Equal(String, PredicateValue),
    /// Inequality comparison: column != value
    NotEqual(String, PredicateValue),
    /// Less than comparison: column < value
    LessThan(String, PredicateValue),
    /// Less than or equal comparison: column <= value
    LessThanOrEqual(String, PredicateValue),
    /// Greater than comparison: column > value
    GreaterThan(String, PredicateValue),
    /// Greater than or equal comparison: column >= value
    GreaterThanOrEqual(String, PredicateValue),
    /// Range comparison: column BETWEEN min AND max (inclusive)
    Between(String, PredicateValue, PredicateValue),
    /// Set membership: column IN (value1, value2, ...)
    In(String, Vec<PredicateValue>),
    /// Negated set membership: column NOT IN (value1, value2, ...)
    NotIn(String, Vec<PredicateValue>),
    /// Null check: column IS NULL
    IsNull(String),
    /// Not null check: column IS NOT NULL
    IsNotNull(String),
    /// Logical AND: all predicates must match
    And(Vec<Predicate>),
    /// Logical OR: any predicate must match
    Or(Vec<Predicate>),
    /// Logical NOT: negate the predicate
    Not(Box<Predicate>),
}

impl Predicate {
    // ==================== Constructors ====================

    /// Create an equality predicate: column = value
    pub fn equal(column: impl Into<String>, value: PredicateValue) -> Self {
        Self::Equal(column.into(), value)
    }

    /// Create an inequality predicate: column != value
    pub fn not_equal(column: impl Into<String>, value: PredicateValue) -> Self {
        Self::NotEqual(column.into(), value)
    }

    /// Create a less than predicate: column < value
    pub fn less_than(column: impl Into<String>, value: PredicateValue) -> Self {
        Self::LessThan(column.into(), value)
    }

    /// Create a less than or equal predicate: column <= value
    pub fn less_than_or_equal(column: impl Into<String>, value: PredicateValue) -> Self {
        Self::LessThanOrEqual(column.into(), value)
    }

    /// Create a greater than predicate: column > value
    pub fn greater_than(column: impl Into<String>, value: PredicateValue) -> Self {
        Self::GreaterThan(column.into(), value)
    }

    /// Create a greater than or equal predicate: column >= value
    pub fn greater_than_or_equal(column: impl Into<String>, value: PredicateValue) -> Self {
        Self::GreaterThanOrEqual(column.into(), value)
    }

    /// Create a range predicate: column BETWEEN min AND max (inclusive)
    pub fn between(column: impl Into<String>, min: PredicateValue, max: PredicateValue) -> Self {
        Self::Between(column.into(), min, max)
    }

    /// Create a set membership predicate: column IN (values...)
    pub fn in_set(column: impl Into<String>, values: Vec<PredicateValue>) -> Self {
        Self::In(column.into(), values)
    }

    /// Create a negated set membership predicate: column NOT IN (values...)
    pub fn not_in_set(column: impl Into<String>, values: Vec<PredicateValue>) -> Self {
        Self::NotIn(column.into(), values)
    }

    /// Create a null check predicate: column IS NULL
    pub fn is_null(column: impl Into<String>) -> Self {
        Self::IsNull(column.into())
    }

    /// Create a not null check predicate: column IS NOT NULL
    pub fn is_not_null(column: impl Into<String>) -> Self {
        Self::IsNotNull(column.into())
    }

    /// Create a logical AND of predicates.
    ///
    /// All predicates must match for a row to pass.
    #[must_use]
    pub fn and(predicates: Vec<Predicate>) -> Self {
        Self::And(predicates)
    }

    /// Create a logical OR of predicates.
    ///
    /// Any predicate matching is sufficient for a row to pass.
    #[must_use]
    pub fn or(predicates: Vec<Predicate>) -> Self {
        Self::Or(predicates)
    }
}

impl std::fmt::Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Equal(col, val) => write!(f, "{col} = {val}"),
            Self::NotEqual(col, val) => write!(f, "{col} != {val}"),
            Self::LessThan(col, val) => write!(f, "{col} < {val}"),
            Self::LessThanOrEqual(col, val) => write!(f, "{col} <= {val}"),
            Self::GreaterThan(col, val) => write!(f, "{col} > {val}"),
            Self::GreaterThanOrEqual(col, val) => write!(f, "{col} >= {val}"),
            Self::Between(col, min, max) => write!(f, "{col} BETWEEN {min} AND {max}"),
            Self::In(col, vals) => {
                let vals_str: Vec<String> =
                    vals.iter().map(std::string::ToString::to_string).collect();
                write!(f, "{} IN ({})", col, vals_str.join(", "))
            }
            Self::NotIn(col, vals) => {
                let vals_str: Vec<String> =
                    vals.iter().map(std::string::ToString::to_string).collect();
                write!(f, "{} NOT IN ({})", col, vals_str.join(", "))
            }
            Self::IsNull(col) => write!(f, "{col} IS NULL"),
            Self::IsNotNull(col) => write!(f, "{col} IS NOT NULL"),
            Self::And(preds) => {
                let strs: Vec<String> = preds.iter().map(|p| format!("({p})")).collect();
                write!(f, "{}", strs.join(" AND "))
            }
            Self::Or(preds) => {
                let strs: Vec<String> = preds.iter().map(|p| format!("({p})")).collect();
                write!(f, "{}", strs.join(" OR "))
            }
            Self::Not(pred) => write!(f, "NOT ({pred})"),
        }
    }
}

impl std::ops::Not for Predicate {
    type Output = Self;

    /// Create a logical NOT of a predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_parquet::predicate::Predicate;
    ///
    /// let pred = !Predicate::is_null("email");
    /// ```
    fn not(self) -> Self::Output {
        Self::Not(Box::new(self))
    }
}
