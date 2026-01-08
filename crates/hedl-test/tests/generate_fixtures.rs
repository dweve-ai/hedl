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

//! Test that generates all fixture files.
//!
//! Run with: cargo test -p hedl-test --test generate_fixtures -- --ignored

use hedl_test::fixtures_as_hedl;
use std::fs;
use std::path::Path;

#[test]
#[ignore = "run manually to regenerate fixture files"]
fn generate_hedl_fixture_files() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    fs::create_dir_all(&fixtures_dir).expect("Failed to create fixtures directory");

    for (name, hedl_text) in fixtures_as_hedl() {
        let path = fixtures_dir.join(format!("{}.hedl", name));
        fs::write(&path, &hedl_text)
            .unwrap_or_else(|_| panic!("Failed to write {}", path.display()));
        println!("Generated: {}", path.display());
    }
}

#[test]
fn verify_fixtures_serialize() {
    // Ensure all fixtures can be serialized without errors
    for (name, hedl_text) in fixtures_as_hedl() {
        assert!(
            !hedl_text.starts_with("# Error"),
            "Fixture '{}' failed to serialize: {}",
            name,
            hedl_text
        );
    }
}
