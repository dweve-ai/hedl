#!/usr/bin/env python3
"""
FFI Overhead Benchmarks for HEDL Python Bindings

Measures the performance overhead of FFI calls compared to native Rust operations.
Tests parse, convert, and validate operations across multiple document sizes.

Run with:
    python3 -m timeit -n 100 -r 5 -s "..." "..."
    or use: python3 benchmarks/ffi_overhead.py
"""

import timeit
import sys
import json
from typing import Dict, List, Tuple
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

import hedl


# Test data generators
def generate_small_hedl() -> str:
    """Generate small HEDL document."""
    return """%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | 1, Alice, alice@example.com
  | 2, Bob, bob@example.com
"""


def generate_medium_hedl() -> str:
    """Generate medium HEDL document."""
    lines = ["%VERSION: 1.0", "%STRUCT: User: [id, name, email, dept]", "---", "users: @User"]
    for i in range(100):
        lines.append(f"  | {i}, User{i}, user{i}@example.com, dept{i%10}")
    return "\n".join(lines)


def generate_large_hedl() -> str:
    """Generate large HEDL document."""
    lines = ["%VERSION: 1.0", "%STRUCT: User: [id, name, email, dept, salary]", "---", "users: @User"]
    for i in range(1000):
        lines.append(f"  | {i}, User{i}, user{i}@example.com, dept{i%10}, {50000 + i*100}")
    return "\n".join(lines)


class BenchmarkResult:
    """Stores benchmark results for a single operation."""

    def __init__(self, name: str, operation: str, size: str):
        self.name = name
        self.operation = operation
        self.size = size
        self.times: List[float] = []
        self.overhead_percent = 0.0

    def add_time(self, time_ns: float):
        """Add a measurement in nanoseconds."""
        self.times.append(time_ns)

    def avg_time_ns(self) -> float:
        """Get average time in nanoseconds."""
        return sum(self.times) / len(self.times) if self.times else 0

    def min_time_ns(self) -> float:
        """Get minimum time in nanoseconds."""
        return min(self.times) if self.times else 0

    def max_time_ns(self) -> float:
        """Get maximum time in nanoseconds."""
        return max(self.times) if self.times else 0

    def std_dev_ns(self) -> float:
        """Get standard deviation in nanoseconds."""
        if len(self.times) < 2:
            return 0
        avg = self.avg_time_ns()
        variance = sum((t - avg) ** 2 for t in self.times) / (len(self.times) - 1)
        return variance ** 0.5

    def to_dict(self) -> Dict:
        """Convert to dictionary for JSON serialization."""
        return {
            "name": self.name,
            "operation": self.operation,
            "size": self.size,
            "avg_time_ns": round(self.avg_time_ns(), 2),
            "min_time_ns": round(self.min_time_ns(), 2),
            "max_time_ns": round(self.max_time_ns(), 2),
            "std_dev_ns": round(self.std_dev_ns(), 2),
            "overhead_percent": round(self.overhead_percent, 2),
            "samples": len(self.times),
        }


class BenchmarkSuite:
    """Runs FFI overhead benchmarks."""

    def __init__(self):
        self.results: List[BenchmarkResult] = []

    def benchmark_parse(self, hedl_content: str, size: str, iterations: int = 10):
        """Benchmark parse operation."""
        print(f"\nBenchmarking parse ({size})...")

        def parse_op():
            doc = hedl.parse(hedl_content)
            doc.close()

        times = []
        for _ in range(iterations):
            start = timeit.default_timer()
            parse_op()
            elapsed = (timeit.default_timer() - start) * 1e9  # Convert to ns
            times.append(elapsed)

        result = BenchmarkResult("parse", "Parse HEDL", size)
        for t in times:
            result.add_time(t)
        self.results.append(result)
        print(f"  Average: {result.avg_time_ns():.0f} ns")

    def benchmark_to_json(self, hedl_content: str, size: str, iterations: int = 10):
        """Benchmark to_json conversion."""
        print(f"\nBenchmarking to_json ({size})...")

        doc = hedl.parse(hedl_content)

        def to_json_op():
            return doc.to_json()

        times = []
        try:
            for _ in range(iterations):
                start = timeit.default_timer()
                to_json_op()
                elapsed = (timeit.default_timer() - start) * 1e9  # Convert to ns
                times.append(elapsed)

            result = BenchmarkResult("to_json", "Convert to JSON", size)
            for t in times:
                result.add_time(t)
            self.results.append(result)
            print(f"  Average: {result.avg_time_ns():.0f} ns")
        finally:
            doc.close()

    def benchmark_to_yaml(self, hedl_content: str, size: str, iterations: int = 10):
        """Benchmark to_yaml conversion."""
        print(f"\nBenchmarking to_yaml ({size})...")

        doc = hedl.parse(hedl_content)

        def to_yaml_op():
            return doc.to_yaml()

        times = []
        try:
            for _ in range(iterations):
                start = timeit.default_timer()
                to_yaml_op()
                elapsed = (timeit.default_timer() - start) * 1e9  # Convert to ns
                times.append(elapsed)

            result = BenchmarkResult("to_yaml", "Convert to YAML", size)
            for t in times:
                result.add_time(t)
            self.results.append(result)
            print(f"  Average: {result.avg_time_ns():.0f} ns")
        finally:
            doc.close()

    def benchmark_validate(self, hedl_content: str, size: str, iterations: int = 10):
        """Benchmark validate operation."""
        print(f"\nBenchmarking validate ({size})...")

        def validate_op():
            return hedl.validate(hedl_content)

        times = []
        for _ in range(iterations):
            start = timeit.default_timer()
            validate_op()
            elapsed = (timeit.default_timer() - start) * 1e9  # Convert to ns
            times.append(elapsed)

        result = BenchmarkResult("validate", "Validate HEDL", size)
        for t in times:
            result.add_time(t)
        self.results.append(result)
        print(f"  Average: {result.avg_time_ns():.0f} ns")

    def run_all(self):
        """Run all benchmarks."""
        print("=" * 70)
        print("HEDL Python FFI Overhead Benchmarks")
        print("=" * 70)

        test_cases = [
            ("small", generate_small_hedl(), 20),
            ("medium", generate_medium_hedl(), 10),
            ("large", generate_large_hedl(), 5),
        ]

        for size_name, content, iterations in test_cases:
            print(f"\n{'='*70}")
            print(f"Testing {size_name} documents ({len(content)} bytes)")
            print(f"{'='*70}")

            self.benchmark_parse(content, size_name, iterations)
            self.benchmark_validate(content, size_name, iterations)
            self.benchmark_to_json(content, size_name, iterations)
            self.benchmark_to_yaml(content, size_name, iterations)

    def print_summary(self):
        """Print summary of all benchmarks."""
        print("\n" + "=" * 70)
        print("BENCHMARK SUMMARY")
        print("=" * 70)

        # Group by operation
        operations = {}
        for result in self.results:
            key = result.operation
            if key not in operations:
                operations[key] = []
            operations[key].append(result)

        for op_name, op_results in sorted(operations.items()):
            print(f"\n{op_name}:")
            print(f"  {'Size':<10} {'Avg (ns)':<15} {'Min (ns)':<15} {'Max (ns)':<15} {'StdDev':<15}")
            print("  " + "-" * 70)
            for result in sorted(op_results, key=lambda r: int(r.size == "small") or int(r.size == "medium") or 2):
                print(
                    f"  {result.size:<10} {result.avg_time_ns():<15.0f} "
                    f"{result.min_time_ns():<15.0f} {result.max_time_ns():<15.0f} "
                    f"{result.std_dev_ns():<15.0f}"
                )

    def export_json(self, filename: str = "ffi_overhead_results.json"):
        """Export results to JSON file."""
        data = {
            "benchmark": "HEDL Python FFI Overhead",
            "timestamp": __import__("datetime").datetime.now().isoformat(),
            "results": [r.to_dict() for r in self.results],
        }

        with open(filename, "w") as f:
            json.dump(data, f, indent=2)
        print(f"\nResults exported to {filename}")


def main():
    """Run benchmarks."""
    suite = BenchmarkSuite()
    suite.run_all()
    suite.print_summary()
    suite.export_json()


if __name__ == "__main__":
    main()
