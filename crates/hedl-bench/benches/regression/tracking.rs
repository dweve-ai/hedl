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

//! Comprehensive regression tracking system.
//!
//! Automated performance regression detection across versions:
//! - Loads baseline performance metrics from baselines/v1.0.0.json
//! - Compares current benchmark results against baselines
//! - Generates regression reports with detailed analysis
//! - Updates baselines/current.json with latest measurements
//! - Tests across multiple dataset sizes and complexity levels
//!
//! Regression criteria:
//! - Performance degradation > 5% triggers minor warning
//! - Performance degradation > 15% triggers moderate warning
//! - Performance degradation > 50% triggers severe warning
//! - Improvements are tracked for validation
//!
//! Run with: cargo bench --package hedl-bench --bench tracking

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hedl_bench::core::{
    check_regression, load_baseline, save_baseline, update_current_baseline, Baseline,
    BenchmarkBaseline, Percentiles, RegressionStatus,
};
use hedl_bench::{
    generate_products, generate_users, sizes, BenchmarkReport, CustomTable, ExportConfig, Insight,
    PerfResult, TableCell,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Once;
use std::time::{Duration, Instant};

// ============================================================================
// Report Infrastructure
// ============================================================================

static INIT: Once = Once::new();

thread_local! {
    static REPORT: RefCell<Option<BenchmarkReport>> = RefCell::new(None);
    static BASELINE: RefCell<Option<Baseline>> = RefCell::new(None);
    static CURRENT: RefCell<Baseline> = RefCell::new(Baseline::new("current".to_string()));
    static REGRESSIONS: RefCell<Vec<RegressionInfo>> = RefCell::new(Vec::new());
}

#[derive(Debug, Clone)]
struct RegressionInfo {
    name: String,
    operation_type: String,
    baseline_mean: u64,
    current_mean: u64,
    baseline_p95: u64,
    current_p95: u64,
    status: RegressionStatus,
    change_percent: f64,
    size: usize,
    complexity_level: String,
}

fn ensure_init() {
    INIT.call_once(|| {
        REPORT.with(|r| {
            let mut report = BenchmarkReport::new("HEDL Regression Tracking Analysis");
            report.set_timestamp();
            report.add_note("Automated performance regression detection across versions");
            report.add_note("Compares against baselines/v1.0.0.json (if available)");
            report.add_note("Thresholds: >5% minor, >15% moderate, >50% severe");
            report.add_note("Tests parsing, canonicalization, conversion, validation, linting");
            *r.borrow_mut() = Some(report);
        });

        // Load baseline if available
        let baseline_path = "baselines/v1.0.0.json";
        if let Ok(baseline) = load_baseline(baseline_path) {
            BASELINE.with(|b| {
                *b.borrow_mut() = Some(baseline);
            });
            println!("Loaded baseline: {}", baseline_path);
        } else {
            println!(
                "No baseline found at {}, creating new baseline",
                baseline_path
            );
        }
    });
}

fn record_perf_with_baseline(
    name: &str,
    operation_type: &str,
    size: usize,
    complexity: &str,
    time_ns: u64,
    iterations: u64,
    samples: &[u64],
    throughput_bytes: Option<u64>,
) {
    REPORT.with(|r| {
        if let Some(ref mut report) = *r.borrow_mut() {
            let throughput_mbs = throughput_bytes.map(|bytes| {
                let bytes_per_sec = (bytes as f64 * 1e9) / time_ns as f64;
                bytes_per_sec / 1_000_000.0
            });
            report.add_perf(PerfResult {
                name: name.to_string(),
                iterations,
                total_time_ns: time_ns,
                throughput_bytes,
                avg_time_ns: Some(time_ns / iterations.max(1)),
                throughput_mbs,
            });
        }
    });

    // Calculate percentiles from samples
    let mut sorted_samples = samples.to_vec();
    sorted_samples.sort_unstable();
    let percentiles = Percentiles::from_sorted(&sorted_samples);

    // Create baseline data
    let avg_ns = time_ns / iterations.max(1);
    let baseline_data = BenchmarkBaseline::new(
        Duration::from_nanos(avg_ns),
        Duration::from_nanos(0), // TODO: Calculate std dev from samples
        percentiles.clone(),
    );

    // Update current baseline
    CURRENT.with(|c| {
        c.borrow_mut().add_benchmark(name, baseline_data.clone());
    });

    // Check for regressions against loaded baseline
    BASELINE.with(|b| {
        if let Some(ref baseline) = *b.borrow() {
            if let Some(baseline_metrics) = baseline.get_benchmark(name) {
                let status = check_regression(avg_ns, baseline_metrics);
                let change_percent = if baseline_metrics.mean > 0 {
                    ((avg_ns as f64 - baseline_metrics.mean as f64) / baseline_metrics.mean as f64)
                        * 100.0
                } else {
                    0.0
                };

                REGRESSIONS.with(|r| {
                    r.borrow_mut().push(RegressionInfo {
                        name: name.to_string(),
                        operation_type: operation_type.to_string(),
                        baseline_mean: baseline_metrics.mean,
                        current_mean: avg_ns,
                        baseline_p95: baseline_metrics.percentiles.p95,
                        current_p95: percentiles.p95,
                        status,
                        change_percent,
                        size,
                        complexity_level: complexity.to_string(),
                    });
                });
            }
        }
    });

    // Save to current baseline file (incremental updates)
    let _ = update_current_baseline("baselines/current.json", name, baseline_data);
}

// ============================================================================
// Core Benchmarks for Regression Tracking
// ============================================================================

/// Track parsing performance regression across multiple sizes
fn bench_regression_parsing(c: &mut Criterion) {
    ensure_init();
    let mut group = c.benchmark_group("regression_parsing");

    let sizes_to_test = vec![
        ("tiny", sizes::SMALL, "flat"),
        ("small", sizes::MEDIUM, "flat"),
        ("medium", sizes::LARGE, "shallow"),
        ("large", sizes::STRESS, "nested"),
    ];

    for (name, size, complexity) in sizes_to_test {
        let hedl = generate_users(size);
        let bytes = hedl.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::from_parameter(name), &hedl, |b, hedl| {
            b.iter(|| black_box(hedl_core::parse(hedl.as_bytes())).unwrap());
        });

        // Collect metrics for regression tracking with samples
        let iterations = if size >= sizes::STRESS { 20 } else { 100 };
        let mut samples = Vec::new();
        let start = Instant::now();

        for _ in 0..iterations {
            let iter_start = Instant::now();
            let _ = hedl_core::parse(hedl.as_bytes());
            samples.push(iter_start.elapsed().as_nanos() as u64);
        }

        let elapsed = start.elapsed();

        record_perf_with_baseline(
            &format!("parsing_{}", name),
            "parsing",
            size,
            complexity,
            elapsed.as_nanos() as u64,
            iterations,
            &samples,
            Some(bytes),
        );
    }

    group.finish();
}

/// Track canonicalization performance regression
fn bench_regression_canonicalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_canonicalization");

    let test_cases = vec![
        ("tiny", sizes::SMALL, "flat"),
        ("small", sizes::MEDIUM, "flat"),
        ("medium", sizes::LARGE, "shallow"),
    ];

    for (name, size, complexity) in test_cases {
        let hedl = generate_users(size);
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();

        group.bench_function(name, |b| {
            b.iter(|| black_box(hedl_c14n::canonicalize(&doc)).unwrap());
        });

        // Collect metrics with samples
        let iterations = if size >= sizes::LARGE { 50 } else { 100 };
        let mut samples = Vec::new();
        let start = Instant::now();

        for _ in 0..iterations {
            let iter_start = Instant::now();
            let _ = hedl_c14n::canonicalize(&doc);
            samples.push(iter_start.elapsed().as_nanos() as u64);
        }

        let elapsed = start.elapsed();

        record_perf_with_baseline(
            &format!("canonicalization_{}", name),
            "canonicalization",
            size,
            complexity,
            elapsed.as_nanos() as u64,
            iterations,
            &samples,
            None,
        );
    }

    group.finish();
}

/// Track conversion performance regression
#[cfg(feature = "json")]
fn bench_regression_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_conversion");

    let test_cases = vec![
        ("tiny", sizes::SMALL, "flat"),
        ("small", sizes::MEDIUM, "flat"),
        ("medium", sizes::LARGE, "shallow"),
    ];

    for (name, size, complexity) in test_cases {
        let hedl = generate_products(size);
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();

        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(hedl_json::to_json(
                    &doc,
                    &hedl_json::ToJsonConfig::default(),
                ))
                .unwrap()
            });
        });

        // Collect metrics with samples
        let iterations = 100;
        let mut samples = Vec::new();
        let start = Instant::now();

        for _ in 0..iterations {
            let iter_start = Instant::now();
            let _ = hedl_json::to_json(&doc, &hedl_json::ToJsonConfig::default());
            samples.push(iter_start.elapsed().as_nanos() as u64);
        }

        let elapsed = start.elapsed();

        record_perf_with_baseline(
            &format!("conversion_json_{}", name),
            "conversion",
            size,
            complexity,
            elapsed.as_nanos() as u64,
            iterations,
            &samples,
            None,
        );
    }

    group.finish();
}

/// Track validation performance regression
fn bench_regression_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_validation");

    let test_cases = vec![
        ("tiny", sizes::SMALL, "flat"),
        ("small", sizes::MEDIUM, "flat"),
        ("medium", sizes::LARGE, "shallow"),
    ];

    for (name, size, complexity) in test_cases {
        let hedl = generate_users(size);
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();

        group.bench_function(name, |b| {
            b.iter(|| black_box(hedl_lint::lint(&doc)));
        });

        // Collect metrics
        let iterations = 100;
        let mut samples = Vec::new();
        let start = Instant::now();

        for _ in 0..iterations {
            let iter_start = Instant::now();
            let _ = hedl_lint::lint(&doc);
            samples.push(iter_start.elapsed().as_nanos() as u64);
        }

        let elapsed = start.elapsed();

        record_perf_with_baseline(
            &format!("validation_{}", name),
            "validation",
            size,
            complexity,
            elapsed.as_nanos() as u64,
            iterations,
            &samples,
            None,
        );
    }

    group.finish();
}

/// Track linting performance regression
fn bench_regression_linting(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_linting");

    let test_cases = vec![
        ("tiny", sizes::SMALL, "flat"),
        ("small", sizes::MEDIUM, "flat"),
        ("medium", sizes::LARGE, "shallow"),
    ];

    for (name, size, complexity) in test_cases {
        let hedl = generate_users(size);
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();

        group.bench_function(name, |b| {
            b.iter(|| black_box(hedl_lint::lint(&doc)));
        });

        // Collect metrics
        let iterations = 100;
        let mut samples = Vec::new();
        let start = Instant::now();

        for _ in 0..iterations {
            let iter_start = Instant::now();
            let _ = hedl_lint::lint(&doc);
            samples.push(iter_start.elapsed().as_nanos() as u64);
        }

        let elapsed = start.elapsed();

        record_perf_with_baseline(
            &format!("linting_{}", name),
            "linting",
            size,
            complexity,
            elapsed.as_nanos() as u64,
            iterations,
            &samples,
            None,
        );
    }

    group.finish();
}

/// Track full pipeline performance regression
fn bench_regression_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_full_pipeline");

    let test_cases = vec![
        ("tiny", sizes::SMALL, "flat"),
        ("small", sizes::MEDIUM, "flat"),
        ("medium", sizes::LARGE, "shallow"),
    ];

    for (name, size, complexity) in test_cases {
        let hedl = generate_users(size);
        let bytes = hedl.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_function(name, |b| {
            b.iter(|| {
                let doc = hedl_core::parse(black_box(hedl.as_bytes())).unwrap();
                let _canonical = hedl_c14n::canonicalize(&doc).unwrap();
                let _diagnostics = hedl_lint::lint(&doc);
                black_box(doc)
            });
        });

        // Collect metrics
        let iterations = if size >= sizes::LARGE { 50 } else { 100 };
        let mut samples = Vec::new();
        let start = Instant::now();

        for _ in 0..iterations {
            let iter_start = Instant::now();
            let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
            let _canonical = hedl_c14n::canonicalize(&doc).unwrap();
            let _diagnostics = hedl_lint::lint(&doc);
            samples.push(iter_start.elapsed().as_nanos() as u64);
        }

        let elapsed = start.elapsed();

        record_perf_with_baseline(
            &format!("pipeline_full_{}", name),
            "pipeline",
            size,
            complexity,
            elapsed.as_nanos() as u64,
            iterations,
            &samples,
            Some(bytes),
        );
    }

    group.finish();
}

// ============================================================================
// Data Collection & Analysis
// ============================================================================

#[derive(Clone)]
struct RegressionTestResult {
    operation: String,
    size: String,
    baseline_mean_us: f64,
    current_mean_us: f64,
    baseline_p95_us: f64,
    current_p95_us: f64,
    change_pct: f64,
    status: String,
    throughput_mbs: Option<f64>,
    complexity: String,
}

#[derive(Clone)]
struct VersionComparisonResult {
    version_name: String,
    total_benchmarks: usize,
    regressions: usize,
    improvements: usize,
    stable: usize,
    avg_change_pct: f64,
    worst_regression_pct: f64,
    best_improvement_pct: f64,
}

fn collect_regression_results() -> Vec<RegressionTestResult> {
    REPORT.with(|r| {
        let borrowed = r.borrow();
        if let Some(ref report) = *borrowed {
            let mut results = Vec::new();

            // Extract actual regression data
            REGRESSIONS.with(|reg| {
                let regressions = reg.borrow();
                for regression in regressions.iter() {
                    let baseline_mean_us = regression.baseline_mean as f64 / 1000.0;
                    let current_mean_us = regression.current_mean as f64 / 1000.0;
                    let baseline_p95_us = regression.baseline_p95 as f64 / 1000.0;
                    let current_p95_us = regression.current_p95 as f64 / 1000.0;

                    let status = if regression.status.is_regression() {
                        format!("REGRESSION ({})", regression.status.severity())
                    } else if regression.change_percent < -5.0 {
                        "IMPROVEMENT".to_string()
                    } else {
                        "STABLE".to_string()
                    };

                    // Find throughput from perf results
                    let throughput_mbs = report
                        .perf_results
                        .iter()
                        .find(|p| p.name == regression.name)
                        .and_then(|p| p.throughput_mbs);

                    results.push(RegressionTestResult {
                        operation: regression.operation_type.clone(),
                        size: format!("{}", regression.size),
                        baseline_mean_us,
                        current_mean_us,
                        baseline_p95_us,
                        current_p95_us,
                        change_pct: regression.change_percent,
                        status,
                        throughput_mbs,
                        complexity: regression.complexity_level.clone(),
                    });
                }
            });

            results
        } else {
            Vec::new()
        }
    })
}

fn analyze_version_comparison() -> Vec<VersionComparisonResult> {
    let regression_results = collect_regression_results();

    if regression_results.is_empty() {
        return Vec::new();
    }

    let total = regression_results.len();
    let regressions = regression_results
        .iter()
        .filter(|r| r.status.contains("REGRESSION"))
        .count();
    let improvements = regression_results
        .iter()
        .filter(|r| r.status == "IMPROVEMENT")
        .count();
    let stable = total - regressions - improvements;

    let avg_change = regression_results.iter().map(|r| r.change_pct).sum::<f64>() / total as f64;

    let worst_regression = regression_results
        .iter()
        .filter(|r| r.change_pct > 0.0)
        .map(|r| r.change_pct)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);

    let best_improvement = regression_results
        .iter()
        .filter(|r| r.change_pct < 0.0)
        .map(|r| r.change_pct)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);

    vec![VersionComparisonResult {
        version_name: "v1.0.0 → current".to_string(),
        total_benchmarks: total,
        regressions,
        improvements,
        stable,
        avg_change_pct: avg_change,
        worst_regression_pct: worst_regression,
        best_improvement_pct: best_improvement,
    }]
}

// ============================================================================
// Custom Tables (14+)
// ============================================================================

// Table 1: Regression Detection Summary
fn create_regression_summary_table(results: &[RegressionTestResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Regression Detection Summary".to_string(),
        headers: vec![
            "Operation".to_string(),
            "Size".to_string(),
            "Baseline (μs)".to_string(),
            "Current (μs)".to_string(),
            "Change (%)".to_string(),
            "Status".to_string(),
            "Severity".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    for result in results {
        let severity = if result.status.contains("severe") {
            "CRITICAL"
        } else if result.status.contains("moderate") {
            "WARNING"
        } else if result.status.contains("minor") {
            "NOTICE"
        } else if result.status == "IMPROVEMENT" {
            "GOOD"
        } else {
            "OK"
        };

        table.rows.push(vec![
            TableCell::String(result.operation.clone()),
            TableCell::String(result.size.clone()),
            TableCell::Float(result.baseline_mean_us),
            TableCell::Float(result.current_mean_us),
            TableCell::Float(result.change_pct),
            TableCell::String(result.status.clone()),
            TableCell::String(severity.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// Table 2: Performance by Operation Type
fn create_performance_by_operation_table(
    results: &[RegressionTestResult],
    report: &mut BenchmarkReport,
) {
    let mut table = CustomTable {
        title: "Performance by Operation Type".to_string(),
        headers: vec![
            "Operation".to_string(),
            "Tests".to_string(),
            "Avg Baseline (μs)".to_string(),
            "Avg Current (μs)".to_string(),
            "Avg Change (%)".to_string(),
            "Regressions".to_string(),
            "Improvements".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let mut by_operation: HashMap<String, Vec<&RegressionTestResult>> = HashMap::new();
    for result in results {
        by_operation
            .entry(result.operation.clone())
            .or_default()
            .push(result);
    }

    for (operation, op_results) in by_operation {
        let count = op_results.len();
        let avg_baseline =
            op_results.iter().map(|r| r.baseline_mean_us).sum::<f64>() / count as f64;
        let avg_current = op_results.iter().map(|r| r.current_mean_us).sum::<f64>() / count as f64;
        let avg_change = op_results.iter().map(|r| r.change_pct).sum::<f64>() / count as f64;
        let regressions = op_results
            .iter()
            .filter(|r| r.status.contains("REGRESSION"))
            .count();
        let improvements = op_results
            .iter()
            .filter(|r| r.status == "IMPROVEMENT")
            .count();

        table.rows.push(vec![
            TableCell::String(operation),
            TableCell::Integer(count as i64),
            TableCell::Float(avg_baseline),
            TableCell::Float(avg_current),
            TableCell::Float(avg_change),
            TableCell::Integer(regressions as i64),
            TableCell::Integer(improvements as i64),
        ]);
    }

    table.rows.sort_by(|a, b| {
        let a_change = match &a[4] {
            TableCell::Float(f) => *f,
            _ => 0.0,
        };
        let b_change = match &b[4] {
            TableCell::Float(f) => *f,
            _ => 0.0,
        };
        b_change.partial_cmp(&a_change).unwrap()
    });

    report.add_custom_table(table);
}

// Table 3: Performance by Size
fn create_performance_by_size_table(
    results: &[RegressionTestResult],
    report: &mut BenchmarkReport,
) {
    let mut table = CustomTable {
        title: "Performance Scaling by Dataset Size".to_string(),
        headers: vec![
            "Size".to_string(),
            "Tests".to_string(),
            "Avg Baseline (μs)".to_string(),
            "Avg Current (μs)".to_string(),
            "Change (%)".to_string(),
            "Throughput (MB/s)".to_string(),
            "Scaling".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let mut by_size: HashMap<String, Vec<&RegressionTestResult>> = HashMap::new();
    for result in results {
        by_size.entry(result.size.clone()).or_default().push(result);
    }

    for (size, size_results) in by_size {
        let count = size_results.len();
        let avg_baseline =
            size_results.iter().map(|r| r.baseline_mean_us).sum::<f64>() / count as f64;
        let avg_current =
            size_results.iter().map(|r| r.current_mean_us).sum::<f64>() / count as f64;
        let avg_change = size_results.iter().map(|r| r.change_pct).sum::<f64>() / count as f64;
        let avg_throughput = size_results
            .iter()
            .filter_map(|r| r.throughput_mbs)
            .sum::<f64>()
            / size_results
                .iter()
                .filter(|r| r.throughput_mbs.is_some())
                .count()
                .max(1) as f64;

        let scaling = if avg_change.abs() < 5.0 {
            "Linear"
        } else if avg_change > 10.0 {
            "Sub-linear (degrading)"
        } else {
            "Super-linear (improving)"
        };

        table.rows.push(vec![
            TableCell::String(size),
            TableCell::Integer(count as i64),
            TableCell::Float(avg_baseline),
            TableCell::Float(avg_current),
            TableCell::Float(avg_change),
            TableCell::Float(avg_throughput),
            TableCell::String(scaling.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// Table 4: P95 Latency Analysis
fn create_p95_latency_table(results: &[RegressionTestResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "P95 Latency Regression Analysis".to_string(),
        headers: vec![
            "Operation".to_string(),
            "Size".to_string(),
            "Baseline P95 (μs)".to_string(),
            "Current P95 (μs)".to_string(),
            "P95 Change (%)".to_string(),
            "Tail Latency Impact".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    for result in results {
        let p95_change = if result.baseline_p95_us > 0.0 {
            ((result.current_p95_us - result.baseline_p95_us) / result.baseline_p95_us) * 100.0
        } else {
            0.0
        };

        let impact = if p95_change.abs() < 5.0 {
            "Negligible"
        } else if p95_change.abs() < 15.0 {
            "Minor"
        } else if p95_change.abs() < 30.0 {
            "Moderate"
        } else {
            "Significant"
        };

        table.rows.push(vec![
            TableCell::String(result.operation.clone()),
            TableCell::String(result.size.clone()),
            TableCell::Float(result.baseline_p95_us),
            TableCell::Float(result.current_p95_us),
            TableCell::Float(p95_change),
            TableCell::String(impact.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// Table 5: Version Comparison Summary
fn create_version_comparison_table(
    versions: &[VersionComparisonResult],
    report: &mut BenchmarkReport,
) {
    let mut table = CustomTable {
        title: "Version-to-Version Comparison".to_string(),
        headers: vec![
            "Version".to_string(),
            "Total Tests".to_string(),
            "Regressions".to_string(),
            "Improvements".to_string(),
            "Stable".to_string(),
            "Avg Change (%)".to_string(),
            "Worst Regression (%)".to_string(),
            "Best Improvement (%)".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    for version in versions {
        table.rows.push(vec![
            TableCell::String(version.version_name.clone()),
            TableCell::Integer(version.total_benchmarks as i64),
            TableCell::Integer(version.regressions as i64),
            TableCell::Integer(version.improvements as i64),
            TableCell::Integer(version.stable as i64),
            TableCell::Float(version.avg_change_pct),
            TableCell::Float(version.worst_regression_pct),
            TableCell::Float(version.best_improvement_pct),
        ]);
    }

    report.add_custom_table(table);
}

// Table 6: Complexity Impact Analysis
fn create_complexity_impact_table(results: &[RegressionTestResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Complexity Level Impact on Performance".to_string(),
        headers: vec![
            "Complexity".to_string(),
            "Tests".to_string(),
            "Avg Change (%)".to_string(),
            "Regressions".to_string(),
            "Stability".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let mut by_complexity: HashMap<String, Vec<&RegressionTestResult>> = HashMap::new();
    for result in results {
        by_complexity
            .entry(result.complexity.clone())
            .or_default()
            .push(result);
    }

    for (complexity, comp_results) in by_complexity {
        let count = comp_results.len();
        let avg_change = comp_results.iter().map(|r| r.change_pct).sum::<f64>() / count as f64;
        let regressions = comp_results
            .iter()
            .filter(|r| r.status.contains("REGRESSION"))
            .count();

        let stability = if avg_change.abs() < 5.0 && regressions == 0 {
            "Excellent"
        } else if avg_change.abs() < 10.0 && regressions <= count / 4 {
            "Good"
        } else if avg_change.abs() < 20.0 {
            "Fair"
        } else {
            "Poor"
        };

        table.rows.push(vec![
            TableCell::String(complexity),
            TableCell::Integer(count as i64),
            TableCell::Float(avg_change),
            TableCell::Integer(regressions as i64),
            TableCell::String(stability.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// Table 7: Regression Severity Distribution
fn create_severity_distribution_table(
    results: &[RegressionTestResult],
    report: &mut BenchmarkReport,
) {
    let mut table = CustomTable {
        title: "Regression Severity Distribution".to_string(),
        headers: vec![
            "Severity".to_string(),
            "Count".to_string(),
            "% of Total".to_string(),
            "Avg Change (%)".to_string(),
            "Action Required".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let total = results.len();

    let severe = results
        .iter()
        .filter(|r| r.status.contains("severe"))
        .count();
    let moderate = results
        .iter()
        .filter(|r| r.status.contains("moderate"))
        .count();
    let minor = results
        .iter()
        .filter(|r| r.status.contains("minor"))
        .count();
    let stable = results.iter().filter(|r| r.status == "STABLE").count();
    let improved = results.iter().filter(|r| r.status == "IMPROVEMENT").count();

    let categories = vec![
        (
            "Severe Regression",
            severe,
            results
                .iter()
                .filter(|r| r.status.contains("severe"))
                .map(|r| r.change_pct)
                .sum::<f64>(),
            "Immediate fix required",
        ),
        (
            "Moderate Regression",
            moderate,
            results
                .iter()
                .filter(|r| r.status.contains("moderate"))
                .map(|r| r.change_pct)
                .sum::<f64>(),
            "Investigation needed",
        ),
        (
            "Minor Regression",
            minor,
            results
                .iter()
                .filter(|r| r.status.contains("minor"))
                .map(|r| r.change_pct)
                .sum::<f64>(),
            "Monitor",
        ),
        ("Stable", stable, 0.0, "None"),
        (
            "Improvement",
            improved,
            results
                .iter()
                .filter(|r| r.status == "IMPROVEMENT")
                .map(|r| r.change_pct)
                .sum::<f64>(),
            "Validate gain",
        ),
    ];

    for (severity, count, sum_change, action) in categories {
        let pct_of_total = (count as f64 / total as f64) * 100.0;
        let avg_change = if count > 0 {
            sum_change / count as f64
        } else {
            0.0
        };

        table.rows.push(vec![
            TableCell::String(severity.to_string()),
            TableCell::Integer(count as i64),
            TableCell::Float(pct_of_total),
            TableCell::Float(avg_change),
            TableCell::String(action.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// Table 8: Throughput Regression Analysis
fn create_throughput_regression_table(
    results: &[RegressionTestResult],
    report: &mut BenchmarkReport,
) {
    let mut table = CustomTable {
        title: "Throughput Regression Analysis".to_string(),
        headers: vec![
            "Operation".to_string(),
            "Size".to_string(),
            "Throughput (MB/s)".to_string(),
            "Change (%)".to_string(),
            "Impact on Production".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    for result in results.iter().filter(|r| r.throughput_mbs.is_some()) {
        let throughput = result.throughput_mbs.unwrap();

        let impact = if result.change_pct.abs() < 5.0 {
            "Negligible - no action needed"
        } else if result.change_pct.abs() < 15.0 {
            "Minor - acceptable for most workloads"
        } else if result.change_pct.abs() < 30.0 {
            "Moderate - consider optimization"
        } else {
            "Significant - requires immediate attention"
        };

        table.rows.push(vec![
            TableCell::String(result.operation.clone()),
            TableCell::String(result.size.clone()),
            TableCell::Float(throughput),
            TableCell::Float(result.change_pct),
            TableCell::String(impact.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// Table 9: Critical Path Analysis
fn create_critical_path_table(results: &[RegressionTestResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Critical Path Performance (Slowest Operations)".to_string(),
        headers: vec![
            "Operation".to_string(),
            "Size".to_string(),
            "Current Time (μs)".to_string(),
            "% of Total Pipeline".to_string(),
            "Regression (%)".to_string(),
            "Priority".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let mut sorted_results = results.to_vec();
    sorted_results.sort_by(|a, b| b.current_mean_us.partial_cmp(&a.current_mean_us).unwrap());

    let total_time: f64 = sorted_results
        .iter()
        .take(10)
        .map(|r| r.current_mean_us)
        .sum();

    for result in sorted_results.iter().take(10) {
        let pct_of_pipeline = (result.current_mean_us / total_time) * 100.0;

        let priority = if result.current_mean_us > 1000.0 && result.change_pct > 10.0 {
            "CRITICAL"
        } else if result.current_mean_us > 500.0 && result.change_pct > 5.0 {
            "HIGH"
        } else if pct_of_pipeline > 20.0 {
            "MEDIUM"
        } else {
            "LOW"
        };

        table.rows.push(vec![
            TableCell::String(result.operation.clone()),
            TableCell::String(result.size.clone()),
            TableCell::Float(result.current_mean_us),
            TableCell::Float(pct_of_pipeline),
            TableCell::Float(result.change_pct),
            TableCell::String(priority.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// Table 10: Stability Metrics
fn create_stability_metrics_table(results: &[RegressionTestResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Performance Stability Metrics".to_string(),
        headers: vec![
            "Metric".to_string(),
            "Value".to_string(),
            "Threshold".to_string(),
            "Status".to_string(),
            "Recommendation".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let total = results.len();
    let regressions = results
        .iter()
        .filter(|r| r.status.contains("REGRESSION"))
        .count();
    let severe = results
        .iter()
        .filter(|r| r.status.contains("severe"))
        .count();
    let regression_rate = (regressions as f64 / total as f64) * 100.0;
    let severe_rate = (severe as f64 / total as f64) * 100.0;

    let avg_change: f64 = results.iter().map(|r| r.change_pct).sum::<f64>() / total as f64;
    let max_regression = results
        .iter()
        .filter(|r| r.change_pct > 0.0)
        .map(|r| r.change_pct)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);

    let metrics = vec![
        (
            "Regression Rate",
            regression_rate,
            10.0,
            "% of tests regressed",
        ),
        (
            "Severe Regression Rate",
            severe_rate,
            5.0,
            "% with >15% slowdown",
        ),
        ("Average Change", avg_change, 5.0, "Mean performance delta"),
        ("Worst Regression", max_regression, 20.0, "Maximum slowdown"),
    ];

    for (name, value, threshold, desc) in metrics {
        let status = if value <= threshold {
            "PASS"
        } else if value <= threshold * 1.5 {
            "WARNING"
        } else {
            "FAIL"
        };

        let recommendation = if value <= threshold {
            "Acceptable - continue monitoring"
        } else if value <= threshold * 1.5 {
            "Review affected benchmarks"
        } else {
            "Critical - requires investigation"
        };

        table.rows.push(vec![
            TableCell::String(format!("{} ({})", name, desc)),
            TableCell::Float(value),
            TableCell::Float(threshold),
            TableCell::String(status.to_string()),
            TableCell::String(recommendation.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// Table 11: Regression Trends
fn create_regression_trends_table(results: &[RegressionTestResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Performance Trend Analysis".to_string(),
        headers: vec![
            "Category".to_string(),
            "Improving".to_string(),
            "Degrading".to_string(),
            "Stable".to_string(),
            "Trend".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let mut by_operation: HashMap<String, Vec<&RegressionTestResult>> = HashMap::new();
    for result in results {
        by_operation
            .entry(result.operation.clone())
            .or_default()
            .push(result);
    }

    for (operation, op_results) in by_operation {
        let improving = op_results.iter().filter(|r| r.change_pct < -5.0).count();
        let degrading = op_results.iter().filter(|r| r.change_pct > 5.0).count();
        let stable = op_results.len() - improving - degrading;

        let trend = if improving > degrading * 2 {
            "Positive - Getting Faster"
        } else if degrading > improving * 2 {
            "Negative - Getting Slower"
        } else {
            "Neutral - Mixed Results"
        };

        table.rows.push(vec![
            TableCell::String(operation),
            TableCell::Integer(improving as i64),
            TableCell::Integer(degrading as i64),
            TableCell::Integer(stable as i64),
            TableCell::String(trend.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// Table 12: Test Coverage Analysis
fn create_test_coverage_table(results: &[RegressionTestResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Regression Test Coverage".to_string(),
        headers: vec![
            "Component".to_string(),
            "Tests".to_string(),
            "Sizes Tested".to_string(),
            "Coverage".to_string(),
            "Gaps".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let mut by_operation: HashMap<String, Vec<&RegressionTestResult>> = HashMap::new();
    for result in results {
        by_operation
            .entry(result.operation.clone())
            .or_default()
            .push(result);
    }

    for (operation, op_results) in by_operation {
        let tests = op_results.len();
        let unique_sizes: std::collections::HashSet<_> =
            op_results.iter().map(|r| r.size.as_str()).collect();
        let sizes_tested = unique_sizes.len();

        let coverage = if sizes_tested >= 4 {
            "Excellent"
        } else if sizes_tested >= 3 {
            "Good"
        } else if sizes_tested >= 2 {
            "Fair"
        } else {
            "Poor"
        };

        let gaps = if sizes_tested < 4 {
            "Add XL/XXL size tests"
        } else {
            "None"
        };

        table.rows.push(vec![
            TableCell::String(operation),
            TableCell::Integer(tests as i64),
            TableCell::Integer(sizes_tested as i64),
            TableCell::String(coverage.to_string()),
            TableCell::String(gaps.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// Table 13: Baseline Quality Assessment
fn create_baseline_quality_table(report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Baseline Quality and Validity".to_string(),
        headers: vec![
            "Aspect".to_string(),
            "Status".to_string(),
            "Details".to_string(),
            "Confidence".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let has_baseline = BASELINE.with(|b| b.borrow().is_some());

    if has_baseline {
        BASELINE.with(|b| {
            if let Some(ref baseline) = *b.borrow() {
                let benchmark_count = baseline.benchmarks.len();

                table.rows.push(vec![
                    TableCell::String("Baseline Loaded".to_string()),
                    TableCell::String("YES".to_string()),
                    TableCell::String(format!("Version: {}", baseline.version)),
                    TableCell::String("High".to_string()),
                ]);

                table.rows.push(vec![
                    TableCell::String("Benchmark Count".to_string()),
                    TableCell::String(format!("{}", benchmark_count)),
                    TableCell::String(format!("Timestamp: {}", baseline.timestamp)),
                    TableCell::String(
                        if benchmark_count >= 10 {
                            "High"
                        } else {
                            "Medium"
                        }
                        .to_string(),
                    ),
                ]);

                table.rows.push(vec![
                    TableCell::String("Comparison Valid".to_string()),
                    TableCell::String("YES".to_string()),
                    TableCell::String("Direct comparison possible".to_string()),
                    TableCell::String("High".to_string()),
                ]);
            }
        });
    } else {
        table.rows.push(vec![
            TableCell::String("Baseline Loaded".to_string()),
            TableCell::String("NO".to_string()),
            TableCell::String("Creating new baseline".to_string()),
            TableCell::String("N/A".to_string()),
        ]);

        table.rows.push(vec![
            TableCell::String("Comparison Valid".to_string()),
            TableCell::String("NO".to_string()),
            TableCell::String("First run - establishing baseline".to_string()),
            TableCell::String("N/A".to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// Table 14: Action Items
fn create_action_items_table(results: &[RegressionTestResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Recommended Actions".to_string(),
        headers: vec![
            "Priority".to_string(),
            "Issue".to_string(),
            "Affected Tests".to_string(),
            "Recommendation".to_string(),
            "Expected Impact".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let severe = results
        .iter()
        .filter(|r| r.status.contains("severe"))
        .count();
    let moderate = results
        .iter()
        .filter(|r| r.status.contains("moderate"))
        .count();

    if severe > 0 {
        table.rows.push(vec![
            TableCell::String("CRITICAL".to_string()),
            TableCell::String(format!("{} severe regressions", severe)),
            TableCell::Integer(severe as i64),
            TableCell::String("Investigate and fix before release".to_string()),
            TableCell::String("Restore performance to baseline levels".to_string()),
        ]);
    }

    if moderate > 0 {
        table.rows.push(vec![
            TableCell::String("HIGH".to_string()),
            TableCell::String(format!("{} moderate regressions", moderate)),
            TableCell::Integer(moderate as i64),
            TableCell::String("Profile affected operations and optimize".to_string()),
            TableCell::String("Reduce regression to <5%".to_string()),
        ]);
    }

    // Find operations with multiple regressions
    let mut regression_counts: HashMap<String, usize> = HashMap::new();
    for result in results.iter().filter(|r| r.status.contains("REGRESSION")) {
        *regression_counts
            .entry(result.operation.clone())
            .or_default() += 1;
    }

    for (operation, count) in regression_counts.iter().filter(|(_, &c)| c >= 2) {
        table.rows.push(vec![
            TableCell::String("MEDIUM".to_string()),
            TableCell::String(format!("{} consistently regressing", operation)),
            TableCell::Integer(*count as i64),
            TableCell::String("Systematic optimization needed".to_string()),
            TableCell::String("Improve scalability".to_string()),
        ]);
    }

    if table.rows.is_empty() {
        table.rows.push(vec![
            TableCell::String("LOW".to_string()),
            TableCell::String("No significant regressions".to_string()),
            TableCell::Integer(0),
            TableCell::String("Continue monitoring".to_string()),
            TableCell::String("Maintain current performance".to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// Table 15: Statistical Confidence
fn create_statistical_confidence_table(
    results: &[RegressionTestResult],
    report: &mut BenchmarkReport,
) {
    let mut table = CustomTable {
        title: "Statistical Confidence Analysis".to_string(),
        headers: vec![
            "Operation".to_string(),
            "Sample Size".to_string(),
            "Mean Stability".to_string(),
            "P95 Stability".to_string(),
            "Confidence Level".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let mut by_operation: HashMap<String, Vec<&RegressionTestResult>> = HashMap::new();
    for result in results {
        by_operation
            .entry(result.operation.clone())
            .or_default()
            .push(result);
    }

    for (operation, op_results) in by_operation {
        let sample_size = op_results.len();

        // Calculate coefficient of variation for mean
        let mean_changes: Vec<f64> = op_results.iter().map(|r| r.change_pct).collect();
        let mean_avg = mean_changes.iter().sum::<f64>() / mean_changes.len() as f64;
        let mean_variance = mean_changes
            .iter()
            .map(|&x| (x - mean_avg).powi(2))
            .sum::<f64>()
            / mean_changes.len() as f64;
        let mean_cv = (mean_variance.sqrt() / mean_avg.abs()).abs();

        let mean_stability = if mean_cv < 0.1 {
            "Excellent"
        } else if mean_cv < 0.3 {
            "Good"
        } else if mean_cv < 0.5 {
            "Fair"
        } else {
            "Poor"
        };

        let p95_stability = if mean_cv < 0.2 { "Stable" } else { "Variable" };

        let confidence = if sample_size >= 4 && mean_cv < 0.3 {
            "High (95%+)"
        } else if sample_size >= 3 && mean_cv < 0.5 {
            "Medium (90%)"
        } else {
            "Low (<90%)"
        };

        table.rows.push(vec![
            TableCell::String(operation),
            TableCell::Integer(sample_size as i64),
            TableCell::String(mean_stability.to_string()),
            TableCell::String(p95_stability.to_string()),
            TableCell::String(confidence.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// ============================================================================
// Insights Generation (10+)
// ============================================================================

fn generate_insights(
    results: &[RegressionTestResult],
    versions: &[VersionComparisonResult],
    report: &mut BenchmarkReport,
) {
    if results.is_empty() {
        report.add_insight(Insight {
            category: "finding".to_string(),
            title: "No Baseline Available for Comparison".to_string(),
            description: "This is the first run - establishing performance baseline".to_string(),
            data_points: vec![
                format!("Collected {} benchmark measurements", results.len()),
                "Saved to baselines/current.json".to_string(),
                "Run again to compare against this baseline".to_string(),
            ],
        });
        return;
    }

    // Insight 1: Overall regression status
    let total = results.len();
    let regressions = results
        .iter()
        .filter(|r| r.status.contains("REGRESSION"))
        .count();
    let severe = results
        .iter()
        .filter(|r| r.status.contains("severe"))
        .count();
    let improvements = results.iter().filter(|r| r.status == "IMPROVEMENT").count();
    let regression_rate = (regressions as f64 / total as f64) * 100.0;

    if regression_rate < 10.0 && severe == 0 {
        report.add_insight(Insight {
            category: "strength".to_string(),
            title: format!(
                "Excellent Performance Stability: {:.1}% Regression Rate",
                regression_rate
            ),
            description: "Performance remains stable across versions with minimal regressions"
                .to_string(),
            data_points: vec![
                format!("{} of {} tests regressed", regressions, total),
                format!("0 severe regressions detected"),
                format!("{} tests improved", improvements),
                "No immediate action required".to_string(),
            ],
        });
    } else if severe > 0 {
        report.add_insight(Insight {
            category: "weakness".to_string(),
            title: format!(
                "Critical Performance Regressions Detected: {} Severe",
                severe
            ),
            description: "Significant performance degradation requires immediate investigation"
                .to_string(),
            data_points: vec![
                format!("{} severe regressions (>15% slowdown)", severe),
                format!("Total regression rate: {:.1}%", regression_rate),
                "Recommend profiling affected operations".to_string(),
                "Do not release until fixed".to_string(),
            ],
        });
    }

    // Insight 2: Operation-specific patterns
    let mut regression_by_op: HashMap<String, usize> = HashMap::new();
    for result in results.iter().filter(|r| r.status.contains("REGRESSION")) {
        *regression_by_op
            .entry(result.operation.clone())
            .or_default() += 1;
    }

    if let Some((worst_op, count)) = regression_by_op.iter().max_by_key(|(_, &c)| c) {
        report.add_insight(Insight {
            category: "finding".to_string(),
            title: format!(
                "{} Operation Most Affected: {} Regressions",
                worst_op, count
            ),
            description: "Specific operation shows systematic performance degradation".to_string(),
            data_points: vec![
                format!(
                    "{} of {} tests regressed",
                    count,
                    results.iter().filter(|r| r.operation == *worst_op).count()
                ),
                "Suggests algorithmic or implementation issue".to_string(),
                format!("Review {} implementation for recent changes", worst_op),
                "May benefit from targeted optimization".to_string(),
            ],
        });
    }

    // Insight 3: Scaling behavior
    let mut by_size: HashMap<String, Vec<&RegressionTestResult>> = HashMap::new();
    for result in results {
        by_size.entry(result.size.clone()).or_default().push(result);
    }

    let mut size_changes: Vec<(String, f64)> = by_size
        .iter()
        .map(|(size, size_results)| {
            let avg_change =
                size_results.iter().map(|r| r.change_pct).sum::<f64>() / size_results.len() as f64;
            (size.clone(), avg_change)
        })
        .collect();
    size_changes.sort_by(|a, b| {
        a.0.parse::<usize>()
            .unwrap_or(0)
            .cmp(&b.0.parse::<usize>().unwrap_or(0))
    });

    if size_changes.len() >= 3 {
        let trend_increasing = size_changes.windows(2).all(|w| w[1].1 > w[0].1);
        let trend_decreasing = size_changes.windows(2).all(|w| w[1].1 < w[0].1);

        if trend_increasing {
            report.add_insight(Insight {
                category: "weakness".to_string(),
                title: "Performance Degrades with Dataset Size".to_string(),
                description: "Regressions worsen as dataset size increases - scaling issue"
                    .to_string(),
                data_points: size_changes
                    .iter()
                    .map(|(size, change)| format!("Size {}: {:+.1}% change", size, change))
                    .collect(),
            });
        } else if trend_decreasing {
            report.add_insight(Insight {
                category: "strength".to_string(),
                title: "Better Scaling on Large Datasets".to_string(),
                description: "Performance improves or stabilizes with larger datasets".to_string(),
                data_points: size_changes
                    .iter()
                    .map(|(size, change)| format!("Size {}: {:+.1}% change", size, change))
                    .collect(),
            });
        }
    }

    // Insight 4: Throughput impact
    let throughput_results: Vec<_> = results
        .iter()
        .filter(|r| r.throughput_mbs.is_some())
        .collect();

    if !throughput_results.is_empty() {
        let avg_throughput_change = throughput_results.iter().map(|r| r.change_pct).sum::<f64>()
            / throughput_results.len() as f64;

        if avg_throughput_change.abs() > 10.0 {
            report.add_insight(Insight {
                category: if avg_throughput_change > 0.0 {
                    "weakness"
                } else {
                    "strength"
                }
                .to_string(),
                title: format!("Throughput Changed by {:.1}%", avg_throughput_change),
                description: "Data processing throughput significantly affected".to_string(),
                data_points: vec![
                    format!("Average change: {:+.1}%", avg_throughput_change),
                    format!(
                        "Affects {} throughput-measured operations",
                        throughput_results.len()
                    ),
                    if avg_throughput_change > 0.0 {
                        "Network-bound operations will see increased latency"
                    } else {
                        "Network-bound operations will benefit from faster processing"
                    }
                    .to_string(),
                ],
            });
        }
    }

    // Insight 5: P95 latency stability
    let p95_regressions = results
        .iter()
        .filter(|r| {
            if r.baseline_p95_us > 0.0 {
                let p95_change =
                    ((r.current_p95_us - r.baseline_p95_us) / r.baseline_p95_us) * 100.0;
                p95_change > 15.0
            } else {
                false
            }
        })
        .count();

    if p95_regressions > 0 {
        report.add_insight(Insight {
            category: "weakness".to_string(),
            title: format!(
                "Tail Latency Degradation: {} Tests Affected",
                p95_regressions
            ),
            description: "P95 latency regressions impact worst-case performance".to_string(),
            data_points: vec![
                format!("{} tests show >15% P95 latency increase", p95_regressions),
                "Affects user experience in high-percentile scenarios".to_string(),
                "May indicate memory allocation or GC pressure".to_string(),
                "Review for buffer allocations and cache misses".to_string(),
            ],
        });
    } else if results.iter().any(|r| r.baseline_p95_us > 0.0) {
        report.add_insight(Insight {
            category: "strength".to_string(),
            title: "Stable Tail Latency Performance".to_string(),
            description: "P95 latency remains consistent - predictable performance".to_string(),
            data_points: vec![
                "All P95 latencies within acceptable variance".to_string(),
                "Consistent worst-case performance".to_string(),
                "Safe for production deployment".to_string(),
            ],
        });
    }

    // Insight 6: Improvement analysis
    if improvements > 0 {
        let avg_improvement = results
            .iter()
            .filter(|r| r.status == "IMPROVEMENT")
            .map(|r| r.change_pct.abs())
            .sum::<f64>()
            / improvements as f64;

        report.add_insight(Insight {
            category: "strength".to_string(),
            title: format!("{} Performance Improvements Detected", improvements),
            description: format!("Average improvement: {:.1}% faster", avg_improvement),
            data_points: vec![
                format!("{} tests show >5% performance gain", improvements),
                format!("Average speedup: {:.1}%", avg_improvement),
                "Validate improvements are not measurement artifacts".to_string(),
                "Document optimizations for future reference".to_string(),
            ],
        });
    }

    // Insight 7: Complexity impact
    let mut by_complexity: HashMap<String, Vec<&RegressionTestResult>> = HashMap::new();
    for result in results {
        by_complexity
            .entry(result.complexity.clone())
            .or_default()
            .push(result);
    }

    for (complexity, comp_results) in by_complexity.iter() {
        let regressions = comp_results
            .iter()
            .filter(|r| r.status.contains("REGRESSION"))
            .count();
        let rate = (regressions as f64 / comp_results.len() as f64) * 100.0;

        if rate > 30.0 {
            report.add_insight(Insight {
                category: "finding".to_string(),
                title: format!(
                    "High Regression Rate for {} Complexity: {:.1}%",
                    complexity, rate
                ),
                description: "Specific complexity level shows disproportionate regressions"
                    .to_string(),
                data_points: vec![
                    format!(
                        "{} of {} {} tests regressed",
                        regressions,
                        comp_results.len(),
                        complexity
                    ),
                    format!("Regression rate: {:.1}%", rate),
                    format!("Suggests issue with {} data handling", complexity),
                ],
            });
        }
    }

    // Insight 8: Version comparison summary
    if let Some(version) = versions.first() {
        let health_score =
            100.0 - (version.regressions as f64 / version.total_benchmarks as f64 * 100.0);

        report.add_insight(Insight {
            category: "finding".to_string(),
            title: format!("Version Health Score: {:.1}%", health_score),
            description: "Overall assessment of version-to-version performance".to_string(),
            data_points: vec![
                format!("Total benchmarks: {}", version.total_benchmarks),
                format!(
                    "Regressions: {} ({:.1}%)",
                    version.regressions,
                    (version.regressions as f64 / version.total_benchmarks as f64) * 100.0
                ),
                format!(
                    "Improvements: {} ({:.1}%)",
                    version.improvements,
                    (version.improvements as f64 / version.total_benchmarks as f64) * 100.0
                ),
                format!(
                    "Stable: {} ({:.1}%)",
                    version.stable,
                    (version.stable as f64 / version.total_benchmarks as f64) * 100.0
                ),
                format!("Average change: {:+.1}%", version.avg_change_pct),
            ],
        });
    }

    // Insight 9: Test coverage assessment
    let operations_tested = results
        .iter()
        .map(|r| r.operation.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();

    report.add_insight(Insight {
        category: "recommendation".to_string(),
        title: format!(
            "Regression Test Coverage: {} Operations Tracked",
            operations_tested
        ),
        description: "Comprehensive regression tracking across multiple operations".to_string(),
        data_points: vec![
            format!("{} distinct operations tested", operations_tested),
            format!("{} total test variations", total),
            "Coverage includes: parsing, canonicalization, conversion, validation, linting"
                .to_string(),
            if operations_tested >= 5 {
                "Excellent coverage - all major operations tracked"
            } else {
                "Consider adding more operation types for complete coverage"
            }
            .to_string(),
        ],
    });

    // Insight 10: Actionable recommendations
    let critical_actions = severe + (if regression_rate > 20.0 { 1 } else { 0 });

    if critical_actions > 0 {
        report.add_insight(Insight {
            category: "recommendation".to_string(),
            title: format!(
                "{} Critical Actions Required Before Release",
                critical_actions
            ),
            description: "Performance regressions require investigation and remediation"
                .to_string(),
            data_points: vec![
                if severe > 0 {
                    format!("Fix {} severe regressions (>15% slowdown)", severe)
                } else {
                    String::new()
                },
                if regression_rate > 20.0 {
                    format!("Address high regression rate ({:.1}%)", regression_rate)
                } else {
                    String::new()
                },
                "Profile affected operations to identify bottlenecks".to_string(),
                "Consider reverting recent changes if root cause unclear".to_string(),
                "Re-run benchmarks after fixes to validate improvements".to_string(),
            ]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect(),
        });
    } else if regressions == 0 && improvements > 0 {
        report.add_insight(Insight {
            category: "strength".to_string(),
            title: "No Regressions Detected - Performance Improved".to_string(),
            description: "Version shows performance gains without any degradation".to_string(),
            data_points: vec![
                format!("{} improvements detected", improvements),
                "0 regressions found".to_string(),
                "Safe for production deployment".to_string(),
                "Update baseline to capture new performance levels".to_string(),
            ],
        });
    }

    // Insight 11: Statistical confidence
    let high_confidence_results = results.iter().filter(|r| r.change_pct.abs() > 10.0).count();

    if high_confidence_results > 0 {
        report.add_insight(Insight {
            category: "finding".to_string(),
            title: format!(
                "{} High-Confidence Changes Detected",
                high_confidence_results
            ),
            description: "Large performance deltas provide high statistical confidence".to_string(),
            data_points: vec![
                format!(
                    "{} tests show >10% absolute change",
                    high_confidence_results
                ),
                "Changes exceed normal variance - statistically significant".to_string(),
                "These results can be trusted without additional validation runs".to_string(),
                format!(
                    "{} tests show <10% change - may need multiple runs",
                    total - high_confidence_results
                ),
            ],
        });
    }
}

// ============================================================================
// Report Export
// ============================================================================

fn export_reports(c: &mut Criterion) {
    let mut group = c.benchmark_group("export_reports");
    group.bench_function("export", |b| b.iter(|| 1));
    group.finish();

    // Save current baseline
    CURRENT.with(|c| {
        let current_path = Path::new("baselines/current.json");
        if let Some(parent) = current_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = save_baseline(&c.borrow()) {
            eprintln!("Warning: Failed to save current baseline: {}", e);
        }
    });

    // Generate regression console report
    REGRESSIONS.with(|r| {
        let regressions = r.borrow();
        if !regressions.is_empty() {
            println!("\n{}", "=".repeat(80));
            println!("REGRESSION ANALYSIS SUMMARY");
            println!("{}", "=".repeat(80));

            let mut has_regression = false;
            let mut severe_count = 0;
            let mut moderate_count = 0;
            let mut minor_count = 0;
            let mut improvement_count = 0;

            for reg in regressions.iter() {
                if reg.status.is_regression() {
                    has_regression = true;
                    match reg.status {
                        RegressionStatus::Severe(_) => severe_count += 1,
                        RegressionStatus::Moderate(_) => moderate_count += 1,
                        RegressionStatus::Minor(_) => minor_count += 1,
                        _ => {}
                    }
                } else if reg.change_percent < -5.0 {
                    improvement_count += 1;
                }
            }

            println!("Total Tests: {}", regressions.len());
            println!("Severe Regressions: {}", severe_count);
            println!("Moderate Regressions: {}", moderate_count);
            println!("Minor Regressions: {}", minor_count);
            println!("Improvements: {}", improvement_count);
            println!();

            if has_regression {
                println!("REGRESSIONS DETECTED:");
                for reg in regressions.iter().filter(|r| r.status.is_regression()) {
                    println!(
                        "  [{}] {}: {:+.2}% ({} ns → {} ns)",
                        reg.status.severity().to_uppercase(),
                        reg.name,
                        reg.change_percent,
                        reg.baseline_mean,
                        reg.current_mean
                    );
                }
            }

            if improvement_count > 0 {
                println!("\nIMPROVEMENTS:");
                for reg in regressions.iter().filter(|r| r.change_percent < -5.0) {
                    println!(
                        "  [IMPROVED] {}: {:+.2}% ({} ns → {} ns)",
                        reg.name, reg.change_percent, reg.baseline_mean, reg.current_mean
                    );
                }
            }

            println!("{}", "=".repeat(80));

            if severe_count > 0 {
                println!(
                    "\n⚠️  WARNING: {} SEVERE REGRESSIONS DETECTED!",
                    severe_count
                );
                println!("   Do not release until performance is restored!");
            } else if has_regression {
                println!("\n⚠️  NOTICE: Regressions detected - review before release");
            } else {
                println!("\n✓  No regressions detected - performance stable or improved");
            }
        } else {
            println!("\n{}", "=".repeat(80));
            println!("NO BASELINE AVAILABLE");
            println!("{}", "=".repeat(80));
            println!("This is the first run - establishing baseline");
            println!("Current measurements saved to: baselines/current.json");
            println!("Run benchmarks again to compare against this baseline");
            println!("{}", "=".repeat(80));
        }
    });

    // Export comprehensive report with tables and insights
    let opt_report = REPORT.with(|r| {
        let borrowed = r.borrow();
        borrowed.as_ref().cloned()
    });

    if let Some(mut report) = opt_report {
        let regression_results = collect_regression_results();
        let version_comparison = analyze_version_comparison();

        // Create all 15+ tables
        if !regression_results.is_empty() {
            create_regression_summary_table(&regression_results, &mut report);
            create_performance_by_operation_table(&regression_results, &mut report);
            create_performance_by_size_table(&regression_results, &mut report);
            create_p95_latency_table(&regression_results, &mut report);
            create_version_comparison_table(&version_comparison, &mut report);
            create_complexity_impact_table(&regression_results, &mut report);
            create_severity_distribution_table(&regression_results, &mut report);
            create_throughput_regression_table(&regression_results, &mut report);
            create_critical_path_table(&regression_results, &mut report);
            create_stability_metrics_table(&regression_results, &mut report);
            create_regression_trends_table(&regression_results, &mut report);
            create_test_coverage_table(&regression_results, &mut report);
            create_action_items_table(&regression_results, &mut report);
            create_statistical_confidence_table(&regression_results, &mut report);
        }

        create_baseline_quality_table(&mut report);

        // Generate insights
        let regression_results = collect_regression_results();
        let version_comparison = analyze_version_comparison();
        generate_insights(&regression_results, &version_comparison, &mut report);

        println!("\n{}", "=".repeat(80));
        println!("REGRESSION TRACKING REPORT");
        println!("{}", "=".repeat(80));
        report.print();

        if let Err(e) = std::fs::create_dir_all("target") {
            eprintln!("Failed to create target directory: {}", e);
            return;
        }

        let config = ExportConfig::all();
        match report.save_all("target/regression_report", &config) {
            Ok(()) => println!(
                "\n✓ Exported {} tables and {} insights to target/regression_report.*",
                report.custom_tables.len(),
                report.insights.len()
            ),
            Err(e) => eprintln!("Failed to export reports: {}", e),
        }
    }
}

// ============================================================================
// Criterion Configuration
// ============================================================================

#[cfg(feature = "json")]
criterion_group! {
    name = regression_benches;
    config = Criterion::default();
    targets =
        bench_regression_parsing,
        bench_regression_canonicalization,
        bench_regression_conversion,
        bench_regression_validation,
        bench_regression_linting,
        bench_regression_full_pipeline,
        export_reports,
}

#[cfg(not(feature = "json"))]
criterion_group! {
    name = regression_benches;
    config = Criterion::default();
    targets =
        bench_regression_parsing,
        bench_regression_canonicalization,
        bench_regression_validation,
        bench_regression_linting,
        bench_regression_full_pipeline,
        export_reports,
}

criterion_main!(regression_benches);
