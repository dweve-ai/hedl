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

//! Comprehensive integration tests for XSD schema validation

use hedl_xml::schema::{SchemaCache, SchemaValidator, ValidationError};
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

// Test Schemas

const PERSON_SCHEMA: &str = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="person">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="name" type="xs:string"/>
        <xs:element name="age" type="xs:integer"/>
        <xs:element name="email" type="xs:string" minOccurs="0"/>
      </xs:sequence>
      <xs:attribute name="id" type="xs:integer" use="required"/>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

const BOOK_SCHEMA: &str = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="library">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="book" maxOccurs="unbounded">
          <xs:complexType>
            <xs:sequence>
              <xs:element name="title" type="xs:string"/>
              <xs:element name="author" type="xs:string"/>
              <xs:element name="year" type="xs:integer"/>
              <xs:element name="price" type="xs:decimal"/>
            </xs:sequence>
            <xs:attribute name="isbn" type="xs:string" use="required"/>
          </xs:complexType>
        </xs:element>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

const PRODUCT_SCHEMA: &str = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="product">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="name" type="xs:string"/>
        <xs:element name="price" type="xs:decimal"/>
        <xs:element name="available" type="xs:boolean"/>
        <xs:element name="stock" type="xs:integer"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

// Basic Validation Tests

#[test]
fn test_valid_person_document() {
    let validator = SchemaValidator::from_xsd(PERSON_SCHEMA).unwrap();

    let xml = r#"<?xml version="1.0"?>
<person id="123">
  <name>Alice Smith</name>
  <age>30</age>
</person>"#;

    assert!(validator.validate(xml).is_ok());
}

#[test]
fn test_valid_person_with_optional_field() {
    let validator = SchemaValidator::from_xsd(PERSON_SCHEMA).unwrap();

    let xml = r#"<?xml version="1.0"?>
<person id="456">
  <name>Bob Johnson</name>
  <age>25</age>
  <email>bob@example.com</email>
</person>"#;

    assert!(validator.validate(xml).is_ok());
}

#[test]
fn test_invalid_integer_type() {
    let validator = SchemaValidator::from_xsd(PERSON_SCHEMA).unwrap();

    let xml = r#"<?xml version="1.0"?>
<person id="789">
  <name>Charlie Brown</name>
  <age>thirty</age>
</person>"#;

    let result = validator.validate(xml);
    assert!(result.is_err());

    match result.unwrap_err() {
        ValidationError::TypeValidationError {
            name,
            expected_type,
            value,
            ..
        } => {
            assert_eq!(name, "age");
            assert_eq!(expected_type, "xs:integer");
            assert_eq!(value, "thirty");
        }
        other => panic!("Expected TypeValidationError, got {:?}", other),
    }
}

#[test]
fn test_unknown_element_error() {
    let validator = SchemaValidator::from_xsd(PERSON_SCHEMA).unwrap();

    let xml = r#"<?xml version="1.0"?>
<person id="101">
  <name>Dave Wilson</name>
  <age>35</age>
  <phone>555-1234</phone>
</person>"#;

    let result = validator.validate(xml);
    assert!(result.is_err());

    match result.unwrap_err() {
        ValidationError::UnknownElement { element, .. } => {
            assert_eq!(element, "phone");
        }
        other => panic!("Expected UnknownElement, got {:?}", other),
    }
}

#[test]
fn test_malformed_xml_error() {
    let validator = SchemaValidator::from_xsd(PERSON_SCHEMA).unwrap();

    let xml = r#"<?xml version="1.0"?>
<person id="202">
  <name>Eve Davis
  <age>28</age>
</person>"#;

    let result = validator.validate(xml);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ValidationError::DocumentParseError { .. }
    ));
}

// Type Validation Tests

#[test]
fn test_boolean_type_validation() {
    let validator = SchemaValidator::from_xsd(PRODUCT_SCHEMA).unwrap();

    // Valid boolean values
    for bool_val in &["true", "false", "1", "0"] {
        let xml = format!(
            r#"<?xml version="1.0"?>
<product>
  <name>Widget</name>
  <price>19.99</price>
  <available>{}</available>
  <stock>100</stock>
</product>"#,
            bool_val
        );

        assert!(
            validator.validate(&xml).is_ok(),
            "Failed to validate boolean value: {}",
            bool_val
        );
    }

    // Invalid boolean value
    let xml = r#"<?xml version="1.0"?>
<product>
  <name>Widget</name>
  <price>19.99</price>
  <available>yes</available>
  <stock>100</stock>
</product>"#;

    let result = validator.validate(xml);
    assert!(result.is_err());
}

#[test]
fn test_decimal_type_validation() {
    let validator = SchemaValidator::from_xsd(PRODUCT_SCHEMA).unwrap();

    // Valid decimal
    let xml = r#"<?xml version="1.0"?>
<product>
  <name>Gadget</name>
  <price>29.95</price>
  <available>true</available>
  <stock>50</stock>
</product>"#;

    assert!(validator.validate(xml).is_ok());

    // Invalid decimal
    let xml = r#"<?xml version="1.0"?>
<product>
  <name>Gadget</name>
  <price>twenty-nine dollars</price>
  <available>true</available>
  <stock>50</stock>
</product>"#;

    let result = validator.validate(xml);
    assert!(result.is_err());
}

#[test]
fn test_integer_type_validation() {
    let validator = SchemaValidator::from_xsd(PRODUCT_SCHEMA).unwrap();

    // Valid integer
    let xml = r#"<?xml version="1.0"?>
<product>
  <name>Doohickey</name>
  <price>9.99</price>
  <available>true</available>
  <stock>200</stock>
</product>"#;

    assert!(validator.validate(xml).is_ok());

    // Invalid integer (decimal)
    let xml = r#"<?xml version="1.0"?>
<product>
  <name>Doohickey</name>
  <price>9.99</price>
  <available>true</available>
  <stock>50.5</stock>
</product>"#;

    let result = validator.validate(xml);
    assert!(result.is_err());
}

// Complex Document Tests

#[test]
fn test_valid_library_document() {
    let validator = SchemaValidator::from_xsd(BOOK_SCHEMA).unwrap();

    let xml = r#"<?xml version="1.0"?>
<library>
  <book isbn="978-0-123456-78-9">
    <title>The Great Book</title>
    <author>Jane Author</author>
    <year>2023</year>
    <price>29.99</price>
  </book>
  <book isbn="978-0-987654-32-1">
    <title>Another Great Book</title>
    <author>John Writer</author>
    <year>2024</year>
    <price>34.99</price>
  </book>
</library>"#;

    assert!(validator.validate(xml).is_ok());
}

#[test]
fn test_nested_element_validation() {
    let validator = SchemaValidator::from_xsd(BOOK_SCHEMA).unwrap();

    // Invalid nested element type
    let xml = r#"<?xml version="1.0"?>
<library>
  <book isbn="978-0-123456-78-9">
    <title>The Great Book</title>
    <author>Jane Author</author>
    <year>twenty-twenty-three</year>
    <price>29.99</price>
  </book>
</library>"#;

    let result = validator.validate(xml);
    assert!(result.is_err());
}

// Schema Cache Tests

#[test]
fn test_schema_cache_basic() {
    let cache = SchemaCache::new(10);
    assert_eq!(cache.size(), 0);

    // Create temporary schema file
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(PERSON_SCHEMA.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // First load
    let validator1 = cache.get_or_load(temp_file.path()).unwrap();
    assert_eq!(cache.size(), 1);

    // Second load should use cache
    let validator2 = cache.get_or_load(temp_file.path()).unwrap();
    assert_eq!(cache.size(), 1);

    // Should be same Arc instance
    assert!(std::sync::Arc::ptr_eq(&validator1, &validator2));
}

#[test]
fn test_schema_cache_eviction() {
    let cache = SchemaCache::new(2);

    let temp_dir = TempDir::new().unwrap();

    // Create 3 schema files
    let mut paths = vec![];
    for i in 0..3 {
        let path = temp_dir.path().join(format!("schema{}.xsd", i));
        fs::write(&path, PERSON_SCHEMA).unwrap();
        paths.push(path);
    }

    // Load first two
    cache.get_or_load(&paths[0]).unwrap();
    cache.get_or_load(&paths[1]).unwrap();
    assert_eq!(cache.size(), 2);

    // Load third - should evict oldest
    cache.get_or_load(&paths[2]).unwrap();
    assert_eq!(cache.size(), 2);
}

#[test]
fn test_schema_cache_clear() {
    let cache = SchemaCache::new(10);

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(PERSON_SCHEMA.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    cache.get_or_load(temp_file.path()).unwrap();
    assert_eq!(cache.size(), 1);

    cache.clear();
    assert_eq!(cache.size(), 0);
}

#[test]
fn test_schema_cache_multiple_schemas() {
    let cache = SchemaCache::new(10);
    let temp_dir = TempDir::new().unwrap();

    // Create multiple different schema files
    let schemas = vec![
        ("person.xsd", PERSON_SCHEMA),
        ("book.xsd", BOOK_SCHEMA),
        ("product.xsd", PRODUCT_SCHEMA),
    ];

    let mut paths = vec![];
    for (name, content) in schemas {
        let path = temp_dir.path().join(name);
        fs::write(&path, content).unwrap();
        paths.push(path);
    }

    // Load all schemas
    for path in &paths {
        cache.get_or_load(path).unwrap();
    }

    assert_eq!(cache.size(), 3);
}

// File-based Validation Tests

#[test]
fn test_validator_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(PERSON_SCHEMA.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let validator = SchemaValidator::from_file(temp_file.path()).unwrap();

    let xml = r#"<?xml version="1.0"?>
<person id="123">
  <name>Test User</name>
  <age>25</age>
</person>"#;

    assert!(validator.validate(xml).is_ok());
}

#[test]
fn test_validator_file_not_found() {
    use std::path::Path;

    let result = SchemaValidator::from_file(Path::new("/nonexistent/path/schema.xsd"));
    assert!(result.is_err());

    match result.unwrap_err() {
        ValidationError::SchemaNotFound { path } => {
            assert_eq!(path.to_str().unwrap(), "/nonexistent/path/schema.xsd");
        }
        other => panic!("Expected SchemaNotFound, got {:?}", other),
    }
}

// Error Display Tests

#[test]
fn test_error_display_type_validation() {
    let err = ValidationError::TypeValidationError {
        name: "age".to_string(),
        expected_type: "xs:integer".to_string(),
        value: "thirty".to_string(),
        line: Some(5),
    };

    let display = err.to_string();
    assert!(display.contains("age"));
    assert!(display.contains("xs:integer"));
    assert!(display.contains("thirty"));
    assert!(display.contains("line 5"));
}

#[test]
fn test_error_display_unknown_element() {
    let err = ValidationError::UnknownElement {
        element: "phone".to_string(),
        line: Some(10),
    };

    let display = err.to_string();
    assert!(display.contains("phone"));
    assert!(display.contains("line 10"));
}

#[test]
fn test_error_display_schema_not_found() {
    use std::path::PathBuf;

    let err = ValidationError::SchemaNotFound {
        path: PathBuf::from("/path/to/schema.xsd"),
    };

    let display = err.to_string();
    assert!(display.contains("schema.xsd"));
}

// Edge Cases and Security Tests

#[test]
fn test_empty_document() {
    let validator = SchemaValidator::from_xsd(PERSON_SCHEMA).unwrap();

    let xml = r#"<?xml version="1.0"?>"#;

    let result = validator.validate(xml);
    assert!(result.is_err());
}

#[test]
fn test_special_characters_in_content() {
    let validator = SchemaValidator::from_xsd(PERSON_SCHEMA).unwrap();

    let xml = r#"<?xml version="1.0"?>
<person id="123">
  <name>O'Connor &amp; Associates</name>
  <age>30</age>
</person>"#;

    assert!(validator.validate(xml).is_ok());
}

#[test]
fn test_unicode_content() {
    let validator = SchemaValidator::from_xsd(PERSON_SCHEMA).unwrap();

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<person id="123">
  <name>José García</name>
  <age>30</age>
</person>"#;

    assert!(validator.validate(xml).is_ok());
}

// Concurrent Access Tests

#[test]
fn test_concurrent_cache_access() {
    use std::sync::Arc;
    use std::thread;

    let cache = Arc::new(SchemaCache::new(10));

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(PERSON_SCHEMA.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    let path = temp_file.path().to_path_buf();

    // Spawn multiple threads
    let mut handles = vec![];
    for _ in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let path_clone = path.clone();
        let handle = thread::spawn(move || {
            for _ in 0..50 {
                let validator = cache_clone.get_or_load(&path_clone).unwrap();

                let xml = r#"<?xml version="1.0"?>
<person id="123">
  <name>Thread Test</name>
  <age>30</age>
</person>"#;

                validator.validate(xml).unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Should only be cached once
    assert_eq!(cache.size(), 1);
}

#[test]
fn test_concurrent_validation() {
    use std::sync::Arc;
    use std::thread;

    let validator = Arc::new(SchemaValidator::from_xsd(PERSON_SCHEMA).unwrap());

    let mut handles = vec![];
    for i in 0..10 {
        let validator_clone = Arc::clone(&validator);
        let handle = thread::spawn(move || {
            for j in 0..50 {
                let xml = format!(
                    r#"<?xml version="1.0"?>
<person id="{}">
  <name>Person {}</name>
  <age>{}</age>
</person>"#,
                    i * 100 + j,
                    i,
                    20 + j % 50
                );

                validator_clone.validate(&xml).unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

// Comprehensive Integration Test

#[test]
fn test_full_validation_workflow() {
    // Create cache
    let cache = SchemaCache::new(10);

    // Create temp directory for schemas
    let temp_dir = TempDir::new().unwrap();

    // Write schema to file
    let schema_path = temp_dir.path().join("person.xsd");
    fs::write(&schema_path, PERSON_SCHEMA).unwrap();

    // Load validator from cache
    let validator = cache.get_or_load(&schema_path).unwrap();

    // Valid document
    let valid_xml = r#"<?xml version="1.0"?>
<person id="100">
  <name>Integration Test User</name>
  <age>42</age>
  <email>test@example.com</email>
</person>"#;

    assert!(validator.validate(valid_xml).is_ok());

    // Invalid document - wrong type
    let invalid_xml = r#"<?xml version="1.0"?>
<person id="200">
  <name>Invalid User</name>
  <age>not a number</age>
</person>"#;

    assert!(validator.validate(invalid_xml).is_err());

    // Load same schema again - should use cache
    let validator2 = cache.get_or_load(&schema_path).unwrap();
    assert!(std::sync::Arc::ptr_eq(&validator, &validator2));

    // Validate with cached instance
    assert!(validator2.validate(valid_xml).is_ok());
}
