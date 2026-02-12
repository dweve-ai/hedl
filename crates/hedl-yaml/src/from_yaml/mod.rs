// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! YAML to HEDL conversion

mod cache;
mod config;
mod conversion;
mod detection;
mod matrix_list;
mod schema;
mod tensor;
mod validation;
mod value_conversion;

pub use config::{
    FromYamlConfig, FromYamlConfigBuilder, DEFAULT_MAX_ARRAY_LENGTH, DEFAULT_MAX_DOCUMENT_SIZE,
    DEFAULT_MAX_NESTING_DEPTH,
};
pub use conversion::{from_yaml, from_yaml_value};

#[cfg(test)]
mod tests;
