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

//! Baseline management and regression detection.
//!
//! Manages performance baselines for regression detection across benchmark runs.

use crate::core::name_validation::validate_version_string;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Errors that can occur during baseline loading and management.
#[derive(Debug, thiserror::Error)]
pub enum BaselineError {
    /// Invalid baseline version string.
    #[error("Invalid baseline version: {0}")]
    InvalidVersion(
        /// Description of the validation error.
        String,
    ),

    /// Path traversal attempt detected.
    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(
        /// Description of the path traversal attempt.
        String,
    ),

    /// Cannot access the specified path.
    #[error("Cannot access path: {0}")]
    InvalidPath(
        /// Description of the path access error.
        String,
    ),

    /// Failed to load baseline file.
    #[error("Failed to load baseline '{0}': {1}")]
    LoadFailed(
        /// Baseline version that failed to load.
        String,
        /// Underlying I/O error.
        #[source]
        std::io::Error,
    ),

    /// Failed to parse baseline JSON.
    #[error("Failed to parse baseline '{0}': {1}")]
    ParseFailed(
        /// Baseline version that failed to parse.
        String,
        /// Underlying JSON error.
        #[source]
        serde_json::Error,
    ),

    /// No baseline directory found.
    #[error("No baseline directory found. Create ./baselines/ or set HEDL_BASELINE_DIR")]
    NoBaselineDirectory,

    /// Invalid baseline directory configuration.
    #[error("Invalid baseline directory: {0}")]
    InvalidBaselineDir(
        /// Description of the directory error.
        String,
    ),
}

/// Regression severity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RegressionStatus {
    /// No regression detected (0-4% slower).
    None,
    /// Minor regression (5-14% slower).
    Minor(
        /// Regression percentage.
        u8,
    ),
    /// Moderate regression (15-49% slower).
    Moderate(
        /// Regression percentage.
        u8,
    ),
    /// Severe regression (50%+ slower).
    Severe(
        /// Regression percentage.
        u8,
    ),
}

impl RegressionStatus {
    /// Returns the regression percentage.
    #[must_use]
    pub fn percentage(&self) -> u8 {
        match self {
            RegressionStatus::None => 0,
            RegressionStatus::Minor(p)
            | RegressionStatus::Moderate(p)
            | RegressionStatus::Severe(p) => *p,
        }
    }

    /// Returns whether this represents a regression.
    #[must_use]
    pub fn is_regression(&self) -> bool {
        !matches!(self, RegressionStatus::None)
    }

    /// Returns the severity level as a string.
    #[must_use]
    pub fn severity(&self) -> &str {
        match self {
            RegressionStatus::None => "none",
            RegressionStatus::Minor(_) => "minor",
            RegressionStatus::Moderate(_) => "moderate",
            RegressionStatus::Severe(_) => "severe",
        }
    }
}

/// Percentile measurements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Percentiles {
    /// 50th percentile (median).
    pub p50: u64,
    /// 95th percentile.
    pub p95: u64,
    /// 99th percentile.
    pub p99: u64,
}

impl Percentiles {
    /// Creates percentiles from a sorted vector of durations in nanoseconds.
    #[must_use]
    pub fn from_sorted(sorted_ns: &[u64]) -> Self {
        if sorted_ns.is_empty() {
            return Self {
                p50: 0,
                p95: 0,
                p99: 0,
            };
        }

        let len = sorted_ns.len();
        Self {
            p50: sorted_ns[len / 2],
            p95: sorted_ns[(len * 95) / 100],
            p99: sorted_ns[(len * 99) / 100],
        }
    }
}

/// Baseline data for a single benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkBaseline {
    /// Mean duration in nanoseconds.
    pub mean: u64,
    /// Standard deviation in nanoseconds.
    pub std_dev: u64,
    /// Percentile measurements.
    pub percentiles: Percentiles,
}

impl BenchmarkBaseline {
    /// Creates a new baseline from duration statistics.
    #[must_use]
    pub fn new(mean: Duration, std_dev: Duration, percentiles: Percentiles) -> Self {
        Self {
            mean: mean.as_nanos() as u64,
            std_dev: std_dev.as_nanos() as u64,
            percentiles,
        }
    }

    /// Returns the mean as a Duration.
    #[must_use]
    pub fn mean_duration(&self) -> Duration {
        Duration::from_nanos(self.mean)
    }

    /// Returns the standard deviation as a Duration.
    #[must_use]
    pub fn std_dev_duration(&self) -> Duration {
        Duration::from_nanos(self.std_dev)
    }
}

/// Complete baseline data for a version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    /// Version identifier (e.g., "1.0.0").
    pub version: String,
    /// Timestamp when baseline was created.
    pub timestamp: String,
    /// Map of benchmark name to baseline data.
    pub benchmarks: HashMap<String, BenchmarkBaseline>,
}

impl Baseline {
    /// Creates a new baseline for a version.
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            benchmarks: HashMap::new(),
        }
    }

    /// Adds a benchmark baseline.
    pub fn add_benchmark(&mut self, name: impl Into<String>, baseline: BenchmarkBaseline) {
        self.benchmarks.insert(name.into(), baseline);
    }

    /// Gets a benchmark baseline by name.
    #[must_use]
    pub fn get_benchmark(&self, name: &str) -> Option<&BenchmarkBaseline> {
        self.benchmarks.get(name)
    }
}

/// Gets the baseline directory, checking multiple sources.
///
/// Priority order:
/// 1. `HEDL_BASELINE_DIR` environment variable (for custom deployments)
/// 2. `./baselines/` (relative to current working directory)
/// 3. `$CARGO_MANIFEST_DIR/baselines/` (when run via cargo)
fn get_baseline_directory() -> Result<PathBuf, BaselineError> {
    // Check environment variable first
    if let Ok(dir) = std::env::var("HEDL_BASELINE_DIR") {
        let path = PathBuf::from(&dir);
        if path.is_dir() {
            return Ok(path);
        }
        return Err(BaselineError::InvalidBaselineDir(format!(
            "HEDL_BASELINE_DIR points to non-existent directory: {}",
            path.display()
        )));
    }

    // Check ./baselines/ relative to current directory
    let cwd_baselines = PathBuf::from("baselines");
    if cwd_baselines.is_dir() {
        return Ok(cwd_baselines);
    }

    // Fallback to crate's bundled baselines (when run via cargo)
    let manifest_dir = option_env!("CARGO_MANIFEST_DIR").unwrap_or(".");
    let bundled = PathBuf::from(manifest_dir).join("baselines");
    if bundled.is_dir() {
        return Ok(bundled);
    }

    Err(BaselineError::NoBaselineDirectory)
}

/// Sanitizes a version string to prevent path traversal attacks.
///
/// # Arguments
///
/// * `version` - The version string to sanitize
///
/// # Returns
///
/// Ok with the sanitized version string, or Err if malicious patterns are detected.
fn sanitize_version(version: &str) -> Result<String, BaselineError> {
    // Use the centralized validation function
    validate_version_string(version).map_err(|e| BaselineError::InvalidVersion(format!("{e}")))?;

    Ok(version.to_string())
}

/// Loads a baseline from a JSON file with security protections.
///
/// # Arguments
///
/// * `version` - Version identifier (e.g., "v1.0", "2024/q1")
///
/// # Returns
///
/// Result containing the loaded baseline or an error.
///
/// # Security
///
/// This function implements multiple layers of protection against path traversal:
/// - Input sanitization to reject obvious malicious patterns
/// - Path sandboxing to ensure files are within the baseline directory
/// - Canonical path verification to prevent symlink escape attacks
///
/// # Examples
///
/// ```ignore
/// // Load from baselines/v1.0.json
/// let baseline = load_baseline("v1.0")?;
///
/// // Load from subdirectory baselines/2024/q1.json
/// let baseline = load_baseline("2024/q1")?;
///
/// // Load with explicit .json extension
/// let baseline = load_baseline("v1.0.json")?;
/// ```
pub fn load_baseline(version: &str) -> Result<Baseline, BaselineError> {
    let baseline_dir = get_baseline_directory()?;

    // Sanitize input to prevent path traversal
    let safe_version = sanitize_version(version)?;

    // Build path within baseline directory
    let filename = if safe_version.ends_with(".json") {
        safe_version
    } else {
        format!("{safe_version}.json")
    };

    let requested_path = baseline_dir.join(&filename);

    // Canonicalize both paths for comparison
    let baseline_dir_canonical = baseline_dir.canonicalize().map_err(|e| {
        BaselineError::InvalidBaselineDir(format!("Cannot access baseline directory: {e}"))
    })?;

    let requested_canonical = requested_path.canonicalize().map_err(|e| {
        BaselineError::InvalidPath(format!(
            "Cannot access baseline '{version}': {e}. \
             Make sure the file exists and is readable."
        ))
    })?;

    // Security check: ensure resolved path is within baseline directory
    if !requested_canonical.starts_with(&baseline_dir_canonical) {
        return Err(BaselineError::PathTraversal(format!(
            "Baseline '{version}' resolves outside baselines directory"
        )));
    }

    // Safe to read
    let contents = fs::read_to_string(&requested_canonical)
        .map_err(|e| BaselineError::LoadFailed(version.to_string(), e))?;

    serde_json::from_str(&contents).map_err(|e| BaselineError::ParseFailed(version.to_string(), e))
}

/// Saves a baseline to a JSON file.
///
/// # Arguments
///
/// * `baseline` - The baseline to save
///
/// # Returns
///
/// Result indicating success or failure.
pub fn save_baseline(baseline: &Baseline) -> Result<(), BaselineError> {
    let baseline_dir = get_baseline_directory()?;
    let path = baseline_dir.join(format!("{}.json", baseline.version));

    // Ensure baselines directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| BaselineError::LoadFailed(baseline.version.clone(), e))?;
    }

    let json = serde_json::to_string_pretty(baseline)
        .map_err(|e| BaselineError::ParseFailed(baseline.version.clone(), e))?;
    fs::write(&path, json).map_err(|e| BaselineError::LoadFailed(baseline.version.clone(), e))?;
    Ok(())
}

/// Checks for regression between current measurement and baseline.
///
/// # Arguments
///
/// * `current_ns` - Current measurement duration in nanoseconds
/// * `baseline` - Baseline to compare against
///
/// # Returns
///
/// `RegressionStatus` indicating the regression severity.
#[must_use]
pub fn check_regression(current_ns: u64, baseline: &BenchmarkBaseline) -> RegressionStatus {
    if current_ns <= baseline.mean {
        return RegressionStatus::None;
    }

    let diff = current_ns - baseline.mean;
    let percentage = ((diff as f64 / baseline.mean as f64) * 100.0) as u8;

    match percentage {
        0..=4 => RegressionStatus::None,
        5..=14 => RegressionStatus::Minor(percentage),
        15..=49 => RegressionStatus::Moderate(percentage),
        _ => RegressionStatus::Severe(percentage),
    }
}

/// Updates the current baseline with new measurement.
///
/// # Arguments
///
/// * `current_path` - Path to current baseline file
/// * `benchmark_name` - Name of the benchmark
/// * `baseline_data` - New baseline data to add/update
pub fn update_current_baseline(
    current_path: &str,
    benchmark_name: &str,
    baseline_data: BenchmarkBaseline,
) -> Result<(), BaselineError> {
    let mut baseline = load_baseline(current_path).unwrap_or_else(|_| Baseline::new("current"));
    baseline.add_benchmark(benchmark_name, baseline_data);
    baseline.timestamp = chrono::Utc::now().to_rfc3339();

    let json = serde_json::to_string_pretty(&baseline)
        .map_err(|e| BaselineError::ParseFailed(current_path.to_string(), e))?;
    fs::write(current_path, json)
        .map_err(|e| BaselineError::LoadFailed(current_path.to_string(), e))?;
    Ok(())
}

// We need chrono for timestamps
use chrono;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regression_status() {
        let status = RegressionStatus::Minor(7);
        assert_eq!(status.percentage(), 7);
        assert!(status.is_regression());
        assert_eq!(status.severity(), "minor");

        let none = RegressionStatus::None;
        assert!(!none.is_regression());
    }

    #[test]
    fn test_percentiles() {
        let values = vec![100, 200, 300, 400, 500];
        let p = Percentiles::from_sorted(&values);
        assert_eq!(p.p50, 300);
        assert_eq!(p.p95, 500);
        assert_eq!(p.p99, 500);
    }

    #[test]
    fn test_benchmark_baseline() {
        let percentiles = Percentiles {
            p50: 1000,
            p95: 2000,
            p99: 3000,
        };
        let baseline = BenchmarkBaseline::new(
            Duration::from_millis(1),
            Duration::from_micros(100),
            percentiles,
        );

        assert_eq!(baseline.mean_duration(), Duration::from_millis(1));
        assert!(baseline.std_dev_duration() > Duration::ZERO);
    }

    #[test]
    fn test_baseline_management() {
        let mut baseline = Baseline::new("test");
        let bench_baseline = BenchmarkBaseline {
            mean: 1_000_000,
            std_dev: 100_000,
            percentiles: Percentiles {
                p50: 1_000_000,
                p95: 1_200_000,
                p99: 1_500_000,
            },
        };

        baseline.add_benchmark("test_bench", bench_baseline);
        assert!(baseline.get_benchmark("test_bench").is_some());
        assert_eq!(baseline.version, "test");
    }

    #[test]
    fn test_check_regression() {
        let baseline = BenchmarkBaseline {
            mean: 1_000_000,
            std_dev: 50_000,
            percentiles: Percentiles {
                p50: 1_000_000,
                p95: 1_100_000,
                p99: 1_200_000,
            },
        };

        // No regression
        assert_eq!(
            check_regression(1_000_000, &baseline),
            RegressionStatus::None
        );

        // Minor regression (7%)
        let status = check_regression(1_070_000, &baseline);
        assert!(matches!(status, RegressionStatus::Minor(_)));

        // Minor regression (10% - still in 5-14% range)
        let status = check_regression(1_100_000, &baseline);
        assert!(matches!(status, RegressionStatus::Minor(_)));

        // Moderate regression (20% - in 15-49% range)
        let status = check_regression(1_200_000, &baseline);
        assert!(matches!(status, RegressionStatus::Moderate(_)));

        // Severe regression (50%+)
        let status = check_regression(1_500_000, &baseline);
        assert!(matches!(status, RegressionStatus::Severe(_)));
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    /// Creates a temporary baselines directory with test files.
    fn setup_test_baselines() -> TempDir {
        let dir = TempDir::new().unwrap();
        let baselines = dir.path().join("baselines");
        fs::create_dir(&baselines).unwrap();

        // Create valid baseline
        fs::write(
            baselines.join("v1.0.json"),
            r#"{"version":"v1.0","timestamp":"2025-01-01T00:00:00Z","benchmarks":{}}"#,
        )
        .unwrap();

        // Create baseline with .json extension
        fs::write(
            baselines.join("v2.0.json"),
            r#"{"version":"v2.0","timestamp":"2025-01-01T00:00:00Z","benchmarks":{}}"#,
        )
        .unwrap();

        // Create baseline in subdirectory
        fs::create_dir(baselines.join("2024")).unwrap();
        fs::write(
            baselines.join("2024/q1.json"),
            r#"{"version":"2024-q1","timestamp":"2025-01-01T00:00:00Z","benchmarks":{}}"#,
        )
        .unwrap();

        dir
    }

    #[test]
    #[serial]
    fn test_path_traversal_double_dot_rejected() {
        let _tmp = setup_test_baselines();

        let attacks = vec![
            "../etc/passwd",
            "../../secrets/keys.json",
            "./../../../etc/passwd",
            "valid/../../../etc/passwd",
            "..",
            "../",
            "./..",
            "test/../etc/passwd",
        ];

        for attack in attacks {
            let result = load_baseline(attack);
            assert!(
                result.is_err(),
                "Path traversal '{attack}' should be rejected"
            );

            match result.unwrap_err() {
                BaselineError::InvalidVersion(_) => {}
                other => panic!(
                    "Wrong error type for '{attack}': expected InvalidVersion, got {other:?}"
                ),
            }
        }
    }

    #[test]
    #[serial]
    fn test_absolute_path_rejected() {
        let _tmp = setup_test_baselines();

        let absolute_paths = vec![
            "/etc/passwd.json",
            "/home/user/.ssh/config.json",
            "/tmp/test.json",
            "\\Windows\\System32\\config.json", // Windows path
        ];

        for path in absolute_paths {
            let result = load_baseline(path);
            assert!(result.is_err(), "Absolute path '{path}' should be rejected");

            match result.unwrap_err() {
                BaselineError::InvalidVersion(_) => {}
                other => {
                    panic!("Wrong error type for '{path}': expected InvalidVersion, got {other:?}")
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_windows_drive_letter_rejected() {
        let _tmp = setup_test_baselines();

        let windows_paths = vec![
            "C:/Windows/System32/config.json",
            "D:\\data\\secrets.json",
            "E:/test.json",
            "c:/windows/test.json", // lowercase drive letter
        ];

        for path in windows_paths {
            let result = load_baseline(path);
            assert!(result.is_err(), "Windows path '{path}' should be rejected");

            match result.unwrap_err() {
                BaselineError::InvalidVersion(_) => {}
                other => {
                    panic!("Wrong error type for '{path}': expected InvalidVersion, got {other:?}")
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_invalid_characters_rejected() {
        let _tmp = setup_test_baselines();

        let invalid_inputs = vec![
            "test\x00null", // Null byte
            "test<script>", // HTML tags
            "test|pipe",    // Pipe character
            "test;cmd",     // Command separator
            "test&args",    // Command separator
            "test$VAR",     // Variable expansion
            "test`cmd`",    // Command substitution
            "test$(cmd)",   // Command substitution
            "test\ninject", // Newline
            "test\tinject", // Tab
            "test\rinject", // Carriage return
            "test*",        // Wildcard
            "test?",        // Wildcard
            "test<>",       // Redirect
            "test{}",       // Brace expansion
        ];

        for invalid in invalid_inputs {
            let result = load_baseline(invalid);
            assert!(
                result.is_err(),
                "Invalid input '{invalid}' should be rejected"
            );

            match result.unwrap_err() {
                BaselineError::InvalidVersion(_) => {}
                other => panic!(
                    "Wrong error type for '{invalid}': expected InvalidVersion, got {other:?}"
                ),
            }
        }
    }

    #[test]
    #[serial]
    fn test_valid_baseline_loads() {
        let tmp = setup_test_baselines();
        std::env::set_var("HEDL_BASELINE_DIR", tmp.path().join("baselines"));

        let result = load_baseline("v1.0");
        assert!(result.is_ok(), "Should load valid baseline");
        assert_eq!(result.unwrap().version, "v1.0");

        std::env::remove_var("HEDL_BASELINE_DIR");
    }

    #[test]
    #[serial]
    fn test_valid_baseline_with_json_extension() {
        let tmp = setup_test_baselines();
        std::env::set_var("HEDL_BASELINE_DIR", tmp.path().join("baselines"));

        let result = load_baseline("v2.0.json");
        assert!(
            result.is_ok(),
            "Should load valid baseline with .json extension"
        );
        assert_eq!(result.unwrap().version, "v2.0");

        std::env::remove_var("HEDL_BASELINE_DIR");
    }

    #[test]
    #[serial]
    fn test_subdirectory_baseline_loads() {
        let tmp = setup_test_baselines();
        std::env::set_var("HEDL_BASELINE_DIR", tmp.path().join("baselines"));

        let result = load_baseline("2024/q1");
        assert!(result.is_ok(), "Should load baseline from subdirectory");
        assert_eq!(result.unwrap().version, "2024-q1");

        std::env::remove_var("HEDL_BASELINE_DIR");
    }

    #[test]
    #[serial]
    fn test_nonexistent_baseline_error() {
        let tmp = setup_test_baselines();
        std::env::set_var("HEDL_BASELINE_DIR", tmp.path().join("baselines"));

        let result = load_baseline("nonexistent");
        assert!(result.is_err(), "Should fail for nonexistent baseline");

        match result.unwrap_err() {
            BaselineError::InvalidPath(_) => {}
            other => panic!(
                "Wrong error type for nonexistent baseline: expected InvalidPath, got {other:?}"
            ),
        }

        std::env::remove_var("HEDL_BASELINE_DIR");
    }

    #[test]
    #[serial]
    fn test_symlink_escape_blocked() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let tmp = setup_test_baselines();
            let baselines = tmp.path().join("baselines");

            // Create file outside baselines directory
            let outside = tmp.path().join("secret.json");
            fs::write(
                &outside,
                r#"{"version":"secret","timestamp":"2025-01-01T00:00:00Z","benchmarks":{}}"#,
            )
            .unwrap();

            // Create symlink pointing outside baselines dir
            let link = baselines.join("evil.json");
            symlink(&outside, &link).unwrap();

            std::env::set_var("HEDL_BASELINE_DIR", &baselines);

            // Should be blocked because canonical path is outside baseline dir
            let result = load_baseline("evil");
            assert!(result.is_err(), "Symlink escape should be blocked");

            match result.unwrap_err() {
                BaselineError::PathTraversal(_) => {}
                other => panic!(
                    "Wrong error type for symlink escape: expected PathTraversal, got {other:?}"
                ),
            }

            std::env::remove_var("HEDL_BASELINE_DIR");
        }

        #[cfg(not(unix))]
        {
            // Skip symlink test on non-Unix systems
            println!("Skipping symlink test on non-Unix system");
        }
    }

    #[test]
    #[serial]
    fn test_env_var_baseline_directory() {
        let tmp = setup_test_baselines();
        let custom_dir = tmp.path().join("custom_baselines");
        fs::create_dir(&custom_dir).unwrap();

        fs::write(
            custom_dir.join("custom.json"),
            r#"{"version":"custom","timestamp":"2025-01-01T00:00:00Z","benchmarks":{}}"#,
        )
        .unwrap();

        std::env::set_var("HEDL_BASELINE_DIR", &custom_dir);

        let result = load_baseline("custom");
        assert!(result.is_ok(), "Should load from custom directory");
        assert_eq!(result.unwrap().version, "custom");

        std::env::remove_var("HEDL_BASELINE_DIR");
    }

    #[test]
    #[serial]
    fn test_env_var_nonexistent_directory() {
        std::env::set_var(
            "HEDL_BASELINE_DIR",
            "/nonexistent/directory/that/does/not/exist",
        );

        let result = load_baseline("test");
        assert!(result.is_err(), "Should fail with nonexistent directory");

        match result.unwrap_err() {
            BaselineError::InvalidBaselineDir(_) => {}
            other => panic!(
                "Wrong error type for nonexistent directory: expected InvalidBaselineDir, got {other:?}"
            ),
        }

        std::env::remove_var("HEDL_BASELINE_DIR");
    }

    #[test]
    #[serial]
    fn test_valid_version_names() {
        let tmp = setup_test_baselines();
        let baselines = tmp.path().join("baselines");

        // Create test files with valid names
        let valid_names = vec![
            "v1.0.0",
            "v2.1-beta",
            "release_2024",
            "test-123",
            "2024/q1",
            "2024/06/release",
            "current",
        ];

        for name in &valid_names {
            let path = baselines.join(format!("{name}.json"));
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(
                &path,
                format!(
                    r#"{{"version":"{name}","timestamp":"2025-01-01T00:00:00Z","benchmarks":{{}}}}"#
                ),
            )
            .unwrap();
        }

        std::env::set_var("HEDL_BASELINE_DIR", &baselines);

        for name in &valid_names {
            let result = load_baseline(name);
            assert!(
                result.is_ok(),
                "Should load valid version name '{name}': {result:?}"
            );
        }

        std::env::remove_var("HEDL_BASELINE_DIR");
    }

    #[test]
    #[serial]
    fn test_empty_version_rejected() {
        let _tmp = setup_test_baselines();

        let result = load_baseline("");
        assert!(result.is_err(), "Empty version should be rejected");

        match result.unwrap_err() {
            BaselineError::InvalidVersion(_) => {}
            other => {
                panic!("Wrong error type for empty version: expected InvalidVersion, got {other:?}")
            }
        }
    }

    #[test]
    #[serial]
    fn test_dot_and_dotdot_variations() {
        let _tmp = setup_test_baselines();

        let variations = vec![
            "...", "....", "./.", "././", "../", "..//", "./../", "/..", "/../", "a/.", "a/..",
            "a/./b", "a/../b",
        ];

        for variation in variations {
            let result = load_baseline(variation);
            // Some may be rejected by sanitization, others by path resolution
            // The important thing is they don't allow unauthorized access
            if result.is_err() {
                match result.unwrap_err() {
                    BaselineError::InvalidVersion(_)
                    | BaselineError::InvalidPath(_)
                    | BaselineError::PathTraversal(_) => {
                        // Expected
                    }
                    other => panic!("Unexpected error type for '{variation}': {other:?}"),
                }
            }
        }
    }
}
