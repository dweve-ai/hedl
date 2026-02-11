// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Tests for CSV to HEDL conversion

#[cfg(test)]
mod csv_conversion_tests {
    use crate::from_csv::parsing::{parse_csv_value, parse_reference};
    use crate::{
        from_csv, from_csv_reader, from_csv_reader_with_config, from_csv_with_config, CsvError,
        FromCsvConfig, DEFAULT_MAX_CELL_SIZE, DEFAULT_MAX_COLUMNS, DEFAULT_MAX_HEADER_SIZE,
        DEFAULT_MAX_ROWS, DEFAULT_MAX_TOTAL_SIZE,
    };
    use hedl_core::lex::Tensor;
    use hedl_core::Value;
    use hedl_test::expr_value;

    // ==================== FromCsvConfig tests ====================

    #[test]
    fn test_from_csv_config_default() {
        let config = FromCsvConfig::default();
        assert_eq!(config.delimiter, b',');
        assert!(config.has_headers);
        assert!(config.trim);
        assert_eq!(config.max_rows, DEFAULT_MAX_ROWS);
    }

    #[test]
    fn test_from_csv_config_debug() {
        let config = FromCsvConfig::default();
        let debug = format!("{config:?}");
        assert!(debug.contains("FromCsvConfig"));
        assert!(debug.contains("delimiter"));
        assert!(debug.contains("has_headers"));
        assert!(debug.contains("trim"));
    }

    #[test]
    fn test_from_csv_config_clone() {
        let config = FromCsvConfig {
            delimiter: b'\t',
            has_headers: false,
            trim: false,
            max_rows: 500_000,
            infer_schema: false,
            sample_rows: 100,
            list_key: None,
            max_columns: 5_000,
            max_cell_size: 2_000_000,
            max_total_size: 200_000_000,
            max_header_size: 2_000_000,
        };
        let cloned = config.clone();
        assert_eq!(cloned.delimiter, b'\t');
        assert!(!cloned.has_headers);
        assert!(!cloned.trim);
        assert_eq!(cloned.max_rows, 500_000);
        assert!(!cloned.infer_schema);
        assert_eq!(cloned.sample_rows, 100);
        assert_eq!(cloned.list_key, None);
        assert_eq!(cloned.max_columns, 5_000);
        assert_eq!(cloned.max_cell_size, 2_000_000);
        assert_eq!(cloned.max_total_size, 200_000_000);
        assert_eq!(cloned.max_header_size, 2_000_000);
    }

    #[test]
    fn test_from_csv_config_all_options() {
        let config = FromCsvConfig {
            delimiter: b';',
            has_headers: true,
            trim: true,
            max_rows: 2_000_000,
            infer_schema: true,
            sample_rows: 200,
            list_key: Some("custom".to_string()),
            max_columns: 15_000,
            max_cell_size: 3_000_000,
            max_total_size: 300_000_000,
            max_header_size: 3_000_000,
        };
        assert_eq!(config.delimiter, b';');
        assert!(config.has_headers);
        assert!(config.trim);
        assert_eq!(config.max_rows, 2_000_000);
        assert!(config.infer_schema);
        assert_eq!(config.sample_rows, 200);
        assert_eq!(config.list_key, Some("custom".to_string()));
        assert_eq!(config.max_columns, 15_000);
        assert_eq!(config.max_cell_size, 3_000_000);
        assert_eq!(config.max_total_size, 300_000_000);
        assert_eq!(config.max_header_size, 3_000_000);
    }

    #[test]
    fn test_max_rows_limit_enforcement() {
        // Create CSV with exactly max_rows + 1 rows
        let mut csv_data = String::from("id,value\n");
        let max_rows = 100;
        for i in 0..=max_rows {
            csv_data.push_str(&format!("{i},test{i}\n"));
        }

        let config = FromCsvConfig {
            max_rows,
            infer_schema: false,
            sample_rows: 100,
            ..Default::default()
        };

        let result = from_csv_with_config(&csv_data, "Item", &["value"], config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CsvError::SecurityLimit { .. }));
        assert!(err.to_string().contains("Security limit"));
        assert!(err.to_string().contains(&max_rows.to_string()));
    }

    #[test]
    fn test_max_rows_limit_not_exceeded() {
        // Create CSV with exactly max_rows rows
        let mut csv_data = String::from("id,value\n");
        let max_rows = 100;
        for i in 0..(max_rows - 1) {
            csv_data.push_str(&format!("{i},test{i}\n"));
        }

        let config = FromCsvConfig {
            max_rows,
            infer_schema: false,
            sample_rows: 100,
            ..Default::default()
        };

        let result = from_csv_with_config(&csv_data, "Item", &["value"], config);
        assert!(result.is_ok());
        let doc = result.unwrap();
        let list = doc.get("items").unwrap().as_list().unwrap();
        assert_eq!(list.rows.len(), max_rows - 1);
    }

    // ==================== from_csv basic tests ====================

    #[test]
    fn test_from_csv_basic() {
        let csv_data = "id,name,age,active\n1,Alice,30,true\n2,Bob,25,false\n";
        let doc = from_csv(csv_data, "Person", &["name", "age", "active"]).unwrap();

        // Check document structure
        assert_eq!(doc.version, (2, 0));

        // Check schema registration
        let schema = doc.get_schema("Person").unwrap();
        assert_eq!(schema, &["id", "name", "age", "active"]);

        // Check matrix list
        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.type_name, "Person");
        assert_eq!(list.rows.len(), 2);

        // Check first row
        let row1 = &list.rows[0];
        assert_eq!(row1.id, "1");
        assert_eq!(row1.fields.len(), schema.len()); // schema includes ID
        assert_eq!(row1.fields[0], Value::Int(1)); // ID field
        assert_eq!(row1.fields[1], Value::String("Alice".into()));
        assert_eq!(row1.fields[2], Value::Int(30));
        assert_eq!(row1.fields[3], Value::Bool(true));

        // Check second row
        let row2 = &list.rows[1];
        assert_eq!(row2.id, "2");
        assert_eq!(row2.fields.len(), schema.len()); // schema includes ID
        assert_eq!(row2.fields[0], Value::Int(2)); // ID field
        assert_eq!(row2.fields[1], Value::String("Bob".into()));
        assert_eq!(row2.fields[2], Value::Int(25));
        assert_eq!(row2.fields[3], Value::Bool(false));
    }

    #[test]
    fn test_from_csv_without_headers() {
        let csv_data = "1,Alice,30\n2,Bob,25\n";
        let config = FromCsvConfig {
            has_headers: false,
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name", "age"], config).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows.len(), 2);
    }

    #[test]
    fn test_from_csv_custom_delimiter() {
        let csv_data = "id\tname\tage\n1\tAlice\t30\n2\tBob\t25\n";
        let config = FromCsvConfig {
            delimiter: b'\t',
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name", "age"], config).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows.len(), 2);
    }

    #[test]
    fn test_from_csv_semicolon_delimiter() {
        let csv_data = "id;name;age\n1;Alice;30\n";
        let config = FromCsvConfig {
            delimiter: b';',
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name", "age"], config).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows.len(), 1);
        assert_eq!(list.rows[0].fields[1], Value::String("Alice".into()));
    }

    #[test]
    fn test_from_csv_empty_file() {
        let csv_data = "id,name\n";
        let doc = from_csv(csv_data, "Person", &["name"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert!(list.rows.is_empty());
    }

    #[test]
    fn test_from_csv_single_row() {
        let csv_data = "id,name\n1,Alice\n";
        let doc = from_csv(csv_data, "Person", &["name"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows.len(), 1);
    }

    // ==================== parse_csv_value tests ====================

    #[test]
    fn test_parse_csv_value_null_empty() {
        assert_eq!(parse_csv_value("").unwrap(), Value::Null);
    }

    #[test]
    fn test_parse_csv_value_null_tilde() {
        assert_eq!(parse_csv_value("~").unwrap(), Value::Null);
    }

    #[test]
    fn test_parse_csv_value_null_whitespace() {
        assert_eq!(parse_csv_value("   ").unwrap(), Value::Null);
    }

    #[test]
    fn test_parse_csv_value_bool_true() {
        assert_eq!(parse_csv_value("true").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_parse_csv_value_bool_false() {
        assert_eq!(parse_csv_value("false").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_parse_csv_value_int_positive() {
        assert_eq!(parse_csv_value("42").unwrap(), Value::Int(42));
    }

    #[test]
    fn test_parse_csv_value_int_negative() {
        assert_eq!(parse_csv_value("-123").unwrap(), Value::Int(-123));
    }

    #[test]
    fn test_parse_csv_value_int_zero() {
        assert_eq!(parse_csv_value("0").unwrap(), Value::Int(0));
    }

    #[test]
    fn test_parse_csv_value_int_large() {
        assert_eq!(
            parse_csv_value("9223372036854775807").unwrap(),
            Value::Int(i64::MAX)
        );
    }

    #[test]
    fn test_parse_csv_value_float_positive() {
        assert_eq!(parse_csv_value("3.25").unwrap(), Value::Float(3.25));
    }

    #[test]
    fn test_parse_csv_value_float_negative() {
        assert_eq!(parse_csv_value("-2.5").unwrap(), Value::Float(-2.5));
    }

    #[test]
    fn test_parse_csv_value_float_zero() {
        assert_eq!(parse_csv_value("0.0").unwrap(), Value::Float(0.0));
    }

    #[test]
    fn test_parse_csv_value_float_scientific() {
        let val = parse_csv_value("1.5e10").unwrap();
        if let Value::Float(f) = val {
            assert!((f - 1.5e10).abs() < 1e5);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_parse_csv_value_string() {
        assert_eq!(
            parse_csv_value("hello").unwrap(),
            Value::String("hello".into())
        );
    }

    #[test]
    fn test_parse_csv_value_string_with_spaces() {
        assert_eq!(
            parse_csv_value("  hello world  ").unwrap(),
            Value::String("  hello world  ".into())
        );
    }

    #[test]
    fn test_parse_csv_value_string_numeric_looking() {
        // Strings that look like numbers but have leading zeros
        assert_eq!(
            parse_csv_value("007").unwrap(),
            Value::Int(7) // Parsed as int
        );
    }

    // ==================== Special float values ====================

    #[test]
    fn test_parse_csv_value_nan() {
        let nan = parse_csv_value("NaN").unwrap();
        assert!(matches!(nan, Value::Float(f) if f.is_nan()));
    }

    #[test]
    fn test_parse_csv_value_infinity() {
        let inf = parse_csv_value("Infinity").unwrap();
        assert_eq!(inf, Value::Float(f64::INFINITY));
    }

    #[test]
    fn test_parse_csv_value_neg_infinity() {
        let neg_inf = parse_csv_value("-Infinity").unwrap();
        assert_eq!(neg_inf, Value::Float(f64::NEG_INFINITY));
    }

    // ==================== Reference tests ====================

    #[test]
    fn test_parse_csv_value_reference_local() {
        let ref_val = parse_csv_value("@user1").unwrap();
        if let Value::Reference(r) = ref_val {
            assert_eq!(&*r.id, "user1");
            assert_eq!(r.type_name, None);
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_parse_csv_value_reference_qualified() {
        let ref_val = parse_csv_value("@User:user1").unwrap();
        if let Value::Reference(r) = ref_val {
            assert_eq!(&*r.id, "user1");
            assert_eq!(r.type_name.as_deref(), Some("User"));
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_parse_csv_value_reference_with_dashes() {
        let ref_val = parse_csv_value("@my-item-123").unwrap();
        if let Value::Reference(r) = ref_val {
            assert_eq!(&*r.id, "my-item-123");
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_parse_reference_empty_error() {
        let result = parse_reference("@");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Empty reference ID"));
    }

    #[test]
    fn test_parse_reference_empty_type_error() {
        let result = parse_reference("@:id");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid reference format"));
    }

    #[test]
    fn test_parse_reference_empty_id_error() {
        let result = parse_reference("@Type:");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid reference format"));
    }

    // ==================== Expression tests ====================

    #[test]
    fn test_parse_csv_value_expression_identifier() {
        let expr = parse_csv_value("$(foo)").unwrap();
        assert_eq!(expr, expr_value("foo"));
    }

    #[test]
    fn test_parse_csv_value_expression_call() {
        let expr = parse_csv_value("$(add(x, y))").unwrap();
        assert_eq!(expr, expr_value("add(x, y)"));
    }

    #[test]
    fn test_parse_csv_value_expression_nested() {
        let expr = parse_csv_value("$(outer(inner(x)))").unwrap();
        if let Value::Expression(e) = expr {
            assert_eq!(e.to_string(), "outer(inner(x))");
        } else {
            panic!("Expected expression");
        }
    }

    // ==================== Tensor tests ====================

    #[test]
    fn test_parse_csv_value_tensor_1d() {
        let val = parse_csv_value("[1, 2, 3]").unwrap();
        if let Value::Tensor(tensor) = val {
            if let Tensor::Array(arr) = tensor.as_ref() {
                assert_eq!(arr.len(), 3);
            } else {
                panic!("Expected tensor array");
            }
        } else {
            panic!("Expected tensor");
        }
    }

    #[test]
    fn test_parse_csv_value_tensor_2d() {
        let val = parse_csv_value("[[1, 2], [3, 4]]").unwrap();
        if let Value::Tensor(tensor) = val {
            if let Tensor::Array(outer) = tensor.as_ref() {
                assert_eq!(outer.len(), 2);
                if let Tensor::Array(inner) = &outer[0] {
                    assert_eq!(inner.len(), 2);
                } else {
                    panic!("Expected nested array");
                }
            } else {
                panic!("Expected tensor array");
            }
        } else {
            panic!("Expected tensor");
        }
    }

    #[test]
    fn test_parse_csv_value_tensor_empty_is_string() {
        // Empty tensors are not valid in HEDL (must have at least one element)
        // So "[]" falls through to being treated as a string
        let val = parse_csv_value("[]").unwrap();
        assert_eq!(val, Value::String("[]".into()));
    }

    // ==================== Error cases ====================

    #[test]
    fn test_empty_id_error() {
        let csv_data = "id,name\n,Alice\n";
        let result = from_csv(csv_data, "Person", &["name"]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CsvError::EmptyId { .. }));
    }

    #[test]
    fn test_mismatched_field_count() {
        let csv_data = "id,name,age\n1,Alice\n";
        let result = from_csv(csv_data, "Person", &["name", "age"]);
        assert!(result.is_err());
        // CSV parser returns Syntax error for malformed records
        assert!(matches!(result.unwrap_err(), CsvError::ParseError { .. }));
    }

    // ==================== Whitespace handling ====================

    #[test]
    fn test_whitespace_trimming_enabled() {
        let csv_data = "id,name,age\n1,  Alice  ,  30  \n";
        let doc = from_csv(csv_data, "Person", &["name", "age"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        let row = &list.rows[0];

        assert_eq!(row.fields[0], Value::Int(1)); // ID field
        assert_eq!(row.fields[1], Value::String("Alice".into()));
        assert_eq!(row.fields[2], Value::Int(30));
    }

    #[test]
    fn test_whitespace_trimming_disabled() {
        let csv_data = "id,name\n1,  Alice  \n";
        let config = FromCsvConfig {
            trim: false,
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name"], config).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        // With trim disabled, whitespace is preserved
        assert_eq!(list.rows[0].fields[1], Value::String("  Alice  ".into()));
    }

    // ==================== from_csv_reader tests ====================

    #[test]
    fn test_from_csv_reader_basic() {
        let csv_data = "id,name\n1,Alice\n".as_bytes();
        let doc = from_csv_reader(csv_data, "Person", &["name"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows.len(), 1);
    }

    #[test]
    fn test_from_csv_reader_with_config() {
        let csv_data = "1\tAlice\n".as_bytes();
        let config = FromCsvConfig {
            delimiter: b'\t',
            has_headers: false,
            trim: true,
            ..Default::default()
        };
        let doc = from_csv_reader_with_config(csv_data, "Person", &["name"], config).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows.len(), 1);
    }

    // ==================== Type naming tests ====================

    #[test]
    fn test_type_naming_singularization() {
        let csv_data = "id,name\n1,Alice\n";
        let doc = from_csv(csv_data, "User", &["name"]).unwrap();

        // Matrix list should use "users" as key (lowercase + pluralized)
        let item = doc.get("users").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.type_name, "User");
    }

    // ==================== Quoted fields ====================

    #[test]
    fn test_quoted_fields() {
        let csv_data = "id,name,bio\n1,Alice,\"Hello, World\"\n";
        let doc = from_csv(csv_data, "Person", &["name", "bio"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows[0].fields[2], Value::String("Hello, World".into()));
    }

    #[test]
    fn test_quoted_fields_with_newline() {
        let csv_data = "id,name,bio\n1,Alice,\"Line 1\nLine 2\"\n";
        let doc = from_csv(csv_data, "Person", &["name", "bio"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(
            list.rows[0].fields[2],
            Value::String("Line 1\nLine 2".into())
        );
    }

    #[test]
    fn test_quoted_fields_with_quotes() {
        let csv_data = "id,name\n1,\"Alice \"\"Bob\"\" Smith\"\n";
        let doc = from_csv(csv_data, "Person", &["name"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(
            list.rows[0].fields[1],
            Value::String("Alice \"Bob\" Smith".into())
        );
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_unicode_values() {
        let csv_data = "id,name\n1,héllo 世界\n";
        let doc = from_csv(csv_data, "Person", &["name"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows[0].fields[1], Value::String("héllo 世界".into()));
    }

    #[test]
    fn test_string_id() {
        let csv_data = "id,name\nabc,Alice\n";
        let doc = from_csv(csv_data, "Person", &["name"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows[0].id, "abc");
        assert_eq!(list.rows[0].fields[0], Value::String("abc".into()));
    }

    #[test]
    fn test_many_columns() {
        let csv_data = "id,a,b,c,d,e\n1,2,3,4,5,6\n";
        let doc = from_csv(csv_data, "Item", &["a", "b", "c", "d", "e"]).unwrap();

        let item = doc.get("items").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.schema.len(), 6); // id + 5 columns
        assert_eq!(list.rows[0].fields.len(), 6);
    }

    // ==================== Custom list_key tests ====================

    #[test]
    fn test_custom_list_key_basic() {
        let csv_data = "id,name\n1,Alice\n";
        let config = FromCsvConfig {
            list_key: Some("people".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name"], config).unwrap();

        // Custom plural should exist
        assert!(doc.get("people").is_some());
        // Default plural should not exist
        assert!(doc.get("persons").is_none());

        let list = doc.get("people").unwrap().as_list().unwrap();
        assert_eq!(list.type_name, "Person");
        assert_eq!(list.rows.len(), 1);
    }

    #[test]
    fn test_custom_list_key_irregular_plurals() {
        // Test common irregular plurals
        let test_cases = vec![
            ("Person", "people"),
            ("Child", "children"),
            ("Tooth", "teeth"),
            ("Foot", "feet"),
            ("Mouse", "mice"),
            ("Goose", "geese"),
            ("Man", "men"),
            ("Woman", "women"),
            ("Ox", "oxen"),
            ("Datum", "data"),
        ];

        for (type_name, plural) in test_cases {
            let csv_data = "id,value\n1,test\n".to_string();
            let config = FromCsvConfig {
                list_key: Some(plural.to_string()),
                ..Default::default()
            };
            let doc = from_csv_with_config(&csv_data, type_name, &["value"], config).unwrap();

            assert!(
                doc.get(plural).is_some(),
                "Failed to find {plural} for type {type_name}"
            );
        }
    }

    #[test]
    fn test_custom_list_key_collective_nouns() {
        let csv_data = "id,value\n1,42\n";

        // Test collective nouns
        let test_cases = vec![
            ("Data", "dataset"),
            ("Information", "info_collection"),
            ("Equipment", "gear"),
            ("Furniture", "furnishings"),
        ];

        for (type_name, collective) in test_cases {
            let config = FromCsvConfig {
                list_key: Some(collective.to_string()),
                ..Default::default()
            };
            let doc = from_csv_with_config(csv_data, type_name, &["value"], config).unwrap();

            assert!(
                doc.get(collective).is_some(),
                "Failed to find {collective} for type {type_name}"
            );
        }
    }

    #[test]
    fn test_custom_list_key_case_sensitive() {
        let csv_data = "id,value\n1,test\n";
        let config = FromCsvConfig {
            list_key: Some("MyCustomList".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Item", &["value"], config).unwrap();

        // Exact case should exist
        assert!(doc.get("MyCustomList").is_some());
        // Different case should not exist
        assert!(doc.get("mycustomlist").is_none());
        assert!(doc.get("items").is_none());
    }

    #[test]
    fn test_custom_list_key_empty_string() {
        // Empty string is technically allowed as a key
        let csv_data = "id,value\n1,test\n";
        let config = FromCsvConfig {
            list_key: Some(String::new()),
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Item", &["value"], config).unwrap();

        assert!(doc.get("").is_some());
    }

    #[test]
    fn test_custom_list_key_with_special_chars() {
        let csv_data = "id,value\n1,test\n";
        let config = FromCsvConfig {
            list_key: Some("my-custom_list.v2".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Item", &["value"], config).unwrap();

        assert!(doc.get("my-custom_list.v2").is_some());
    }

    #[test]
    fn test_custom_list_key_unicode() {
        let csv_data = "id,value\n1,test\n";
        let config = FromCsvConfig {
            list_key: Some("人々".to_string()), // Japanese for "people"
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["value"], config).unwrap();

        assert!(doc.get("人々").is_some());
    }

    #[test]
    fn test_custom_list_key_with_schema_inference() {
        let csv_data = "id,value\n1,42\n2,43\n3,44\n";
        let config = FromCsvConfig {
            list_key: Some("people".to_string()),
            infer_schema: true,
            sample_rows: 10,
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["value"], config).unwrap();

        assert!(doc.get("people").is_some());
        let list = doc.get("people").unwrap().as_list().unwrap();
        assert_eq!(list.rows.len(), 3);
        // Schema inference should still work
        assert_eq!(list.rows[0].fields[1], Value::Int(42));
    }

    #[test]
    fn test_custom_list_key_none_uses_default() {
        let csv_data = "id,name\n1,Alice\n";
        let config = FromCsvConfig {
            list_key: None,
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name"], config).unwrap();

        // Should use default pluralization
        assert!(doc.get("persons").is_some());
        assert!(doc.get("people").is_none());
    }

    #[test]
    fn test_custom_list_key_default_config() {
        let csv_data = "id,name\n1,Alice\n";
        let doc = from_csv(csv_data, "User", &["name"]).unwrap();

        // Default should use simple pluralization
        assert!(doc.get("users").is_some());
    }

    #[test]
    fn test_custom_list_key_preserves_type_name() {
        let csv_data = "id,name\n1,Alice\n";
        let config = FromCsvConfig {
            list_key: Some("people".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name"], config).unwrap();

        let list = doc.get("people").unwrap().as_list().unwrap();
        // Type name should still be "Person", not "people"
        assert_eq!(list.type_name, "Person");
    }

    #[test]
    fn test_custom_list_key_with_multiple_types() {
        // This test ensures each call can have its own list_key
        let csv1 = "id,name\n1,Alice\n";
        let config1 = FromCsvConfig {
            list_key: Some("people".to_string()),
            ..Default::default()
        };
        let doc1 = from_csv_with_config(csv1, "Person", &["name"], config1).unwrap();

        let csv2 = "id,name\n1,Fluffy\n";
        let config2 = FromCsvConfig {
            list_key: Some("mice".to_string()),
            ..Default::default()
        };
        let doc2 = from_csv_with_config(csv2, "Mouse", &["name"], config2).unwrap();

        assert!(doc1.get("people").is_some());
        assert!(doc1.get("persons").is_none());

        assert!(doc2.get("mice").is_some());
        assert!(doc2.get("mouses").is_none());
    }

    #[test]
    fn test_custom_list_key_numbers_in_name() {
        let csv_data = "id,value\n1,test\n";
        let config = FromCsvConfig {
            list_key: Some("items_v2".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Item", &["value"], config).unwrap();

        assert!(doc.get("items_v2").is_some());
    }

    #[test]
    fn test_custom_list_key_round_trip_compatibility() {
        // Ensure custom list keys work with to_csv_list
        let csv_data = "id,name\n1,Alice\n2,Bob\n";
        let config = FromCsvConfig {
            list_key: Some("people".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name"], config).unwrap();

        // Export the list using the custom key
        use crate::to_csv_list;
        let exported_csv = to_csv_list(&doc, "people").unwrap();
        assert!(exported_csv.contains("Alice"));
        assert!(exported_csv.contains("Bob"));

        // Should not be accessible via default key
        assert!(to_csv_list(&doc, "persons").is_err());
    }

    #[test]
    fn test_from_csv_config_clone_with_list_key() {
        let config = FromCsvConfig {
            delimiter: b',',
            has_headers: true,
            trim: true,
            max_rows: 1000,
            infer_schema: false,
            sample_rows: 50,
            list_key: Some("people".to_string()),
            max_columns: DEFAULT_MAX_COLUMNS,
            max_cell_size: DEFAULT_MAX_CELL_SIZE,
            max_total_size: DEFAULT_MAX_TOTAL_SIZE,
            max_header_size: DEFAULT_MAX_HEADER_SIZE,
        };
        let cloned = config.clone();
        assert_eq!(cloned.list_key, Some("people".to_string()));
    }

    #[test]
    fn test_from_csv_config_debug_with_list_key() {
        let config = FromCsvConfig {
            list_key: Some("people".to_string()),
            ..Default::default()
        };
        let debug = format!("{config:?}");
        assert!(debug.contains("list_key"));
        assert!(debug.contains("people"));
    }

    // ==================== Security Limit Tests ====================

    #[test]
    fn test_from_csv_config_default_security_limits() {
        let config = FromCsvConfig::default();
        assert_eq!(config.max_columns, DEFAULT_MAX_COLUMNS);
        assert_eq!(config.max_cell_size, DEFAULT_MAX_CELL_SIZE);
        assert_eq!(config.max_total_size, DEFAULT_MAX_TOTAL_SIZE);
        assert_eq!(config.max_header_size, DEFAULT_MAX_HEADER_SIZE);
    }

    #[test]
    fn test_from_csv_config_clone_with_security_limits() {
        let config = FromCsvConfig {
            max_columns: 5_000,
            max_cell_size: 2_000_000,
            max_total_size: 200_000_000,
            max_header_size: 2_000_000,
            ..Default::default()
        };
        let cloned = config.clone();
        assert_eq!(cloned.max_columns, 5_000);
        assert_eq!(cloned.max_cell_size, 2_000_000);
        assert_eq!(cloned.max_total_size, 200_000_000);
        assert_eq!(cloned.max_header_size, 2_000_000);
    }

    #[test]
    fn test_from_csv_config_unlimited() {
        let config = FromCsvConfig::unlimited();
        assert_eq!(config.max_rows, usize::MAX);
        assert_eq!(config.max_columns, usize::MAX);
        assert_eq!(config.max_cell_size, usize::MAX);
        assert_eq!(config.max_total_size, usize::MAX);
        assert_eq!(config.max_header_size, usize::MAX);
    }

    #[test]
    fn test_from_csv_config_strict() {
        let config = FromCsvConfig::strict();
        assert_eq!(config.max_rows, 1_000_000);
        assert_eq!(config.max_columns, 1_000);
        assert_eq!(config.max_cell_size, 65_536);
        assert_eq!(config.max_total_size, 10_485_760);
        assert_eq!(config.max_header_size, 65_536);
    }

    #[test]
    fn test_column_count_limit_enforcement() {
        // Create CSV with 11,000 columns (exceeds default 10,000)
        let mut csv = String::from("col0");
        for i in 1..11_000 {
            csv.push_str(&format!(",col{i}"));
        }
        csv.push('\n');
        csv.push_str("a,");
        csv.push_str(&"b,".repeat(10_999));
        csv.push('b');

        let result = from_csv_with_config(&csv, "Item", &[], FromCsvConfig::default());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CsvError::Security { .. }));
        assert!(err.to_string().contains("exceeds limit"));
    }

    #[test]
    fn test_cell_size_limit_enforcement() {
        // Create CSV with 2MB cell (exceeds default 1MB)
        let huge_cell = "x".repeat(2_000_000);
        let csv = format!("id,data\n1,\"{huge_cell}\"\n");

        let result = from_csv_with_config(&csv, "Item", &["data"], FromCsvConfig::default());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CsvError::Security { .. }));
        // Check that the error message contains information about the limit
        let err_msg = err.to_string();
        assert!(err_msg.contains("exceeds limit") || err_msg.contains("Security"));
    }

    #[test]
    fn test_total_size_limit_enforcement() {
        // Create CSV with 110MB total data (exceeds default 100MB)
        let mut csv = String::from("id,data\n");
        let row_data = "x".repeat(100_000); // 100KB per row

        for i in 0..1_100 {
            csv.push_str(&format!("{i},\"{row_data}\"\n"));
        }

        let result = from_csv_with_config(&csv, "Item", &["data"], FromCsvConfig::default());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CsvError::Security { .. }));
        assert!(err.to_string().contains("total size"));
    }

    #[test]
    fn test_header_size_limit_enforcement() {
        // Create CSV with 2MB total header size (exceeds default 1MB)
        let mut csv = String::new();
        for i in 0..20_000 {
            if i > 0 {
                csv.push(',');
            }
            csv.push_str(&format!("column_{i}_very_long_name_{i}"));
        }
        csv.push_str("\n1\n");

        let result = from_csv_with_config(&csv, "Item", &[], FromCsvConfig::default());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CsvError::Security { .. }));
        // Check that the error message contains information about the limit
        let err_msg = err.to_string();
        assert!(err_msg.contains("exceeds limit") || err_msg.contains("Security"));
    }

    #[test]
    fn test_normal_csv_within_limits() {
        // Normal CSV should work fine with default limits
        let csv_data = "id,name,age\n1,Alice,30\n2,Bob,25\n";

        let result = from_csv_with_config(
            csv_data,
            "Person",
            &["name", "age"],
            FromCsvConfig::default(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_unlimited_config_allows_large_csvs() {
        // Verify that unlimited() config allows huge CSVs
        let huge_cell = "x".repeat(10_000_000);
        let csv = format!("id,data\n1,\"{huge_cell}\"\n");

        let config = FromCsvConfig::unlimited();
        let result = from_csv_with_config(&csv, "Item", &["data"], config);

        // Should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_strict_config_blocks_large_cells() {
        // Even a moderately large cell should fail with strict config
        let csv = format!("id,data\n1,\"{}\"\n", "x".repeat(100_000));

        let config = FromCsvConfig::strict();
        let result = from_csv_with_config(&csv, "Item", &["data"], config);

        // Should fail - 100KB exceeds strict 64KB limit
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CsvError::Security { .. }));
    }

    #[test]
    fn test_strict_config_allows_small_csvs() {
        // Small CSV should work with strict config
        let csv = "id,data\n1,small_data\n";

        let config = FromCsvConfig::strict();
        let result = from_csv_with_config(csv, "Item", &["data"], config);

        assert!(result.is_ok());
    }
}
