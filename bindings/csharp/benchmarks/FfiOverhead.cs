// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using Dweve.Hedl;

namespace Dweve.Hedl.Benchmarks
{
    /// <summary>
    /// FFI Overhead Benchmarks for HEDL C# Bindings
    ///
    /// Measures the performance overhead of FFI calls compared to native Rust operations.
    /// Tests parse, convert, and validate operations across multiple document sizes.
    ///
    /// Run with:
    ///   dotnet run --project benchmarks/FfiOverhead.csproj
    /// </summary>
    class Program
    {
        private const string SmallSize = "small";
        private const string MediumSize = "medium";
        private const string LargeSize = "large";

        static void Main(string[] args)
        {
            var suite = new BenchmarkSuite();
            suite.RunAll();
            suite.PrintSummary();
            suite.ExportJson();
        }

        static string GenerateSmallHedl()
        {
            return @"%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | 1, Alice, alice@example.com
  | 2, Bob, bob@example.com";
        }

        static string GenerateMediumHedl()
        {
            var sb = new StringBuilder();
            sb.AppendLine("%VERSION: 1.0");
            sb.AppendLine("%STRUCT: User: [id, name, email, dept]");
            sb.AppendLine("---");
            sb.AppendLine("users: @User");
            for (int i = 0; i < 100; i++)
            {
                sb.AppendLine($"  | {i}, User{i}, user{i}@example.com, dept{i % 10}");
            }
            return sb.ToString().TrimEnd();
        }

        static string GenerateLargeHedl()
        {
            var sb = new StringBuilder();
            sb.AppendLine("%VERSION: 1.0");
            sb.AppendLine("%STRUCT: User: [id, name, email, dept, salary]");
            sb.AppendLine("---");
            sb.AppendLine("users: @User");
            for (int i = 0; i < 1000; i++)
            {
                sb.AppendLine($"  | {i}, User{i}, user{i}@example.com, dept{i % 10}, {50000 + i * 100}");
            }
            return sb.ToString().TrimEnd();
        }

        /// <summary>
        /// Stores benchmark results for a single operation
        /// </summary>
        class BenchmarkResult
        {
            [JsonPropertyName("name")]
            public string Name { get; set; }

            [JsonPropertyName("operation")]
            public string Operation { get; set; }

            [JsonPropertyName("size")]
            public string Size { get; set; }

            [JsonPropertyName("avg_time_ns")]
            public double AvgTimeNs { get; set; }

            [JsonPropertyName("min_time_ns")]
            public double MinTimeNs { get; set; }

            [JsonPropertyName("max_time_ns")]
            public double MaxTimeNs { get; set; }

            [JsonPropertyName("std_dev_ns")]
            public double StdDevNs { get; set; }

            [JsonPropertyName("overhead_percent")]
            public double OverheadPercent { get; set; }

            [JsonPropertyName("samples")]
            public int Samples { get; set; }

            private List<long> _times = new();

            public BenchmarkResult(string name, string operation, string size)
            {
                Name = name;
                Operation = operation;
                Size = size;
                OverheadPercent = 0.0;
            }

            public void AddTime(long timeNs)
            {
                _times.Add(timeNs);
            }

            public void CalculateStats()
            {
                if (_times.Count == 0)
                    return;

                AvgTimeNs = _times.Average();
                MinTimeNs = _times.Min();
                MaxTimeNs = _times.Max();
                Samples = _times.Count;

                if (_times.Count >= 2)
                {
                    var variance = _times.Sum(t => Math.Pow(t - AvgTimeNs, 2)) / (_times.Count - 1);
                    StdDevNs = Math.Sqrt(variance);
                }
            }
        }

        /// <summary>
        /// Runs FFI overhead benchmarks
        /// </summary>
        class BenchmarkSuite
        {
            private List<BenchmarkResult> _results = new();

            public void BenchmarkParse(string content, string size, int iterations = 10)
            {
                Console.WriteLine($"\nBenchmarking parse ({size})...");

                var times = new List<long>();
                for (int i = 0; i < iterations; i++)
                {
                    var sw = Stopwatch.StartNew();
                    using var doc = Hedl.Parse(content);
                    sw.Stop();
                    times.Add(sw.Elapsed.Ticks * 100); // Convert to nanoseconds
                }

                var result = new BenchmarkResult("parse", "Parse HEDL", size);
                foreach (var t in times)
                {
                    result.AddTime(t);
                }
                result.CalculateStats();
                _results.Add(result);
                Console.WriteLine($"  Average: {result.AvgTimeNs:F0} ns");
            }

            public void BenchmarkToJson(string content, string size, int iterations = 10)
            {
                Console.WriteLine($"\nBenchmarking to_json ({size})...");

                using var doc = Hedl.Parse(content);

                var times = new List<long>();
                for (int i = 0; i < iterations; i++)
                {
                    var sw = Stopwatch.StartNew();
                    doc.ToJson();
                    sw.Stop();
                    times.Add(sw.Elapsed.Ticks * 100); // Convert to nanoseconds
                }

                var result = new BenchmarkResult("to_json", "Convert to JSON", size);
                foreach (var t in times)
                {
                    result.AddTime(t);
                }
                result.CalculateStats();
                _results.Add(result);
                Console.WriteLine($"  Average: {result.AvgTimeNs:F0} ns");
            }

            public void BenchmarkToYaml(string content, string size, int iterations = 10)
            {
                Console.WriteLine($"\nBenchmarking to_yaml ({size})...");

                using var doc = Hedl.Parse(content);

                var times = new List<long>();
                for (int i = 0; i < iterations; i++)
                {
                    var sw = Stopwatch.StartNew();
                    doc.ToYaml();
                    sw.Stop();
                    times.Add(sw.Elapsed.Ticks * 100); // Convert to nanoseconds
                }

                var result = new BenchmarkResult("to_yaml", "Convert to YAML", size);
                foreach (var t in times)
                {
                    result.AddTime(t);
                }
                result.CalculateStats();
                _results.Add(result);
                Console.WriteLine($"  Average: {result.AvgTimeNs:F0} ns");
            }

            public void BenchmarkValidate(string content, string size, int iterations = 10)
            {
                Console.WriteLine($"\nBenchmarking validate ({size})...");

                var times = new List<long>();
                for (int i = 0; i < iterations; i++)
                {
                    var sw = Stopwatch.StartNew();
                    Hedl.Validate(content);
                    sw.Stop();
                    times.Add(sw.Elapsed.Ticks * 100); // Convert to nanoseconds
                }

                var result = new BenchmarkResult("validate", "Validate HEDL", size);
                foreach (var t in times)
                {
                    result.AddTime(t);
                }
                result.CalculateStats();
                _results.Add(result);
                Console.WriteLine($"  Average: {result.AvgTimeNs:F0} ns");
            }

            public void RunAll()
            {
                Console.WriteLine(new string('=', 70));
                Console.WriteLine("HEDL C# FFI Overhead Benchmarks");
                Console.WriteLine(new string('=', 70));

                var testCases = new[]
                {
                    (SmallSize, GenerateSmallHedl(), 20),
                    (MediumSize, GenerateMediumHedl(), 10),
                    (LargeSize, GenerateLargeHedl(), 5)
                };

                foreach (var (sizeName, content, iterations) in testCases)
                {
                    Console.WriteLine($"\n{new string('=', 70)}");
                    Console.WriteLine($"Testing {sizeName} documents ({content.Length} bytes)");
                    Console.WriteLine(new string('=', 70));

                    BenchmarkParse(content, sizeName, iterations);
                    BenchmarkValidate(content, sizeName, iterations);
                    BenchmarkToJson(content, sizeName, iterations);
                    BenchmarkToYaml(content, sizeName, iterations);
                }
            }

            public void PrintSummary()
            {
                Console.WriteLine("\n" + new string('=', 70));
                Console.WriteLine("BENCHMARK SUMMARY");
                Console.WriteLine(new string('=', 70));

                var operations = _results.GroupBy(r => r.Operation)
                    .OrderBy(g => g.Key)
                    .ToList();

                foreach (var opGroup in operations)
                {
                    Console.WriteLine($"\n{opGroup.Key}:");
                    Console.WriteLine($"  {"Size",-10} {"Avg (ns)",-15} {"Min (ns)",-15} {"Max (ns)",-15} {"StdDev",-15}");
                    Console.WriteLine("  " + new string('-', 70));

                    var sizeOrder = new[] { SmallSize, MediumSize, LargeSize };
                    var sorted = opGroup.OrderBy(r => Array.IndexOf(sizeOrder, r.Size));

                    foreach (var result in sorted)
                    {
                        Console.WriteLine(
                            $"  {result.Size,-10} {result.AvgTimeNs,15:F0} " +
                            $"{result.MinTimeNs,15:F0} {result.MaxTimeNs,15:F0} " +
                            $"{result.StdDevNs,15:F0}"
                        );
                    }
                }
            }

            public void ExportJson(string filename = "ffi_overhead_results.json")
            {
                var data = new
                {
                    benchmark = "HEDL C# FFI Overhead",
                    timestamp = DateTime.UtcNow.ToString("O"),
                    results = _results
                };

                var options = new JsonSerializerOptions
                {
                    WriteIndented = true,
                    PropertyNamingPolicy = JsonNamingPolicy.CamelCase
                };

                var json = JsonSerializer.Serialize(data, options);
                File.WriteAllText(filename, json);
                Console.WriteLine($"\nResults exported to {filename}");
            }
        }
    }
}
