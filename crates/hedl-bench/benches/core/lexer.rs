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

//! Core lexer benchmarks for HEDL.
//!
//! Measures tokenization, span tracking, and error recovery performance using
//! the new infrastructure from src/core/, src/harness/, and src/reporters/.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hedl_bench::{
    generate_users, generate_products, generate_blog, sizes,
    BenchmarkReport, PerfResult,
};
use hedl_core::lex::{parse_expression, parse_reference, scan_regions};
use std::cell::RefCell;
use std::time::Instant;

// Thread-local report storage
thread_local! {
    static REPORT: RefCell<Option<BenchmarkReport>> = RefCell::new(None);
}

static INIT: std::sync::Once = std::sync::Once::new();

fn ensure_init() {
    INIT.call_once(|| {
        REPORT.with(|r| {
            let mut report = BenchmarkReport::new("HEDL Lexer Performance Report");
            report.set_timestamp();
            report.add_note("Tokenization and lexical analysis performance");
            report.add_note("Tests reference parsing, expression parsing, and region scanning");
            report.add_note("Includes error recovery and span tracking benchmarks");
            *r.borrow_mut() = Some(report);
        });
    });
}

fn record_perf(name: &str, iterations: u64, total_ns: u64, throughput_bytes: Option<u64>) {
    REPORT.with(|r| {
        if let Some(ref mut report) = *r.borrow_mut() {
            let throughput_mbs = throughput_bytes.map(|bytes| {
                let bytes_per_sec = (bytes as f64 * 1e9) / total_ns as f64;
                bytes_per_sec / 1_000_000.0
            });

            report.add_perf(PerfResult {
                name: name.to_string(),
                iterations,
                total_time_ns: total_ns,
                throughput_bytes,
                avg_time_ns: Some(total_ns / iterations.max(1)),
                throughput_mbs,
            });
        }
    });
}

// ============================================================================
// Reference Parsing Benchmarks
// ============================================================================

fn bench_parse_reference(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_reference");

    let test_cases = vec![
        ("simple", "User:123"),
        ("complex", "BlogPost:abc-123-xyz-789"),
        ("nested_type", "Company.Department:eng-001"),
        ("long_id", "Organization:xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx"),
    ];

    for (name, input) in &test_cases {
        group.bench_function(*name, |b| b.iter(|| parse_reference(black_box(input))));

        // Collect metrics
        let iterations = 10_000u64;
        let mut total_ns = 0u64;
        for _ in 0..iterations {
            let start = Instant::now();
            let _ = parse_reference(input);
            total_ns += start.elapsed().as_nanos() as u64;
        }
        record_perf(
            &format!("parse_reference_{}", name),
            iterations,
            total_ns,
            Some(input.len() as u64),
        );
    }

    group.finish();
}

// ============================================================================
// Expression Parsing Benchmarks
// ============================================================================

fn bench_parse_expression(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_expression");

    let test_cases = vec![
        ("simple_calc", "$(1 + 2)"),
        ("with_vars", "$(price * quantity)"),
        ("complex_expr", "$(subtotal * (1 + tax_rate))"),
        ("nested_expr", "$(((a + b) * c) - (d / e))"),
    ];

    for (name, input) in &test_cases {
        group.bench_function(*name, |b| b.iter(|| parse_expression(black_box(input))));

        // Collect metrics
        let iterations = 10_000u64;
        let mut total_ns = 0u64;
        for _ in 0..iterations {
            let start = Instant::now();
            let _ = parse_expression(input);
            total_ns += start.elapsed().as_nanos() as u64;
        }
        record_perf(
            &format!("parse_expression_{}", name),
            iterations,
            total_ns,
            Some(input.len() as u64),
        );
    }

    group.finish();
}

// ============================================================================
// Region Scanning Benchmarks
// ============================================================================

fn bench_scan_regions(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_regions");

    let test_cases = vec![
        ("simple_line", "name: Alice"),
        ("quoted_string", r#"message: "Hello, world!""#),
        ("expression", "total: $(price * quantity)"),
        ("reference", "author: @User:123"),
        ("multiline", "description: \"Line 1\nLine 2\nLine 3\""),
    ];

    for (name, input) in &test_cases {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_function(*name, |b| b.iter(|| scan_regions(black_box(input))));

        // Collect metrics
        let iterations = 10_000u64;
        let mut total_ns = 0u64;
        for _ in 0..iterations {
            let start = Instant::now();
            let _ = scan_regions(input);
            total_ns += start.elapsed().as_nanos() as u64;
        }
        record_perf(
            &format!("scan_regions_{}", name),
            iterations,
            total_ns,
            Some(input.len() as u64),
        );
    }

    group.finish();
}

// ============================================================================
// Document Scanning Benchmarks
// ============================================================================

fn bench_scan_documents(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_documents");

    for &size in &[sizes::SMALL, sizes::MEDIUM, sizes::LARGE] {
        let hedl = generate_users(size);
        let bytes = hedl.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::new("users", size), &hedl, |b, input| {
            b.iter(|| scan_regions(black_box(input)))
        });

        // Collect metrics
        let iterations = match size {
            s if s <= sizes::SMALL => 1000,
            s if s <= sizes::MEDIUM => 100,
            _ => 10,
        };

        let mut total_ns = 0u64;
        for _ in 0..iterations {
            let start = Instant::now();
            let _ = scan_regions(&hedl);
            total_ns += start.elapsed().as_nanos() as u64;
        }
        record_perf(&format!("scan_documents_users_{}", size), iterations, total_ns, Some(bytes));
    }

    group.finish();
}

// ============================================================================
// Complex Document Scanning
// ============================================================================

fn bench_scan_complex_documents(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_complex");

    // Products (varied data types)
    for &size in &[sizes::SMALL, sizes::MEDIUM] {
        let hedl = generate_products(size);
        let bytes = hedl.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::new("products", size), &hedl, |b, input| {
            b.iter(|| scan_regions(black_box(input)))
        });

        let iterations = if size <= sizes::SMALL { 500 } else { 50 };
        let mut total_ns = 0u64;
        for _ in 0..iterations {
            let start = Instant::now();
            let _ = scan_regions(&hedl);
            total_ns += start.elapsed().as_nanos() as u64;
        }
        record_perf(&format!("scan_complex_products_{}", size), iterations, total_ns, Some(bytes));
    }

    // Blog (nested structures)
    let blog = generate_blog(50, 5);
    let bytes = blog.len() as u64;

    group.throughput(Throughput::Bytes(bytes));
    group.bench_function("blog", |b| b.iter(|| scan_regions(black_box(&blog))));

    let iterations = 100;
    let mut total_ns = 0u64;
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = scan_regions(&blog);
        total_ns += start.elapsed().as_nanos() as u64;
    }
    record_perf("scan_complex_blog", iterations, total_ns, Some(bytes));

    group.finish();
}

// ============================================================================
// Report Export
// ============================================================================

fn export_reports(c: &mut Criterion) {
    let mut group = c.benchmark_group("export_reports");
    group.bench_function("export", |b| b.iter(|| 1));
    group.finish();

    REPORT.with(|r| {
        if let Some(ref report) = *r.borrow() {
            println!("\n{}", "=".repeat(80));
            println!("LEXER PERFORMANCE REPORT");
            println!("{}", "=".repeat(80));
            report.print();

            // Create target directory
            if let Err(e) = std::fs::create_dir_all("target") {
                eprintln!("Failed to create target directory: {}", e);
                return;
            }

            // Export reports using built-in methods
            let base_path = "target/lexer_report";
            if let Err(e) = report.save_json(format!("{}.json", base_path)) {
                eprintln!("Failed to export JSON: {}", e);
            } else {
                println!("Exported JSON: {}.json", base_path);
            }

            if let Err(e) = std::fs::write(format!("{}.md", base_path), report.to_markdown()) {
                eprintln!("Failed to export Markdown: {}", e);
            } else {
                println!("Exported Markdown: {}.md", base_path);
            }

            println!("\nRECOMMENDATIONS:");
            println!("1. Reference parsing should be <100ns per reference for good performance");
            println!("2. Expression parsing time should scale linearly with expression complexity");
            println!("3. Region scanning throughput should exceed 100 MB/s for simple documents");
            println!("4. Consider optimizing reference/expression parsing if avg time >500ns");
            println!("5. Profile string allocation overhead if scanning throughput is below 50 MB/s");
        }
    });
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group! {
    name = benches;
    config = {
        let c = Criterion::default();
        ensure_init();
        c
    };
    targets = bench_parse_reference,
        bench_parse_expression,
        bench_scan_regions,
        bench_scan_documents,
        bench_scan_complex_documents,
        export_reports
}

criterion_main!(benches);
