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

//! Real WebAssembly Performance Benchmarks
//!
//! This benchmark suite measures ACTUAL WebAssembly performance by:
//! 1. Loading pre-compiled hedl-wasm.wasm module
//! 2. Running benchmarks through wasmtime runtime
//! 3. Comparing real WASM vs native execution times
//!
//! ## Prerequisites
//! Build the WASM module first:
//! ```bash
//! cd crates/hedl-wasm && wasm-pack build --release --target web
//! ```

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hedl_bench::{generate_users, sizes, BenchmarkReport, CustomTable, ExportConfig, Insight, PerfResult, TableCell};
use std::cell::RefCell;
use std::path::PathBuf;
use std::time::Instant;

#[cfg(feature = "wasm-runtime")]
use wasmtime::*;

thread_local! {
    static REPORT: RefCell<Option<BenchmarkReport>> = RefCell::new(None);
}

fn init_report() {
    REPORT.with(|r| {
        let mut report = BenchmarkReport::new("HEDL WASM Performance Report");
        report.set_timestamp();
        report.add_note("Real WebAssembly benchmarks using wasmtime runtime");
        report.add_note("Compares actual WASM execution vs native Rust");
        *r.borrow_mut() = Some(report);
    });
}

fn add_perf_result(name: &str, time_ns: u64, iterations: u64, throughput_bytes: Option<u64>) {
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
}

// ============================================================================
// WASM Module Loading
// ============================================================================

#[cfg(feature = "wasm-runtime")]
fn find_wasm_module() -> Option<PathBuf> {
    let candidates = [
        "crates/hedl-wasm/pkg/hedl_wasm_bg.wasm",
        "../hedl-wasm/pkg/hedl_wasm_bg.wasm",
        "target/wasm32-unknown-unknown/release/hedl_wasm.wasm",
    ];

    for candidate in &candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

#[cfg(feature = "wasm-runtime")]
struct WasmBenchContext {
    store: Store<()>,
    memory: Memory,
    alloc_fn: TypedFunc<i32, i32>,
    dealloc_fn: TypedFunc<(i32, i32), ()>,
    parse_fn: TypedFunc<(i32, i32), i32>,
}

#[cfg(feature = "wasm-runtime")]
impl WasmBenchContext {
    fn new(wasm_bytes: &[u8]) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::new(&engine, wasm_bytes)?;
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;

        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("memory export not found"))?;

        let alloc_fn = instance.get_typed_func::<i32, i32>(&mut store, "__wbindgen_malloc")?;
        let dealloc_fn = instance.get_typed_func::<(i32, i32), ()>(&mut store, "__wbindgen_free")?;
        let parse_fn = instance.get_typed_func::<(i32, i32), i32>(&mut store, "parse")?;

        Ok(Self {
            store,
            memory,
            alloc_fn,
            dealloc_fn,
            parse_fn,
        })
    }

    fn parse_hedl(&mut self, input: &str) -> Result<bool> {
        let input_bytes = input.as_bytes();
        let len = input_bytes.len() as i32;

        // Allocate memory in WASM
        let ptr = self.alloc_fn.call(&mut self.store, len)?;

        // Write input to WASM memory
        self.memory.write(&mut self.store, ptr as usize, input_bytes)?;

        // Call parse function
        let result = self.parse_fn.call(&mut self.store, (ptr, len))?;

        // Free the allocated memory
        self.dealloc_fn.call(&mut self.store, (ptr, len))?;

        Ok(result != 0)
    }
}

// ============================================================================
// Native Baseline Benchmarks
// ============================================================================

fn bench_native_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("native_parse");

    for size in [sizes::SMALL, sizes::MEDIUM, sizes::LARGE] {
        let hedl = generate_users(size);
        let bytes = hedl.len();

        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &hedl, |b, input| {
            b.iter(|| hedl_core::parse(black_box(input.as_bytes())))
        });

        // Collect metrics
        let iterations = 100u64;
        let mut total_ns = 0u64;
        for _ in 0..iterations {
            let start = Instant::now();
            let _ = hedl_core::parse(hedl.as_bytes());
            total_ns += start.elapsed().as_nanos() as u64;
        }
        add_perf_result(
            &format!("native_parse_{}", size),
            total_ns,
            iterations,
            Some(bytes as u64),
        );
    }

    group.finish();
}

// ============================================================================
// WASM Runtime Benchmarks
// ============================================================================

#[cfg(feature = "wasm-runtime")]
fn bench_wasm_parse(c: &mut Criterion) {
    let wasm_path = match find_wasm_module() {
        Some(path) => path,
        None => {
            eprintln!("WASM module not found. Build it first:");
            eprintln!("  cd crates/hedl-wasm && wasm-pack build --release --target web");
            // Don't record a result with 0 iterations - it would cause divide by zero in avg_time_ns()
            return;
        }
    };

    let wasm_bytes = match std::fs::read(&wasm_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to read WASM module: {}", e);
            return;
        }
    };

    // Record WASM binary size
    let wasm_size = wasm_bytes.len();
    REPORT.with(|r| {
        if let Some(ref mut report) = *r.borrow_mut() {
            report.add_note(&format!("WASM binary size: {} bytes ({:.1} KB)", wasm_size, wasm_size as f64 / 1024.0));
        }
    });

    let mut group = c.benchmark_group("wasm_parse");

    for size in [sizes::SMALL, sizes::MEDIUM, sizes::LARGE] {
        let hedl = generate_users(size);
        let bytes = hedl.len();

        // Create fresh WASM context for each benchmark
        let mut ctx = match WasmBenchContext::new(&wasm_bytes) {
            Ok(ctx) => ctx,
            Err(e) => {
                eprintln!("Failed to create WASM context: {}", e);
                continue;
            }
        };

        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &hedl, |b, input| {
            b.iter(|| ctx.parse_hedl(black_box(input)))
        });

        // Collect metrics
        let iterations = 100u64;
        let mut total_ns = 0u64;
        for _ in 0..iterations {
            let start = Instant::now();
            let _ = ctx.parse_hedl(&hedl);
            total_ns += start.elapsed().as_nanos() as u64;
        }
        add_perf_result(
            &format!("wasm_parse_{}", size),
            total_ns,
            iterations,
            Some(bytes as u64),
        );
    }

    group.finish();
}

#[cfg(not(feature = "wasm-runtime"))]
fn bench_wasm_parse(_c: &mut Criterion) {
    eprintln!("WASM benchmarks disabled. Enable with: --features wasm-runtime");
    // Don't record a result with 0 iterations - it would cause divide by zero in avg_time_ns()
}

// ============================================================================
// WASM Instantiation Benchmark
// ============================================================================

#[cfg(feature = "wasm-runtime")]
fn bench_wasm_instantiation(c: &mut Criterion) {
    let wasm_path = match find_wasm_module() {
        Some(path) => path,
        None => return,
    };

    let wasm_bytes = match std::fs::read(&wasm_path) {
        Ok(bytes) => bytes,
        Err(_) => return,
    };

    let mut group = c.benchmark_group("wasm_instantiation");

    // Benchmark module compilation
    group.bench_function("compile", |b| {
        let engine = Engine::default();
        b.iter(|| Module::new(&engine, black_box(&wasm_bytes)))
    });

    // Collect compilation metrics
    let engine = Engine::default();
    let iterations = 10u64;
    let mut total_ns = 0u64;
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = Module::new(&engine, &wasm_bytes);
        total_ns += start.elapsed().as_nanos() as u64;
    }
    add_perf_result("wasm_compile", total_ns, iterations, Some(wasm_bytes.len() as u64));

    // Benchmark instantiation (with pre-compiled module)
    let module = Module::new(&engine, &wasm_bytes).unwrap();
    group.bench_function("instantiate", |b| {
        b.iter(|| {
            let mut store = Store::new(&engine, ());
            Instance::new(&mut store, black_box(&module), &[])
        })
    });

    // Collect instantiation metrics
    let mut total_ns = 0u64;
    for _ in 0..iterations {
        let start = Instant::now();
        let mut store = Store::new(&engine, ());
        let _ = Instance::new(&mut store, &module, &[]);
        total_ns += start.elapsed().as_nanos() as u64;
    }
    add_perf_result("wasm_instantiate", total_ns, iterations, None);

    group.finish();
}

#[cfg(not(feature = "wasm-runtime"))]
fn bench_wasm_instantiation(_c: &mut Criterion) {}

// ============================================================================
// Report Generation
// ============================================================================

fn create_comparison_table(report: &mut BenchmarkReport) {
    let results = report.perf_results.clone();

    let mut table = CustomTable {
        title: "WASM vs Native Performance".to_string(),
        headers: vec![
            "Size".to_string(),
            "Native (ms)".to_string(),
            "WASM (ms)".to_string(),
            "Slowdown".to_string(),
            "Native Throughput (MB/s)".to_string(),
            "WASM Throughput (MB/s)".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    for size in [sizes::SMALL, sizes::MEDIUM, sizes::LARGE] {
        let native_name = format!("native_parse_{}", size);
        let wasm_name = format!("wasm_parse_{}", size);

        let native = results.iter().find(|r| r.name == native_name);
        let wasm = results.iter().find(|r| r.name == wasm_name);

        if let (Some(n), Some(w)) = (native, wasm) {
            let native_ms = n.avg_time_ns.unwrap_or(0) as f64 / 1_000_000.0;
            let wasm_ms = w.avg_time_ns.unwrap_or(0) as f64 / 1_000_000.0;
            let slowdown = if native_ms > 0.0 { wasm_ms / native_ms } else { 0.0 };

            table.rows.push(vec![
                TableCell::Integer(size as i64),
                TableCell::Float(native_ms),
                TableCell::Float(wasm_ms),
                TableCell::Float(slowdown),
                TableCell::Float(n.throughput_mbs.unwrap_or(0.0)),
                TableCell::Float(w.throughput_mbs.unwrap_or(0.0)),
            ]);
        }
    }

    if !table.rows.is_empty() {
        report.add_custom_table(table);
    }
}

fn create_instantiation_table(report: &mut BenchmarkReport) {
    let results = report.perf_results.clone();

    let compile = results.iter().find(|r| r.name == "wasm_compile");
    let instantiate = results.iter().find(|r| r.name == "wasm_instantiate");

    if compile.is_none() && instantiate.is_none() {
        return;
    }

    let mut table = CustomTable {
        title: "WASM Startup Overhead".to_string(),
        headers: vec![
            "Phase".to_string(),
            "Time (ms)".to_string(),
            "Notes".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    if let Some(c) = compile {
        let ms = c.avg_time_ns.unwrap_or(0) as f64 / 1_000_000.0;
        let size_kb = c.throughput_bytes.unwrap_or(0) as f64 / 1024.0;
        table.rows.push(vec![
            TableCell::String("Compilation".to_string()),
            TableCell::Float(ms),
            TableCell::String(format!("Module size: {:.1} KB", size_kb)),
        ]);
    }

    if let Some(i) = instantiate {
        let ms = i.avg_time_ns.unwrap_or(0) as f64 / 1_000_000.0;
        table.rows.push(vec![
            TableCell::String("Instantiation".to_string()),
            TableCell::Float(ms),
            TableCell::String("Per-instance overhead".to_string()),
        ]);
    }

    report.add_custom_table(table);
}

fn generate_insights(report: &mut BenchmarkReport) {
    let results = &report.perf_results;

    // Calculate average slowdown
    let mut slowdowns = Vec::new();
    for size in [sizes::SMALL, sizes::MEDIUM, sizes::LARGE] {
        let native = results.iter().find(|r| r.name == format!("native_parse_{}", size));
        let wasm = results.iter().find(|r| r.name == format!("wasm_parse_{}", size));

        if let (Some(n), Some(w)) = (native, wasm) {
            let native_ns = n.avg_time_ns.unwrap_or(1);
            let wasm_ns = w.avg_time_ns.unwrap_or(1);
            if native_ns > 0 && wasm_ns > 0 {
                slowdowns.push(wasm_ns as f64 / native_ns as f64);
            }
        }
    }

    if !slowdowns.is_empty() {
        let avg_slowdown = slowdowns.iter().sum::<f64>() / slowdowns.len() as f64;

        report.add_insight(Insight {
            category: "finding".to_string(),
            title: format!("WASM is {:.2}x slower than native on average", avg_slowdown),
            description: "Measured slowdown factor from real WASM execution".to_string(),
            data_points: slowdowns.iter().enumerate().map(|(i, s)| {
                let size = [sizes::SMALL, sizes::MEDIUM, sizes::LARGE][i];
                format!("Size {}: {:.2}x slowdown", size, s)
            }).collect(),
        });

        let recommendation = if avg_slowdown < 2.0 {
            "WASM overhead is acceptable for most use cases"
        } else if avg_slowdown < 5.0 {
            "WASM overhead is moderate - consider native for performance-critical paths"
        } else {
            "WASM overhead is significant - use native code where possible"
        };

        report.add_insight(Insight {
            category: "recommendation".to_string(),
            title: recommendation.to_string(),
            description: format!("Based on {:.2}x average slowdown", avg_slowdown),
            data_points: vec![
                "Use WASM for browser/sandboxed execution".to_string(),
                "Use native for server-side batch processing".to_string(),
            ],
        });
    } else {
        report.add_insight(Insight {
            category: "warning".to_string(),
            title: "WASM benchmark data not available".to_string(),
            description: "Build WASM module to enable real WASM benchmarks".to_string(),
            data_points: vec![
                "Run: cd crates/hedl-wasm && wasm-pack build --release".to_string(),
                "Then re-run benchmarks".to_string(),
            ],
        });
    }
}

fn export_reports(c: &mut Criterion) {
    let mut group = c.benchmark_group("export_reports");
    group.bench_function("export", |b| b.iter(|| 1));
    group.finish();

    let opt_report = REPORT.with(|r| r.borrow().clone());

    if let Some(mut report) = opt_report {
        create_comparison_table(&mut report);
        create_instantiation_table(&mut report);
        generate_insights(&mut report);

        println!("\n{}", "=".repeat(80));
        println!("WASM PERFORMANCE REPORT");
        println!("{}", "=".repeat(80));
        report.print();

        if let Err(e) = std::fs::create_dir_all("target") {
            eprintln!("Failed to create target directory: {}", e);
            return;
        }

        let config = ExportConfig::all();
        match report.save_all("target/wasm_report", &config) {
            Ok(()) => println!(
                "\n✓ Exported {} tables and {} insights to target/wasm_report.*",
                report.custom_tables.len(),
                report.insights.len()
            ),
            Err(e) => eprintln!("Failed to export reports: {}", e),
        }
    }
}

static INIT: std::sync::Once = std::sync::Once::new();

fn ensure_init() {
    INIT.call_once(|| {
        init_report();
    });
}

criterion_group! {
    name = benches;
    config = {
        let c = Criterion::default();
        ensure_init();
        c
    };
    targets =
        bench_native_parse,
        bench_wasm_parse,
        bench_wasm_instantiation,
        export_reports
}

criterion_main!(benches);
