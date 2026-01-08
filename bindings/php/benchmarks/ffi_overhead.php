<?php
/**
 * FFI Overhead Benchmarks for HEDL PHP Bindings
 *
 * Measures the performance overhead of FFI calls compared to native Rust operations.
 * Tests parse, convert, and validate operations across multiple document sizes.
 *
 * Requirements:
 * - PHP 8.0+ with FFI extension enabled
 * - libhedl_ffi shared library
 *
 * Run with:
 *   php benchmarks/ffi_overhead.php
 *   or: time php benchmarks/ffi_overhead.php
 */

namespace Dweve\Hedl\Benchmarks;

require_once __DIR__ . '/../Hedl.php';

use Dweve\Hedl\Hedl;

// Test data generators
function generateSmallHedl(): string
{
    return <<<'HEDL'
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | 1, Alice, alice@example.com
  | 2, Bob, bob@example.com
HEDL;
}

function generateMediumHedl(): string
{
    $lines = [
        '%VERSION: 1.0',
        '%STRUCT: User: [id, name, email, dept]',
        '---',
        'users: @User'
    ];
    for ($i = 0; $i < 100; $i++) {
        $lines[] = "  | $i, User$i, user$i@example.com, dept" . ($i % 10);
    }
    return implode("\n", $lines);
}

function generateLargeHedl(): string
{
    $lines = [
        '%VERSION: 1.0',
        '%STRUCT: User: [id, name, email, dept, salary]',
        '---',
        'users: @User'
    ];
    for ($i = 0; $i < 1000; $i++) {
        $salary = 50000 + $i * 100;
        $lines[] = "  | $i, User$i, user$i@example.com, dept" . ($i % 10) . ", $salary";
    }
    return implode("\n", $lines);
}

/**
 * Stores benchmark results for a single operation
 */
class BenchmarkResult
{
    private string $name;
    private string $operation;
    private string $size;
    private array $times = [];
    private float $overheadPercent = 0.0;

    public function __construct(string $name, string $operation, string $size)
    {
        $this->name = $name;
        $this->operation = $operation;
        $this->size = $size;
    }

    public function addTime(float $timeNs): void
    {
        $this->times[] = $timeNs;
    }

    public function avgTimeNs(): float
    {
        return empty($this->times) ? 0.0 : array_sum($this->times) / count($this->times);
    }

    public function minTimeNs(): float
    {
        return empty($this->times) ? 0.0 : min($this->times);
    }

    public function maxTimeNs(): float
    {
        return empty($this->times) ? 0.0 : max($this->times);
    }

    public function stdDevNs(): float
    {
        if (count($this->times) < 2) {
            return 0.0;
        }
        $avg = $this->avgTimeNs();
        $variance = array_sum(array_map(fn($t) => pow($t - $avg, 2), $this->times)) / (count($this->times) - 1);
        return sqrt($variance);
    }

    public function toArray(): array
    {
        return [
            'name' => $this->name,
            'operation' => $this->operation,
            'size' => $this->size,
            'avg_time_ns' => round($this->avgTimeNs(), 2),
            'min_time_ns' => round($this->minTimeNs(), 2),
            'max_time_ns' => round($this->maxTimeNs(), 2),
            'std_dev_ns' => round($this->stdDevNs(), 2),
            'overhead_percent' => round($this->overheadPercent, 2),
            'samples' => count($this->times),
        ];
    }

    public function getName(): string
    {
        return $this->name;
    }

    public function getOperation(): string
    {
        return $this->operation;
    }

    public function getSize(): string
    {
        return $this->size;
    }
}

/**
 * Runs FFI overhead benchmarks
 */
class BenchmarkSuite
{
    private array $results = [];

    public function benchmarkParse(string $content, string $size, int $iterations = 10): void
    {
        printf("\nBenchmarking parse (%s)...\n", $size);

        $times = [];
        for ($i = 0; $i < $iterations; $i++) {
            $start = hrtime(true);
            $doc = Hedl::parse($content);
            $doc->close();
            $elapsed = hrtime(true) - $start;
            $times[] = $elapsed;
        }

        $result = new BenchmarkResult('parse', 'Parse HEDL', $size);
        foreach ($times as $t) {
            $result->addTime($t);
        }
        $this->results[] = $result;
        printf("  Average: %.0f ns\n", $result->avgTimeNs());
    }

    public function benchmarkToJson(string $content, string $size, int $iterations = 10): void
    {
        printf("\nBenchmarking to_json (%s)...\n", $size);

        $doc = Hedl::parse($content);

        $times = [];
        for ($i = 0; $i < $iterations; $i++) {
            $start = hrtime(true);
            $doc->toJson();
            $elapsed = hrtime(true) - $start;
            $times[] = $elapsed;
        }

        $doc->close();

        $result = new BenchmarkResult('to_json', 'Convert to JSON', $size);
        foreach ($times as $t) {
            $result->addTime($t);
        }
        $this->results[] = $result;
        printf("  Average: %.0f ns\n", $result->avgTimeNs());
    }

    public function benchmarkToYaml(string $content, string $size, int $iterations = 10): void
    {
        printf("\nBenchmarking to_yaml (%s)...\n", $size);

        $doc = Hedl::parse($content);

        $times = [];
        for ($i = 0; $i < $iterations; $i++) {
            $start = hrtime(true);
            $doc->toYaml();
            $elapsed = hrtime(true) - $start;
            $times[] = $elapsed;
        }

        $doc->close();

        $result = new BenchmarkResult('to_yaml', 'Convert to YAML', $size);
        foreach ($times as $t) {
            $result->addTime($t);
        }
        $this->results[] = $result;
        printf("  Average: %.0f ns\n", $result->avgTimeNs());
    }

    public function benchmarkValidate(string $content, string $size, int $iterations = 10): void
    {
        printf("\nBenchmarking validate (%s)...\n", $size);

        $times = [];
        for ($i = 0; $i < $iterations; $i++) {
            $start = hrtime(true);
            Hedl::validate($content);
            $elapsed = hrtime(true) - $start;
            $times[] = $elapsed;
        }

        $result = new BenchmarkResult('validate', 'Validate HEDL', $size);
        foreach ($times as $t) {
            $result->addTime($t);
        }
        $this->results[] = $result;
        printf("  Average: %.0f ns\n", $result->avgTimeNs());
    }

    public function runAll(): void
    {
        echo str_repeat('=', 70) . "\n";
        echo "HEDL PHP FFI Overhead Benchmarks\n";
        echo str_repeat('=', 70) . "\n";

        $testCases = [
            ['small', generateSmallHedl(), 20],
            ['medium', generateMediumHedl(), 10],
            ['large', generateLargeHedl(), 5],
        ];

        foreach ($testCases as [$sizeName, $content, $iterations]) {
            printf("\n%s\n", str_repeat('=', 70));
            printf("Testing %s documents (%d bytes)\n", $sizeName, strlen($content));
            printf("%s\n", str_repeat('=', 70));

            $this->benchmarkParse($content, $sizeName, $iterations);
            $this->benchmarkValidate($content, $sizeName, $iterations);
            $this->benchmarkToJson($content, $sizeName, $iterations);
            $this->benchmarkToYaml($content, $sizeName, $iterations);
        }
    }

    public function printSummary(): void
    {
        echo "\n" . str_repeat('=', 70) . "\n";
        echo "BENCHMARK SUMMARY\n";
        echo str_repeat('=', 70) . "\n";

        $operations = [];
        foreach ($this->results as $result) {
            $op = $result->getOperation();
            if (!isset($operations[$op])) {
                $operations[$op] = [];
            }
            $operations[$op][] = $result;
        }

        $sizeOrder = ['small' => 0, 'medium' => 1, 'large' => 2];

        ksort($operations);
        foreach ($operations as $opName => $opResults) {
            printf("\n%s:\n", $opName);
            printf("  %-10s %-15s %-15s %-15s %-15s\n", 'Size', 'Avg (ns)', 'Min (ns)', 'Max (ns)', 'StdDev');
            echo "  " . str_repeat('-', 70) . "\n";

            usort($opResults, fn($a, $b) => ($sizeOrder[$a->getSize()] ?? 999) <=> ($sizeOrder[$b->getSize()] ?? 999));

            foreach ($opResults as $result) {
                printf(
                    "  %-10s %-15.0f %-15.0f %-15.0f %-15.0f\n",
                    $result->getSize(),
                    $result->avgTimeNs(),
                    $result->minTimeNs(),
                    $result->maxTimeNs(),
                    $result->stdDevNs()
                );
            }
        }
    }

    public function exportJson(string $filename = 'ffi_overhead_results.json'): void
    {
        $data = [
            'benchmark' => 'HEDL PHP FFI Overhead',
            'timestamp' => (new \DateTime())->format(\DateTime::RFC3339),
            'results' => array_map(fn($r) => $r->toArray(), $this->results),
        ];

        file_put_contents($filename, json_encode($data, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES));
        printf("\nResults exported to %s\n", $filename);
    }
}

function main(): void
{
    $suite = new BenchmarkSuite();
    $suite->runAll();
    $suite->printSummary();
    $suite->exportJson();
}

main();
