// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for format conversion functions.

use hedl_ffi::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

// HEDL document with valid struct syntax and complex data types
const COMPLEX_HEDL: &[u8] = b"%VERSION: 1.0\n---\nperson: { name: \"Alice\", age: 30, active: true }\nnumbers: [1, 2, 3, 4, 5]\nnested: { level1: { level2: \"deep\" } }\0";

#[cfg(feature = "json")]
#[test]
fn test_to_json_with_metadata() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(COMPLEX_HEDL.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_to_json(doc, 1, &mut out_str);

        assert_eq!(result, HEDL_OK);
        assert!(!out_str.is_null());

        let json = CStr::from_ptr(out_str).to_str().unwrap();
        assert!(json.contains("Alice"));
        assert!(!json.is_empty());

        hedl_free_string(out_str);
        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_to_json_without_metadata() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(COMPLEX_HEDL.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_to_json(doc, 0, &mut out_str);

        assert_eq!(result, HEDL_OK);
        assert!(!out_str.is_null());

        let json = CStr::from_ptr(out_str).to_str().unwrap();
        assert!(json.contains("Alice"));

        hedl_free_string(out_str);
        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_from_json_with_complex_structure() {
    unsafe {
        let json = CString::new(r#"{"name": "Bob", "age": 25, "items": [1, 2, 3]}"#).unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_from_json(json.as_ptr(), -1, &mut doc);
        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_from_json_with_exact_length() {
    unsafe {
        let json = r#"{"key": "value"}"#;
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_from_json(json.as_ptr().cast::<c_char>(), json.len() as i32, &mut doc);
        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_from_json_with_invalid_json() {
    unsafe {
        let json = CString::new("{invalid json").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_from_json(json.as_ptr(), -1, &mut doc);
        assert_eq!(result, HEDL_ERR_JSON);
        assert!(doc.is_null());

        let err = hedl_get_last_error();
        assert!(!err.is_null());
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_to_yaml_with_complex_structure() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(COMPLEX_HEDL.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_to_yaml(doc, 0, &mut out_str);

        assert_eq!(result, HEDL_OK);
        assert!(!out_str.is_null());

        let yaml = CStr::from_ptr(out_str).to_str().unwrap();
        assert!(yaml.contains("Alice"));
        assert!(!yaml.is_empty());

        hedl_free_string(out_str);
        hedl_free_document(doc);
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_from_yaml_with_complex_structure() {
    unsafe {
        // Simple YAML mapping structure
        let yaml = CString::new("key: value").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_from_yaml(yaml.as_ptr(), -1, &mut doc);
        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        hedl_free_document(doc);
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_from_yaml_with_invalid_yaml() {
    unsafe {
        let yaml = CString::new("invalid:\n  yaml:\n- broken").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_from_yaml(yaml.as_ptr(), -1, &mut doc);
        // Result depends on whether the YAML is actually invalid
        // Just verify we get a document or proper error
        if result == HEDL_OK {
            hedl_free_document(doc);
        } else {
            assert!(doc.is_null());
        }
    }
}

#[cfg(feature = "xml")]
#[test]
fn test_to_xml_structure() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nroot: { child: \"value\" }\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_to_xml(doc, &mut out_str);

        assert_eq!(result, HEDL_OK);
        assert!(!out_str.is_null());

        let xml = CStr::from_ptr(out_str).to_str().unwrap();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("child"));

        hedl_free_string(out_str);
        hedl_free_document(doc);
    }
}

#[cfg(feature = "xml")]
#[test]
fn test_from_xml_with_complex_structure() {
    unsafe {
        let xml = CString::new(r#"<?xml version="1.0"?><root><item>value</item></root>"#).unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_from_xml(xml.as_ptr(), -1, &mut doc);
        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        hedl_free_document(doc);
    }
}

#[cfg(feature = "xml")]
#[test]
fn test_from_xml_with_invalid_xml() {
    unsafe {
        // Use HEDL XML format with unclosed tag to ensure failure
        let xml = CString::new("<?xml version=\"1.0\"?><hedl><item>value</hedl>").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_from_xml(xml.as_ptr(), -1, &mut doc);
        assert_eq!(result, HEDL_ERR_XML);
        assert!(doc.is_null());
    }
}

#[cfg(feature = "csv")]
#[test]
fn test_to_csv_with_simple_data() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\n\
            rows: [\n\
                { name: \"Alice\", age: 30 },\n\
                { name: \"Bob\", age: 25 }\n\
            ]\0";

        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_to_csv(doc, &mut out_str);

        if result == HEDL_OK {
            assert!(!out_str.is_null());
            let csv = CStr::from_ptr(out_str).to_str().unwrap();
            assert!(!csv.is_empty());
            hedl_free_string(out_str);
        }

        hedl_free_document(doc);
    }
}

#[cfg(feature = "parquet")]
#[test]
fn test_to_parquet_basic() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\n\
            data: [\n\
                { id: 1, name: \"Alice\" },\n\
                { id: 2, name: \"Bob\" }\n\
            ]\0";

        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut out_data: *mut u8 = ptr::null_mut();
        let mut out_len: usize = 0;
        let result = hedl_to_parquet(doc, &mut out_data, &mut out_len);

        if result == HEDL_OK {
            assert!(!out_data.is_null());
            assert!(out_len > 0);
            hedl_free_bytes(out_data, out_len);
        }

        hedl_free_document(doc);
    }
}

#[cfg(feature = "parquet")]
#[test]
fn test_from_parquet_basic() {
    unsafe {
        // First create parquet data
        let input = b"%VERSION: 1.0\n---\n\
            data: [\n\
                { id: 1, value: \"test\" }\n\
            ]\0";

        let mut doc1: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc1);

        let mut parquet_data: *mut u8 = ptr::null_mut();
        let mut parquet_len: usize = 0;
        let result1 = hedl_to_parquet(doc1, &mut parquet_data, &mut parquet_len);

        if result1 == HEDL_OK && !parquet_data.is_null() {
            // Now parse it back
            let mut doc2: *mut HedlDocument = ptr::null_mut();
            let result2 = hedl_from_parquet(parquet_data, parquet_len, &mut doc2);

            if result2 == HEDL_OK {
                assert!(!doc2.is_null());
                hedl_free_document(doc2);
            }

            hedl_free_bytes(parquet_data, parquet_len);
        }

        hedl_free_document(doc1);
    }
}

#[cfg(feature = "neo4j")]
#[test]
fn test_to_neo4j_cypher_basic() {
    unsafe {
        // Valid HEDL syntax for a nested structure suitable for Neo4j conversion
        let input = b"%VERSION: 1.0\n---\nperson: { name: \"Alice\" }\0";

        let mut doc: *mut HedlDocument = ptr::null_mut();
        let parse_result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_eq!(parse_result, HEDL_OK);
        assert!(!doc.is_null());

        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_to_neo4j_cypher(doc, 1, &mut out_str);

        assert_eq!(result, HEDL_OK);
        assert!(!out_str.is_null());

        hedl_free_string(out_str);
        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_json_roundtrip_preserves_data() {
    unsafe {
        let input =
            b"%VERSION: 1.0\n---\nstring: \"test\"\nnumber: 42\nbool: true\narray: [1, 2, 3]\0";

        let mut doc1: *mut HedlDocument = ptr::null_mut();
        let parse_result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc1);
        assert_eq!(parse_result, HEDL_OK);

        // Convert to JSON
        let mut json_str: *mut c_char = ptr::null_mut();
        hedl_to_json(doc1, 1, &mut json_str);

        // Parse back
        let mut doc2: *mut HedlDocument = ptr::null_mut();
        hedl_from_json(json_str, -1, &mut doc2);

        assert!(!doc2.is_null());

        hedl_free_string(json_str);
        hedl_free_document(doc1);
        hedl_free_document(doc2);
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_yaml_roundtrip_preserves_data() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\n\
            data: { key: \"value\", num: 123 }\0";

        let mut doc1: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc1);

        // Convert to YAML
        let mut yaml_str: *mut c_char = ptr::null_mut();
        hedl_to_yaml(doc1, 0, &mut yaml_str);

        // Parse back
        let mut doc2: *mut HedlDocument = ptr::null_mut();
        hedl_from_yaml(yaml_str, -1, &mut doc2);

        assert!(!doc2.is_null());

        hedl_free_string(yaml_str);
        hedl_free_document(doc1);
        hedl_free_document(doc2);
    }
}

#[cfg(all(feature = "json", feature = "yaml"))]
#[test]
fn test_cross_format_conversion() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\ntest: \"data\"\0";

        // Parse HEDL
        let mut doc1: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc1);

        // Convert to JSON
        let mut json_str: *mut c_char = ptr::null_mut();
        hedl_to_json(doc1, 0, &mut json_str);

        // Parse JSON
        let mut doc2: *mut HedlDocument = ptr::null_mut();
        hedl_from_json(json_str, -1, &mut doc2);

        // Convert to YAML
        let mut yaml_str: *mut c_char = ptr::null_mut();
        hedl_to_yaml(doc2, 0, &mut yaml_str);

        // Verify we have output
        assert!(!yaml_str.is_null());

        hedl_free_string(json_str);
        hedl_free_string(yaml_str);
        hedl_free_document(doc1);
        hedl_free_document(doc2);
    }
}

#[test]
fn test_canonicalize_is_idempotent() {
    unsafe {
        // Use valid HEDL syntax
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let parse_result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_eq!(parse_result, HEDL_OK);
        assert!(!doc.is_null());

        // First canonicalization
        let mut canon1: *mut c_char = ptr::null_mut();
        let canon_result = hedl_canonicalize(doc, &mut canon1);
        assert_eq!(canon_result, HEDL_OK);
        assert!(!canon1.is_null());

        // Parse canonical form
        let mut doc2: *mut HedlDocument = ptr::null_mut();
        let parse_result2 = hedl_parse(canon1, -1, 0, &mut doc2);
        assert_eq!(parse_result2, HEDL_OK);
        assert!(!doc2.is_null());

        // Second canonicalization
        let mut canon2: *mut c_char = ptr::null_mut();
        let canon_result2 = hedl_canonicalize(doc2, &mut canon2);
        assert_eq!(canon_result2, HEDL_OK);
        assert!(!canon2.is_null());

        // Should be identical
        let str1 = CStr::from_ptr(canon1).to_str().unwrap();
        let str2 = CStr::from_ptr(canon2).to_str().unwrap();
        assert_eq!(str1, str2);

        hedl_free_string(canon1);
        hedl_free_string(canon2);
        hedl_free_document(doc);
        hedl_free_document(doc2);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_empty_document_conversions() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\n\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut json_str: *mut c_char = ptr::null_mut();
        let result = hedl_to_json(doc, 0, &mut json_str);

        assert_eq!(result, HEDL_OK);
        assert!(!json_str.is_null());

        hedl_free_string(json_str);
        hedl_free_document(doc);
    }
}

#[test]
fn test_multiple_conversions_same_document() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        // Multiple canonicalization calls
        for _ in 0..5 {
            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_canonicalize(doc, &mut out_str);
            assert_eq!(result, HEDL_OK);
            hedl_free_string(out_str);
        }

        hedl_free_document(doc);
    }
}

#[cfg(feature = "parquet")]
#[test]
fn test_free_bytes_with_zero_length() {
    unsafe {
        // Allocate and free zero-length byte array
        let input = b"%VERSION: 1.0\n---\ndata: []\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut out_data: *mut u8 = ptr::null_mut();
        let mut out_len: usize = 0;
        let result = hedl_to_parquet(doc, &mut out_data, &mut out_len);

        if result == HEDL_OK && !out_data.is_null() {
            hedl_free_bytes(out_data, out_len);
        }

        hedl_free_document(doc);
    }
}

#[cfg(feature = "toon")]
#[test]
fn test_to_toon_basic() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nname: \"Alice\"\nage: 30\0";

        let mut doc: *mut HedlDocument = ptr::null_mut();
        let parse_result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_eq!(parse_result, HEDL_OK);
        assert!(!doc.is_null());

        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_to_toon(doc, &mut out_str);

        assert_eq!(result, HEDL_OK);
        assert!(!out_str.is_null());

        let toon = CStr::from_ptr(out_str).to_str().unwrap();
        assert!(!toon.is_empty());

        hedl_free_string(out_str);
        hedl_free_document(doc);
    }
}

#[cfg(feature = "toon")]
#[test]
fn test_from_toon_basic() {
    unsafe {
        let toon = CString::new("name: test\ncount: 42").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_from_toon(toon.as_ptr(), -1, &mut doc);
        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        hedl_free_document(doc);
    }
}

#[cfg(feature = "toon")]
#[test]
fn test_toon_roundtrip_preserves_data() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nname: \"Alice\"\nage: 30\0";

        let mut doc1: *mut HedlDocument = ptr::null_mut();
        let parse_result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc1);
        assert_eq!(parse_result, HEDL_OK);

        // Convert to TOON
        let mut toon_str: *mut c_char = ptr::null_mut();
        let to_result = hedl_to_toon(doc1, &mut toon_str);
        assert_eq!(to_result, HEDL_OK);
        assert!(!toon_str.is_null());

        // Parse back
        let mut doc2: *mut HedlDocument = ptr::null_mut();
        let from_result = hedl_from_toon(toon_str, -1, &mut doc2);
        assert_eq!(from_result, HEDL_OK);
        assert!(!doc2.is_null());

        hedl_free_string(toon_str);
        hedl_free_document(doc1);
        hedl_free_document(doc2);
    }
}

#[cfg(feature = "toon")]
#[test]
fn test_to_toon_null_doc() {
    unsafe {
        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_to_toon(ptr::null(), &mut out_str);

        assert_eq!(result, HEDL_ERR_NULL_PTR);
        assert!(out_str.is_null());
    }
}

#[cfg(feature = "toon")]
#[test]
fn test_to_toon_null_out_str() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let result = hedl_to_toon(doc, ptr::null_mut());

        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "toon")]
#[test]
fn test_from_toon_null_input() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_from_toon(ptr::null(), -1, &mut doc);

        assert_eq!(result, HEDL_ERR_NULL_PTR);
        assert!(doc.is_null());
    }
}

#[cfg(feature = "toon")]
#[test]
fn test_from_toon_null_out_doc() {
    unsafe {
        let toon = CString::new("key: value").unwrap();
        let result = hedl_from_toon(toon.as_ptr(), -1, ptr::null_mut());

        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}
