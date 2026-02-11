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

//! Batch operation traits.

use crate::error::CliError;
use std::path::Path;

/// Trait for batch operations on HEDL files.
///
/// Implement this trait to define custom batch operations. The operation must be
/// thread-safe (Send + Sync) to support parallel processing.
///
/// # Type Parameters
///
/// * `Output` - The type returned on successful processing of a file
///
/// # Examples
///
/// ```rust
/// use hedl_cli::batch::BatchOperation;
/// use hedl_cli::error::CliError;
/// use std::path::Path;
///
/// struct CountLinesOperation;
///
/// impl BatchOperation for CountLinesOperation {
///     type Output = usize;
///
///     fn process_file(&self, path: &Path) -> Result<Self::Output, CliError> {
///         let content = std::fs::read_to_string(path)
///             .map_err(|e| CliError::io_error(path, e))?;
///         Ok(content.lines().count())
///     }
///
///     fn name(&self) -> &str {
///         "count-lines"
///     }
/// }
/// ```
pub trait BatchOperation: Send + Sync {
    /// The output type for successful processing
    type Output: Send;

    /// Process a single file and return the result.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to process
    ///
    /// # Returns
    ///
    /// * `Ok(Output)` - On successful processing
    /// * `Err(CliError)` - On any error
    ///
    /// # Errors
    ///
    /// Should return appropriate `CliError` variants for different failure modes.
    fn process_file(&self, path: &Path) -> Result<Self::Output, CliError>;

    /// Get a human-readable name for this operation.
    ///
    /// Used for progress reporting and logging.
    fn name(&self) -> &str;
}

/// Trait for streaming batch operations on HEDL files.
///
/// Unlike `BatchOperation` which loads entire files into memory,
/// streaming operations process files incrementally with constant memory usage.
/// This is ideal for processing large files (>100MB) or when memory is constrained.
///
/// # Memory Characteristics
///
/// - **Standard operations**: `O(num_threads` × `file_size`)
/// - **Streaming operations**: `O(buffer_size` + `ID_set`) ≈ constant
///
/// # Type Parameters
///
/// * `Output` - The type returned on successful processing of a file
///
/// # Examples
///
/// ```rust
/// use hedl_cli::batch::StreamingBatchOperation;
/// use hedl_cli::error::CliError;
/// use std::path::Path;
///
/// struct StreamingCountOperation;
///
/// impl StreamingBatchOperation for StreamingCountOperation {
///     type Output = usize;
///
///     fn process_file_streaming(&self, path: &Path) -> Result<Self::Output, CliError> {
///         use std::io::BufReader;
///         use std::fs::File;
///         use hedl_stream::StreamingParser;
///
///         let file = File::open(path).map_err(|e| CliError::io_error(path, e))?;
///         let reader = BufReader::new(file);
///         let parser = StreamingParser::new(reader)
///             .map_err(|e| CliError::parse(e.to_string()))?;
///
///         let count = parser.filter(|e| {
///             matches!(e, Ok(hedl_stream::NodeEvent::Node(_)))
///         }).count();
///
///         Ok(count)
///     }
///
///     fn name(&self) -> &str {
///         "count-streaming"
///     }
/// }
/// ```
pub trait StreamingBatchOperation: Send + Sync {
    /// The output type for successful processing
    type Output: Send;

    /// Process a file using streaming parser.
    ///
    /// # Arguments
    ///
    /// * `path` - File path to process
    ///
    /// # Returns
    ///
    /// * `Ok(Output)` - On successful processing
    /// * `Err(CliError)` - On any error
    ///
    /// # Memory Guarantee
    ///
    /// Implementations should maintain O(1) memory usage regardless of file size,
    /// processing the file incrementally using the streaming parser.
    fn process_file_streaming(&self, path: &Path) -> Result<Self::Output, CliError>;

    /// Get operation name for progress reporting
    fn name(&self) -> &str;

    /// Indicate if this operation can run in streaming mode.
    ///
    /// Some operations (like formatting) may require full document.
    /// Default: true
    fn supports_streaming(&self) -> bool {
        true
    }
}
