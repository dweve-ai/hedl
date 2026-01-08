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

//! Fixtures for scalar values, special strings, references, and edge cases.

use hedl_core::{Document, Item, Reference, Tensor, Value};
use std::collections::BTreeMap;

/// Document with all scalar value types.
///
/// Tests: null, bool, int, float, string values.
pub fn scalars() -> Document {
    let mut root = BTreeMap::new();

    root.insert("null_val".to_string(), Item::Scalar(Value::Null));
    root.insert("bool_true".to_string(), Item::Scalar(Value::Bool(true)));
    root.insert("bool_false".to_string(), Item::Scalar(Value::Bool(false)));
    root.insert("int_positive".to_string(), Item::Scalar(Value::Int(42)));
    root.insert("int_negative".to_string(), Item::Scalar(Value::Int(-17)));
    root.insert("int_zero".to_string(), Item::Scalar(Value::Int(0)));
    root.insert(
        "float_positive".to_string(),
        Item::Scalar(Value::Float(3.5)),
    );
    root.insert(
        "float_negative".to_string(),
        Item::Scalar(Value::Float(-2.5)),
    );
    root.insert("float_zero".to_string(), Item::Scalar(Value::Float(0.0)));
    root.insert(
        "string_simple".to_string(),
        Item::Scalar(Value::String("hello world".to_string())),
    );
    root.insert(
        "string_empty".to_string(),
        Item::Scalar(Value::String(String::new())),
    );

    Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

/// Document with special string values.
///
/// Tests: quotes, backslashes, newlines, tabs, unicode.
pub fn special_strings() -> Document {
    let mut root = BTreeMap::new();

    root.insert(
        "with_quotes".to_string(),
        Item::Scalar(Value::String("He said \"hello\" and 'goodbye'".to_string())),
    );
    root.insert(
        "with_backslash".to_string(),
        Item::Scalar(Value::String("path\\to\\file".to_string())),
    );
    root.insert(
        "with_newline".to_string(),
        Item::Scalar(Value::String("line1\nline2\nline3".to_string())),
    );
    root.insert(
        "with_tab".to_string(),
        Item::Scalar(Value::String("col1\tcol2\tcol3".to_string())),
    );
    root.insert(
        "with_unicode".to_string(),
        Item::Scalar(Value::String("日本語 中文 한국어 emoji: 🎉".to_string())),
    );
    root.insert(
        "with_mixed".to_string(),
        Item::Scalar(Value::String(
            "It's a \"test\" with\ttabs and\nnewlines".to_string(),
        )),
    );

    Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

/// Document with reference values.
///
/// Tests: local references, typed references.
pub fn references() -> Document {
    let mut root = BTreeMap::new();

    // Local reference (no type)
    root.insert(
        "local_ref".to_string(),
        Item::Scalar(Value::Reference(Reference {
            type_name: None,
            id: "some_id".to_string(),
        })),
    );

    // Typed reference
    root.insert(
        "typed_ref".to_string(),
        Item::Scalar(Value::Reference(Reference {
            type_name: Some("User".to_string()),
            id: "alice".to_string(),
        })),
    );

    Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

/// Document with tensor values.
///
/// Tests: 1D tensors, 2D tensors, nested tensors.
pub fn tensors() -> Document {
    let mut root = BTreeMap::new();

    // 1D tensor [1.0, 2.0, 3.0]
    root.insert(
        "tensor_1d".to_string(),
        Item::Scalar(Value::Tensor(Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
            Tensor::Scalar(3.0),
        ]))),
    );

    // 2D tensor (matrix) [[1.0, 2.0], [3.0, 4.0]]
    root.insert(
        "tensor_2d".to_string(),
        Item::Scalar(Value::Tensor(Tensor::Array(vec![
            Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
            Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
        ]))),
    );

    // 3D tensor
    root.insert(
        "tensor_3d".to_string(),
        Item::Scalar(Value::Tensor(Tensor::Array(vec![
            Tensor::Array(vec![
                Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
                Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
            ]),
            Tensor::Array(vec![
                Tensor::Array(vec![Tensor::Scalar(5.0), Tensor::Scalar(6.0)]),
                Tensor::Array(vec![Tensor::Scalar(7.0), Tensor::Scalar(8.0)]),
            ]),
        ]))),
    );

    // Empty tensor
    root.insert(
        "tensor_empty".to_string(),
        Item::Scalar(Value::Tensor(Tensor::Array(vec![]))),
    );

    Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

/// Document with multiple scalar types as named values.
///
/// Tests: Various scalar types organized by name.
pub fn named_values() -> Document {
    let mut root = BTreeMap::new();

    // Config-style named values
    root.insert(
        "app_name".to_string(),
        Item::Scalar(Value::String("MyApp".to_string())),
    );
    root.insert(
        "version".to_string(),
        Item::Scalar(Value::String("1.0.0".to_string())),
    );
    root.insert("debug_mode".to_string(), Item::Scalar(Value::Bool(true)));
    root.insert("max_connections".to_string(), Item::Scalar(Value::Int(100)));
    root.insert(
        "timeout_seconds".to_string(),
        Item::Scalar(Value::Float(30.5)),
    );
    root.insert("deprecated_feature".to_string(), Item::Scalar(Value::Null));

    Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

/// Document with edge case values.
///
/// Tests: Large numbers, very long strings, extreme floats.
pub fn edge_cases() -> Document {
    let mut root = BTreeMap::new();

    // Large integer
    root.insert("large_int".to_string(), Item::Scalar(Value::Int(i64::MAX)));
    root.insert("small_int".to_string(), Item::Scalar(Value::Int(i64::MIN)));

    // Extreme floats
    root.insert(
        "tiny_float".to_string(),
        Item::Scalar(Value::Float(f64::MIN_POSITIVE)),
    );
    root.insert(
        "large_float".to_string(),
        Item::Scalar(Value::Float(f64::MAX)),
    );

    // Long string
    root.insert(
        "long_string".to_string(),
        Item::Scalar(Value::String("x".repeat(10000))),
    );

    // String with only special chars
    root.insert(
        "special_only".to_string(),
        Item::Scalar(Value::String("\n\t\r\\\"'".to_string())),
    );

    Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

/// Empty document.
///
/// Tests: Minimal valid document.
pub fn empty() -> Document {
    Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root: BTreeMap::new(),
    }
}
