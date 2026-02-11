// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Performance benchmarks for the HEDL linter.
//!
//! Measures lint execution time across different document sizes and rule configurations.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hedl_core::parse;
use hedl_lint::lint;
use std::hint::black_box;

/// Generate a HEDL document with the specified number of nodes.
fn generate_document(node_count: usize) -> String {
    let mut doc =
        String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:User:[id,name,email,age]\n---\nusers:@User\n");
    for i in 0..node_count {
        doc.push_str(&format!(
            " |user{},User {},user{}@example.com,{}\n",
            i,
            i,
            i,
            i % 100
        ));
    }
    doc
}

/// Benchmark linting with varying document sizes.
fn bench_lint_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("lint_scaling");

    for size in &[10, 100, 500, 1000, 5000] {
        let doc_str = generate_document(*size);
        let doc = parse(doc_str.as_bytes()).expect("Failed to parse document");

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("nodes", size), &doc, |b, doc| {
            b.iter(|| {
                let diagnostics = lint(black_box(doc));
                black_box(diagnostics)
            });
        });
    }

    group.finish();
}

/// Benchmark linting a small document (baseline).
fn bench_lint_small(c: &mut Criterion) {
    let doc_str = "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:User:[id,name]\n---\nusers:@User\n |alice,Alice Smith\n |bob,Bob Jones\n";
    let doc = parse(doc_str.as_bytes()).expect("Failed to parse document");

    c.bench_function("lint_small", |b| {
        b.iter(|| {
            let diagnostics = lint(black_box(&doc));
            black_box(diagnostics)
        });
    });
}

/// Benchmark linting a document with deeply nested structures.
fn bench_lint_nested(c: &mut Criterion) {
    let mut doc_str = String::from(
        "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:Department:[id,name]\n%S:Team:[id,name]\n%S:Employee:[id,name]\n%N:Department>Team\n%N:Team>Employee\n---\ndepartments:@Department\n",
    );

    // Create 10 departments with 5 teams each with 10 employees
    for d in 0..10 {
        doc_str.push_str(&format!(" |dept{d},Department {d}\n"));
        for t in 0..5 {
            doc_str.push_str(&format!("  |team{d}_{t},Team {t}\n"));
            for e in 0..10 {
                doc_str.push_str(&format!("   |emp{d}_{t}_{e},Employee {e}\n"));
            }
        }
    }

    let doc = parse(doc_str.as_bytes()).expect("Failed to parse nested document");

    c.bench_function("lint_nested", |b| {
        b.iter(|| {
            let diagnostics = lint(black_box(&doc));
            black_box(diagnostics)
        });
    });
}

/// Benchmark linting a document with many references.
fn bench_lint_references(c: &mut Criterion) {
    let mut doc_str = String::from(
        "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:User:[id,name]\n%S:Post:[id,title,author]\n---\nusers:@User\n",
    );

    // Create 100 users
    for i in 0..100 {
        doc_str.push_str(&format!(" |user{i},User {i}\n"));
    }

    doc_str.push_str("posts:@Post\n");
    // Create 500 posts referencing users
    for i in 0..500 {
        doc_str.push_str(&format!(" |post{},Post {},@User:user{}\n", i, i, i % 100));
    }

    let doc = parse(doc_str.as_bytes()).expect("Failed to parse document with references");

    c.bench_function("lint_references", |b| {
        b.iter(|| {
            let diagnostics = lint(black_box(&doc));
            black_box(diagnostics)
        });
    });
}

criterion_group!(
    benches,
    bench_lint_scaling,
    bench_lint_small,
    bench_lint_nested,
    bench_lint_references
);
criterion_main!(benches);
