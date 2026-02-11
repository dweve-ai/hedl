// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! CSV validation and size tracking

use crate::error::{CsvError, Result};
use crate::from_csv::config::FromCsvConfig;

pub(crate) fn validate_headers(headers: &csv::StringRecord, config: &FromCsvConfig) -> Result<()> {
    // Check column count
    let column_count = headers.len();
    if column_count > config.max_columns {
        return Err(CsvError::Security {
            limit_type: "column count".to_string(),
            limit: config.max_columns,
            actual: column_count,
            message: format!(
                "CSV has {} columns, exceeds limit of {}",
                column_count, config.max_columns
            ),
        });
    }

    // Check total header size
    let header_size: usize = headers.iter().map(str::len).sum();
    if header_size > config.max_header_size {
        return Err(CsvError::Security {
            limit_type: "header size".to_string(),
            limit: config.max_header_size,
            actual: header_size,
            message: format!(
                "CSV header size {} bytes, exceeds limit of {} bytes",
                header_size, config.max_header_size
            ),
        });
    }

    // Check for individual column name size (prevent single huge name)
    for (i, header) in headers.iter().enumerate() {
        if header.len() > config.max_cell_size {
            // Safely create preview by finding the last complete character before byte 100
            let preview = if header.len() > 100 {
                // Find the last character boundary before position 100
                let mut preview_end = 100;
                while !header.is_char_boundary(preview_end) && preview_end > 0 {
                    preview_end -= 1;
                }
                format!("{}...", &header[..preview_end])
            } else {
                header.to_string()
            };
            return Err(CsvError::Security {
                limit_type: "column name size".to_string(),
                limit: config.max_cell_size,
                actual: header.len(),
                message: format!(
                    "Column name '{}' at index {} is {} bytes, exceeds cell size limit of {} bytes",
                    preview,
                    i,
                    header.len(),
                    config.max_cell_size
                ),
            });
        }
    }

    Ok(())
}

/// Validate a single cell against security limits.
///
/// This function checks that the cell size does not exceed `max_cell_size`.
///
/// # Arguments
///
/// * `cell` - The cell content to validate
/// * `row` - Row number (1-based, for error messages)
/// * `column` - Column index (0-based, for error messages)
/// * `config` - Configuration containing security limits
///
/// # Returns
///
/// `Ok(())` if the cell is within limits, otherwise an error.
pub(crate) fn validate_cell(
    cell: &str,
    row: usize,
    column: usize,
    config: &FromCsvConfig,
) -> Result<()> {
    if cell.len() > config.max_cell_size {
        // Safely create preview by finding the last complete character before byte 100
        let preview = if cell.len() > 100 {
            // Find the last character boundary before position 100
            let mut preview_end = 100;
            while !cell.is_char_boundary(preview_end) && preview_end > 0 {
                preview_end -= 1;
            }
            format!("{}...", &cell[..preview_end])
        } else {
            cell.to_string()
        };
        return Err(CsvError::Security {
            limit_type: "cell size".to_string(),
            limit: config.max_cell_size,
            actual: cell.len(),
            message: format!(
                "Cell at row {}, column {} is {} bytes, exceeds limit of {} bytes. Content preview: '{}'",
                row,
                column,
                cell.len(),
                config.max_cell_size,
                preview
            ),
        });
    }
    Ok(())
}

/// Tracker for CSV size during parsing.
///
/// This struct tracks the total bytes read during CSV parsing to prevent
/// decompression bomb attacks.
pub(crate) struct CsvSizeTracker {
    pub(crate) bytes_read: usize,
    max_total_size: usize,
}

impl CsvSizeTracker {
    /// Create a new size tracker with the specified maximum.
    pub(crate) fn new(max_total_size: usize) -> Self {
        Self {
            bytes_read: 0,
            max_total_size,
        }
    }

    /// Track a record and check if the total size exceeds the limit.
    ///
    /// # Arguments
    ///
    /// * `record` - The CSV record to track
    ///
    /// # Returns
    ///
    /// `Ok(())` if within limits, otherwise an error.
    pub(crate) fn track_record(&mut self, record: &csv::StringRecord) -> Result<()> {
        let record_size: usize = record.iter().map(str::len).sum();
        self.bytes_read += record_size;

        if self.bytes_read > self.max_total_size {
            return Err(CsvError::Security {
                limit_type: "total size".to_string(),
                limit: self.max_total_size,
                actual: self.bytes_read,
                message: format!(
                    "CSV total size {} bytes exceeds limit of {} bytes",
                    self.bytes_read, self.max_total_size
                ),
            });
        }

        Ok(())
    }
}
