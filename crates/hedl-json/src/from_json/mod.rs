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

//! JSON to HEDL conversion

mod array_conversion;
mod auto_nesting;
mod config;
mod converter;
mod partial;
mod surrogate;
mod uniform_schema;
mod value_conversion;

// Re-export public API from config module
pub use config::{
    FromJsonConfig, FromJsonConfigBuilder, JsonConversionError, SurrogatePolicy,
    DEFAULT_MAX_ARRAY_SIZE, DEFAULT_MAX_DEPTH, DEFAULT_MAX_OBJECT_SIZE, DEFAULT_MAX_STRING_LENGTH,
};

// Re-export public API from converter module
pub use converter::{from_json, from_json_value, from_json_value_owned};

// Re-export public API from partial module
pub use partial::{
    partial_parse_json, partial_parse_json_value, ErrorLocation, ErrorTolerance, ParseError,
    PartialConfig, PartialConfigBuilder, PartialResult,
};

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most tests are in crates/hedl-json/tests/
    // This is a basic smoke test to ensure the modules work together

    #[test]
    fn test_basic_conversion() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let config = FromJsonConfig::default();
        let result = from_json(json, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = FromJsonConfig::builder()
            .max_depth(1000)
            .max_array_size(100_000)
            .build();
        assert_eq!(config.max_depth, Some(1000));
        assert_eq!(config.max_array_size, Some(100_000));
    }

    #[test]
    fn test_partial_parse() {
        let json = r#"{"valid": "data", "users": [{"id": "1"}]}"#;
        let config = PartialConfig::builder()
            .tolerance(ErrorTolerance::CollectAll)
            .build();
        let result = partial_parse_json(json, &config);
        assert!(result.is_complete());
    }
}
