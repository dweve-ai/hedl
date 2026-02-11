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

//! Batch processing result types.

use crate::error::CliError;
use std::path::PathBuf;

/// Result of processing a single file in a batch operation.
///
/// Contains the file path and either a success value or an error.
///
/// # Type Parameters
///
/// * `T` - The success type returned by the operation
#[derive(Debug, Clone)]
pub struct FileResult<T> {
    /// The file path that was processed
    pub path: PathBuf,
    /// The result of processing (Ok or Err)
    pub result: Result<T, CliError>,
}

impl<T> FileResult<T> {
    /// Create a successful file result.
    pub fn success(path: PathBuf, value: T) -> Self {
        Self {
            path,
            result: Ok(value),
        }
    }

    /// Create a failed file result.
    #[must_use]
    pub fn failure(path: PathBuf, error: CliError) -> Self {
        Self {
            path,
            result: Err(error),
        }
    }

    /// Check if the result is successful.
    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }

    /// Check if the result is a failure.
    pub fn is_failure(&self) -> bool {
        self.result.is_err()
    }
}

/// Aggregated results from a batch processing operation.
///
/// Contains all individual file results and provides statistics.
///
/// # Type Parameters
///
/// * `T` - The success type returned by the operation
#[derive(Debug, Clone)]
pub struct BatchResults<T> {
    /// Individual results for each processed file
    pub results: Vec<FileResult<T>>,
    /// Total processing time in milliseconds
    pub elapsed_ms: u128,
}

impl<T> BatchResults<T> {
    /// Create new batch results from a vector of file results.
    #[must_use]
    pub fn new(results: Vec<FileResult<T>>, elapsed_ms: u128) -> Self {
        Self {
            results,
            elapsed_ms,
        }
    }

    /// Get the total number of files processed.
    #[must_use]
    pub fn total_files(&self) -> usize {
        self.results.len()
    }

    /// Get the number of successfully processed files.
    #[must_use]
    pub fn success_count(&self) -> usize {
        self.results.iter().filter(|r| r.is_success()).count()
    }

    /// Get the number of failed files.
    #[must_use]
    pub fn failure_count(&self) -> usize {
        self.results.iter().filter(|r| r.is_failure()).count()
    }

    /// Check if all files were processed successfully.
    #[must_use]
    pub fn all_succeeded(&self) -> bool {
        self.results.iter().all(FileResult::is_success)
    }

    /// Check if any files failed.
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.results.iter().any(FileResult::is_failure)
    }

    /// Get an iterator over successful results.
    pub fn successes(&self) -> impl Iterator<Item = &FileResult<T>> {
        self.results.iter().filter(|r| r.is_success())
    }

    /// Get an iterator over failed results.
    pub fn failures(&self) -> impl Iterator<Item = &FileResult<T>> {
        self.results.iter().filter(|r| r.is_failure())
    }

    /// Get processing throughput in files per second.
    #[must_use]
    pub fn throughput(&self) -> f64 {
        if self.elapsed_ms == 0 {
            0.0
        } else {
            (self.total_files() as f64) / (self.elapsed_ms as f64 / 1000.0)
        }
    }
}
