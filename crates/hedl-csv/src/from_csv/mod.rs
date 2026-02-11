// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! CSV to HEDL conversion

mod config;
mod conversion;
mod parsing;
mod schema_inference;
mod validation;

pub use config::{
    FromCsvConfig, DEFAULT_MAX_CELL_SIZE, DEFAULT_MAX_COLUMNS, DEFAULT_MAX_HEADER_SIZE,
    DEFAULT_MAX_ROWS, DEFAULT_MAX_TOTAL_SIZE,
};
pub use conversion::{
    from_csv, from_csv_reader, from_csv_reader_with_config, from_csv_with_config,
};

#[cfg(test)]
mod tests;
