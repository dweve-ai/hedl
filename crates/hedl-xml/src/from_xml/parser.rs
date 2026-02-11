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

//! XML parsing functions for HEDL conversion

use super::config::{EntityPolicy, FromXmlConfig};
use super::conversion::{
    items_are_list_elements, items_are_tensor_elements, items_to_list,
    items_to_matrix_list_with_type, items_to_tensor, to_hedl_key,
};
use super::values::{parse_value_with_config, parse_version};
use hedl_core::convert::parse_reference;
use hedl_core::lex::singularize_and_capitalize;
use hedl_core::{Document, Item, Value};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::BTreeMap;

/// Maximum recursion depth for XML parsing (prevents stack overflow).
const MAX_RECURSION_DEPTH: usize = 100;

/// Convert XML string to HEDL Document
pub fn from_xml(xml: &str, config: &FromXmlConfig) -> Result<Document, String> {
    // Pre-scan for DOCTYPE declarations if strict policy
    if config.entity_policy == EntityPolicy::RejectDtd
        && (xml.contains("<!DOCTYPE") || xml.contains("<!ENTITY"))
    {
        return Err("DOCTYPE declarations rejected by entity policy (XXE prevention)".to_string());
    }

    let mut reader = Reader::from_str(xml);
    // Note: trim_text disabled to preserve whitespace around entity references
    // In quick-xml 0.38+, entities like &amp; are separate Event::GeneralRef events
    reader.config_mut().trim_text(false);

    let mut doc = Document::new(config.version);

    // Skip XML declaration and find root element
    loop {
        match reader.read_event() {
            Ok(Event::DocType(e)) => {
                if config.log_security_events {
                    eprintln!(
                        "[SECURITY] DTD detected in XML input at position {}: {:?}",
                        reader.buffer_position(),
                        String::from_utf8_lossy(&e)
                    );
                }

                match config.entity_policy {
                    EntityPolicy::RejectDtd => {
                        return Err(format!(
                            "DOCTYPE declaration rejected at position {} (XXE prevention policy)",
                            reader.buffer_position()
                        ));
                    }
                    EntityPolicy::WarnOnEntities => {
                        eprintln!(
                            "[WARNING] DOCTYPE detected in XML. External entities are NOT processed by quick-xml."
                        );
                    }
                    EntityPolicy::AllowDtdNoExternal => {
                        // Continue parsing, entities won't be resolved anyway
                    }
                }
            }
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                // Parse version from root if present
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    if key == "version" {
                        if let Some((major, minor)) = parse_version(&value) {
                            doc.version = (major, minor);
                        }
                    }
                }

                // Parse root content
                doc.root = parse_children(&mut reader, &name, config, &mut doc.structs, 0)?;
                break;
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(format!(
                    "XML parse error at position {}: {}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => {}
        }
    }

    Ok(doc)
}

pub(crate) fn parse_children(
    reader: &mut Reader<&[u8]>,
    parent_name: &str,
    config: &FromXmlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<BTreeMap<String, Item>, String> {
    // Security: Prevent stack overflow via deep recursion
    if depth > MAX_RECURSION_DEPTH {
        return Err(format!(
            "XML recursion depth exceeded (max: {})",
            MAX_RECURSION_DEPTH
        ));
    }
    let mut children = BTreeMap::new();
    // Track element items and explicit type attributes
    let mut element_counts: BTreeMap<String, (Vec<Item>, Option<String>)> = BTreeMap::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let raw_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let name = to_hedl_key(&raw_name);

                // Extract explicit type attribute if it looks like HEDL metadata
                // HEDL type names are PascalCase (e.g., type="Company" on <companies>)
                let explicit_type = e.attributes().flatten().find_map(|attr| {
                    let key = String::from_utf8_lossy(attr.key.as_ref());
                    if key == "type" {
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        // Only treat as HEDL metadata if value looks like a type name
                        // (starts with uppercase letter)
                        if value
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_uppercase())
                            .unwrap_or(false)
                        {
                            Some(value)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                let elem_owned = e.to_owned();
                let item = parse_element(reader, &elem_owned, config, structs, depth + 1)?;

                // Track repeated elements for list inference
                if config.infer_lists {
                    let entry = element_counts
                        .entry(name.clone())
                        .or_insert((Vec::new(), None));
                    entry.0.push(item);
                    // Store explicit type if provided (first one wins)
                    if entry.1.is_none() && explicit_type.is_some() {
                        entry.1 = explicit_type;
                    }
                } else {
                    // ISSUE 2 FIX: Detect duplicate elements when infer_lists is false
                    if children.contains_key(&name) {
                        return Err(format!(
                            "Duplicate element '{}' found with infer_lists=false. \
                             Enable infer_lists to automatically collect duplicates into a list.",
                            name
                        ));
                    }
                    children.insert(name, item);
                }
            }
            Ok(Event::Empty(e)) => {
                let raw_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let name = to_hedl_key(&raw_name);

                // Extract explicit type attribute if it looks like HEDL metadata
                // HEDL type names are PascalCase (e.g., type="Company" on <companies>)
                let explicit_type = e.attributes().flatten().find_map(|attr| {
                    let key = String::from_utf8_lossy(attr.key.as_ref());
                    if key == "type" {
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        // Only treat as HEDL metadata if value looks like a type name
                        // (starts with uppercase letter)
                        if value
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_uppercase())
                            .unwrap_or(false)
                        {
                            Some(value)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                let elem_owned = e.to_owned();
                let item = parse_empty_element(&elem_owned, config)?;

                if config.infer_lists {
                    let entry = element_counts
                        .entry(name.clone())
                        .or_insert((Vec::new(), None));
                    entry.0.push(item);
                    if entry.1.is_none() && explicit_type.is_some() {
                        entry.1 = explicit_type;
                    }
                } else {
                    // ISSUE 2 FIX: Detect duplicate elements when infer_lists is false
                    if children.contains_key(&name) {
                        return Err(format!(
                            "Duplicate element '{}' found with infer_lists=false. \
                             Enable infer_lists to automatically collect duplicates into a list.",
                            name
                        ));
                    }
                    children.insert(name, item);
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == parent_name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
    }

    // Process element counts to infer lists
    if config.infer_lists {
        for (name, (items, explicit_type)) in element_counts {
            if items.len() > 1 {
                // Multiple elements - convert to matrix list
                let list =
                    items_to_matrix_list_with_type(&name, items, explicit_type, config, structs)?;
                children.insert(name, Item::List(list));
            } else if let Some(item) = items.into_iter().next() {
                // Single item - but check if we have explicit type metadata
                if let Some(ref type_name) = explicit_type {
                    // Explicit type attribute means this should be a list
                    // The item might be wrapped: <companies type="Company"><company>...</company></companies>
                    // We need to extract the inner <company> element(s)
                    if let Item::Object(inner) = &item {
                        // Look for a child that matches the expected singular form
                        let expected_child = type_name.to_lowercase();
                        if let Some((_, child_item)) = inner
                            .iter()
                            .find(|(k, _)| k.to_lowercase() == expected_child)
                        {
                            match child_item {
                                Item::List(list) => {
                                    // Already a list - use it directly but update type name
                                    let mut new_list = list.clone();
                                    new_list.type_name = type_name.clone();
                                    structs.insert(type_name.clone(), new_list.schema.clone());
                                    children.insert(name, Item::List(new_list));
                                    continue;
                                }
                                Item::Object(obj) => {
                                    // Single object - wrap it in a list
                                    let list = items_to_matrix_list_with_type(
                                        &expected_child,
                                        vec![Item::Object(obj.clone())],
                                        Some(type_name.clone()),
                                        config,
                                        structs,
                                    )?;
                                    children.insert(name, Item::List(list));
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                    // Fallback: wrap the single item as a list
                    let list = items_to_matrix_list_with_type(
                        &name,
                        vec![item],
                        explicit_type,
                        config,
                        structs,
                    )?;
                    children.insert(name, Item::List(list));
                } else {
                    children.insert(name, item);
                }
            }
        }
    }

    Ok(children)
}

pub(crate) fn parse_element(
    reader: &mut Reader<&[u8]>,
    elem: &quick_xml::events::BytesStart<'_>,
    config: &FromXmlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<Item, String> {
    // Security: Prevent stack overflow via deep recursion
    if depth > MAX_RECURSION_DEPTH {
        return Err(format!(
            "XML recursion depth exceeded (max: {})",
            MAX_RECURSION_DEPTH
        ));
    }
    let name = String::from_utf8_lossy(elem.name().as_ref()).to_string();

    // Extract attributes (convert keys to valid HEDL format)
    let mut attributes = BTreeMap::new();
    let mut is_reference = false;
    for attr in elem.attributes().flatten() {
        let raw_key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let value = String::from_utf8_lossy(&attr.value).to_string();

        // Check for HEDL marker attributes
        if raw_key == "__hedl_type__" {
            if value == "ref" {
                is_reference = true;
            }
            continue; // Don't include in regular attributes
        }
        if raw_key == "__hedl_child__" {
            continue; // Skip child marker, it's handled separately
        }
        // Note: "type" attribute is NOT skipped here since it may be regular data.
        // List type inference is handled separately in parse_children using explicit_type.

        let key = to_hedl_key(&raw_key);
        attributes.insert(key, value);
    }

    // Parse content
    let mut text_content = String::new();
    let mut child_elements: BTreeMap<String, Vec<Item>> = BTreeMap::new();
    let mut marked_children: BTreeMap<String, Vec<Item>> = BTreeMap::new(); // Elements with __hedl_child__
    let mut has_children = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                has_children = true;
                let raw_child_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let child_name = to_hedl_key(&raw_child_name);

                // Check for __hedl_child__ marker attribute
                let is_marked_child = e.attributes().any(|attr| {
                    if let Ok(attr) = attr {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        let val = String::from_utf8_lossy(&attr.value);
                        key == "__hedl_child__" && val == "true"
                    } else {
                        false
                    }
                });

                let elem_owned = e.to_owned();
                let child_item = parse_element(reader, &elem_owned, config, structs, depth + 1)?;

                if is_marked_child {
                    marked_children
                        .entry(raw_child_name)
                        .or_default()
                        .push(child_item);
                } else {
                    child_elements
                        .entry(child_name)
                        .or_default()
                        .push(child_item);
                }
            }
            Ok(Event::Empty(e)) => {
                has_children = true;
                let raw_child_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let child_name = to_hedl_key(&raw_child_name);

                // Check for __hedl_child__ marker attribute
                let is_marked_child = e.attributes().any(|attr| {
                    if let Ok(attr) = attr {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        let val = String::from_utf8_lossy(&attr.value);
                        key == "__hedl_child__" && val == "true"
                    } else {
                        false
                    }
                });

                let elem_owned = e.to_owned();
                let child_item = parse_empty_element(&elem_owned, config)?;

                if is_marked_child {
                    marked_children
                        .entry(raw_child_name)
                        .or_default()
                        .push(child_item);
                } else {
                    child_elements
                        .entry(child_name)
                        .or_default()
                        .push(child_item);
                }
            }
            Ok(Event::Text(e)) => {
                let content = e
                    .xml_content()
                    .map_err(|e| format!("Text decode error: {}", e))?;
                text_content.push_str(&content);
            }
            Ok(Event::GeneralRef(e)) => {
                // Handle entity references (quick-xml 0.38+ reports these as separate events)
                let ref_name = e.decode().map_err(|e| format!("Ref decode error: {}", e))?;
                let unescaped = match ref_name.as_ref() {
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "quot" => "\"",
                    "apos" => "'",
                    _ => return Err(format!("Unknown entity reference: {}", ref_name)),
                };
                text_content.push_str(unescaped);
            }
            Ok(Event::End(e)) => {
                let end_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if end_name == name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
    }

    // Determine item type
    if has_children {
        // Convert collected child elements, inferring lists for repeated elements
        let mut result_children = BTreeMap::new();
        for (child_name, items) in child_elements {
            if items.len() > 1 {
                if config.infer_lists {
                    // Check element type and convert appropriately:
                    // 1. Numeric scalars -> tensor (numeric arrays with [...] syntax)
                    // 2. Non-numeric scalars -> list (non-numeric arrays with (...) syntax)
                    // 3. Objects -> matrix list
                    //
                    // IMPORTANT: Check tensor first, because numeric values are also valid list elements,
                    // but we want to preserve backward compatibility where numeric <item> elements
                    // become tensors, not lists.
                    if child_name == "item" && items_are_tensor_elements(&items) {
                        // Convert to tensor (numeric arrays with [...] syntax)
                        let tensor = items_to_tensor(&items)?;
                        result_children
                            .insert(child_name, Item::Scalar(Value::Tensor(Box::new(tensor))));
                    } else if child_name == "item" && items_are_list_elements(&items) {
                        // Convert to list (non-numeric arrays with (...) syntax)
                        let list = items_to_list(&items)?;
                        result_children
                            .insert(child_name, Item::Scalar(Value::List(Box::new(list))));
                    } else {
                        // Multiple elements with same name - convert to matrix list
                        use super::conversion::items_to_matrix_list;
                        let list = items_to_matrix_list(&child_name, items, config, structs)?;
                        result_children.insert(child_name, Item::List(list));
                    }
                } else {
                    // ISSUE 2 FIX: Error when duplicates found with infer_lists=false
                    return Err(format!(
                        "Duplicate element '{}' found with infer_lists=false. \
                         Enable infer_lists to automatically collect duplicates into a list.",
                        child_name
                    ));
                }
            } else if let Some(item) = items.into_iter().next() {
                result_children.insert(child_name, item);
            }
        }

        // Convert marked children (elements with __hedl_child__="true") to lists
        // These represent NEST hierarchical children that should be attached to nodes
        for (child_type_raw, child_items) in marked_children {
            if !child_items.is_empty() {
                // Convert to matrix list (even a single child becomes a list)
                use super::conversion::items_to_matrix_list;
                let list = items_to_matrix_list(&child_type_raw, child_items, config, structs)?;
                let child_key = to_hedl_key(&child_type_raw);
                result_children.insert(child_key, Item::List(list));
            }
        }

        // ISSUE 1 FIX: Merge attributes into the result object
        for (key, value_str) in attributes {
            let value = parse_value_with_config(&value_str, config)?;
            result_children.insert(key, Item::Scalar(value));
        }

        // Handle mixed content (text + children/attributes)
        if !text_content.trim().is_empty() {
            let value = if is_reference {
                Value::Reference(parse_reference(text_content.trim())?)
            } else {
                parse_value_with_config(&text_content, config)?
            };
            result_children.insert("_text".to_string(), Item::Scalar(value));
        }

        // Check if we should flatten: if object has single child that's a list or Value::List,
        // and the child name is the singular of the parent name, promote the list.
        // This handles XML patterns like:
        //   - <users><user>...</user><user>...</user></users> -> users:@User[...]
        //   - <roles><item>admin</item><item>editor</item></roles> -> roles: (admin, editor)
        // BUT: don't flatten if the list has hierarchical children (NEST structures)
        // ALSO: don't flatten if we have attributes or text content
        if result_children.len() == 1 {
            // SAFETY: len() == 1 guarantees at least one element
            let (child_key, child_item) = result_children.iter().next().expect("len == 1");

            // Check for MatrixList (Item::List)
            if let Item::List(list) = child_item {
                // Don't flatten if any rows have children (hierarchical nesting)
                let has_nested_children = list
                    .rows
                    .iter()
                    .any(|node| node.children().map(|c| !c.is_empty()).unwrap_or(false));
                if !has_nested_children {
                    // Check if child is singular form of parent
                    // Compare case-insensitively because XML element names may have different casing
                    // e.g., post_tags -> PostTag, but child element might be posttag -> Posttag
                    let parent_singular =
                        singularize_and_capitalize(&to_hedl_key(&name)).to_lowercase();
                    let child_type = singularize_and_capitalize(child_key).to_lowercase();
                    if parent_singular == child_type {
                        // Flatten: return the list directly
                        // SAFETY: len() == 1 guarantees at least one element
                        return Ok(result_children.into_values().next().expect("len == 1"));
                    }
                }
            }

            // Check for Value::List (Item::Scalar(Value::List))
            // This handles non-numeric arrays like (admin, editor, viewer)
            if let Item::Scalar(Value::List(_)) = child_item {
                // For Value::List, always flatten if the key is "item"
                // <roles><item>x</item><item>y</item></roles> -> roles: (x, y) not roles: { item: (x, y) }
                if child_key == "item" {
                    // SAFETY: len() == 1 guarantees at least one element
                    return Ok(result_children.into_values().next().expect("len == 1"));
                }
            }
        }

        // Object with nested elements
        Ok(Item::Object(result_children))
    } else if !text_content.trim().is_empty() {
        // Scalar with text content (and possibly attributes)
        let value = if is_reference {
            // Explicitly marked as reference
            Value::Reference(parse_reference(text_content.trim())?)
        } else {
            parse_value_with_config(&text_content, config)?
        };

        // ISSUE 1 FIX: If we have both text and attributes, create an object
        if !attributes.is_empty() {
            let mut obj = BTreeMap::new();
            obj.insert("_text".to_string(), Item::Scalar(value));
            for (key, value_str) in attributes {
                let attr_value = parse_value_with_config(&value_str, config)?;
                obj.insert(key, Item::Scalar(attr_value));
            }
            Ok(Item::Object(obj))
        } else {
            Ok(Item::Scalar(value))
        }
    } else if !attributes.is_empty() {
        // Empty element with attributes - convert to object
        let mut obj = BTreeMap::new();
        for (key, value_str) in attributes {
            let value = parse_value_with_config(&value_str, config)?;
            obj.insert(key, Item::Scalar(value));
        }
        Ok(Item::Object(obj))
    } else {
        // Empty element - null value
        Ok(Item::Scalar(Value::Null))
    }
}

pub(crate) fn parse_empty_element(
    elem: &quick_xml::events::BytesStart<'_>,
    config: &FromXmlConfig,
) -> Result<Item, String> {
    let mut attributes = BTreeMap::new();

    for attr in elem.attributes().flatten() {
        let raw_key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let key = to_hedl_key(&raw_key);
        let value = String::from_utf8_lossy(&attr.value).to_string();
        attributes.insert(key, value);
    }

    if attributes.is_empty() {
        Ok(Item::Scalar(Value::Null))
    } else if attributes.len() == 1 && attributes.contains_key("value") {
        // Special case: <elem value="x"/> -> scalar x
        // SAFETY: contains_key("value") guarantees get() succeeds
        let value_str = attributes.get("value").expect("key exists");
        let value = parse_value_with_config(value_str, config)?;
        Ok(Item::Scalar(value))
    } else {
        // Multiple attributes - convert to object
        let mut obj = BTreeMap::new();
        for (key, value_str) in attributes {
            let value = parse_value_with_config(&value_str, config)?;
            obj.insert(key, Item::Scalar(value));
        }
        Ok(Item::Object(obj))
    }
}
