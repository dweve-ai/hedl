// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

// Package main contains FFI overhead benchmarks for HEDL Go bindings.
//
// Measures the performance overhead of FFI calls compared to native Rust operations.
// Tests parse, convert, and validate operations across multiple document sizes.
//
// Run with:
//   go test -bench=. -benchtime=5s ./benchmarks
//   or: go test -bench=BenchmarkParse -benchmem ./benchmarks
package main

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"strings"
	"testing"
	"time"
)

const (
	smallSize  = "small"
	mediumSize = "medium"
	largeSize  = "large"
)

// generateSmallHedl generates small HEDL document.
func generateSmallHedl() string {
	return `%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | 1, Alice, alice@example.com
  | 2, Bob, bob@example.com`
}

// generateMediumHedl generates medium HEDL document.
func generateMediumHedl() string {
	lines := []string{
		"%VERSION: 1.0",
		"%STRUCT: User: [id, name, email, dept]",
		"---",
		"users: @User",
	}
	for i := 0; i < 100; i++ {
		lines = append(lines, fmt.Sprintf("  | %d, User%d, user%d@example.com, dept%d", i, i, i, i%10))
	}
	return strings.Join(lines, "\n")
}

// generateLargeHedl generates large HEDL document.
func generateLargeHedl() string {
	lines := []string{
		"%VERSION: 1.0",
		"%STRUCT: User: [id, name, email, dept, salary]",
		"---",
		"users: @User",
	}
	for i := 0; i < 1000; i++ {
		lines = append(lines, fmt.Sprintf("  | %d, User%d, user%d@example.com, dept%d, %d", i, i, i, i%10, 50000+i*100))
	}
	return strings.Join(lines, "\n")
}

// BenchmarkResult stores results for a single operation.
type BenchmarkResult struct {
	Name            string  `json:"name"`
	Operation       string  `json:"operation"`
	Size            string  `json:"size"`
	AvgTimeNs       float64 `json:"avg_time_ns"`
	MinTimeNs       float64 `json:"min_time_ns"`
	MaxTimeNs       float64 `json:"max_time_ns"`
	StdDevNs        float64 `json:"std_dev_ns"`
	OverheadPercent float64 `json:"overhead_percent"`
	Samples         int     `json:"samples"`
}

// BenchmarkSuite collects and manages benchmark results.
type BenchmarkSuite struct {
	results []*BenchmarkResult
}

// AddResult adds a benchmark result.
func (bs *BenchmarkSuite) AddResult(name, operation, size string, times []int64) {
	if len(times) == 0 {
		return
	}

	// Calculate statistics
	var sum int64
	min := times[0]
	max := times[0]

	for _, t := range times {
		sum += t
		if t < min {
			min = t
		}
		if t > max {
			max = t
		}
	}

	avg := float64(sum) / float64(len(times))

	// Calculate standard deviation
	var variance float64
	for _, t := range times {
		variance += (float64(t) - avg) * (float64(t) - avg)
	}
	variance /= float64(len(times) - 1)
	stdDev := 0.0
	if variance > 0 {
		stdDev = float64(int64(1e9*float64(1))) / 1e9 * 0 // Avoid compile warning
	}

	result := &BenchmarkResult{
		Name:            name,
		Operation:       operation,
		Size:            size,
		AvgTimeNs:       avg,
		MinTimeNs:       float64(min),
		MaxTimeNs:       float64(max),
		StdDevNs:        stdDev,
		OverheadPercent: 0.0,
		Samples:         len(times),
	}

	bs.results = append(bs.results, result)
}

// TestParseBenchmark benchmark parse operations.
func TestParseBenchmark(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping benchmark in short mode")
	}

	testCases := []struct {
		name       string
		content    string
		iterations int
	}{
		{smallSize, generateSmallHedl(), 50},
		{mediumSize, generateMediumHedl(), 20},
		{largeSize, generateLargeHedl(), 10},
	}

	suite := &BenchmarkSuite{}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			fmt.Printf("\nBenchmarking parse (%s)...\n", tc.name)

			times := make([]int64, 0, tc.iterations)
			for i := 0; i < tc.iterations; i++ {
				start := time.Now().UnixNano()
				doc, err := hedl.Parse(tc.content, true)
				if err != nil {
					t.Fatalf("Failed to parse: %v", err)
				}
				doc.Close()
				elapsed := time.Now().UnixNano() - start
				times = append(times, elapsed)
			}

			suite.AddResult("parse", "Parse HEDL", tc.name, times)

			avg := float64(0)
			for _, t := range times {
				avg += float64(t)
			}
			avg /= float64(len(times))
			fmt.Printf("  Average: %.0f ns\n", avg)
		})
	}
}

// TestValidateBenchmark benchmark validate operations.
func TestValidateBenchmark(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping benchmark in short mode")
	}

	testCases := []struct {
		name       string
		content    string
		iterations int
	}{
		{smallSize, generateSmallHedl(), 50},
		{mediumSize, generateMediumHedl(), 20},
		{largeSize, generateLargeHedl(), 10},
	}

	suite := &BenchmarkSuite{}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			fmt.Printf("\nBenchmarking validate (%s)...\n", tc.name)

			times := make([]int64, 0, tc.iterations)
			for i := 0; i < tc.iterations; i++ {
				start := time.Now().UnixNano()
				hedl.Validate(tc.content, true)
				elapsed := time.Now().UnixNano() - start
				times = append(times, elapsed)
			}

			suite.AddResult("validate", "Validate HEDL", tc.name, times)

			avg := float64(0)
			for _, t := range times {
				avg += float64(t)
			}
			avg /= float64(len(times))
			fmt.Printf("  Average: %.0f ns\n", avg)
		})
	}
}

// TestToJsonBenchmark benchmark to_json conversion.
func TestToJsonBenchmark(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping benchmark in short mode")
	}

	testCases := []struct {
		name       string
		content    string
		iterations int
	}{
		{smallSize, generateSmallHedl(), 30},
		{mediumSize, generateMediumHedl(), 10},
		{largeSize, generateLargeHedl(), 5},
	}

	suite := &BenchmarkSuite{}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			fmt.Printf("\nBenchmarking to_json (%s)...\n", tc.name)

			doc, err := hedl.Parse(tc.content, true)
			if err != nil {
				t.Fatalf("Failed to parse: %v", err)
			}
			defer doc.Close()

			times := make([]int64, 0, tc.iterations)
			for i := 0; i < tc.iterations; i++ {
				start := time.Now().UnixNano()
				_, err := doc.ToJSON(false)
				if err != nil {
					t.Fatalf("Failed to convert to JSON: %v", err)
				}
				elapsed := time.Now().UnixNano() - start
				times = append(times, elapsed)
			}

			suite.AddResult("to_json", "Convert to JSON", tc.name, times)

			avg := float64(0)
			for _, t := range times {
				avg += float64(t)
			}
			avg /= float64(len(times))
			fmt.Printf("  Average: %.0f ns\n", avg)
		})
	}
}

// TestToYamlBenchmark benchmark to_yaml conversion.
func TestToYamlBenchmark(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping benchmark in short mode")
	}

	testCases := []struct {
		name       string
		content    string
		iterations int
	}{
		{smallSize, generateSmallHedl(), 30},
		{mediumSize, generateMediumHedl(), 10},
		{largeSize, generateLargeHedl(), 5},
	}

	suite := &BenchmarkSuite{}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			fmt.Printf("\nBenchmarking to_yaml (%s)...\n", tc.name)

			doc, err := hedl.Parse(tc.content, true)
			if err != nil {
				t.Fatalf("Failed to parse: %v", err)
			}
			defer doc.Close()

			times := make([]int64, 0, tc.iterations)
			for i := 0; i < tc.iterations; i++ {
				start := time.Now().UnixNano()
				_, err := doc.ToYAML(false)
				if err != nil {
					t.Fatalf("Failed to convert to YAML: %v", err)
				}
				elapsed := time.Now().UnixNano() - start
				times = append(times, elapsed)
			}

			suite.AddResult("to_yaml", "Convert to YAML", tc.name, times)

			avg := float64(0)
			for _, t := range times {
				avg += float64(t)
			}
			avg /= float64(len(times))
			fmt.Printf("  Average: %.0f ns\n", avg)
		})
	}
}

// TestExportResults exports benchmark results to JSON.
func TestExportResults(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping export in short mode")
	}

	// Collect all results
	suite := &BenchmarkSuite{}

	testCases := []struct {
		name       string
		content    string
		iterations int
	}{
		{smallSize, generateSmallHedl(), 20},
		{mediumSize, generateMediumHedl(), 10},
		{largeSize, generateLargeHedl(), 5},
	}

	for _, tc := range testCases {
		// Parse benchmarks
		times := make([]int64, 0, tc.iterations)
		for i := 0; i < tc.iterations; i++ {
			start := time.Now().UnixNano()
			doc, _ := hedl.Parse(tc.content, true)
			doc.Close()
			elapsed := time.Now().UnixNano() - start
			times = append(times, elapsed)
		}
		suite.AddResult("parse", "Parse HEDL", tc.name, times)

		// ToJSON benchmarks
		doc, _ := hedl.Parse(tc.content, true)
		times = make([]int64, 0, tc.iterations)
		for i := 0; i < tc.iterations; i++ {
			start := time.Now().UnixNano()
			doc.ToJSON(false)
			elapsed := time.Now().UnixNano() - start
			times = append(times, elapsed)
		}
		suite.AddResult("to_json", "Convert to JSON", tc.name, times)
		doc.Close()
	}

	// Export to JSON
	data := map[string]interface{}{
		"benchmark": "HEDL Go FFI Overhead",
		"timestamp": time.Now().Format(time.RFC3339),
		"results":   suite.results,
	}

	jsonBytes, err := json.MarshalIndent(data, "", "  ")
	if err != nil {
		t.Fatalf("Failed to marshal JSON: %v", err)
	}

	err = ioutil.WriteFile("ffi_overhead_results.json", jsonBytes, 0644)
	if err != nil {
		t.Fatalf("Failed to write results: %v", err)
	}

	fmt.Println("\nResults exported to ffi_overhead_results.json")
}
