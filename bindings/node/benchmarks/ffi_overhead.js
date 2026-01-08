#!/usr/bin/env node
/**
 * FFI Overhead Benchmarks for HEDL Node.js Bindings
 *
 * Measures the performance overhead of FFI calls compared to native Rust operations.
 * Tests parse, convert, and validate operations across multiple document sizes.
 *
 * Run with:
 *   node benchmarks/ffi_overhead.js
 *   or: time node benchmarks/ffi_overhead.js
 */

const hedl = require('../dist/index.js');
const fs = require('fs');
const path = require('path');

// Test data generators
function generateSmallHedl() {
  return `%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | 1, Alice, alice@example.com
  | 2, Bob, bob@example.com`;
}

function generateMediumHedl() {
  const lines = [
    '%VERSION: 1.0',
    '%STRUCT: User: [id, name, email, dept]',
    '---',
    'users: @User'
  ];
  for (let i = 0; i < 100; i++) {
    lines.push(`  | ${i}, User${i}, user${i}@example.com, dept${i % 10}`);
  }
  return lines.join('\n');
}

function generateLargeHedl() {
  const lines = [
    '%VERSION: 1.0',
    '%STRUCT: User: [id, name, email, dept, salary]',
    '---',
    'users: @User'
  ];
  for (let i = 0; i < 1000; i++) {
    lines.push(`  | ${i}, User${i}, user${i}@example.com, dept${i % 10}, ${50000 + i * 100}`);
  }
  return lines.join('\n');
}

/**
 * Stores benchmark results for a single operation
 */
class BenchmarkResult {
  constructor(name, operation, size) {
    this.name = name;
    this.operation = operation;
    this.size = size;
    this.times = [];
    this.overheadPercent = 0.0;
  }

  addTime(timeNs) {
    this.times.push(timeNs);
  }

  avgTimeNs() {
    return this.times.length > 0 ? this.times.reduce((a, b) => a + b, 0) / this.times.length : 0;
  }

  minTimeNs() {
    return Math.min(...this.times);
  }

  maxTimeNs() {
    return Math.max(...this.times);
  }

  stdDevNs() {
    if (this.times.length < 2) {
      return 0;
    }
    const avg = this.avgTimeNs();
    const variance = this.times.reduce((sum, t) => sum + Math.pow(t - avg, 2), 0) / (this.times.length - 1);
    return Math.sqrt(variance);
  }

  toObject() {
    return {
      name: this.name,
      operation: this.operation,
      size: this.size,
      avg_time_ns: Math.round(this.avgTimeNs() * 100) / 100,
      min_time_ns: Math.round(this.minTimeNs() * 100) / 100,
      max_time_ns: Math.round(this.maxTimeNs() * 100) / 100,
      std_dev_ns: Math.round(this.stdDevNs() * 100) / 100,
      overhead_percent: Math.round(this.overheadPercent * 100) / 100,
      samples: this.times.length
    };
  }
}

/**
 * Runs FFI overhead benchmarks
 */
class BenchmarkSuite {
  constructor() {
    this.results = [];
  }

  benchmarkParse(content, size, iterations = 10) {
    console.log(`\nBenchmarking parse (${size})...`);

    const times = [];
    for (let i = 0; i < iterations; i++) {
      const start = process.hrtime.bigint();
      const doc = hedl.parse(content);
      doc.close();
      const elapsed = process.hrtime.bigint() - start;
      times.push(Number(elapsed));
    }

    const result = new BenchmarkResult('parse', 'Parse HEDL', size);
    times.forEach(t => result.addTime(t));
    this.results.push(result);
    console.log(`  Average: ${Math.round(result.avgTimeNs())} ns`);
  }

  benchmarkToJson(content, size, iterations = 10) {
    console.log(`\nBenchmarking to_json (${size})...`);

    const doc = hedl.parse(content);
    const times = [];

    for (let i = 0; i < iterations; i++) {
      const start = process.hrtime.bigint();
      doc.toJson();
      const elapsed = process.hrtime.bigint() - start;
      times.push(Number(elapsed));
    }

    doc.close();

    const result = new BenchmarkResult('to_json', 'Convert to JSON', size);
    times.forEach(t => result.addTime(t));
    this.results.push(result);
    console.log(`  Average: ${Math.round(result.avgTimeNs())} ns`);
  }

  benchmarkToYaml(content, size, iterations = 10) {
    console.log(`\nBenchmarking to_yaml (${size})...`);

    const doc = hedl.parse(content);
    const times = [];

    for (let i = 0; i < iterations; i++) {
      const start = process.hrtime.bigint();
      doc.toYaml();
      const elapsed = process.hrtime.bigint() - start;
      times.push(Number(elapsed));
    }

    doc.close();

    const result = new BenchmarkResult('to_yaml', 'Convert to YAML', size);
    times.forEach(t => result.addTime(t));
    this.results.push(result);
    console.log(`  Average: ${Math.round(result.avgTimeNs())} ns`);
  }

  benchmarkValidate(content, size, iterations = 10) {
    console.log(`\nBenchmarking validate (${size})...`);

    const times = [];
    for (let i = 0; i < iterations; i++) {
      const start = process.hrtime.bigint();
      hedl.validate(content);
      const elapsed = process.hrtime.bigint() - start;
      times.push(Number(elapsed));
    }

    const result = new BenchmarkResult('validate', 'Validate HEDL', size);
    times.forEach(t => result.addTime(t));
    this.results.push(result);
    console.log(`  Average: ${Math.round(result.avgTimeNs())} ns`);
  }

  runAll() {
    console.log('='.repeat(70));
    console.log('HEDL Node.js FFI Overhead Benchmarks');
    console.log('='.repeat(70));

    const testCases = [
      ['small', generateSmallHedl(), 20],
      ['medium', generateMediumHedl(), 10],
      ['large', generateLargeHedl(), 5]
    ];

    for (const [sizeName, content, iterations] of testCases) {
      console.log(`\n${'='.repeat(70)}`);
      console.log(`Testing ${sizeName} documents (${content.length} bytes)`);
      console.log(`${'='.repeat(70)}`);

      this.benchmarkParse(content, sizeName, iterations);
      this.benchmarkValidate(content, sizeName, iterations);
      this.benchmarkToJson(content, sizeName, iterations);
      this.benchmarkToYaml(content, sizeName, iterations);
    }
  }

  printSummary() {
    console.log('\n' + '='.repeat(70));
    console.log('BENCHMARK SUMMARY');
    console.log('='.repeat(70));

    const operations = {};
    for (const result of this.results) {
      if (!operations[result.operation]) {
        operations[result.operation] = [];
      }
      operations[result.operation].push(result);
    }

    const sizeOrder = { small: 0, medium: 1, large: 2 };

    for (const [opName, opResults] of Object.entries(operations).sort()) {
      console.log(`\n${opName}:`);
      console.log(`  ${'Size':<10} ${'Avg (ns)':<15} ${'Min (ns)':<15} ${'Max (ns)':<15} ${'StdDev':<15}`);
      console.log('  ' + '-'.repeat(70));

      const sorted = opResults.sort((a, b) => (sizeOrder[a.size] || 999) - (sizeOrder[b.size] || 999));
      for (const result of sorted) {
        console.log(
          `  ${result.size.padEnd(10)} ${Math.round(result.avgTimeNs()).toString().padEnd(15)} ` +
          `${Math.round(result.minTimeNs()).toString().padEnd(15)} ${Math.round(result.maxTimeNs()).toString().padEnd(15)} ` +
          `${Math.round(result.stdDevNs()).toString().padEnd(15)}`
        );
      }
    }
  }

  exportJson(filename = 'ffi_overhead_results.json') {
    const data = {
      benchmark: 'HEDL Node.js FFI Overhead',
      timestamp: new Date().toISOString(),
      results: this.results.map(r => r.toObject())
    };

    fs.writeFileSync(filename, JSON.stringify(data, null, 2));
    console.log(`\nResults exported to ${filename}`);
  }
}

function main() {
  const suite = new BenchmarkSuite();
  suite.runAll();
  suite.printSummary();
  suite.exportJson();
}

main();
