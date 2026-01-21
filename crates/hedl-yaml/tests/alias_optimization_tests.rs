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

//! Tests for YAML Alias Resolution Optimization (Task 92)
//!
//! This test suite validates the correctness of the alias resolution optimization.
//! All tests must pass to ensure the optimization doesn't change behavior.

use hedl_core::{Item, Value};
use hedl_yaml::{from_yaml, FromYamlConfig};

// ==================== Correctness Tests ====================

#[test]
fn test_alias_resolution_correctness() {
    let yaml = r"
defaults: &defaults
  timeout: 30
  retries: 3
  config:
    nested: true

production:
  settings: *defaults
  override: custom

staging:
  settings: *defaults
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    // Verify both references resolve correctly
    let prod = doc.root.get("production").unwrap();
    if let Item::Object(prod_obj) = prod {
        let settings = prod_obj.get("settings").unwrap();
        if let Item::Object(settings_obj) = settings {
            // Check timeout
            if let Item::Scalar(Value::Int(timeout)) = settings_obj.get("timeout").unwrap() {
                assert_eq!(*timeout, 30);
            } else {
                panic!("Expected timeout to be Int(30)");
            }

            // Check retries
            if let Item::Scalar(Value::Int(retries)) = settings_obj.get("retries").unwrap() {
                assert_eq!(*retries, 3);
            } else {
                panic!("Expected retries to be Int(3)");
            }
        } else {
            panic!("Expected settings to be Object");
        }
    } else {
        panic!("Expected production to be Object");
    }

    // Verify staging has same content
    let staging = doc.root.get("staging").unwrap();
    if let Item::Object(staging_obj) = staging {
        let settings = staging_obj.get("settings").unwrap();
        if let Item::Object(settings_obj) = settings {
            if let Item::Scalar(Value::Int(timeout)) = settings_obj.get("timeout").unwrap() {
                assert_eq!(*timeout, 30);
            } else {
                panic!("Expected timeout to be Int(30)");
            }
        } else {
            panic!("Expected settings to be Object");
        }
    } else {
        panic!("Expected staging to be Object");
    }
}

#[test]
fn test_deeply_nested_aliases() {
    let yaml = r"
level1: &l1
  data: base
  nested: &l2
    value: middle
    deep: &l3
      final: end

ref1: *l1
ref2: *l2
ref3: *l3
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    // Verify all levels resolve correctly
    let ref3 = doc.root.get("ref3").unwrap();
    if let Item::Object(ref3_obj) = ref3 {
        if let Item::Scalar(Value::String(s)) = ref3_obj.get("final").unwrap() {
            assert_eq!(&**s, "end");
        } else {
            panic!("Expected 'final' to be a string");
        }
    } else {
        panic!("Expected ref3 to be Object");
    }

    // Verify ref2 contains nested structure
    let ref2 = doc.root.get("ref2").unwrap();
    if let Item::Object(ref2_obj) = ref2 {
        assert!(ref2_obj.contains_key("value"));
        assert!(ref2_obj.contains_key("deep"));
    } else {
        panic!("Expected ref2 to be Object");
    }

    // Verify ref1 contains all levels
    let ref1 = doc.root.get("ref1").unwrap();
    if let Item::Object(ref1_obj) = ref1 {
        assert!(ref1_obj.contains_key("data"));
        assert!(ref1_obj.contains_key("nested"));
    } else {
        panic!("Expected ref1 to be Object");
    }
}

#[test]
fn test_alias_with_many_references() {
    // 100 references to same anchor - memory optimization critical
    let mut yaml = String::from("shared: &shared\n  large_data: ");
    yaml.push_str(&"x".repeat(1000)); // 1KB shared block
    yaml.push_str("\n\nrefs:\n");

    for i in 0..100 {
        yaml.push_str(&format!("  ref{i}: *shared\n"));
    }

    let config = FromYamlConfig::default();
    let doc = from_yaml(&yaml, &config).unwrap();

    let refs = doc.root.get("refs").unwrap();
    if let Item::Object(refs_obj) = refs {
        assert_eq!(refs_obj.len(), 100);

        // Verify all references resolve to same content
        let first = refs_obj.get("ref0").unwrap();
        if let Item::Object(first_obj) = first {
            let first_data = first_obj.get("large_data").unwrap();
            if let Item::Scalar(Value::String(first_str)) = first_data {
                // Verify all other refs have same content
                for i in 1..100 {
                    let current = refs_obj.get(&format!("ref{i}")).unwrap();
                    if let Item::Object(current_obj) = current {
                        let current_data = current_obj.get("large_data").unwrap();
                        if let Item::Scalar(Value::String(current_str)) = current_data {
                            assert_eq!(current_str, first_str);
                        } else {
                            panic!("Expected large_data to be String");
                        }
                    } else {
                        panic!("Expected ref{i} to be Object");
                    }
                }
            } else {
                panic!("Expected large_data to be String");
            }
        } else {
            panic!("Expected ref0 to be Object");
        }
    } else {
        panic!("Expected refs to be Object");
    }
}

#[test]
fn test_multiple_anchors_independent() {
    let yaml = r"
anchor1: &a1
  value: 100

anchor2: &a2
  value: 200

anchor3: &a3
  value: 300

refs:
  ref_a1_1: *a1
  ref_a2_1: *a2
  ref_a3_1: *a3
  ref_a1_2: *a1
  ref_a2_2: *a2
  ref_a3_2: *a3
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let refs = doc.root.get("refs").unwrap();
    if let Item::Object(refs_obj) = refs {
        // Verify each anchor has correct value
        for (key, expected) in &[
            ("ref_a1_1", 100),
            ("ref_a1_2", 100),
            ("ref_a2_1", 200),
            ("ref_a2_2", 200),
            ("ref_a3_1", 300),
            ("ref_a3_2", 300),
        ] {
            let item = refs_obj.get(*key).unwrap();
            if let Item::Object(obj) = item {
                if let Item::Scalar(Value::Int(val)) = obj.get("value").unwrap() {
                    assert_eq!(*val, i64::from(*expected));
                } else {
                    panic!("Expected value to be Int");
                }
            } else {
                panic!("Expected {key} to be Object");
            }
        }
    } else {
        panic!("Expected refs to be Object");
    }
}

#[test]
fn test_anchor_references_in_simple_objects() {
    // Test that anchor references work correctly when used multiple times
    let yaml = r"
template: &item_template
  type: widget
  enabled: true
  priority: 5

collection:
  item_a: *item_template
  item_b: *item_template
  item_c: *item_template
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let collection = doc.root.get("collection").unwrap();
    if let Item::Object(collection_obj) = collection {
        // All three items should have the same content from template
        for key in &["item_a", "item_b", "item_c"] {
            let item = collection_obj.get(*key).unwrap();
            if let Item::Object(item_obj) = item {
                // Check type
                if let Item::Scalar(Value::String(type_val)) = item_obj.get("type").unwrap() {
                    assert_eq!(&**type_val, "widget");
                } else {
                    panic!("Expected type to be String");
                }

                // Check enabled
                if let Item::Scalar(Value::Bool(enabled)) = item_obj.get("enabled").unwrap() {
                    assert!(*enabled);
                } else {
                    panic!("Expected enabled to be Bool");
                }

                // Check priority
                if let Item::Scalar(Value::Int(priority)) = item_obj.get("priority").unwrap() {
                    assert_eq!(*priority, 5);
                } else {
                    panic!("Expected priority to be Int");
                }
            } else {
                panic!("Expected {key} to be Object");
            }
        }
    } else {
        panic!("Expected collection to be Object");
    }
}

#[test]
fn test_nested_anchors_in_objects() {
    // Test anchors within nested object structures
    let yaml = r"
database: &db_config
  host: localhost
  port: 5432
  timeout: 30

cache: &cache_config
  host: localhost
  port: 6379
  timeout: 60

services:
  primary_db: *db_config
  secondary_db: *db_config
  redis: *cache_config
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    // Verify services use the anchored configs
    let services = doc.root.get("services").unwrap();
    if let Item::Object(services_obj) = services {
        // primary_db should have database config
        let primary = services_obj.get("primary_db").unwrap();
        if let Item::Object(primary_obj) = primary {
            if let Item::Scalar(Value::Int(port)) = primary_obj.get("port").unwrap() {
                assert_eq!(*port, 5432);
            } else {
                panic!("Expected port to be Int");
            }
        } else {
            panic!("Expected primary_db to be Object");
        }

        // redis should have cache config
        let redis = services_obj.get("redis").unwrap();
        if let Item::Object(redis_obj) = redis {
            if let Item::Scalar(Value::Int(port)) = redis_obj.get("port").unwrap() {
                assert_eq!(*port, 6379);
            } else {
                panic!("Expected port to be Int");
            }
        } else {
            panic!("Expected redis to be Object");
        }
    } else {
        panic!("Expected services to be Object");
    }
}

#[test]
fn test_no_anchors_unchanged() {
    // Documents without anchors should parse identically
    let yaml = r"
user1:
  name: Alice
  age: 30

user2:
  name: Bob
  age: 25
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let user1 = doc.root.get("user1").unwrap();
    if let Item::Object(user1_obj) = user1 {
        if let Item::Scalar(Value::String(name)) = user1_obj.get("name").unwrap() {
            assert_eq!(&**name, "Alice");
        } else {
            panic!("Expected name to be String");
        }
    } else {
        panic!("Expected user1 to be Object");
    }
}

#[test]
fn test_anchor_with_complex_structure() {
    let yaml = r"
template: &template
  metadata:
    version: 1.0
    author: test
  settings:
    debug: true
    timeout: 30
  nested:
    deep:
      value: 42

instance1:
  config: *template

instance2:
  config: *template
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    // Verify both instances have the complex structure
    for instance in &["instance1", "instance2"] {
        let inst = doc.root.get(*instance).unwrap();
        if let Item::Object(inst_obj) = inst {
            let config_item = inst_obj.get("config").unwrap();
            if let Item::Object(config_obj) = config_item {
                // Verify metadata
                assert!(config_obj.contains_key("metadata"));

                // Verify settings
                assert!(config_obj.contains_key("settings"));

                // Verify nested
                assert!(config_obj.contains_key("nested"));
            } else {
                panic!("Expected config to be Object");
            }
        } else {
            panic!("Expected {instance} to be Object");
        }
    }
}

#[test]
fn test_sequential_anchors() {
    let yaml = r"
first: &a
  value: 1

second: &b
  value: 2
  ref: *a

third: &c
  value: 3
  ref: *b

result: *c
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let result = doc.root.get("result").unwrap();
    if let Item::Object(result_obj) = result {
        // Should have value 3
        if let Item::Scalar(Value::Int(val)) = result_obj.get("value").unwrap() {
            assert_eq!(*val, 3);
        } else {
            panic!("Expected value to be Int");
        }

        // Should have nested reference
        assert!(result_obj.contains_key("ref"));
    } else {
        panic!("Expected result to be Object");
    }
}

#[test]
fn test_empty_anchor() {
    let yaml = r"
empty: &empty {}

ref1: *empty
ref2: *empty
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let ref1 = doc.root.get("ref1").unwrap();
    if let Item::Object(ref1_obj) = ref1 {
        assert_eq!(ref1_obj.len(), 0);
    } else {
        panic!("Expected ref1 to be Object");
    }
}

#[test]
fn test_anchor_with_null_values() {
    let yaml = r"
template: &template
  value1: null
  value2: 42
  value3: null

ref: *template
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let ref_item = doc.root.get("ref").unwrap();
    if let Item::Object(ref_obj) = ref_item {
        // value1 should be null
        if let Item::Scalar(Value::Null) = ref_obj.get("value1").unwrap() {
            // OK
        } else {
            panic!("Expected value1 to be Null");
        }

        // value2 should be 42
        if let Item::Scalar(Value::Int(val)) = ref_obj.get("value2").unwrap() {
            assert_eq!(*val, 42);
        } else {
            panic!("Expected value2 to be Int");
        }

        // value3 should be null
        if let Item::Scalar(Value::Null) = ref_obj.get("value3").unwrap() {
            // OK
        } else {
            panic!("Expected value3 to be Null");
        }
    } else {
        panic!("Expected ref to be Object");
    }
}

// ==================== Edge Case Tests ====================

#[test]
fn test_anchor_same_as_key_name() {
    let yaml = r"
data: &data
  value: 100

reference:
  data: *data
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let reference = doc.root.get("reference").unwrap();
    if let Item::Object(ref_obj) = reference {
        let data = ref_obj.get("data").unwrap();
        if let Item::Object(data_obj) = data {
            if let Item::Scalar(Value::Int(val)) = data_obj.get("value").unwrap() {
                assert_eq!(*val, 100);
            } else {
                panic!("Expected value to be Int");
            }
        } else {
            panic!("Expected data to be Object");
        }
    } else {
        panic!("Expected reference to be Object");
    }
}

#[test]
fn test_multiple_anchors_in_complex_structure() {
    // Test multiple anchors with complex nested structures
    let yaml = r"
defaults: &defaults
  retry: 3
  timeout: 30

logging: &logging
  level: info
  format: json

service_a:
  name: ServiceA
  config: *defaults
  log: *logging

service_b:
  name: ServiceB
  config: *defaults
  log: *logging
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    // Verify service_a uses both anchors
    let service_a = doc.root.get("service_a").unwrap();
    if let Item::Object(service_a_obj) = service_a {
        // Check config
        if let Item::Object(config_obj) = service_a_obj.get("config").unwrap() {
            if let Item::Scalar(Value::Int(retry)) = config_obj.get("retry").unwrap() {
                assert_eq!(*retry, 3);
            } else {
                panic!("Expected retry to be Int");
            }
        } else {
            panic!("Expected config to be Object");
        }

        // Check log
        if let Item::Object(log_obj) = service_a_obj.get("log").unwrap() {
            if let Item::Scalar(Value::String(level)) = log_obj.get("level").unwrap() {
                assert_eq!(&**level, "info");
            } else {
                panic!("Expected level to be String");
            }
        } else {
            panic!("Expected log to be Object");
        }
    } else {
        panic!("Expected service_a to be Object");
    }

    // Verify service_b uses the same anchors
    let service_b = doc.root.get("service_b").unwrap();
    if let Item::Object(service_b_obj) = service_b {
        if let Item::Object(config_obj) = service_b_obj.get("config").unwrap() {
            if let Item::Scalar(Value::Int(retry)) = config_obj.get("retry").unwrap() {
                assert_eq!(*retry, 3);
            } else {
                panic!("Expected retry to be Int");
            }
        } else {
            panic!("Expected config to be Object");
        }
    } else {
        panic!("Expected service_b to be Object");
    }
}

#[test]
fn test_anchor_with_boolean_values() {
    let yaml = r"
flags: &flags
  enabled: true
  debug: false
  verbose: true

config: *flags
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let config_item = doc.root.get("config").unwrap();
    if let Item::Object(config_obj) = config_item {
        if let Item::Scalar(Value::Bool(enabled)) = config_obj.get("enabled").unwrap() {
            assert!(*enabled);
        } else {
            panic!("Expected enabled to be Bool");
        }

        if let Item::Scalar(Value::Bool(debug)) = config_obj.get("debug").unwrap() {
            assert!(!*debug);
        } else {
            panic!("Expected debug to be Bool");
        }
    } else {
        panic!("Expected config to be Object");
    }
}

#[test]
fn test_large_number_of_unique_anchors() {
    // Test with many different anchors
    let mut yaml = String::from("---\n");

    // Create 50 unique anchors
    for i in 0..50 {
        yaml.push_str(&format!("anchor{}: &a{}\n  value: {}\n", i, i, i * 10));
    }

    yaml.push_str("refs:\n");
    // Reference each anchor twice
    for i in 0..50 {
        yaml.push_str(&format!("  ref{i}_1: *a{i}\n"));
        yaml.push_str(&format!("  ref{i}_2: *a{i}\n"));
    }

    let config = FromYamlConfig::default();
    let doc = from_yaml(&yaml, &config).unwrap();

    let refs = doc.root.get("refs").unwrap();
    if let Item::Object(refs_obj) = refs {
        // Should have 100 references (50 anchors * 2 references each)
        assert_eq!(refs_obj.len(), 100);

        // Verify some random values
        for i in [0, 10, 25, 49] {
            let ref1 = refs_obj.get(&format!("ref{i}_1")).unwrap();
            let ref2 = refs_obj.get(&format!("ref{i}_2")).unwrap();

            if let (Item::Object(obj1), Item::Object(obj2)) = (ref1, ref2) {
                if let (Item::Scalar(Value::Int(val1)), Item::Scalar(Value::Int(val2))) =
                    (obj1.get("value").unwrap(), obj2.get("value").unwrap())
                {
                    assert_eq!(*val1, i * 10);
                    assert_eq!(*val2, i * 10);
                    assert_eq!(val1, val2);
                } else {
                    panic!("Expected value to be Int");
                }
            } else {
                panic!("Expected refs to be Objects");
            }
        }
    } else {
        panic!("Expected refs to be Object");
    }
}
