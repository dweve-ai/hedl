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

//! Tensor literal parsing for HEDL format.
//!
//! Tensors are multi-dimensional numerical arrays like `[1, 2, 3]` or `[[1, 2], [3, 4]]`.
//!
//! # Examples
//!
//! ```
//! use hedl_core::lex::{parse_tensor, is_tensor_literal, Tensor};
//!
//! // Parse a 1D tensor
//! let tensor = parse_tensor("[1, 2, 3]").unwrap();
//! assert_eq!(tensor.shape(), vec![3]);
//! assert_eq!(tensor.flatten(), vec![1.0, 2.0, 3.0]);
//!
//! // Parse a 2D tensor (matrix)
//! let matrix = parse_tensor("[[1, 2], [3, 4]]").unwrap();
//! assert_eq!(matrix.shape(), vec![2, 2]);
//!
//! // Quick validation
//! assert!(is_tensor_literal("[1, 2, 3]"));
//! assert!(!is_tensor_literal("not a tensor"));
//! ```
//!
//! # Security
//!
//! This module includes multiple security protections:
//! - Maximum recursion depth of 100 to prevent stack overflow
//! - Maximum element count of 10 million to prevent memory exhaustion
//! - Rejection of NaN and Infinity values for predictable behavior
//! - Error message truncation to prevent DoS attacks

use crate::lex::error::LexError;

/// Maximum recursion depth for tensor parsing (prevents stack overflow).
const MAX_RECURSION_DEPTH: usize = 100;

/// Maximum number of elements in a tensor (prevents memory exhaustion).
const MAX_TENSOR_ELEMENTS: usize = 10_000_000;

/// A multi-dimensional numerical array.
///
/// Tensors can be scalars (single values) or arrays of nested tensors.
/// All leaf values are stored as f64.
///
/// # Examples
///
/// ```
/// use hedl_core::lex::{parse_tensor, Tensor};
///
/// // Scalar
/// let scalar = Tensor::Scalar(42.0);
/// assert_eq!(scalar.shape(), vec![]);
/// assert_eq!(scalar.flatten(), vec![42.0]);
///
/// // 1D array
/// let vec = parse_tensor("[1, 2, 3]").unwrap();
/// assert_eq!(vec.shape(), vec![3]);
/// assert!(vec.is_integer());
///
/// // 2D matrix
/// let matrix = parse_tensor("[[1.5, 2.5], [3.5, 4.5]]").unwrap();
/// assert_eq!(matrix.shape(), vec![2, 2]);
/// assert!(!matrix.is_integer());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Tensor {
    /// A scalar number (integer or float).
    Scalar(f64),
    /// A nested array of tensors.
    Array(Vec<Tensor>),
}

impl std::fmt::Display for Tensor {
    /// Formats the tensor as a parseable HEDL tensor literal.
    ///
    /// This produces output that can be parsed back by `parse_tensor()`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tensor::Scalar(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    if *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                        write!(f, "{}", *n as i64)
                    } else {
                        write!(f, "{}", n)
                    }
                } else {
                    write!(f, "{}", n)
                }
            }
            Tensor::Array(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
        }
    }
}

impl Tensor {
    /// Returns `true` if this tensor contains only integers (no decimal points).
    ///
    /// A number is considered an integer if its fractional part is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::{parse_tensor, Tensor};
    ///
    /// let integers = parse_tensor("[1, 2, 3]").unwrap();
    /// assert!(integers.is_integer());
    ///
    /// let floats = parse_tensor("[1.5, 2.5]").unwrap();
    /// assert!(!floats.is_integer());
    /// ```
    pub fn is_integer(&self) -> bool {
        match self {
            Tensor::Scalar(n) => n.fract() == 0.0,
            Tensor::Array(items) => items.iter().all(|t| t.is_integer()),
        }
    }

    /// Returns the shape of the tensor as a vector of dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::{parse_tensor, Tensor};
    ///
    /// let scalar = Tensor::Scalar(42.0);
    /// assert_eq!(scalar.shape(), vec![]);
    ///
    /// let vec = parse_tensor("[1, 2, 3]").unwrap();
    /// assert_eq!(vec.shape(), vec![3]);
    ///
    /// let matrix = parse_tensor("[[1, 2], [3, 4]]").unwrap();
    /// assert_eq!(matrix.shape(), vec![2, 2]);
    /// ```
    pub fn shape(&self) -> Vec<usize> {
        match self {
            Tensor::Scalar(_) => vec![],
            Tensor::Array(items) => {
                if items.is_empty() {
                    vec![0]
                } else {
                    let mut shape = vec![items.len()];
                    shape.extend(items[0].shape());
                    shape
                }
            }
        }
    }

    /// Flattens the tensor into a 1D vector of f64 values in row-major order.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::parse_tensor;
    ///
    /// let matrix = parse_tensor("[[1, 2], [3, 4]]").unwrap();
    /// assert_eq!(matrix.flatten(), vec![1.0, 2.0, 3.0, 4.0]);
    /// ```
    pub fn flatten(&self) -> Vec<f64> {
        let capacity = self.count_elements();
        let mut result = Vec::with_capacity(capacity);
        self.flatten_into(&mut result);
        result
    }

    /// Counts the total number of scalar elements.
    fn count_elements(&self) -> usize {
        match self {
            Tensor::Scalar(_) => 1,
            Tensor::Array(items) => items.iter().map(|t| t.count_elements()).sum(),
        }
    }

    /// Flattens into a pre-allocated vector.
    fn flatten_into(&self, result: &mut Vec<f64>) {
        match self {
            Tensor::Scalar(n) => result.push(*n),
            Tensor::Array(items) => {
                for item in items {
                    item.flatten_into(result);
                }
            }
        }
    }

    /// Returns `true` if this is a scalar value.
    #[inline]
    pub fn is_scalar(&self) -> bool {
        matches!(self, Tensor::Scalar(_))
    }

    /// Returns `true` if this is an array.
    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Tensor::Array(_))
    }

    /// Returns the number of dimensions (0 for scalar).
    #[inline]
    pub fn ndim(&self) -> usize {
        self.shape().len()
    }

    /// Returns the total number of elements.
    #[inline]
    pub fn len(&self) -> usize {
        self.count_elements()
    }

    /// Returns `true` if the tensor has no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            Tensor::Scalar(_) => false,
            Tensor::Array(items) => items.is_empty(),
        }
    }
}

/// Checks if a string looks like it could be a tensor literal.
///
/// This is a quick check that doesn't fully validate - use `parse_tensor` for that.
///
/// # Examples
///
/// ```
/// use hedl_core::lex::is_tensor_literal;
///
/// assert!(is_tensor_literal("[1, 2, 3]"));
/// assert!(is_tensor_literal("[[1, 2], [3, 4]]"));
/// assert!(!is_tensor_literal("hello"));
/// assert!(!is_tensor_literal("@reference"));
/// ```
#[inline]
pub fn is_tensor_literal(s: &str) -> bool {
    let s = s.trim();
    let bytes = s.as_bytes();
    if bytes.first() != Some(&b'[') || bytes.last() != Some(&b']') {
        return false;
    }

    let mut depth: i32 = 0;
    for &b in bytes {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            b'0'..=b'9' | b'.' | b'-' | b',' | b' ' | b'\t' => {}
            _ => return false,
        }
    }

    depth == 0
}

/// Parses a tensor literal string into a `Tensor` structure.
///
/// # Examples
///
/// ```
/// use hedl_core::lex::parse_tensor;
///
/// // Parse 1D tensor
/// let t = parse_tensor("[1, 2, 3]").unwrap();
/// assert_eq!(t.shape(), vec![3]);
///
/// // Parse 2D tensor
/// let t = parse_tensor("[[1, 2], [3, 4]]").unwrap();
/// assert_eq!(t.shape(), vec![2, 2]);
///
/// // Parse with floats
/// let t = parse_tensor("[1.5, 2.5]").unwrap();
/// assert!(!t.is_integer());
/// ```
///
/// # Errors
///
/// Returns error for:
/// - Unbalanced brackets
/// - Empty tensor
/// - Invalid numbers
/// - Inconsistent dimensions
/// - Exceeding security limits
pub fn parse_tensor(s: &str) -> Result<Tensor, LexError> {
    let s = s.trim();
    if !s.starts_with('[') {
        return Err(LexError::UnexpectedChar(s.chars().next().unwrap_or(' ')));
    }

    let (tensor, remaining) = parse_tensor_inner(s, 0)?;

    if !remaining.trim().is_empty() {
        return Err(LexError::UnexpectedChar(
            remaining.trim().chars().next().unwrap_or('?'),
        ));
    }

    // Validate total element count
    let element_count = tensor.count_elements();
    if element_count > MAX_TENSOR_ELEMENTS {
        return Err(LexError::InvalidStructure(format!(
            "Tensor element count exceeds maximum: {} (max: {})",
            element_count, MAX_TENSOR_ELEMENTS
        )));
    }

    Ok(tensor)
}

/// Estimates array size by counting commas.
fn estimate_array_size(s: &str) -> usize {
    let mut depth = 0;
    let mut comma_count = 0;

    for ch in s.chars() {
        match ch {
            '[' => depth += 1,
            ']' => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            ',' if depth == 0 => comma_count += 1,
            _ => {}
        }
    }

    if comma_count > 0 {
        comma_count + 1
    } else {
        2
    }
}

fn parse_tensor_inner(s: &str, depth: usize) -> Result<(Tensor, &str), LexError> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(LexError::InvalidStructure(format!(
            "Recursion depth exceeded (max: {})",
            MAX_RECURSION_DEPTH
        )));
    }
    let s = s.trim();

    if let Some(remaining_str) = s.strip_prefix('[') {
        let mut remaining = remaining_str;
        let estimated_capacity = estimate_array_size(remaining_str);
        let mut items = Vec::with_capacity(estimated_capacity);

        loop {
            remaining = remaining.trim_start();

            if remaining.is_empty() {
                return Err(LexError::UnbalancedBrackets);
            }

            if remaining.starts_with(']') {
                remaining = &remaining[1..];
                break;
            }

            if !items.is_empty() {
                if !remaining.starts_with(',') {
                    return Err(LexError::UnexpectedChar(
                        remaining.chars().next().unwrap_or('?'),
                    ));
                }
                remaining = remaining[1..].trim_start();
            }

            if remaining.starts_with(']') {
                remaining = &remaining[1..];
                break;
            }

            if remaining.starts_with('[') {
                let (tensor, rest) = parse_tensor_inner(remaining, depth + 1)?;
                items.push(tensor);
                remaining = rest;
            } else {
                let (num, rest) = parse_number(remaining)?;
                items.push(Tensor::Scalar(num));
                remaining = rest;
            }
        }

        if items.is_empty() {
            return Err(LexError::EmptyTensor);
        }

        // Validate consistent dimensions
        if items.len() > 1 {
            let first_shape = items[0].shape();
            for item in &items[1..] {
                if item.shape() != first_shape {
                    return Err(LexError::InconsistentDimensions);
                }
            }
        }

        Ok((Tensor::Array(items), remaining))
    } else {
        let (num, rest) = parse_number(s)?;
        Ok((Tensor::Scalar(num), rest))
    }
}

fn parse_number(s: &str) -> Result<(f64, &str), LexError> {
    let s = s.trim_start();
    let bytes = s.as_bytes();

    let mut end = 0;
    let mut has_dot = false;

    if bytes.first() == Some(&b'-') {
        end = 1;
    }

    while end < bytes.len() {
        match bytes[end] {
            b'0'..=b'9' => end += 1,
            b'.' if !has_dot => {
                has_dot = true;
                end += 1;
            }
            _ => break,
        }
    }

    if end == 0 || (end == 1 && bytes[0] == b'-') {
        return Err(LexError::InvalidNumber(
            s.chars().take(10).collect::<String>(),
        ));
    }

    let num_str = &s[..end];
    let num: f64 = num_str.parse().map_err(|_| {
        let context = if num_str.len() > 80 {
            format!("{}...", &num_str[..80])
        } else {
            num_str.to_string()
        };
        LexError::InvalidNumber(context)
    })?;

    if !num.is_finite() {
        return Err(LexError::InvalidNumber(format!(
            "{} (non-finite values not allowed)",
            num_str
        )));
    }

    Ok((num, &s[end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== is_tensor_literal tests ====================

    #[test]
    fn test_is_tensor_literal_valid() {
        assert!(is_tensor_literal("[1, 2, 3]"));
        assert!(is_tensor_literal("[[1, 2], [3, 4]]"));
        assert!(is_tensor_literal("[1.5, 2.5]"));
        assert!(is_tensor_literal("[-1, -2]"));
        assert!(is_tensor_literal("  [1, 2, 3]  "));
    }

    #[test]
    fn test_is_tensor_literal_invalid() {
        assert!(!is_tensor_literal("hello"));
        assert!(!is_tensor_literal("@reference"));
        assert!(!is_tensor_literal("123"));
        assert!(!is_tensor_literal(""));
        assert!(!is_tensor_literal("[1, 2"));
        assert!(!is_tensor_literal("[a, b]"));
    }

    // ==================== parse_tensor tests ====================

    #[test]
    fn test_parse_1d() {
        let t = parse_tensor("[1, 2, 3]").unwrap();
        assert_eq!(t.shape(), vec![3]);
        assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_parse_2d() {
        let t = parse_tensor("[[1, 2], [3, 4]]").unwrap();
        assert_eq!(t.shape(), vec![2, 2]);
        assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_parse_floats() {
        let t = parse_tensor("[1.5, 2.5, 3.5]").unwrap();
        assert_eq!(t.flatten(), vec![1.5, 2.5, 3.5]);
        assert!(!t.is_integer());
    }

    #[test]
    fn test_parse_negatives() {
        let t = parse_tensor("[-1, -2, -3]").unwrap();
        assert_eq!(t.flatten(), vec![-1.0, -2.0, -3.0]);
    }

    #[test]
    fn test_parse_trailing_comma() {
        let t = parse_tensor("[1, 2, 3,]").unwrap();
        assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0]);
    }

    // ==================== Error tests ====================

    #[test]
    fn test_empty_tensor_error() {
        assert!(matches!(parse_tensor("[]"), Err(LexError::EmptyTensor)));
    }

    #[test]
    fn test_unbalanced_brackets_error() {
        assert!(matches!(
            parse_tensor("[1, 2"),
            Err(LexError::UnbalancedBrackets)
        ));
    }

    #[test]
    fn test_inconsistent_dimensions_error() {
        assert!(matches!(
            parse_tensor("[[1, 2], [3]]"),
            Err(LexError::InconsistentDimensions)
        ));
    }

    #[test]
    fn test_invalid_number_error() {
        assert!(matches!(
            parse_tensor("[abc]"),
            Err(LexError::InvalidNumber(_))
        ));
    }

    // ==================== Tensor struct tests ====================

    #[test]
    fn test_tensor_display() {
        let t = parse_tensor("[1, 2, 3]").unwrap();
        assert_eq!(format!("{}", t), "[1, 2, 3]");

        let t = parse_tensor("[[1, 2], [3, 4]]").unwrap();
        assert_eq!(format!("{}", t), "[[1, 2], [3, 4]]");
    }

    #[test]
    fn test_tensor_methods() {
        let scalar = Tensor::Scalar(42.0);
        assert!(scalar.is_scalar());
        assert!(!scalar.is_array());
        assert_eq!(scalar.ndim(), 0);
        assert_eq!(scalar.len(), 1);
        assert!(!scalar.is_empty());

        let array = parse_tensor("[1, 2, 3]").unwrap();
        assert!(!array.is_scalar());
        assert!(array.is_array());
        assert_eq!(array.ndim(), 1);
        assert_eq!(array.len(), 3);
        assert!(!array.is_empty());
    }

    #[test]
    fn test_tensor_equality() {
        let t1 = parse_tensor("[1, 2, 3]").unwrap();
        let t2 = parse_tensor("[1, 2, 3]").unwrap();
        assert_eq!(t1, t2);

        let t3 = parse_tensor("[1, 2, 4]").unwrap();
        assert_ne!(t1, t3);
    }

    #[test]
    fn test_tensor_clone() {
        let t1 = parse_tensor("[[1, 2], [3, 4]]").unwrap();
        let t2 = t1.clone();
        assert_eq!(t1, t2);
    }

    // ==================== Round-trip tests ====================

    #[test]
    fn test_display_roundtrip() {
        let original = parse_tensor("[1, 2, 3]").unwrap();
        let serialized = original.to_string();
        let parsed = parse_tensor(&serialized).unwrap();
        assert_eq!(original, parsed);

        let original = parse_tensor("[[1.5, 2.5], [3.5, 4.5]]").unwrap();
        let serialized = original.to_string();
        let parsed = parse_tensor(&serialized).unwrap();
        assert_eq!(original, parsed);
    }
}
