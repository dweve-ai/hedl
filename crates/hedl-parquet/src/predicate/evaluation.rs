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

//! In-memory predicate evaluation on Arrow record batches.

use std::sync::Arc;

use arrow::array::{
    Array, BooleanArray, Float32Array, Float64Array, Int16Array, Int32Array, Int64Array, Int8Array,
    RecordBatch, Scalar, StringArray, UInt16Array, UInt32Array, UInt64Array, UInt8Array,
};
use arrow::compute;
use arrow::compute::kernels::cmp;
use arrow::datatypes::DataType;

use hedl_core::{HedlError, HedlErrorKind};

use super::types::{Predicate, PredicateValue};

/// Evaluate a predicate on a record batch, returning a boolean mask.
///
/// Each element in the returned array indicates whether the corresponding
/// row in the batch matches the predicate.
pub fn evaluate_predicate(
    batch: &RecordBatch,
    predicate: &Predicate,
) -> Result<BooleanArray, HedlError> {
    match predicate {
        Predicate::Equal(col, val) => evaluate_comparison(batch, col, val, ComparisonOp::Eq),
        Predicate::NotEqual(col, val) => evaluate_comparison(batch, col, val, ComparisonOp::Ne),
        Predicate::LessThan(col, val) => evaluate_comparison(batch, col, val, ComparisonOp::Lt),
        Predicate::LessThanOrEqual(col, val) => {
            evaluate_comparison(batch, col, val, ComparisonOp::Le)
        }
        Predicate::GreaterThan(col, val) => evaluate_comparison(batch, col, val, ComparisonOp::Gt),
        Predicate::GreaterThanOrEqual(col, val) => {
            evaluate_comparison(batch, col, val, ComparisonOp::Ge)
        }
        Predicate::Between(col, min_val, max_val) => {
            // BETWEEN is: col >= min AND col <= max
            let ge_min = evaluate_comparison(batch, col, min_val, ComparisonOp::Ge)?;
            let le_max = evaluate_comparison(batch, col, max_val, ComparisonOp::Le)?;
            compute::and(&ge_min, &le_max).map_err(|e| {
                HedlError::new(
                    HedlErrorKind::IO,
                    format!("Failed to combine predicates: {e}"),
                    0,
                )
            })
        }
        Predicate::In(col, values) => evaluate_in(batch, col, values, false),
        Predicate::NotIn(col, values) => evaluate_in(batch, col, values, true),
        Predicate::IsNull(col) => evaluate_is_null(batch, col, true),
        Predicate::IsNotNull(col) => evaluate_is_null(batch, col, false),
        Predicate::And(predicates) => {
            if predicates.is_empty() {
                // Empty AND matches everything
                return Ok(BooleanArray::from(vec![true; batch.num_rows()]));
            }
            let mut result = evaluate_predicate(batch, &predicates[0])?;
            for pred in &predicates[1..] {
                let mask = evaluate_predicate(batch, pred)?;
                result = compute::and(&result, &mask).map_err(|e| {
                    HedlError::new(
                        HedlErrorKind::IO,
                        format!("Failed to combine predicates: {e}"),
                        0,
                    )
                })?;
            }
            Ok(result)
        }
        Predicate::Or(predicates) => {
            if predicates.is_empty() {
                // Empty OR matches nothing
                return Ok(BooleanArray::from(vec![false; batch.num_rows()]));
            }
            let mut result = evaluate_predicate(batch, &predicates[0])?;
            for pred in &predicates[1..] {
                let mask = evaluate_predicate(batch, pred)?;
                result = compute::or(&result, &mask).map_err(|e| {
                    HedlError::new(
                        HedlErrorKind::IO,
                        format!("Failed to combine predicates: {e}"),
                        0,
                    )
                })?;
            }
            Ok(result)
        }
        Predicate::Not(pred) => {
            let mask = evaluate_predicate(batch, pred)?;
            compute::not(&mask).map_err(|e| {
                HedlError::new(
                    HedlErrorKind::IO,
                    format!("Failed to negate predicate: {e}"),
                    0,
                )
            })
        }
    }
}

/// Apply a predicate filter to a record batch, returning filtered batch.
pub fn filter_batch(batch: &RecordBatch, predicate: &Predicate) -> Result<RecordBatch, HedlError> {
    let mask = evaluate_predicate(batch, predicate)?;

    // Use Arrow's filter kernel to select matching rows
    let filtered_columns: Result<Vec<Arc<dyn Array>>, _> = batch
        .columns()
        .iter()
        .map(|col| compute::filter(col, &mask))
        .collect();

    let filtered_columns = filtered_columns.map_err(|e| {
        HedlError::new(HedlErrorKind::IO, format!("Failed to filter batch: {e}"), 0)
    })?;

    RecordBatch::try_new(batch.schema(), filtered_columns).map_err(|e| {
        HedlError::new(
            HedlErrorKind::IO,
            format!("Failed to create filtered batch: {e}"),
            0,
        )
    })
}

// ==================== Helper Functions ====================

#[derive(Clone, Copy)]
enum ComparisonOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

fn evaluate_comparison(
    batch: &RecordBatch,
    col_name: &str,
    value: &PredicateValue,
    op: ComparisonOp,
) -> Result<BooleanArray, HedlError> {
    let col_idx = batch.schema().index_of(col_name).map_err(|_| {
        HedlError::new(
            HedlErrorKind::Syntax,
            format!("Column not found: {col_name}"),
            0,
        )
    })?;

    let array = batch.column(col_idx);

    match (array.data_type(), value) {
        (DataType::Int64, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<Int64Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected Int64 array", 0))?;
            compare_int64(int_array, *v, op)
        }
        (DataType::Int32, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<Int32Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected Int32 array", 0))?;
            compare_int32(int_array, *v as i32, op)
        }
        (DataType::Int16, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<Int16Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected Int16 array", 0))?;
            compare_int16(int_array, *v as i16, op)
        }
        (DataType::Int8, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<Int8Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected Int8 array", 0))?;
            compare_int8(int_array, *v as i8, op)
        }
        (DataType::UInt64, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected UInt64 array", 0))?;
            if *v < 0 {
                // All uint64 values are >= 0, so comparison with negative is straightforward
                match op {
                    ComparisonOp::Eq => Ok(BooleanArray::from(vec![false; batch.num_rows()])),
                    ComparisonOp::Ne => Ok(BooleanArray::from(vec![true; batch.num_rows()])),
                    ComparisonOp::Lt => Ok(BooleanArray::from(vec![false; batch.num_rows()])),
                    ComparisonOp::Le => Ok(BooleanArray::from(vec![false; batch.num_rows()])),
                    ComparisonOp::Gt => Ok(BooleanArray::from(vec![true; batch.num_rows()])),
                    ComparisonOp::Ge => Ok(BooleanArray::from(vec![true; batch.num_rows()])),
                }
            } else {
                compare_uint64(int_array, *v as u64, op)
            }
        }
        (DataType::UInt32, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<UInt32Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected UInt32 array", 0))?;
            compare_uint32(int_array, *v as u32, op)
        }
        (DataType::UInt16, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<UInt16Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected UInt16 array", 0))?;
            compare_uint16(int_array, *v as u16, op)
        }
        (DataType::UInt8, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<UInt8Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected UInt8 array", 0))?;
            compare_uint8(int_array, *v as u8, op)
        }
        (DataType::Float64, PredicateValue::Float(v)) => {
            let float_array = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| {
                    HedlError::new(HedlErrorKind::Syntax, "Expected Float64 array", 0)
                })?;
            compare_float64(float_array, *v, op)
        }
        (DataType::Float32, PredicateValue::Float(v)) => {
            let float_array = array
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| {
                    HedlError::new(HedlErrorKind::Syntax, "Expected Float32 array", 0)
                })?;
            compare_float32(float_array, *v as f32, op)
        }
        (DataType::Float64, PredicateValue::Int(v)) => {
            let float_array = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| {
                    HedlError::new(HedlErrorKind::Syntax, "Expected Float64 array", 0)
                })?;
            compare_float64(float_array, *v as f64, op)
        }
        (DataType::Float32, PredicateValue::Int(v)) => {
            let float_array = array
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| {
                    HedlError::new(HedlErrorKind::Syntax, "Expected Float32 array", 0)
                })?;
            compare_float32(float_array, *v as f32, op)
        }
        (DataType::Utf8, PredicateValue::String(v)) => {
            let str_array = array
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected String array", 0))?;
            compare_string(str_array, v, op)
        }
        (DataType::Boolean, PredicateValue::Bool(v)) => {
            let bool_array = array
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or_else(|| {
                    HedlError::new(HedlErrorKind::Syntax, "Expected Boolean array", 0)
                })?;
            compare_bool(bool_array, *v, op)
        }
        _ => Err(HedlError::new(
            HedlErrorKind::Syntax,
            format!(
                "Type mismatch: column '{}' has type {:?}, predicate value is {:?}",
                col_name,
                array.data_type(),
                value
            ),
            0,
        )),
    }
}

fn evaluate_in(
    batch: &RecordBatch,
    col_name: &str,
    values: &[PredicateValue],
    negate: bool,
) -> Result<BooleanArray, HedlError> {
    if values.is_empty() {
        let result = vec![negate; batch.num_rows()];
        return Ok(BooleanArray::from(result));
    }

    // Evaluate equality for each value and OR them together
    let mut result = evaluate_comparison(batch, col_name, &values[0], ComparisonOp::Eq)?;
    for val in &values[1..] {
        let mask = evaluate_comparison(batch, col_name, val, ComparisonOp::Eq)?;
        result = compute::or(&result, &mask).map_err(|e| {
            HedlError::new(
                HedlErrorKind::IO,
                format!("Failed to combine IN predicates: {e}"),
                0,
            )
        })?;
    }

    if negate {
        compute::not(&result).map_err(|e| {
            HedlError::new(
                HedlErrorKind::IO,
                format!("Failed to negate predicate: {e}"),
                0,
            )
        })
    } else {
        Ok(result)
    }
}

fn evaluate_is_null(
    batch: &RecordBatch,
    col_name: &str,
    is_null: bool,
) -> Result<BooleanArray, HedlError> {
    let col_idx = batch.schema().index_of(col_name).map_err(|_| {
        HedlError::new(
            HedlErrorKind::Syntax,
            format!("Column not found: {col_name}"),
            0,
        )
    })?;

    let array = batch.column(col_idx);

    if is_null {
        compute::is_null(array).map_err(|e| {
            HedlError::new(HedlErrorKind::IO, format!("Failed to check nulls: {e}"), 0)
        })
    } else {
        compute::is_not_null(array).map_err(|e| {
            HedlError::new(
                HedlErrorKind::IO,
                format!("Failed to check not nulls: {e}"),
                0,
            )
        })
    }
}

// Comparison helper macros using Arrow 57+ Datum-based API
macro_rules! impl_compare {
    ($fn_name:ident, $array_type:ty, $value_type:ty, $wrapper_array:ty) => {
        fn $fn_name(
            array: &$array_type,
            value: $value_type,
            op: ComparisonOp,
        ) -> Result<BooleanArray, HedlError> {
            // Create a scalar value as a single-element array wrapped in Scalar
            let scalar_array = <$wrapper_array>::from(vec![value]);
            let scalar = Scalar::new(&scalar_array);

            let result = match op {
                ComparisonOp::Eq => cmp::eq(array, &scalar),
                ComparisonOp::Ne => cmp::neq(array, &scalar),
                ComparisonOp::Lt => cmp::lt(array, &scalar),
                ComparisonOp::Le => cmp::lt_eq(array, &scalar),
                ComparisonOp::Gt => cmp::gt(array, &scalar),
                ComparisonOp::Ge => cmp::gt_eq(array, &scalar),
            };
            result.map_err(|e| {
                HedlError::new(HedlErrorKind::IO, format!("Comparison failed: {}", e), 0)
            })
        }
    };
}

impl_compare!(compare_int64, Int64Array, i64, Int64Array);
impl_compare!(compare_int32, Int32Array, i32, Int32Array);
impl_compare!(compare_int16, Int16Array, i16, Int16Array);
impl_compare!(compare_int8, Int8Array, i8, Int8Array);
impl_compare!(compare_uint64, UInt64Array, u64, UInt64Array);
impl_compare!(compare_uint32, UInt32Array, u32, UInt32Array);
impl_compare!(compare_uint16, UInt16Array, u16, UInt16Array);
impl_compare!(compare_uint8, UInt8Array, u8, UInt8Array);
impl_compare!(compare_float64, Float64Array, f64, Float64Array);
impl_compare!(compare_float32, Float32Array, f32, Float32Array);

fn compare_string(
    array: &StringArray,
    value: &str,
    op: ComparisonOp,
) -> Result<BooleanArray, HedlError> {
    let scalar_array = StringArray::from(vec![value]);
    let scalar = Scalar::new(&scalar_array);

    let result = match op {
        ComparisonOp::Eq => cmp::eq(array, &scalar),
        ComparisonOp::Ne => cmp::neq(array, &scalar),
        ComparisonOp::Lt => cmp::lt(array, &scalar),
        ComparisonOp::Le => cmp::lt_eq(array, &scalar),
        ComparisonOp::Gt => cmp::gt(array, &scalar),
        ComparisonOp::Ge => cmp::gt_eq(array, &scalar),
    };
    result.map_err(|e| {
        HedlError::new(
            HedlErrorKind::IO,
            format!("String comparison failed: {e}"),
            0,
        )
    })
}

fn compare_bool(
    array: &BooleanArray,
    value: bool,
    op: ComparisonOp,
) -> Result<BooleanArray, HedlError> {
    let scalar_array = BooleanArray::from(vec![value]);
    let scalar = Scalar::new(&scalar_array);

    let result = match op {
        ComparisonOp::Eq => cmp::eq(array, &scalar),
        ComparisonOp::Ne => cmp::neq(array, &scalar),
        // For boolean, Lt/Le/Gt/Ge use Arrow's default ordering (false < true)
        ComparisonOp::Lt => cmp::lt(array, &scalar),
        ComparisonOp::Le => cmp::lt_eq(array, &scalar),
        ComparisonOp::Gt => cmp::gt(array, &scalar),
        ComparisonOp::Ge => cmp::gt_eq(array, &scalar),
    };
    result.map_err(|e| {
        HedlError::new(
            HedlErrorKind::IO,
            format!("Boolean comparison failed: {e}"),
            0,
        )
    })
}
