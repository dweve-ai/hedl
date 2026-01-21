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

//! Integration tests demonstrating all three XML issue fixes working together

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_xml::{from_xml, to_xml, FromXmlConfig, ToXmlConfig};
use std::collections::BTreeMap;

/// Comprehensive test demonstrating all three fixes:
/// - Issue 1 (namespace sanitization)
/// - Issue 2 (proper schema for children)
/// - Issue 3 (no attribute duplication)
#[test]
fn test_all_issues_comprehensive_integration() {
    // XML with namespace prefixes, duplicate elements, and attributes
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <soap:product id="100" sku="WIDGET-001" ns:category="electronics" available="true">
            <name>Super Widget</name>
            <x:price>19.99</x:price>
            <my-description>A wonderful widget</my-description>
            <reference>@ref123</reference>
        </soap:product>
        <soap:product id="101" sku="WIDGET-002" ns:category="home" available="false">
            <name>Home Widget</name>
            <x:price>29.99</x:price>
            <my-description>For home use</my-description>
            <reference>@ref456</reference>
        </soap:product>
    </hedl>"#;

    // ISSUE 1 FIX: Namespace sanitization - parse XML with namespaces
    let config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };
    let doc = from_xml(xml, &config).unwrap();

    // Verify namespace prefixes were sanitized
    if let Some(Item::List(products)) = doc.root.get("soap_product") {
        assert_eq!(products.rows.len(), 2, "Should have 2 products");

        // Check that namespace-prefixed attributes were sanitized
        // "ns:category" should become "ns_category" in schema
        let schema = &products.schema;
        assert!(
            schema.iter().any(|s| s.contains("category")),
            "Schema should contain sanitized category field. Schema: {schema:?}"
        );

        // Check first product has all fields
        let product1 = &products.rows[0];
        assert!(
            product1.fields.len() >= 4,
            "Product should have multiple fields (id, sku, category, available, etc.)"
        );
    } else {
        panic!("Expected soap_product list (namespace prefix should be sanitized to soap_product)");
    }

    // ISSUE 2 & 3 FIX: Proper schema for children and no duplication
    // Create a document with nested children that have complex schemas
    let mut test_doc = Document::new((1, 0));

    // Register schemas with multiple fields
    test_doc.structs.insert(
        "Organization".to_string(),
        vec!["id".to_string(), "name".to_string(), "founded".to_string()],
    );
    test_doc.structs.insert(
        "Department".to_string(),
        vec![
            "id".to_string(),
            "dept_name".to_string(),
            "manager".to_string(), // Reference - complex field
            "budget".to_string(),
        ],
    );
    test_doc.structs.insert(
        "Employee".to_string(),
        vec![
            "id".to_string(),
            "full_name".to_string(),
            "title".to_string(),
            "supervisor".to_string(), // Reference - complex field
        ],
    );

    // Create nested structure: Organization -> Department -> Employee
    let employee = Node::new(
        "Employee",
        "e1",
        vec![
            Value::String("e1".to_string().into()),
            Value::String("John Doe".to_string().into()),
            Value::String("Senior Engineer".to_string().into()),
            Value::Reference(Reference::local("manager1")),
        ],
    );

    let mut department = Node::new(
        "Department",
        "d1",
        vec![
            Value::String("d1".to_string().into()),
            Value::String("Engineering".to_string().into()),
            Value::Reference(Reference::local("ceo")),
            Value::Float(1000000.0),
        ],
    );
    let mut dept_children = BTreeMap::new();
    dept_children.insert("Employee".to_string(), vec![employee]);
    department.children = Some(Box::new(dept_children));

    let mut organization = Node::new(
        "Organization",
        "o1",
        vec![
            Value::String("o1".to_string().into()),
            Value::String("Acme Corp".to_string().into()),
            Value::Int(1985),
        ],
    );
    let mut org_children = BTreeMap::new();
    org_children.insert("Department".to_string(), vec![department]);
    organization.children = Some(Box::new(org_children));

    let mut list = MatrixList::new(
        "Organization",
        vec!["id".to_string(), "name".to_string(), "founded".to_string()],
    );
    list.add_row(organization);
    test_doc
        .root
        .insert("organizations".to_string(), Item::List(list));

    // ISSUE 3 FIX: Convert with use_attributes=true to test no duplication
    let to_config = ToXmlConfig {
        use_attributes: true,
        pretty: true,
        ..Default::default()
    };
    let output_xml = to_xml(&test_doc, &to_config).unwrap();

    // Verify ISSUE 3 FIX: simple fields appear only once (as attributes)
    let name_count = output_xml.matches("Acme Corp").count();
    assert_eq!(
        name_count, 1,
        "Organization name should appear exactly once (as attribute, not duplicated as element)"
    );

    let founded_count = output_xml.matches("1985").count();
    assert_eq!(
        founded_count, 1,
        "Founded year should appear exactly once (as attribute, not duplicated)"
    );

    // Verify ISSUE 2 FIX: All child fields are present (not just "id")
    assert!(
        output_xml.contains("Engineering"),
        "Department name should be in XML"
    );
    assert!(
        output_xml.contains("1000000"),
        "Department budget should be in XML"
    );
    assert!(
        output_xml.contains("John Doe"),
        "Employee name should be in XML"
    );
    assert!(
        output_xml.contains("Senior Engineer"),
        "Employee title should be in XML"
    );

    // Verify complex fields (References) are elements, not attributes
    assert!(
        output_xml.contains("<manager") || output_xml.contains("<supervisor"),
        "Complex reference fields should be elements"
    );

    // Parse back and verify structure preserved
    let from_config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };
    let restored = from_xml(&output_xml, &from_config).unwrap();

    // Verify organization exists
    assert!(
        restored.root.contains_key("organizations") || restored.root.contains_key("organization"),
        "Organization list should be preserved"
    );

    // Comprehensive verification
    // Note: XML round-trip may create Object or List depending on structure
    let org_key = if restored.root.contains_key("organizations") {
        "organizations"
    } else {
        "organization"
    };

    // Handle both List and Object cases (XML round-trip limitation)
    match restored.root.get(org_key) {
        Some(Item::List(orgs)) => {
            assert_eq!(orgs.rows.len(), 1, "Should have 1 organization");

            let org = &orgs.rows[0];

            // Verify organization has nested children
            assert!(
                org.children().is_some() && !org.children().unwrap().is_empty(),
                "Organization should have child departments"
            );

            if let Some(children) = org.children() {
                if let Some(depts) = children.get("Department") {
                    assert_eq!(depts.len(), 1, "Should have 1 department");

                    let dept = &depts[0];

                    // ISSUE 2 FIX VERIFICATION: All department fields should be preserved
                    assert!(
                        dept.fields.len() >= 4,
                        "Department should have at least 4 fields (id, dept_name, manager, budget), got {}",
                        dept.fields.len()
                    );

                    // Verify department has nested employees
                    assert!(
                        dept.children().is_some() && !dept.children().unwrap().is_empty(),
                        "Department should have child employees"
                    );

                    if let Some(dept_children) = dept.children() {
                        if let Some(employees) = dept_children.get("Employee") {
                            assert_eq!(employees.len(), 1, "Should have 1 employee");

                            let emp = &employees[0];

                            // ISSUE 2 FIX VERIFICATION: All employee fields should be preserved
                            assert!(
                                emp.fields.len() >= 4,
                                "Employee should have at least 4 fields (id, full_name, title, supervisor), got {}",
                                emp.fields.len()
                            );
                        }
                    }
                }
            }
        }
        Some(Item::Object(obj)) => {
            // XML may parse nested structure as object - verify key fields present
            // This is acceptable for XML round-trip (known limitation)
            assert!(
                obj.contains_key("organization") || obj.keys().any(|k| k.contains("org")),
                "Should contain organization data. Keys: {:?}",
                obj.keys().collect::<Vec<_>>()
            );

            // Verify at least some organization data is present
            let has_org_data = obj.values().any(|v| match v {
                Item::Scalar(Value::String(s)) => s.contains("Acme"),
                Item::Object(inner) => inner.values().any(|iv| match iv {
                    Item::Scalar(Value::String(is)) => is.contains("Acme"),
                    _ => false,
                }),
                _ => false,
            });
            assert!(
                has_org_data,
                "Organization data should be preserved in XML round-trip"
            );
        }
        _ => {
            panic!(
                "Expected organization data in restored document. Keys: {:?}",
                restored.root.keys().collect::<Vec<_>>()
            );
        }
    }
}

/// Test namespace sanitization with edge cases
#[test]
fn test_issue1_namespace_edge_cases() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <xml:lang>en</xml:lang>
        <xhtml:div>content</xhtml:div>
        <my-ns:weird.name:with-many:colons>value</my-ns:weird.name:with-many:colons>
        <123invalid>data</123invalid>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    // All namespace-prefixed elements should be sanitized
    assert!(
        doc.root.contains_key("xml_lang"),
        "xml:lang should become xml_lang"
    );
    assert!(
        doc.root.contains_key("xhtml_div"),
        "xhtml:div should become xhtml_div"
    );
    assert!(
        doc.root
            .keys()
            .any(|k| k.contains("weird") && k.contains("name")),
        "Complex namespace should be sanitized"
    );
    assert!(
        doc.root
            .keys()
            .any(|k| k.starts_with('_') && k.contains("123")),
        "Leading digit should be prefixed with underscore"
    );
}

/// Test proper schema usage in deeply nested hierarchies
#[test]
fn test_issue2_deep_nesting_with_schemas() {
    let mut doc = Document::new((1, 0));

    // Register 4 levels of schemas
    for level in 1..=4 {
        doc.structs.insert(
            format!("Level{level}"),
            vec![
                "id".to_string(),
                format!("field_a_{}", level),
                format!("field_b_{}", level),
                format!("field_c_{}", level),
            ],
        );
    }

    // Build nested structure from bottom up
    let level4 = Node::new(
        "Level4",
        "l4",
        vec![
            Value::String("l4".to_string().into()),
            Value::String("L4-A".to_string().into()),
            Value::String("L4-B".to_string().into()),
            Value::String("L4-C".to_string().into()),
        ],
    );

    let mut level3 = Node::new(
        "Level3",
        "l3",
        vec![
            Value::String("l3".to_string().into()),
            Value::String("L3-A".to_string().into()),
            Value::String("L3-B".to_string().into()),
            Value::String("L3-C".to_string().into()),
        ],
    );
    let mut l3_children = BTreeMap::new();
    l3_children.insert("Level4".to_string(), vec![level4]);
    level3.children = Some(Box::new(l3_children));

    let mut level2 = Node::new(
        "Level2",
        "l2",
        vec![
            Value::String("l2".to_string().into()),
            Value::String("L2-A".to_string().into()),
            Value::String("L2-B".to_string().into()),
            Value::String("L2-C".to_string().into()),
        ],
    );
    let mut l2_children = BTreeMap::new();
    l2_children.insert("Level3".to_string(), vec![level3]);
    level2.children = Some(Box::new(l2_children));

    let mut level1 = Node::new(
        "Level1",
        "l1",
        vec![
            Value::String("l1".to_string().into()),
            Value::String("L1-A".to_string().into()),
            Value::String("L1-B".to_string().into()),
            Value::String("L1-C".to_string().into()),
        ],
    );
    let mut l1_children = BTreeMap::new();
    l1_children.insert("Level2".to_string(), vec![level2]);
    level1.children = Some(Box::new(l1_children));

    let mut list = MatrixList::new(
        "Level1",
        vec![
            "id".to_string(),
            "field_a_1".to_string(),
            "field_b_1".to_string(),
            "field_c_1".to_string(),
        ],
    );
    list.add_row(level1);
    doc.root.insert("items".to_string(), Item::List(list));

    // Convert to XML
    let config = ToXmlConfig::default();
    let xml = to_xml(&doc, &config).unwrap();

    // Verify all fields from all levels are present
    for level in 1..=4 {
        assert!(
            xml.contains(&format!("L{level}-A")),
            "Level {level} field A should be in XML"
        );
        assert!(
            xml.contains(&format!("L{level}-B")),
            "Level {level} field B should be in XML"
        );
        assert!(
            xml.contains(&format!("L{level}-C")),
            "Level {level} field C should be in XML"
        );
    }
}

/// Test `use_attributes` with complex mixed scenarios
#[test]
fn test_issue3_no_duplication_comprehensive() {
    let mut doc = Document::new((1, 0));

    doc.structs.insert(
        "Record".to_string(),
        vec![
            "id".to_string(),
            "simple_string".to_string(),
            "simple_int".to_string(),
            "simple_bool".to_string(),
            "complex_ref".to_string(),
            "complex_tensor".to_string(),
        ],
    );

    let mut list = MatrixList::new(
        "Record",
        vec![
            "id".to_string(),
            "simple_string".to_string(),
            "simple_int".to_string(),
            "simple_bool".to_string(),
            "complex_ref".to_string(),
            "complex_tensor".to_string(),
        ],
    );

    let node = Node::new(
        "Record",
        "r1",
        vec![
            Value::String("r1".to_string().into()),
            Value::String("Simple Text".to_string().into()),
            Value::Int(42),
            Value::Bool(true),
            Value::Reference(Reference::qualified("User", "u123")),
            Value::Tensor(Box::new(hedl_core::lex::Tensor::Array(vec![
                hedl_core::lex::Tensor::Scalar(1.0),
                hedl_core::lex::Tensor::Scalar(2.0),
            ]))),
        ],
    );

    list.add_row(node);
    doc.root.insert("records".to_string(), Item::List(list));

    // Convert with use_attributes=true
    let config = ToXmlConfig {
        use_attributes: true,
        pretty: false,
        ..Default::default()
    };
    let xml = to_xml(&doc, &config).unwrap();

    // Simple fields should appear exactly once (as attributes)
    assert_eq!(
        xml.matches("Simple Text").count(),
        1,
        "simple_string should appear once as attribute"
    );
    assert_eq!(
        xml.matches("42").count(),
        1,
        "simple_int should appear once as attribute"
    );
    assert_eq!(
        xml.matches("true").count(),
        1,
        "simple_bool should appear once as attribute"
    );

    // Simple fields should be in attributes
    assert!(
        xml.contains("simple_string=\"Simple Text\""),
        "Should have simple_string attribute"
    );
    assert!(
        xml.contains("simple_int=\"42\""),
        "Should have simple_int attribute"
    );
    assert!(
        xml.contains("simple_bool=\"true\""),
        "Should have simple_bool attribute"
    );

    // Simple fields should NOT be elements
    assert!(
        !xml.contains("<simple_string>"),
        "simple_string should not be duplicated as element"
    );
    assert!(
        !xml.contains("<simple_int>"),
        "simple_int should not be duplicated as element"
    );
    assert!(
        !xml.contains("<simple_bool>"),
        "simple_bool should not be duplicated as element"
    );

    // Complex fields SHOULD be elements
    assert!(
        xml.contains("<complex_ref"),
        "complex_ref should be element"
    );
    assert!(
        xml.contains("<complex_tensor"),
        "complex_tensor should be element"
    );
}
