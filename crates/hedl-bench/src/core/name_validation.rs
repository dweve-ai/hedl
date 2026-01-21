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

//! Benchmark name validation for path safety.
//!
//! Provides strict validation of benchmark names to prevent:
//! - Path traversal attacks
//! - Cross-platform compatibility issues
//! - Unicode security issues

use crate::error::{BenchError, Result};

/// Maximum allowed length for benchmark names
pub const MAX_NAME_LENGTH: usize = 128;

/// Minimum allowed length for benchmark names
pub const MIN_NAME_LENGTH: usize = 1;

/// Characters allowed in benchmark names (ASCII subset)
/// Pattern: [a-zA-Z0-9_-]
const ALLOWED_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";

/// Windows reserved device names (case-insensitive)
const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Result of name validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// Name is valid.
    Valid,
    /// Name is invalid with specific error.
    Invalid(NameValidationError),
}

/// Specific validation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameValidationError {
    /// Name is empty
    Empty,
    /// Name exceeds maximum length
    TooLong {
        /// Actual length of the name.
        length: usize,
        /// Maximum allowed length.
        max: usize,
    },
    /// Name contains path separator
    ContainsPathSeparator {
        /// The separator character found.
        char: char,
        /// Position in the string.
        position: usize,
    },
    /// Name contains path traversal sequence
    ContainsPathTraversal,
    /// Name contains reserved character
    ContainsReservedChar {
        /// The reserved character found.
        char: char,
        /// Position in the string.
        position: usize,
    },
    /// Name contains control character
    ContainsControlChar {
        /// The control character byte value.
        byte: u8,
        /// Position in the string.
        position: usize,
    },
    /// Name contains non-ASCII character
    ContainsNonAscii {
        /// The non-ASCII character found.
        char: char,
        /// Position in the string.
        position: usize,
    },
    /// Name matches Windows reserved device name
    WindowsReservedName {
        /// The reserved name matched.
        name: String,
    },
    /// Name has problematic prefix or suffix
    ProblematicPrefixSuffix {
        /// Description of the issue.
        issue: String,
    },
}

impl std::fmt::Display for NameValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "benchmark name cannot be empty"),
            Self::TooLong { length, max } => {
                write!(f, "benchmark name too long: {length} chars (max: {max})")
            }
            Self::ContainsPathSeparator { char, position } => write!(
                f,
                "benchmark name contains path separator '{char}' at position {position}"
            ),
            Self::ContainsPathTraversal => {
                write!(f, "benchmark name contains path traversal sequence '..'")
            }
            Self::ContainsReservedChar { char, position } => write!(
                f,
                "benchmark name contains reserved character '{char}' at position {position}"
            ),
            Self::ContainsControlChar { byte, position } => write!(
                f,
                "benchmark name contains control character 0x{byte:02X} at position {position}"
            ),
            Self::ContainsNonAscii { char, position } => write!(
                f,
                "benchmark name contains non-ASCII character '{}' (U+{:04X}) at position {}",
                char, *char as u32, position
            ),
            Self::WindowsReservedName { name } => write!(
                f,
                "benchmark name '{name}' is a Windows reserved device name"
            ),
            Self::ProblematicPrefixSuffix { issue } => {
                write!(f, "benchmark name has problematic prefix/suffix: {issue}")
            }
        }
    }
}

/// Validates a benchmark name for path safety.
///
/// # Validation Rules
///
/// 1. Length: 1-128 characters
/// 2. Allowed characters: [a-zA-Z0-9_-]
/// 3. No path separators (/ or \)
/// 4. No path traversal (..)
/// 5. No Windows reserved names
/// 6. No leading/trailing dots or spaces
/// 7. ASCII only (no Unicode)
///
/// # Arguments
///
/// * `name` - The benchmark name to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err(BenchError::InvalidBenchmarkName)` otherwise
///
/// # Examples
///
/// ```
/// use hedl_bench::core::name_validation::validate_benchmark_name;
///
/// assert!(validate_benchmark_name("parse_users_1000").is_ok());
/// assert!(validate_benchmark_name("convert-json-to-hedl").is_ok());
/// assert!(validate_benchmark_name("../etc/passwd").is_err());
/// assert!(validate_benchmark_name("test<>name").is_err());
/// ```
pub fn validate_benchmark_name(name: &str) -> Result<()> {
    // Check for empty name
    if name.is_empty() {
        return Err(BenchError::InvalidBenchmarkName {
            name: name.to_string(),
            reason: NameValidationError::Empty.to_string(),
        });
    }

    // Check length
    if name.len() > MAX_NAME_LENGTH {
        return Err(BenchError::InvalidBenchmarkName {
            name: name.to_string(),
            reason: NameValidationError::TooLong {
                length: name.len(),
                max: MAX_NAME_LENGTH,
            }
            .to_string(),
        });
    }

    // Check for path traversal
    if name.contains("..") {
        return Err(BenchError::InvalidBenchmarkName {
            name: name.to_string(),
            reason: NameValidationError::ContainsPathTraversal.to_string(),
        });
    }

    // Check each character
    for (pos, c) in name.chars().enumerate() {
        // Check for path separators
        if c == '/' || c == '\\' {
            return Err(BenchError::InvalidBenchmarkName {
                name: name.to_string(),
                reason: NameValidationError::ContainsPathSeparator {
                    char: c,
                    position: pos,
                }
                .to_string(),
            });
        }

        // Check for non-ASCII
        if !c.is_ascii() {
            return Err(BenchError::InvalidBenchmarkName {
                name: name.to_string(),
                reason: NameValidationError::ContainsNonAscii {
                    char: c,
                    position: pos,
                }
                .to_string(),
            });
        }

        // Check for control characters
        if c.is_ascii_control() {
            return Err(BenchError::InvalidBenchmarkName {
                name: name.to_string(),
                reason: NameValidationError::ContainsControlChar {
                    byte: c as u8,
                    position: pos,
                }
                .to_string(),
            });
        }

        // Check for allowed characters
        if !ALLOWED_CHARS.contains(c) {
            return Err(BenchError::InvalidBenchmarkName {
                name: name.to_string(),
                reason: NameValidationError::ContainsReservedChar {
                    char: c,
                    position: pos,
                }
                .to_string(),
            });
        }
    }

    // Check for Windows reserved names
    let upper = name.to_uppercase();
    for reserved in WINDOWS_RESERVED {
        if upper == *reserved || upper.starts_with(&format!("{reserved}.")) {
            return Err(BenchError::InvalidBenchmarkName {
                name: name.to_string(),
                reason: NameValidationError::WindowsReservedName {
                    name: (*reserved).to_string(),
                }
                .to_string(),
            });
        }
    }

    // Check for problematic prefix/suffix
    if name.starts_with('-') {
        return Err(BenchError::InvalidBenchmarkName {
            name: name.to_string(),
            reason: NameValidationError::ProblematicPrefixSuffix {
                issue: "starts with hyphen".to_string(),
            }
            .to_string(),
        });
    }

    if name.starts_with('.') || name.ends_with('.') {
        return Err(BenchError::InvalidBenchmarkName {
            name: name.to_string(),
            reason: NameValidationError::ProblematicPrefixSuffix {
                issue: "starts or ends with dot".to_string(),
            }
            .to_string(),
        });
    }

    Ok(())
}

/// Validates a version string for baseline file paths.
///
/// Same rules as benchmark names, but also allows dots for semver and forward slashes
/// for subdirectories (e.g., "2024/q1"). Backslashes are always rejected.
pub fn validate_version_string(version: &str) -> Result<()> {
    // Allow dots and forward slashes in version strings for semver and subdirectories
    // But still validate other rules

    if version.is_empty() {
        return Err(BenchError::InvalidConfig {
            parameter: "version".to_string(),
            reason: "version string cannot be empty".to_string(),
        });
    }

    if version.len() > MAX_NAME_LENGTH {
        return Err(BenchError::InvalidConfig {
            parameter: "version".to_string(),
            reason: format!(
                "version string too long: {} chars (max: {})",
                version.len(),
                MAX_NAME_LENGTH
            ),
        });
    }

    // Reject path traversal sequences
    if version.contains("..") {
        return Err(BenchError::InvalidConfig {
            parameter: "version".to_string(),
            reason: "version string contains path traversal sequence '..'".to_string(),
        });
    }

    // Reject absolute paths
    if version.starts_with('/') {
        return Err(BenchError::InvalidConfig {
            parameter: "version".to_string(),
            reason: "version string cannot start with '/' (absolute path)".to_string(),
        });
    }

    for (pos, c) in version.chars().enumerate() {
        // Backslash is always rejected
        if c == '\\' {
            return Err(BenchError::InvalidConfig {
                parameter: "version".to_string(),
                reason: format!("version string contains backslash at position {pos}"),
            });
        }

        if !c.is_ascii() {
            return Err(BenchError::InvalidConfig {
                parameter: "version".to_string(),
                reason: format!("version string contains non-ASCII character at position {pos}"),
            });
        }

        if c.is_ascii_control() {
            return Err(BenchError::InvalidConfig {
                parameter: "version".to_string(),
                reason: format!("version string contains control character at position {pos}"),
            });
        }

        // Allow alphanumeric, underscore, hyphen, dot for semver, and forward slash for subdirectories
        if !c.is_ascii_alphanumeric() && c != '_' && c != '-' && c != '.' && c != '/' {
            return Err(BenchError::InvalidConfig {
                parameter: "version".to_string(),
                reason: format!(
                    "version string contains invalid character '{c}' at position {pos}"
                ),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert!(validate_benchmark_name("parse_users").is_ok());
        assert!(validate_benchmark_name("convert-json").is_ok());
        assert!(validate_benchmark_name("benchmark123").is_ok());
        assert!(validate_benchmark_name("UPPERCASE").is_ok());
        assert!(validate_benchmark_name("a").is_ok());
        assert!(validate_benchmark_name("a_b-c").is_ok());
    }

    #[test]
    fn test_empty_name() {
        assert!(validate_benchmark_name("").is_err());
    }

    #[test]
    fn test_too_long_name() {
        let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
        assert!(validate_benchmark_name(&long_name).is_err());
    }

    #[test]
    fn test_path_traversal() {
        assert!(validate_benchmark_name("..").is_err());
        assert!(validate_benchmark_name("../etc/passwd").is_err());
        assert!(validate_benchmark_name("test/../other").is_err());
        assert!(validate_benchmark_name("..\\windows").is_err());
    }

    #[test]
    fn test_path_separators() {
        assert!(validate_benchmark_name("test/bench").is_err());
        assert!(validate_benchmark_name("test\\bench").is_err());
        assert!(validate_benchmark_name("/absolute").is_err());
        assert!(validate_benchmark_name("\\unc\\path").is_err());
    }

    #[test]
    fn test_reserved_chars() {
        assert!(validate_benchmark_name("test<name").is_err());
        assert!(validate_benchmark_name("test>name").is_err());
        assert!(validate_benchmark_name("test:name").is_err());
        assert!(validate_benchmark_name("test\"name").is_err());
        assert!(validate_benchmark_name("test|name").is_err());
        assert!(validate_benchmark_name("test?name").is_err());
        assert!(validate_benchmark_name("test*name").is_err());
    }

    #[test]
    fn test_control_chars() {
        assert!(validate_benchmark_name("test\0name").is_err());
        assert!(validate_benchmark_name("test\nname").is_err());
        assert!(validate_benchmark_name("test\tname").is_err());
    }

    #[test]
    fn test_unicode() {
        assert!(validate_benchmark_name("test\u{200B}name").is_err()); // Zero-width space
        assert!(validate_benchmark_name("test\u{202E}name").is_err()); // RTL override
        assert!(validate_benchmark_name("cafe\u{0301}").is_err()); // Combining accent
        assert!(validate_benchmark_name("test\u{0430}").is_err()); // Cyrillic 'a'
    }

    #[test]
    fn test_windows_reserved() {
        assert!(validate_benchmark_name("CON").is_err());
        assert!(validate_benchmark_name("con").is_err());
        assert!(validate_benchmark_name("PRN").is_err());
        assert!(validate_benchmark_name("AUX").is_err());
        assert!(validate_benchmark_name("NUL").is_err());
        assert!(validate_benchmark_name("COM1").is_err());
        assert!(validate_benchmark_name("LPT1").is_err());
    }

    #[test]
    fn test_problematic_prefix_suffix() {
        assert!(validate_benchmark_name("-test").is_err());
        assert!(validate_benchmark_name(".hidden").is_err());
        assert!(validate_benchmark_name("test.").is_err());
    }

    #[test]
    fn test_valid_versions() {
        assert!(validate_version_string("1.0.0").is_ok());
        assert!(validate_version_string("v2.3.4-beta").is_ok());
        assert!(validate_version_string("current").is_ok());
        assert!(validate_version_string("2024/q1").is_ok());
        assert!(validate_version_string("2024/06/release").is_ok());
    }

    #[test]
    fn test_invalid_versions() {
        assert!(validate_version_string("").is_err());
        assert!(validate_version_string("../../../etc/passwd").is_err());
        assert!(validate_version_string("1.0.0\0evil").is_err());
        assert!(validate_version_string("/absolute/path").is_err());
        assert!(validate_version_string("test\\backslash").is_err());
    }
}
