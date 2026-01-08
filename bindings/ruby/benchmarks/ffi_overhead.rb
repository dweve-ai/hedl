#!/usr/bin/env ruby
# frozen_string_literal: true

##
# FFI Overhead Benchmarks for HEDL Ruby Bindings
#
# Measures the performance overhead of FFI calls compared to native Rust operations.
# Tests parse, convert, and validate operations across multiple document sizes.
#
# Run with:
#   ruby benchmarks/ffi_overhead.rb
#   or with: time ruby benchmarks/ffi_overhead.rb

require 'benchmark'
require 'json'
require 'time'

# Add lib directory to load path
$LOAD_PATH.unshift(File.expand_path('../lib', __dir__))
require 'hedl'

# Test data generators
def generate_small_hedl
  <<~HEDL
    %VERSION: 1.0
    %STRUCT: User: [id, name, email]
    ---
    users: @User
      | 1, Alice, alice@example.com
      | 2, Bob, bob@example.com
  HEDL
end

def generate_medium_hedl
  lines = [
    '%VERSION: 1.0',
    '%STRUCT: User: [id, name, email, dept]',
    '---',
    'users: @User'
  ]
  100.times { |i| lines << "  | #{i}, User#{i}, user#{i}@example.com, dept#{i % 10}" }
  lines.join("\n")
end

def generate_large_hedl
  lines = [
    '%VERSION: 1.0',
    '%STRUCT: User: [id, name, email, dept, salary]',
    '---',
    'users: @User'
  ]
  1000.times { |i| lines << "  | #{i}, User#{i}, user#{i}@example.com, dept#{i % 10}, #{50_000 + i * 100}" }
  lines.join("\n")
end

##
# Stores benchmark results for a single operation
class BenchmarkResult
  attr_reader :name, :operation, :size
  attr_accessor :overhead_percent

  def initialize(name, operation, size)
    @name = name
    @operation = operation
    @size = size
    @times = []
    @overhead_percent = 0.0
  end

  def add_time(time_ns)
    @times << time_ns
  end

  def avg_time_ns
    return 0 if @times.empty?

    @times.sum.to_f / @times.length
  end

  def min_time_ns
    @times.min || 0
  end

  def max_time_ns
    @times.max || 0
  end

  def std_dev_ns
    return 0 if @times.length < 2

    mean = avg_time_ns
    variance = @times.map { |t| (t - mean) ** 2 }.sum / (@times.length - 1).to_f
    Math.sqrt(variance)
  end

  def to_hash
    {
      name: @name,
      operation: @operation,
      size: @size,
      avg_time_ns: avg_time_ns.round(2),
      min_time_ns: min_time_ns.round(2),
      max_time_ns: max_time_ns.round(2),
      std_dev_ns: std_dev_ns.round(2),
      overhead_percent: @overhead_percent.round(2),
      samples: @times.length
    }
  end
end

##
# Runs FFI overhead benchmarks
class BenchmarkSuite
  def initialize
    @results = []
  end

  def benchmark_parse(content, size, iterations = 10)
    puts "\nBenchmarking parse (#{size})..."

    times = []
    iterations.times do
      start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      doc = Hedl.parse(content)
      doc.close
      elapsed_ns = ((Process.clock_gettime(Process::CLOCK_MONOTONIC) - start) * 1e9).to_i
      times << elapsed_ns
    end

    result = BenchmarkResult.new('parse', 'Parse HEDL', size)
    times.each { |t| result.add_time(t) }
    @results << result
    puts "  Average: #{result.avg_time_ns.to_i} ns"
  end

  def benchmark_to_json(content, size, iterations = 10)
    puts "\nBenchmarking to_json (#{size})..."

    doc = Hedl.parse(content)

    times = []
    iterations.times do
      start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      doc.to_json
      elapsed_ns = ((Process.clock_gettime(Process::CLOCK_MONOTONIC) - start) * 1e9).to_i
      times << elapsed_ns
    end

    doc.close

    result = BenchmarkResult.new('to_json', 'Convert to JSON', size)
    times.each { |t| result.add_time(t) }
    @results << result
    puts "  Average: #{result.avg_time_ns.to_i} ns"
  end

  def benchmark_to_yaml(content, size, iterations = 10)
    puts "\nBenchmarking to_yaml (#{size})..."

    doc = Hedl.parse(content)

    times = []
    iterations.times do
      start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      doc.to_yaml
      elapsed_ns = ((Process.clock_gettime(Process::CLOCK_MONOTONIC) - start) * 1e9).to_i
      times << elapsed_ns
    end

    doc.close

    result = BenchmarkResult.new('to_yaml', 'Convert to YAML', size)
    times.each { |t| result.add_time(t) }
    @results << result
    puts "  Average: #{result.avg_time_ns.to_i} ns"
  end

  def benchmark_validate(content, size, iterations = 10)
    puts "\nBenchmarking validate (#{size})..."

    times = []
    iterations.times do
      start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      Hedl.validate(content)
      elapsed_ns = ((Process.clock_gettime(Process::CLOCK_MONOTONIC) - start) * 1e9).to_i
      times << elapsed_ns
    end

    result = BenchmarkResult.new('validate', 'Validate HEDL', size)
    times.each { |t| result.add_time(t) }
    @results << result
    puts "  Average: #{result.avg_time_ns.to_i} ns"
  end

  def run_all
    puts '=' * 70
    puts 'HEDL Ruby FFI Overhead Benchmarks'
    puts '=' * 70

    test_cases = [
      ['small', generate_small_hedl, 20],
      ['medium', generate_medium_hedl, 10],
      ['large', generate_large_hedl, 5]
    ]

    test_cases.each do |size_name, content, iterations|
      puts "\n#{'=' * 70}"
      puts "Testing #{size_name} documents (#{content.length} bytes)"
      puts "#{'=' * 70}"

      benchmark_parse(content, size_name, iterations)
      benchmark_validate(content, size_name, iterations)
      benchmark_to_json(content, size_name, iterations)
      benchmark_to_yaml(content, size_name, iterations)
    end
  end

  def print_summary
    puts "\n" + '=' * 70
    puts 'BENCHMARK SUMMARY'
    puts '=' * 70

    operations = {}
    @results.each do |result|
      operations[result.operation] ||= []
      operations[result.operation] << result
    end

    operations.sort.each do |op_name, op_results|
      puts "\n#{op_name}:"
      puts "  #{'Size':<10} #{'Avg (ns)':<15} #{'Min (ns)':<15} #{'Max (ns)':<15} #{'StdDev':<15}"
      puts '  ' + '-' * 70
      op_results.sort_by { |r| ['small', 'medium', 'large'].index(r.size) || 999 }.each do |result|
        puts "  #{result.size:<10} #{result.avg_time_ns.to_i.to_s.ljust(15)} " \
             "#{result.min_time_ns.to_i.to_s.ljust(15)} #{result.max_time_ns.to_i.to_s.ljust(15)} " \
             "#{result.std_dev_ns.to_i.to_s.ljust(15)}"
      end
    end
  end

  def export_json(filename = 'ffi_overhead_results.json')
    data = {
      benchmark: 'HEDL Ruby FFI Overhead',
      timestamp: Time.now.iso8601,
      results: @results.map(&:to_hash)
    }

    File.write(filename, JSON.pretty_generate(data))
    puts "\nResults exported to #{filename}"
  end
end

def main
  suite = BenchmarkSuite.new
  suite.run_all
  suite.print_summary
  suite.export_json
end

main if __FILE__ == $PROGRAM_NAME
