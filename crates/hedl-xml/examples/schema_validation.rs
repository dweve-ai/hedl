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

//! XSD Schema Validation Example
//!
//! Demonstrates comprehensive schema validation features including:
//! - Creating validators from XSD strings
//! - Validating XML documents
//! - Handling validation errors
//! - Using schema cache for performance
//! - Working with schema files

use hedl_xml::schema::{SchemaCache, SchemaValidator, ValidationError};
use std::io::Write;
use tempfile::NamedTempFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HEDL XML Schema Validation Examples ===\n");

    // Example 1: Basic Schema Validation
    println!("1. Basic Schema Validation");
    println!("{}", "-".repeat(50));
    basic_validation()?;

    // Example 2: Type Validation
    println!("\n2. Type Validation");
    println!("{}", "-".repeat(50));
    type_validation()?;

    // Example 3: Error Handling
    println!("\n3. Error Handling");
    println!("{}", "-".repeat(50));
    error_handling()?;

    // Example 4: Schema Caching
    println!("\n4. Schema Caching");
    println!("{}", "-".repeat(50));
    schema_caching()?;

    // Example 5: Complex Document Validation
    println!("\n5. Complex Document Validation");
    println!("{}", "-".repeat(50));
    complex_validation()?;

    println!("\n=== All Examples Completed Successfully ===");
    Ok(())
}

/// Example 1: Basic schema validation
fn basic_validation() -> Result<(), Box<dyn std::error::Error>> {
    let schema = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="person">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="name" type="xs:string"/>
        <xs:element name="age" type="xs:integer"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

    println!("Creating validator from XSD schema...");
    let validator = SchemaValidator::from_xsd(schema)?;

    let valid_xml = r#"<?xml version="1.0"?>
<person>
  <name>Alice Smith</name>
  <age>30</age>
</person>"#;

    println!("Validating XML document...");
    match validator.validate(valid_xml) {
        Ok(_) => println!("✓ Document is valid!"),
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    Ok(())
}

/// Example 2: Type validation with different XML Schema types
fn type_validation() -> Result<(), Box<dyn std::error::Error>> {
    let schema = r#"<?xml version="1.0"?>
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

    let validator = SchemaValidator::from_xsd(schema)?;

    let xml = r#"<?xml version="1.0"?>
<product>
  <name>Premium Widget</name>
  <price>29.99</price>
  <available>true</available>
  <stock>150</stock>
</product>"#;

    println!("Validating product with multiple types:");
    println!("  - string: Premium Widget");
    println!("  - decimal: 29.99");
    println!("  - boolean: true");
    println!("  - integer: 150");

    match validator.validate(xml) {
        Ok(_) => println!("✓ All types validated successfully!"),
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    Ok(())
}

/// Example 3: Error handling and detailed error messages
fn error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let schema = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="person">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="name" type="xs:string"/>
        <xs:element name="age" type="xs:integer"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

    let validator = SchemaValidator::from_xsd(schema)?;

    // Test case 1: Invalid type
    println!("Test 1: Invalid type (age is not an integer)");
    let xml1 = r#"<?xml version="1.0"?>
<person>
  <name>Bob Johnson</name>
  <age>thirty</age>
</person>"#;

    match validator.validate(xml1) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => {
            println!("  ✓ Expected error: {}", e);
            match e {
                ValidationError::TypeValidationError {
                    name,
                    expected_type,
                    value,
                    line,
                } => {
                    println!("    Element: {}", name);
                    println!("    Expected: {}", expected_type);
                    println!("    Found: {}", value);
                    if let Some(l) = line {
                        println!("    Line: {}", l);
                    }
                }
                _ => {}
            }
        }
    }

    // Test case 2: Unknown element
    println!("\nTest 2: Unknown element (email not in schema)");
    let xml2 = r#"<?xml version="1.0"?>
<person>
  <name>Charlie Brown</name>
  <age>25</age>
  <email>charlie@example.com</email>
</person>"#;

    match validator.validate(xml2) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => {
            println!("  ✓ Expected error: {}", e);
        }
    }

    // Test case 3: Malformed XML
    println!("\nTest 3: Malformed XML (unclosed tag)");
    let xml3 = r#"<?xml version="1.0"?>
<person>
  <name>Dave Wilson
  <age>35</age>
</person>"#;

    match validator.validate(xml3) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => {
            println!("  ✓ Expected error: {}", e);
        }
    }

    Ok(())
}

/// Example 4: Schema caching for performance
fn schema_caching() -> Result<(), Box<dyn std::error::Error>> {
    let schema = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="message">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="text" type="xs:string"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

    // Create temporary schema file
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(schema.as_bytes())?;
    temp_file.flush()?;

    println!("Creating schema cache (max size: 10)");
    let cache = SchemaCache::new(10);

    println!("Loading schema from file (first time)...");
    let start = std::time::Instant::now();
    let validator1 = cache.get_or_load(temp_file.path())?;
    let first_load = start.elapsed();
    println!("  Time: {:?}", first_load);
    println!("  Cache size: {}", cache.size());

    println!("\nLoading schema from file (second time - cached)...");
    let start = std::time::Instant::now();
    let validator2 = cache.get_or_load(temp_file.path())?;
    let second_load = start.elapsed();
    println!("  Time: {:?}", second_load);
    println!("  Cache size: {}", cache.size());

    // Verify it's the same instance
    if std::sync::Arc::ptr_eq(&validator1, &validator2) {
        println!("  ✓ Using cached validator (same Arc instance)");
    }

    // Use cached validator
    let xml = r#"<?xml version="1.0"?>
<message>
  <text>Hello from cached schema!</text>
</message>"#;

    validator2.validate(xml)?;
    println!("\n✓ Validation with cached schema successful!");

    Ok(())
}

/// Example 5: Complex document validation
fn complex_validation() -> Result<(), Box<dyn std::error::Error>> {
    let schema = r#"<?xml version="1.0"?>
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

    let validator = SchemaValidator::from_xsd(schema)?;

    let xml = r#"<?xml version="1.0"?>
<library>
  <book isbn="978-0-123456-78-9">
    <title>Advanced Rust Programming</title>
    <author>Jane Developer</author>
    <year>2024</year>
    <price>49.99</price>
  </book>
  <book isbn="978-0-987654-32-1">
    <title>Machine Learning with Rust</title>
    <author>John Scientist</author>
    <year>2025</year>
    <price>59.99</price>
  </book>
  <book isbn="978-0-555555-55-5">
    <title>High-Performance Systems</title>
    <author>Alice Engineer</author>
    <year>2024</year>
    <price>54.99</price>
  </book>
</library>"#;

    println!("Validating complex library document with multiple books...");
    match validator.validate(xml) {
        Ok(_) => {
            println!("✓ Complex document validated successfully!");
            println!("  Validated:");
            println!("    - 3 book elements");
            println!("    - Required ISBN attributes");
            println!("    - String, integer, and decimal types");
        }
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    Ok(())
}
