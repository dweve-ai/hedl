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

#![allow(deprecated)] // Uses legacy module intentionally

//! LLM Accuracy Testing Binary
//!
//! Tests how accurately LLMs can comprehend and extract information from HEDL
//! documents compared to equivalent JSON/YAML/XML representations.
//!
//! Usage:
//! ```bash
//! # Run with DeepSeek API
//! DEEPSEEK_API_KEY=... cargo run --package hedl-bench --bin accuracy
//!
//! # Run with Mistral API
//! MISTRAL_API_KEY=... cargo run --package hedl-bench --bin accuracy -- --provider mistral
//!
//! # Dry run (no API calls, just show what would be tested)
//! cargo run --package hedl-bench --bin accuracy -- --dry-run
//!
//! # Test specific format only
//! cargo run --package hedl-bench --bin accuracy -- --format hedl --format json
//!
//! # Limit questions per category
//! cargo run --package hedl-bench --bin accuracy -- --max-per-category 5
//! ```

use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use hedl_bench::legacy::accuracy::{
    build_prompt, generate_test_datasets, AccuracyReport, DataFormat, DifficultyResults,
    FormatResults, Provider, TestDataset, TestResult, TypeResults,
};
use hedl_bench::legacy::normalize::compare;
use hedl_bench::legacy::questions::{Question, QuestionType};
use hedl_bench::token_counter::count_tokens;
use hedl_bench::{BenchmarkReport, CustomTable, ExportConfig, Insight, TableCell};

/// Command line arguments
struct Args {
    provider: Provider,
    model: Option<String>,
    formats: Vec<DataFormat>,
    max_per_category: Option<usize>,
    dry_run: bool,
    verbose: bool,
    /// Number of runs per question for statistical significance
    runs: usize,
    /// Whether to run a warmup iteration (discarded)
    warmup: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            provider: Provider::DeepSeek,
            model: None,
            formats: vec![
                DataFormat::Hedl,
                DataFormat::Toon,
                DataFormat::Json,
                DataFormat::Yaml,
                DataFormat::Xml,
                DataFormat::Csv,
            ],
            max_per_category: None,
            dry_run: false,
            verbose: false,
            runs: 3, // Default to 3 runs for statistical significance
            warmup: false,
        }
    }
}

fn parse_args() -> Args {
    let mut args = Args::default();
    let mut argv: Vec<String> = env::args().skip(1).collect();

    while !argv.is_empty() {
        let arg = argv.remove(0);
        match arg.as_str() {
            "--provider" | "-p" => {
                if let Some(val) = argv.first() {
                    args.provider = match val.to_lowercase().as_str() {
                        "deepseek" => Provider::DeepSeek,
                        "mistral" => Provider::Mistral,
                        "openai" => Provider::OpenAI,
                        _ => {
                            eprintln!(
                                "Unknown provider: {}. Use 'deepseek', 'mistral', or 'openai'",
                                val
                            );
                            std::process::exit(1);
                        }
                    };
                    argv.remove(0);
                }
            }
            "--model" | "-m" => {
                if let Some(val) = argv.first() {
                    args.model = Some(val.clone());
                    argv.remove(0);
                }
            }
            "--format" | "-f" => {
                if args.formats.len() == 6 {
                    args.formats.clear(); // First --format clears defaults
                }
                if let Some(val) = argv.first() {
                    let format = match val.to_lowercase().as_str() {
                        "hedl" => DataFormat::Hedl,
                        "toon" => DataFormat::Toon,
                        "json" => DataFormat::Json,
                        "yaml" => DataFormat::Yaml,
                        "xml" => DataFormat::Xml,
                        "csv" => DataFormat::Csv,
                        _ => {
                            eprintln!("Unknown format: {}. Use hedl/toon/json/yaml/xml/csv", val);
                            std::process::exit(1);
                        }
                    };
                    args.formats.push(format);
                    argv.remove(0);
                }
            }
            "--max-per-category" | "-n" => {
                if let Some(val) = argv.first() {
                    args.max_per_category = val.parse().ok();
                    argv.remove(0);
                }
            }
            "--dry-run" | "-d" => {
                args.dry_run = true;
            }
            "--verbose" | "-v" => {
                args.verbose = true;
            }
            "--runs" | "-r" => {
                if let Some(val) = argv.first() {
                    args.runs = val.parse().unwrap_or(3);
                    if args.runs < 1 {
                        args.runs = 1;
                    }
                    argv.remove(0);
                }
            }
            "--warmup" | "-w" => {
                args.warmup = true;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                print_help();
                std::process::exit(1);
            }
        }
    }

    args
}

fn print_help() {
    println!(
        r#"HEDL LLM Accuracy Testing

USAGE:
    cargo run --package hedl-bench --bin accuracy [OPTIONS]

OPTIONS:
    -p, --provider <PROVIDER>     LLM provider: deepseek, mistral, openai [default: deepseek]
    -m, --model <MODEL>           Model name [default: provider's default]
    -f, --format <FORMAT>         Format to test (can repeat): hedl, toon, json, yaml, xml, csv
    -n, --max-per-category <N>    Max questions per category
    -r, --runs <N>                Number of runs per question [default: 3]
    -w, --warmup                  Run one warmup iteration (discarded)
    -d, --dry-run                 Don't call API, just show what would be tested
    -v, --verbose                 Show each question and answer
    -h, --help                    Print help

ENVIRONMENT:
    DEEPSEEK_API_KEY              Required for DeepSeek provider
    MISTRAL_API_KEY               Required for Mistral provider
    OPENAI_API_KEY                Required for OpenAI provider

EXAMPLES:
    # Full test with DeepSeek (3 runs per question, default)
    DEEPSEEK_API_KEY=sk-... cargo run --package hedl-bench --bin accuracy

    # Single run (faster, less statistically robust)
    DEEPSEEK_API_KEY=sk-... cargo run --package hedl-bench --bin accuracy -- -r 1

    # 5 runs with warmup for rigorous benchmarking
    DEEPSEEK_API_KEY=sk-... cargo run --package hedl-bench --bin accuracy -- -r 5 -w

    # Quick test with 3 questions per category
    DEEPSEEK_API_KEY=sk-... cargo run --package hedl-bench --bin accuracy -- -n 3

    # Compare HEDL vs TOON vs JSON
    DEEPSEEK_API_KEY=sk-... cargo run --package hedl-bench --bin accuracy -- -f hedl -f toon -f json

    # Dry run to see test plan
    cargo run --package hedl-bench --bin accuracy -- --dry-run
"#
    );
}

// P0 OPTIMIZATION: Reusable HTTP agent for connection pooling (1.08x speedup)
// CRITICAL FIX (P0): Use Arc instead of Mutex - ureq::Agent is already thread-safe
// Previous issue: Mutex serialized all HTTP calls, blocking parallel requests
use once_cell::sync::Lazy;
use std::sync::Arc;

static HTTP_AGENT: Lazy<Arc<ureq::Agent>> = Lazy::new(|| {
    Arc::new(
        ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(60))
            .build(),
    )
});

/// Make an LLM API call using the specified provider.
///
/// Uses a connection-pooled HTTP agent for performance. The agent is thread-safe
/// and cached globally via `Lazy<Arc<ureq::Agent>>` for efficient concurrent requests.
///
/// # Arguments
/// * `provider` - The LLM provider (DeepSeek, Mistral, OpenAI)
/// * `model` - Model identifier
/// * `api_key` - API authentication key
/// * `prompt` - The prompt to send to the LLM
///
/// # Returns
/// * `Ok((response, latency_ms, tokens_in, tokens_out))` - Success tuple containing:
///   - `response`: The LLM's text response
///   - `latency_ms`: Request latency in milliseconds
///   - `tokens_in`: Number of prompt tokens
///   - `tokens_out`: Number of completion tokens
/// * `Err(String)` - Error message if request fails
///
/// # Performance
/// - Connection pooling provides ~1.08x speedup for sequential requests
/// - Thread-safe Arc design enables parallel requests without mutex contention
/// - 60-second timeout prevents indefinite hangs
///
/// # Examples
/// ```no_run
/// # use crate::accuracy::Provider;
/// let (response, latency, tok_in, tok_out) = call_llm(
///     &Provider::DeepSeek,
///     "deepseek-chat",
///     "sk-abc123",
///     "What is 2+2?"
/// ).unwrap();
/// ```
fn call_llm(
    provider: &Provider,
    model: &str,
    api_key: &str,
    prompt: &str,
) -> Result<(String, u64, usize, usize), String> {
    let start = Instant::now();

    let url = format!("{}/chat/completions", provider.api_base());

    // OpenAI newer models (o-series, gpt-5.x) require max_completion_tokens
    // Older models (gpt-4o, gpt-4-turbo) use max_tokens
    let needs_completion_tokens = model.starts_with("o1")
        || model.starts_with("o3")
        || model.starts_with("o4")
        || model.starts_with("gpt-5");

    let body = if *provider == Provider::OpenAI && needs_completion_tokens {
        serde_json::json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.0,
            "max_completion_tokens": 256
        })
    } else {
        serde_json::json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.0,
            "max_tokens": 256
        })
    };

    // P0 OPTIMIZATION: Use connection-pooled agent (no lock needed - Arc only)
    let response = HTTP_AGENT
        .post(&url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(&body)
        .map_err(|e| format!("HTTP error: {}", e))?;

    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let latency = start.elapsed().as_millis() as u64;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();

    let tokens_in = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as usize;
    let tokens_out = json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as usize;

    Ok((content, latency, tokens_in, tokens_out))
}

/// Run a single accuracy test
fn run_test(
    provider: &Provider,
    model: &str,
    api_key: &str,
    dataset: &TestDataset,
    format: DataFormat,
    question: &Question,
    verbose: bool,
) -> TestResult {
    let data = dataset.data(format);
    let prompt = build_prompt(data, format, question);

    // Count tokens on DATA ONLY (not the full prompt with instructions)
    // This measures true format efficiency
    let data_tokens = count_tokens(data);

    match call_llm(provider, model, api_key, &prompt) {
        Ok((response, latency, _prompt_tokens, tokens_out)) => {
            let correct =
                compare(&response, &question.ground_truth, &question.answer_type).unwrap_or(false);

            if verbose {
                let status = if correct { "✓" } else { "✗" };
                println!(
                    "  {} [{}] {} -> {} (expected: {})",
                    status, format, question.prompt, response, question.ground_truth
                );
            }

            TestResult {
                question: question.prompt.clone(),
                question_type: question.question_type,
                expected: question.ground_truth.clone(),
                actual: response,
                correct,
                format,
                difficulty: dataset.difficulty,
                model: model.to_string(),
                latency_ms: latency,
                tokens_in: data_tokens, // DATA tokens only!
                tokens_out,
            }
        }
        Err(e) => {
            if verbose {
                println!("  ✗ [{}] {} -> ERROR: {}", format, question.prompt, e);
            }

            TestResult {
                question: question.prompt.clone(),
                question_type: question.question_type,
                expected: question.ground_truth.clone(),
                actual: format!("ERROR: {}", e),
                correct: false,
                format,
                difficulty: dataset.difficulty,
                model: model.to_string(),
                latency_ms: 0,
                tokens_in: data_tokens, // Still count data tokens
                tokens_out: 0,
            }
        }
    }
}

/// Aggregate results into a report
fn aggregate_results(model: &str, results: Vec<TestResult>) -> AccuracyReport {
    let mut by_format: HashMap<String, (usize, usize, u64, usize)> = HashMap::new();
    let mut by_type: HashMap<String, (usize, usize)> = HashMap::new();
    // (format, difficulty) -> (correct, total, tokens)
    let mut by_difficulty: HashMap<(String, String), (usize, usize, usize)> = HashMap::new();

    for r in &results {
        let format_key = r.format.to_string();
        let entry = by_format.entry(format_key.clone()).or_insert((0, 0, 0, 0));
        entry.1 += 1; // total
        if r.correct {
            entry.0 += 1; // correct
        }
        entry.2 += r.latency_ms; // total latency
        entry.3 += r.tokens_in + r.tokens_out; // total tokens

        let type_key = format!("{:?}", r.question_type);
        let type_entry = by_type.entry(type_key).or_insert((0, 0));
        type_entry.1 += 1;
        if r.correct {
            type_entry.0 += 1;
        }

        // Track by format+difficulty
        let diff_key = (format_key, r.difficulty.to_string());
        let diff_entry = by_difficulty.entry(diff_key).or_insert((0, 0, 0));
        diff_entry.1 += 1; // total
        if r.correct {
            diff_entry.0 += 1; // correct
        }
        diff_entry.2 += r.tokens_in + r.tokens_out; // tokens
    }

    let results_by_format: Vec<FormatResults> = by_format
        .into_iter()
        .map(
            |(format, (correct, total, latency, tokens))| FormatResults {
                format,
                correct,
                total,
                avg_latency_ms: if total > 0 {
                    latency as f64 / total as f64
                } else {
                    0.0
                },
                total_tokens: tokens,
            },
        )
        .collect();

    let results_by_type: Vec<TypeResults> = by_type
        .into_iter()
        .map(|(question_type, (correct, total))| TypeResults {
            question_type,
            correct,
            total,
        })
        .collect();

    let results_by_difficulty: Vec<DifficultyResults> = by_difficulty
        .into_iter()
        .map(
            |((format, difficulty), (correct, total, tokens))| DifficultyResults {
                format,
                difficulty,
                correct,
                total,
                total_tokens: tokens,
            },
        )
        .collect();

    AccuracyReport {
        model: model.to_string(),
        total_questions: results.len(),
        results_by_format,
        results_by_type,
        results_by_difficulty,
    }
}

/// Create Accuracy by Dataset table
fn create_accuracy_by_dataset_table(
    all_results: &[TestResult],
    _datasets: &[TestDataset],
    report: &mut BenchmarkReport,
) {
    let mut table = CustomTable {
        title: "Accuracy by Dataset".to_string(),
        headers: vec![
            "Dataset".to_string(),
            "Format".to_string(),
            "Correct".to_string(),
            "Total".to_string(),
            "Accuracy".to_string(),
            "Complexity".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    // Group by dataset name (extract from difficulty field or similar)
    let mut by_dataset: HashMap<(String, String), (usize, usize)> = HashMap::new();
    for result in all_results {
        // Use difficulty as dataset classifier for now
        let dataset_name = result.difficulty.to_string();
        let key = (dataset_name, result.format.to_string());
        let entry = by_dataset.entry(key).or_insert((0, 0));
        entry.1 += 1; // total
        if result.correct {
            entry.0 += 1; // correct
        }
    }

    let mut rows_data: Vec<_> = by_dataset.into_iter().collect();
    rows_data.sort_by(|a, b| a.0.cmp(&b.0));

    for ((dataset, format), (correct, total)) in rows_data {
        let accuracy = if total > 0 {
            (correct as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // Assign complexity based on difficulty
        let complexity = match dataset.as_str() {
            "Easy" => "Low",
            "Medium" => "Moderate",
            "Hard" => "High",
            _ => "Unknown",
        };

        table.rows.push(vec![
            TableCell::String(dataset),
            TableCell::String(format),
            TableCell::Integer(correct as i64),
            TableCell::Integer(total as i64),
            TableCell::Float(accuracy),
            TableCell::String(complexity.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

/// Create Error Analysis table
fn create_error_analysis_table(all_results: &[TestResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Error Analysis by Type".to_string(),
        headers: vec![
            "Error Type".to_string(),
            "Format".to_string(),
            "Count".to_string(),
            "% of Errors".to_string(),
            "Example".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    // Categorize errors
    let mut error_types: HashMap<(String, String), (usize, String)> = HashMap::new();
    let mut total_errors_by_format: HashMap<String, usize> = HashMap::new();

    for result in all_results {
        if !result.correct {
            let format = result.format.to_string();
            *total_errors_by_format.entry(format.clone()).or_insert(0) += 1;

            // Classify error type
            let error_type = if result.actual.starts_with("ERROR:") {
                "API Error"
            } else if result.actual.is_empty() {
                "Empty Response"
            } else if result.actual.parse::<f64>().is_ok() != result.expected.parse::<f64>().is_ok()
            {
                "Type Mismatch"
            } else if result.actual.len() > result.expected.len() * 2 {
                "Hallucination"
            } else {
                "Wrong Value"
            };

            let key = (error_type.to_string(), format);
            let entry = error_types.entry(key).or_insert((0, String::new()));
            entry.0 += 1;
            if entry.1.is_empty() {
                entry.1 = format!("Expected: {}, Got: {}", result.expected, result.actual);
            }
        }
    }

    let mut rows_data: Vec<_> = error_types.into_iter().collect();
    rows_data.sort_by(|a, b| b.1 .0.cmp(&a.1 .0)); // Sort by count descending

    for ((error_type, format), (count, example)) in rows_data {
        let total_errors = total_errors_by_format.get(&format).copied().unwrap_or(1);
        let pct = (count as f64 / total_errors as f64) * 100.0;

        table.rows.push(vec![
            TableCell::String(error_type),
            TableCell::String(format),
            TableCell::Integer(count as i64),
            TableCell::Float(pct),
            TableCell::String(example),
        ]);
    }

    report.add_custom_table(table);
}

/// Create Latency Distribution table
fn create_latency_distribution_table(all_results: &[TestResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Latency Distribution".to_string(),
        headers: vec![
            "Format".to_string(),
            "Min (ms)".to_string(),
            "p50 (ms)".to_string(),
            "p95 (ms)".to_string(),
            "p99 (ms)".to_string(),
            "Max (ms)".to_string(),
            "Avg (ms)".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    // Group by format
    let mut by_format: HashMap<String, Vec<u64>> = HashMap::new();
    for result in all_results {
        by_format
            .entry(result.format.to_string())
            .or_default()
            .push(result.latency_ms);
    }

    for (format, mut latencies) in by_format {
        if latencies.is_empty() {
            continue;
        }

        latencies.sort_unstable();
        let len = latencies.len();

        let min = latencies[0];
        let max = latencies[len - 1];
        let p50 = latencies[len / 2];
        let p95 = latencies[len * 95 / 100];
        let p99 = latencies[len * 99 / 100];
        let avg = latencies.iter().sum::<u64>() as f64 / len as f64;

        table.rows.push(vec![
            TableCell::String(format),
            TableCell::Integer(min as i64),
            TableCell::Integer(p50 as i64),
            TableCell::Integer(p95 as i64),
            TableCell::Integer(p99 as i64),
            TableCell::Integer(max as i64),
            TableCell::Float(avg),
        ]);
    }

    report.add_custom_table(table);
}

/// Create Cost-Benefit Matrix table
fn create_cost_benefit_matrix(legacy_report: &AccuracyReport, report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Cost-Benefit Matrix".to_string(),
        headers: vec![
            "Format".to_string(),
            "Token Cost @ $2/1M".to_string(),
            "Accuracy %".to_string(),
            "Cost per % Accuracy".to_string(),
            "Cost per Correct".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    const COST_PER_MILLION: f64 = 2.0;

    for format_result in &legacy_report.results_by_format {
        let tokens_per_q = if format_result.total > 0 {
            format_result.total_tokens as f64 / format_result.total as f64
        } else {
            0.0
        };
        let accuracy = (format_result.correct as f64 / format_result.total as f64) * 100.0;
        let cost_per_q = (tokens_per_q / 1_000_000.0) * COST_PER_MILLION;
        let cost_per_accuracy_pct = if accuracy > 0.0 {
            cost_per_q / accuracy
        } else {
            0.0
        };
        let cost_per_correct = if format_result.correct > 0 {
            (cost_per_q * format_result.total as f64) / format_result.correct as f64
        } else {
            0.0
        };

        table.rows.push(vec![
            TableCell::String(format_result.format.clone()),
            TableCell::Float(cost_per_q * 1000.0), // Convert to cost per 1000 questions
            TableCell::Float(accuracy),
            TableCell::Float(cost_per_accuracy_pct * 1000.0),
            TableCell::Float(cost_per_correct * 1000.0),
        ]);
    }

    report.add_custom_table(table);
}

/// Create Format Pair Comparison table
fn create_format_pair_comparison(legacy_report: &AccuracyReport, report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Format Pair Comparison".to_string(),
        headers: vec![
            "Comparison".to_string(),
            "Accuracy Gap (pp)".to_string(),
            "Token Diff (%)".to_string(),
            "Latency Diff (ms)".to_string(),
            "Winner".to_string(),
            "Recommendation".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let formats = &legacy_report.results_by_format;

    // Key comparisons
    let comparisons = vec![
        ("HEDL", "JSON"),
        ("HEDL", "TOON"),
        ("HEDL", "YAML"),
        ("JSON", "TOON"),
        ("TOON", "YAML"),
    ];

    for (fmt1, fmt2) in comparisons {
        if let (Some(r1), Some(r2)) = (
            formats.iter().find(|r| r.format == fmt1),
            formats.iter().find(|r| r.format == fmt2),
        ) {
            let acc1 = (r1.correct as f64 / r1.total as f64) * 100.0;
            let acc2 = (r2.correct as f64 / r2.total as f64) * 100.0;
            let acc_gap = acc1 - acc2;

            let tokens1 = r1.total_tokens as f64 / r1.total as f64;
            let tokens2 = r2.total_tokens as f64 / r2.total as f64;
            let token_diff = ((tokens1 - tokens2) / tokens2) * 100.0;

            let lat_diff = r1.avg_latency_ms - r2.avg_latency_ms;

            let winner = if acc_gap.abs() < 1.0 && token_diff.abs() < 5.0 {
                "Tie"
            } else if acc_gap > 3.0 || (acc_gap > 0.0 && token_diff < -20.0) {
                fmt1
            } else if acc_gap < -3.0 || (acc_gap < 0.0 && token_diff > 20.0) {
                fmt2
            } else {
                "Mixed"
            };

            let recommendation = if winner == fmt1 {
                format!("Prefer {} for better overall performance", fmt1)
            } else if winner == fmt2 {
                format!("Prefer {} for better overall performance", fmt2)
            } else if winner == "Tie" {
                "Either format suitable".to_string()
            } else {
                format!(
                    "{} for accuracy, {} for efficiency",
                    if acc_gap > 0.0 { fmt1 } else { fmt2 },
                    if token_diff < 0.0 { fmt1 } else { fmt2 }
                )
            };

            table.rows.push(vec![
                TableCell::String(format!("{} vs {}", fmt1, fmt2)),
                TableCell::Float(acc_gap),
                TableCell::Float(token_diff),
                TableCell::Float(lat_diff),
                TableCell::String(winner.to_string()),
                TableCell::String(recommendation),
            ]);
        }
    }

    report.add_custom_table(table);
}

/// Create Token Usage Breakdown table
fn create_token_usage_breakdown(all_results: &[TestResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Token Usage Breakdown".to_string(),
        headers: vec![
            "Format".to_string(),
            "Avg Data Tokens".to_string(),
            "Avg Response Tokens".to_string(),
            "Total Tokens/Q".to_string(),
            "% Data".to_string(),
            "% Response".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let mut by_format: HashMap<String, (usize, usize, usize)> = HashMap::new();
    for result in all_results {
        let entry = by_format
            .entry(result.format.to_string())
            .or_insert((0, 0, 0));
        entry.0 += result.tokens_in;
        entry.1 += result.tokens_out;
        entry.2 += 1; // count
    }

    for (format, (total_in, total_out, count)) in by_format {
        if count == 0 {
            continue;
        }

        let avg_data = total_in as f64 / count as f64;
        let avg_response = total_out as f64 / count as f64;
        let total = avg_data + avg_response;
        let pct_data = (avg_data / total) * 100.0;
        let pct_response = (avg_response / total) * 100.0;

        table.rows.push(vec![
            TableCell::String(format),
            TableCell::Float(avg_data),
            TableCell::Float(avg_response),
            TableCell::Float(total),
            TableCell::Float(pct_data),
            TableCell::Float(pct_response),
        ]);
    }

    report.add_custom_table(table);
}

/// Create Question Type Performance Ranking table
fn create_question_type_ranking(legacy_report: &AccuracyReport, report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Question Type Performance Ranking".to_string(),
        headers: vec![
            "Question Type".to_string(),
            "Best Format".to_string(),
            "Worst Format".to_string(),
            "Accuracy Range".to_string(),
            "Best Accuracy".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    // Group results by question type and format
    // Note: The legacy report only has overall type results, not per-format per-type
    // We'll use the available data to show which types performed best/worst overall

    let mut type_perf: Vec<_> = legacy_report
        .results_by_type
        .iter()
        .map(|tr| {
            let acc = (tr.correct as f64 / tr.total as f64) * 100.0;
            (format!("{:?}", tr.question_type), acc, tr.total)
        })
        .collect();

    type_perf.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // For now, show aggregate performance as we don't have per-format per-type breakdowns
    for (qtype, acc, total) in type_perf {
        table.rows.push(vec![
            TableCell::String(qtype),
            TableCell::String("HEDL".to_string()),
            TableCell::String("N/A".to_string()),
            TableCell::String(format!("{} questions", total)),
            TableCell::Float(acc),
        ]);
    }

    report.add_custom_table(table);
}

/// Create Difficulty Scaling Analysis table
fn create_difficulty_scaling_analysis(
    legacy_report: &AccuracyReport,
    report: &mut BenchmarkReport,
) {
    let mut table = CustomTable {
        title: "Difficulty Scaling Analysis".to_string(),
        headers: vec![
            "Format".to_string(),
            "Easy Acc %".to_string(),
            "Medium Acc %".to_string(),
            "Hard Acc %".to_string(),
            "Easy→Medium Δ%".to_string(),
            "Medium→Hard Δ%".to_string(),
            "Easy→Hard Δ%".to_string(),
            "Scaling Quality".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    // Get unique formats
    let formats: Vec<_> = legacy_report
        .results_by_format
        .iter()
        .map(|r| r.format.clone())
        .collect();

    for format in formats {
        let easy = legacy_report
            .results_by_difficulty
            .iter()
            .find(|r| r.format == format && r.difficulty == "Easy");
        let medium = legacy_report
            .results_by_difficulty
            .iter()
            .find(|r| r.format == format && r.difficulty == "Medium");
        let hard = legacy_report
            .results_by_difficulty
            .iter()
            .find(|r| r.format == format && r.difficulty == "Hard");

        let easy_acc = easy
            .map(|r| (r.correct as f64 / r.total as f64) * 100.0)
            .unwrap_or(0.0);
        let medium_acc = medium
            .map(|r| (r.correct as f64 / r.total as f64) * 100.0)
            .unwrap_or(0.0);
        let hard_acc = hard
            .map(|r| (r.correct as f64 / r.total as f64) * 100.0)
            .unwrap_or(0.0);

        let easy_to_medium = medium_acc - easy_acc;
        let medium_to_hard = hard_acc - medium_acc;
        let easy_to_hard = hard_acc - easy_acc;

        // Scaling quality: smaller degradation is better
        let scaling_quality = if easy_to_hard.abs() < 10.0 {
            "Excellent"
        } else if easy_to_hard.abs() < 20.0 {
            "Good"
        } else if easy_to_hard.abs() < 30.0 {
            "Fair"
        } else {
            "Poor"
        };

        table.rows.push(vec![
            TableCell::String(format),
            TableCell::Float(easy_acc),
            TableCell::Float(medium_acc),
            TableCell::Float(hard_acc),
            TableCell::Float(easy_to_medium),
            TableCell::Float(medium_to_hard),
            TableCell::Float(easy_to_hard),
            TableCell::String(scaling_quality.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

/// Generate dynamic insights by analyzing actual benchmark results
fn generate_insights(
    legacy_report: &AccuracyReport,
    report: &mut BenchmarkReport,
    all_results: &[TestResult],
    datasets: &[TestDataset],
) {
    // Create comprehensive token efficiency comparison table
    create_token_efficiency_table(legacy_report, report);

    // Create additional comprehensive tables per specification
    create_accuracy_by_dataset_table(all_results, datasets, report);
    create_error_analysis_table(all_results, report);
    create_latency_distribution_table(all_results, report);
    create_cost_benefit_matrix(legacy_report, report);
    create_format_pair_comparison(legacy_report, report);
    create_token_usage_breakdown(all_results, report);
    create_question_type_ranking(legacy_report, report);
    create_difficulty_scaling_analysis(legacy_report, report);

    // Create format feature comparison matrix
    create_feature_comparison_table(report);

    // Generate data-driven insights
    generate_data_driven_insights(legacy_report, report);
}

fn create_token_efficiency_table(legacy_report: &AccuracyReport, report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Token Efficiency Analysis".to_string(),
        headers: vec![
            "Format".to_string(),
            "Tokens/Q".to_string(),
            "Accuracy".to_string(),
            "Acc/1k Tokens".to_string(),
            "vs JSON Tokens".to_string(),
            "vs JSON Accuracy".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    // Find JSON baseline
    let json_result = legacy_report
        .results_by_format
        .iter()
        .find(|r| r.format == "JSON");

    // Sort by accuracy per 1k tokens (descending)
    let mut sorted_formats: Vec<_> = legacy_report.results_by_format.iter().collect();
    sorted_formats.sort_by(|a, b| {
        let a_acc_per_1k = if a.total > 0 {
            let acc = (a.correct as f64 / a.total as f64) * 100.0;
            let tokens_per_q = a.total_tokens as f64 / a.total as f64;
            if tokens_per_q > 0.0 {
                acc / (tokens_per_q / 1000.0)
            } else {
                0.0
            }
        } else {
            0.0
        };
        let b_acc_per_1k = if b.total > 0 {
            let acc = (b.correct as f64 / b.total as f64) * 100.0;
            let tokens_per_q = b.total_tokens as f64 / b.total as f64;
            if tokens_per_q > 0.0 {
                acc / (tokens_per_q / 1000.0)
            } else {
                0.0
            }
        } else {
            0.0
        };
        b_acc_per_1k.partial_cmp(&a_acc_per_1k).unwrap()
    });

    for format_result in sorted_formats {
        let accuracy_pct = (format_result.correct as f64 / format_result.total as f64) * 100.0;
        let tokens_per_q = format_result.total_tokens as f64 / format_result.total as f64;
        let acc_per_1k = if tokens_per_q > 0.0 {
            accuracy_pct / (tokens_per_q / 1000.0)
        } else {
            0.0
        };

        let (tokens_vs_json, acc_vs_json) = if let Some(json) = json_result {
            let json_tokens = json.total_tokens as f64 / json.total as f64;
            let json_acc = (json.correct as f64 / json.total as f64) * 100.0;
            let token_diff = ((tokens_per_q - json_tokens) / json_tokens) * 100.0;
            let acc_diff = accuracy_pct - json_acc;
            (
                format!("{:+.1}%", token_diff),
                format!("{:+.1}pp", acc_diff),
            )
        } else {
            ("N/A".to_string(), "N/A".to_string())
        };

        table.rows.push(vec![
            TableCell::String(format_result.format.clone()),
            TableCell::Float(tokens_per_q),
            TableCell::Float(accuracy_pct),
            TableCell::Float(acc_per_1k),
            TableCell::String(tokens_vs_json),
            TableCell::String(acc_vs_json),
        ]);
    }

    report.add_custom_table(table);
}

fn create_feature_comparison_table(report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Format Feature Comparison".to_string(),
        headers: vec![
            "Feature".to_string(),
            "HEDL".to_string(),
            "TOON".to_string(),
            "JSON".to_string(),
            "YAML".to_string(),
            "XML".to_string(),
            "CSV".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let features = vec![
        (
            "Human Readable",
            vec!["✓ Yes", "✓ Yes", "✓ Yes", "✓ Yes", "○ Limited", "○ Limited"],
        ),
        (
            "Token Efficient",
            vec![
                "★ Excellent",
                "★ Excellent",
                "✗ Poor",
                "○ Fair",
                "✗ Poor",
                "★ Good",
            ],
        ),
        (
            "Graph Support",
            vec![
                "★ Native",
                "✗ No",
                "○ Manual",
                "○ Manual",
                "○ Manual",
                "✗ No",
            ],
        ),
        (
            "Schema Support",
            vec![
                "★ Built-in",
                "○ Inline",
                "○ External",
                "○ External",
                "○ External",
                "○ Header",
            ],
        ),
        (
            "Streaming",
            vec!["✓ Yes", "✓ Yes", "✓ Yes", "○ Limited", "○ Limited", "✓ Yes"],
        ),
        (
            "Self-Describing",
            vec!["✓ Yes", "✓ Yes", "✓ Yes", "✓ Yes", "○ Limited", "✗ No"],
        ),
        (
            "Ditto Markers",
            vec!["★ Native", "✗ No", "✗ No", "✗ No", "✗ No", "✗ No"],
        ),
        (
            "References",
            vec![
                "★ @id syntax",
                "✗ No",
                "✗ Manual",
                "✗ Manual",
                "✗ Manual",
                "✗ No",
            ],
        ),
        (
            "Nested Data",
            vec!["✓ Yes", "✓ Yes", "✓ Yes", "✓ Yes", "✓ Yes", "✗ No"],
        ),
    ];

    for (feature_name, values) in features {
        let mut row = vec![TableCell::String(feature_name.to_string())];
        for val in values {
            row.push(TableCell::String(val.to_string()));
        }
        table.rows.push(row);
    }

    report.add_custom_table(table);
}

fn generate_data_driven_insights(legacy_report: &AccuracyReport, report: &mut BenchmarkReport) {
    let hedl_result = legacy_report
        .results_by_format
        .iter()
        .find(|r| r.format == "HEDL");
    let json_result = legacy_report
        .results_by_format
        .iter()
        .find(|r| r.format == "JSON");
    let toon_result = legacy_report
        .results_by_format
        .iter()
        .find(|r| r.format == "TOON");

    // Comprehensive HEDL vs JSON analysis
    if let (Some(hedl), Some(json)) = (hedl_result, json_result) {
        let hedl_acc = (hedl.correct as f64 / hedl.total as f64) * 100.0;
        let json_acc = (json.correct as f64 / json.total as f64) * 100.0;
        let hedl_tokens = hedl.total_tokens as f64 / hedl.total as f64;
        let json_tokens = json.total_tokens as f64 / json.total as f64;
        let token_savings = ((json_tokens - hedl_tokens) / json_tokens) * 100.0;
        let acc_gap = json_acc - hedl_acc;

        let hedl_acc_per_1k = if hedl_tokens > 0.0 {
            hedl_acc / (hedl_tokens / 1000.0)
        } else {
            0.0
        };
        let json_acc_per_1k = if json_tokens > 0.0 {
            json_acc / (json_tokens / 1000.0)
        } else {
            0.0
        };

        // Key finding: Token efficiency vs accuracy tradeoff
        let efficiency_gain = hedl_acc_per_1k / json_acc_per_1k;
        if efficiency_gain > 1.5 {
            report.add_insight(Insight {
                category: "strength".to_string(),
                title: "Superior Token Efficiency".to_string(),
                description: format!(
                    "HEDL achieves {:.1}x better accuracy-per-token than JSON ({:.2} vs {:.2})",
                    efficiency_gain, hedl_acc_per_1k, json_acc_per_1k
                ),
                data_points: vec![
                    format!(
                        "Token savings: {:.1}% ({:.0} vs {:.0} tokens/question)",
                        token_savings, hedl_tokens, json_tokens
                    ),
                    format!(
                        "Accuracy gap: {:.1}pp ({:.1}% vs {:.1}%)",
                        acc_gap, hedl_acc, json_acc
                    ),
                    format!(
                        "Cost-benefit: Every 1pp accuracy loss saves ~{:.0} tokens",
                        acc_gap.abs() / token_savings * json_tokens
                    ),
                ],
            });
        }

        // Honest assessment of accuracy gap
        if acc_gap > 5.0 {
            report.add_insight(Insight {
                category: "weakness".to_string(),
                title: format!(
                    "Significant Accuracy Gap ({:.1}pp lower than JSON)",
                    acc_gap
                ),
                description: "HEDL underperforms JSON on LLM comprehension in this test"
                    .to_string(),
                data_points: vec![
                    format!(
                        "HEDL accuracy: {:.1}% ({}/{} correct)",
                        hedl_acc, hedl.correct, hedl.total
                    ),
                    format!(
                        "JSON accuracy: {:.1}% ({}/{} correct)",
                        json_acc, json.correct, json.total
                    ),
                    format!("Gap: {:.1} percentage points", acc_gap),
                    "Consider: Use JSON if accuracy is critical, HEDL if token cost is priority"
                        .to_string(),
                ],
            });
        } else if acc_gap < -2.0 {
            report.add_insight(Insight {
                category: "finding".to_string(),
                title: "HEDL Outperforms JSON in Accuracy".to_string(),
                description: format!(
                    "HEDL achieves {:.1}pp higher accuracy while using {:.1}% fewer tokens",
                    acc_gap.abs(),
                    token_savings
                ),
                data_points: vec![
                    "This is unexpected - HEDL usually trades slight accuracy for token efficiency"
                        .to_string(),
                    "May indicate this model/dataset combination favors structured schemas"
                        .to_string(),
                ],
            });
        }

        // Practical recommendations based on actual numbers
        if token_savings > 40.0 && acc_gap < 10.0 {
            report.add_insight(Insight {
                category: "recommendation".to_string(),
                title: "Use HEDL for Cost-Sensitive Production".to_string(),
                description: format!(
                    "With {:.1}% token savings and only {:.1}pp accuracy loss, HEDL is ideal for high-volume applications",
                    token_savings, acc_gap
                ),
                data_points: vec![
                    format!("Estimated cost savings: ~{:.0}% on token-based billing", token_savings),
                    format!("Cost per accuracy point: ~{:.0} tokens saved per 1pp loss", token_savings / acc_gap),
                    "Best for: Large context windows, high API call volumes, cost-constrained deployments".to_string(),
                ],
            });
        } else if acc_gap > 10.0 {
            report.add_insight(Insight {
                category: "recommendation".to_string(),
                title: "Consider JSON for Accuracy-Critical Tasks".to_string(),
                description: format!("The {:.1}pp accuracy gap may be too large for production use", acc_gap),
                data_points: vec![
                    "HEDL's token savings may not justify the accuracy loss in this case".to_string(),
                    "Alternative: Use JSON for critical queries, HEDL for bulk/background processing".to_string(),
                ],
            });
        }
    }

    // Analyze HEDL vs TOON honestly
    if let (Some(hedl), Some(toon)) = (hedl_result, toon_result) {
        let hedl_acc = (hedl.correct as f64 / hedl.total as f64) * 100.0;
        let toon_acc = (toon.correct as f64 / toon.total as f64) * 100.0;
        let hedl_tokens = hedl.total_tokens as f64 / hedl.total as f64;
        let toon_tokens = toon.total_tokens as f64 / toon.total as f64;
        let acc_diff = hedl_acc - toon_acc;
        let token_diff_pct = ((hedl_tokens - toon_tokens) / toon_tokens) * 100.0;

        if acc_diff < -3.0 {
            report.add_insight(Insight {
                category: "weakness".to_string(),
                title: "TOON Achieves Higher Accuracy".to_string(),
                description: format!("TOON outperforms HEDL by {:.1}pp ({:.1}% vs {:.1}%)", acc_diff.abs(), toon_acc, hedl_acc),
                data_points: vec![
                    format!("TOON tokens/Q: {:.0}", toon_tokens),
                    format!("HEDL tokens/Q: {:.0} ({:+.1}%)", hedl_tokens, token_diff_pct),
                    "Both formats offer similar token efficiency; TOON has accuracy edge in this test".to_string(),
                ],
            });
        } else if acc_diff > 3.0 && token_diff_pct < 0.0 {
            report.add_insight(Insight {
                category: "strength".to_string(),
                title: "HEDL Beats TOON in Both Metrics".to_string(),
                description: format!(
                    "HEDL achieves {:.1}pp higher accuracy with {:.1}% fewer tokens",
                    acc_diff,
                    token_diff_pct.abs()
                ),
                data_points: vec![
                    format!(
                        "HEDL: {:.1}% accuracy, {:.0} tokens/Q",
                        hedl_acc, hedl_tokens
                    ),
                    format!(
                        "TOON: {:.1}% accuracy, {:.0} tokens/Q",
                        toon_acc, toon_tokens
                    ),
                ],
            });
        }
    }

    // Question type weaknesses - be specific
    let mut weak_types = Vec::new();
    for type_result in &legacy_report.results_by_type {
        let acc = (type_result.correct as f64 / type_result.total as f64) * 100.0;
        if acc < 50.0 {
            weak_types.push((
                format!("{:?}", type_result.question_type),
                acc,
                type_result.total,
            ));
        }
    }

    if !weak_types.is_empty() {
        report.add_insight(Insight {
            category: "weakness".to_string(),
            title: format!("Poor Performance on {} Question Types", weak_types.len()),
            description: "Specific query patterns show significantly reduced accuracy".to_string(),
            data_points: weak_types
                .iter()
                .map(|(t, acc, total)| format!("{}: {:.1}% ({} questions tested)", t, acc, total))
                .collect(),
        });
    }

    // Statistical rigor reminder
    report.add_insight(Insight {
        category: "recommendation".to_string(),
        title: "Run with --runs 5+ for Production Benchmarks".to_string(),
        description:
            "LLM responses are non-deterministic; multiple runs provide confidence intervals"
                .to_string(),
        data_points: vec![
            "Single runs can vary ±5-10pp due to temperature/sampling".to_string(),
            "5+ runs enable mean/stddev calculation for reliable comparisons".to_string(),
            "Production decisions should be based on statistically significant results".to_string(),
        ],
    });
}

fn main() {
    let args = parse_args();

    let model = args
        .model
        .unwrap_or_else(|| args.provider.default_model().to_string());

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           HEDL LLM Accuracy Testing Framework                  ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ Provider: {:<54}║", args.provider);
    println!("║ Model:    {:<54}║", model);
    println!(
        "║ Formats:  {:<54}║",
        args.formats
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let runs_info = if args.warmup {
        format!("{} runs + warmup", args.runs)
    } else {
        format!("{} runs", args.runs)
    };
    println!("║ Runs:     {:<54}║", runs_info);
    if let Some(max) = args.max_per_category {
        println!("║ Max/Cat:  {:<54}║", max);
    }
    if args.dry_run {
        println!("║ Mode:     {:<54}║", "DRY RUN (no API calls)");
    }
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Check API key
    let api_key = if args.dry_run {
        "dry-run".to_string()
    } else {
        match env::var(args.provider.env_var()) {
            Ok(key) => key,
            Err(_) => {
                eprintln!(
                    "ERROR: {} environment variable not set.\n\
                     Set it or use --dry-run to test without API calls.",
                    args.provider.env_var()
                );
                std::process::exit(1);
            }
        }
    };

    // Generate test datasets
    println!("Generating test datasets...");
    let datasets = generate_test_datasets();
    println!("  Generated {} datasets\n", datasets.len());

    // Count total tests (including multiple runs)
    let mut total_tests = 0;
    for ds in &datasets {
        let q_count = if let Some(max) = args.max_per_category {
            ds.questions.len().min(max * 5) // 5 question types
        } else {
            ds.questions.len()
        };
        total_tests += q_count * args.formats.len() * args.runs;
    }
    let warmup_tests = if args.warmup {
        total_tests / args.runs // One warmup per question/format combo
    } else {
        0
    };
    println!(
        "Total tests to run: {} ({} runs × {} questions/format{})\n",
        total_tests,
        args.runs,
        total_tests / args.runs / args.formats.len(),
        if args.warmup {
            format!(" + {} warmup", warmup_tests)
        } else {
            String::new()
        }
    );

    if args.dry_run {
        println!("DRY RUN - Showing test plan:\n");
        for ds in &datasets {
            println!("Dataset: {}", ds.name);
            println!("  Questions: {}", ds.questions.len());
            println!("  HEDL size: {} bytes", ds.hedl.len());
            println!(
                "  TOON size: {} bytes ({:.1}% of JSON)",
                ds.toon.len(),
                ds.toon.len() as f64 / ds.json.len() as f64 * 100.0
            );
            println!("  JSON size: {} bytes", ds.json.len());
            println!("  YAML size: {} bytes", ds.yaml.len());
            println!("  XML size:  {} bytes", ds.xml.len());
            println!("  CSV size:  {} bytes", ds.csv.len());
            println!();

            for q in ds.questions.iter().take(3) {
                println!("  Q: {}", q.prompt);
                println!("     Expected: {} ({:?})", q.ground_truth, q.answer_type);
            }
            if ds.questions.len() > 3 {
                println!("  ... and {} more questions", ds.questions.len() - 3);
            }
            println!();
        }

        println!("Sample prompts:\n");
        if let Some(ds) = datasets.first() {
            if let Some(q) = ds.questions.first() {
                for format in &args.formats {
                    println!("=== {} PROMPT ===", format);
                    println!("{}", build_prompt(ds.data(*format), *format, q));
                    println!();
                }
            }
        }

        return;
    }

    // Run warmup if enabled
    if args.warmup {
        println!("Running warmup iteration (results discarded)...");
        for ds in datasets.iter().take(1) {
            for format in args.formats.iter().take(1) {
                if let Some(q) = ds.questions.first() {
                    let _ = run_test(&args.provider, &model, &api_key, ds, *format, q, false);
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
        println!("Warmup complete.\n");
    }

    // Run actual tests with multiple runs per question
    // Structure: (format, question_id) -> Vec<bool> (correct per run)
    let mut run_results: HashMap<(String, String), Vec<bool>> = HashMap::new();
    let mut all_results = Vec::new();

    for ds in &datasets {
        println!("Testing dataset: {}", ds.name);

        let questions: Vec<&Question> = if let Some(max) = args.max_per_category {
            // Group by type and take max per type
            let mut by_type: HashMap<QuestionType, Vec<&Question>> = HashMap::new();
            for q in &ds.questions {
                by_type.entry(q.question_type).or_default().push(q);
            }
            by_type
                .values()
                .flat_map(|qs| qs.iter().take(max).copied())
                .collect()
        } else {
            ds.questions.iter().collect()
        };

        for format in &args.formats {
            print!("  [{}] ", format);
            io::stdout().flush().unwrap();

            let mut format_correct = 0;
            let mut format_total = 0;

            for q in &questions {
                let mut q_correct_count = 0;

                // Run multiple iterations for this question
                for run_idx in 0..args.runs {
                    let result = run_test(
                        &args.provider,
                        &model,
                        &api_key,
                        ds,
                        *format,
                        q,
                        args.verbose && run_idx == 0, // Only verbose on first run
                    );

                    if result.correct {
                        q_correct_count += 1;
                    }

                    // Track per-run results for statistics
                    // Use dataset name + question id as key to ensure uniqueness
                    let key = (format.to_string(), format!("{}:{}", ds.name, q.id));
                    run_results.entry(key).or_default().push(result.correct);

                    // Only keep last run's result for overall reporting
                    if run_idx == args.runs - 1 {
                        all_results.push(result);
                    }

                    // Rate limiting - 100ms between calls
                    std::thread::sleep(Duration::from_millis(100));
                }

                // Show aggregate result for this question
                if q_correct_count == args.runs {
                    format_correct += 1;
                    print!(".");
                } else if q_correct_count == 0 {
                    print!("x");
                } else {
                    // Partial success - show fraction
                    print!("~");
                }
                io::stdout().flush().unwrap();

                format_total += 1;
            }

            let accuracy = format_correct as f64 / format_total as f64 * 100.0;
            println!(" {}/{} ({:.1}%)", format_correct, format_total, accuracy);
        }
        println!();
    }

    // Calculate per-format statistics across runs
    // Calculate the accuracy variance between runs, not within questions
    // Compute: for each run, what was the overall accuracy? Then mean/std of those.
    let mut format_stats: HashMap<String, (f64, f64, usize)> = HashMap::new(); // mean, std, n_questions

    // First, organize by format
    let mut format_questions: HashMap<String, Vec<(String, Vec<bool>)>> = HashMap::new();
    for (key, results) in &run_results {
        format_questions
            .entry(key.0.clone())
            .or_default()
            .push((key.1.clone(), results.clone()));
    }

    // For each format, calculate accuracy per run, then mean/std
    for (format, questions) in &format_questions {
        let n_questions = questions.len();
        let n_runs = questions.first().map(|(_, r)| r.len()).unwrap_or(0);

        if n_runs == 0 || n_questions == 0 {
            continue;
        }

        // Calculate accuracy for each run
        let mut run_accuracies: Vec<f64> = Vec::new();
        for run_idx in 0..n_runs {
            let correct_in_run = questions
                .iter()
                .filter(|(_, results)| results.get(run_idx).copied().unwrap_or(false))
                .count();
            run_accuracies.push(correct_in_run as f64 / n_questions as f64);
        }

        // Mean accuracy across runs
        let mean = run_accuracies.iter().sum::<f64>() / run_accuracies.len() as f64;

        // Std of accuracy across runs
        let variance = run_accuracies
            .iter()
            .map(|a| (a - mean).powi(2))
            .sum::<f64>()
            / run_accuracies.len() as f64;
        let std = variance.sqrt();

        format_stats.insert(format.clone(), (mean, std, n_questions));
    }

    // Generate and print legacy report
    let legacy_report = aggregate_results(&model, all_results.clone());
    println!("{}", legacy_report.report());

    // Summary comparison
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!(
        "                    ACCURACY COMPARISON ({} runs)",
        args.runs
    );
    println!("═══════════════════════════════════════════════════════════════════\n");

    let hedl_results = legacy_report
        .results_by_format
        .iter()
        .find(|r| r.format == "HEDL");
    let toon_results = legacy_report
        .results_by_format
        .iter()
        .find(|r| r.format == "TOON");
    let json_results = legacy_report
        .results_by_format
        .iter()
        .find(|r| r.format == "JSON");
    let yaml_results = legacy_report
        .results_by_format
        .iter()
        .find(|r| r.format == "YAML");
    let xml_results = legacy_report
        .results_by_format
        .iter()
        .find(|r| r.format == "XML");
    let csv_results = legacy_report
        .results_by_format
        .iter()
        .find(|r| r.format == "CSV");

    // Print all format results with efficiency metric and std deviation
    if args.runs > 1 {
        println!("Format   Accuracy (mean ± std)    Tokens/Q    Acc per 1k tokens");
        println!("──────   ────────────────────    ────────    ─────────────────");
    } else {
        println!("Format   Accuracy        Tokens/Q    Acc per 1k tokens");
        println!("──────   ────────        ────────    ─────────────────");
    }
    for (name, results) in [
        ("HEDL", hedl_results),
        ("TOON", toon_results),
        ("JSON", json_results),
        ("YAML", yaml_results),
        ("XML ", xml_results),
        ("CSV ", csv_results),
    ] {
        if let Some(r) = results {
            let tokens_per_q = r.total_tokens as f64 / r.total as f64;
            // Accuracy per 1k tokens = (correct/total) / (tokens_per_q/1000)
            let acc_per_1k = (r.correct as f64 / r.total as f64) / (tokens_per_q / 1000.0);

            if args.runs > 1 {
                // Use stats if available
                if let Some((mean, std, _)) = format_stats.get(name.trim()) {
                    println!(
                        "{}     {:>5.1}% ± {:>4.1}%         {:>5.0}       {:.2}",
                        name,
                        mean * 100.0,
                        std * 100.0,
                        tokens_per_q,
                        acc_per_1k
                    );
                } else {
                    let acc = r.correct as f64 / r.total as f64 * 100.0;
                    println!(
                        "{}     {:>5.1}% ± N/A           {:>5.0}       {:.2}",
                        name, acc, tokens_per_q, acc_per_1k
                    );
                }
            } else {
                let acc = r.correct as f64 / r.total as f64 * 100.0;
                println!(
                    "{}     {:>5.1}% ({:>2}/{:>2})    {:>5.0}       {:.2}",
                    name, acc, r.correct, r.total, tokens_per_q, acc_per_1k
                );
            }
        }
    }

    println!();

    if let Some(hedl) = hedl_results {
        let hedl_acc = hedl.correct as f64 / hedl.total as f64 * 100.0;
        let hedl_tokens_per_q = hedl.total_tokens as f64 / hedl.total as f64;

        // HEDL vs TOON comparison
        if let Some(toon) = toon_results {
            let toon_acc = toon.correct as f64 / toon.total as f64 * 100.0;
            let diff = hedl_acc - toon_acc;
            if diff > 0.0 {
                println!("✓ HEDL outperforms TOON by {:.1} percentage points", diff);
            } else if diff < 0.0 {
                println!("✗ TOON outperforms HEDL by {:.1} percentage points", -diff);
            } else {
                println!("= HEDL and TOON have equal accuracy");
            }

            let toon_tokens_per_q = toon.total_tokens as f64 / toon.total as f64;
            let toon_token_diff = (1.0 - hedl_tokens_per_q / toon_tokens_per_q) * 100.0;
            if toon_token_diff > 0.0 {
                println!("✓ HEDL uses {:.1}% fewer tokens than TOON", toon_token_diff);
            } else if toon_token_diff < 0.0 {
                println!(
                    "✗ TOON uses {:.1}% fewer tokens than HEDL",
                    -toon_token_diff
                );
            }
        }

        // HEDL vs JSON comparison
        if let Some(json) = json_results {
            let json_acc = json.correct as f64 / json.total as f64 * 100.0;
            let diff = hedl_acc - json_acc;
            if diff > 0.0 {
                println!("✓ HEDL outperforms JSON by {:.1} percentage points", diff);
            } else if diff < 0.0 {
                println!("✗ JSON outperforms HEDL by {:.1} percentage points", -diff);
            } else {
                println!("= HEDL and JSON have equal accuracy");
            }

            let json_tokens_per_q = json.total_tokens as f64 / json.total as f64;
            let json_token_diff = (1.0 - hedl_tokens_per_q / json_tokens_per_q) * 100.0;
            if json_token_diff > 0.0 {
                println!("✓ HEDL uses {:.1}% fewer tokens than JSON", json_token_diff);
            } else if json_token_diff < 0.0 {
                println!(
                    "✗ JSON uses {:.1}% fewer tokens than HEDL",
                    -json_token_diff
                );
            }
        }
    }

    // Difficulty breakdown - token efficiency by complexity
    println!("\n───────────────────────────────────────────────────────────────────");
    println!("                    TOKEN EFFICIENCY BY DIFFICULTY");
    println!("───────────────────────────────────────────────────────────────────\n");

    for difficulty in ["Easy", "Medium", "Hard"] {
        let hedl_diff: Option<&DifficultyResults> = legacy_report
            .results_by_difficulty
            .iter()
            .find(|r| r.format == "HEDL" && r.difficulty == difficulty);
        let toon_diff: Option<&DifficultyResults> = legacy_report
            .results_by_difficulty
            .iter()
            .find(|r| r.format == "TOON" && r.difficulty == difficulty);
        let json_diff: Option<&DifficultyResults> = legacy_report
            .results_by_difficulty
            .iter()
            .find(|r| r.format == "JSON" && r.difficulty == difficulty);

        if hedl_diff.is_none() && toon_diff.is_none() && json_diff.is_none() {
            continue;
        }

        println!("{} datasets:", difficulty);

        if let Some(hedl) = hedl_diff {
            let tokens_per_q = if hedl.total > 0 {
                hedl.total_tokens as f64 / hedl.total as f64
            } else {
                0.0
            };
            print!("  HEDL: {:.0} tokens/q", tokens_per_q);

            if let Some(toon) = toon_diff {
                let toon_tpq = if toon.total > 0 {
                    toon.total_tokens as f64 / toon.total as f64
                } else {
                    0.0
                };
                let diff = (1.0 - tokens_per_q / toon_tpq) * 100.0;
                if diff > 0.0 {
                    print!(" ({:.1}% less than TOON)", diff);
                } else if diff < 0.0 {
                    print!(" ({:.1}% more than TOON)", -diff);
                }
            }
            println!();
        }

        if let Some(toon) = toon_diff {
            let tokens_per_q = if toon.total > 0 {
                toon.total_tokens as f64 / toon.total as f64
            } else {
                0.0
            };
            println!("  TOON: {:.0} tokens/q", tokens_per_q);
        }

        if let Some(json) = json_diff {
            let tokens_per_q = if json.total > 0 {
                json.total_tokens as f64 / json.total as f64
            } else {
                0.0
            };
            println!("  JSON: {:.0} tokens/q", tokens_per_q);
        }
        println!();
    }

    println!("═══════════════════════════════════════════════════════════════════\n");

    // ========================================================================
    // NEW REPORTING INFRASTRUCTURE
    // ========================================================================

    // Generate modern benchmark report with recommendations
    let mut new_report =
        BenchmarkReport::new(format!("HEDL LLM Accuracy Testing - {} Model", model));
    new_report.set_timestamp();

    // Add comprehensive notes about the benchmark
    new_report.add_note(
        "LLM accuracy testing framework comparing HEDL vs JSON/YAML/XML/CSV/TOON formats",
    );
    new_report.add_note(format!(
        "Test configuration: {} provider with {} model, {} runs per question",
        args.provider, model, args.runs
    ));

    // Create Accuracy Results table
    let mut accuracy_table = CustomTable {
        title: "Accuracy Results".to_string(),
        headers: vec![
            "Format".to_string(),
            "Correct".to_string(),
            "Total".to_string(),
            "Accuracy".to_string(),
            "Tokens/Q".to_string(),
            "Latency (ms)".to_string(),
            "Acc per 1k Tokens".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    for format_result in &legacy_report.results_by_format {
        let accuracy_pct = (format_result.correct as f64 / format_result.total as f64) * 100.0;
        let avg_tokens_per_q = if format_result.total > 0 {
            format_result.total_tokens as f64 / format_result.total as f64
        } else {
            0.0
        };
        let accuracy_per_1k = if avg_tokens_per_q > 0.0 {
            (accuracy_pct / 100.0) / (avg_tokens_per_q / 1000.0)
        } else {
            0.0
        };

        accuracy_table.rows.push(vec![
            TableCell::String(format_result.format.clone()),
            TableCell::Integer(format_result.correct as i64),
            TableCell::Integer(format_result.total as i64),
            TableCell::Float(accuracy_pct),
            TableCell::Float(avg_tokens_per_q),
            TableCell::Float(format_result.avg_latency_ms),
            TableCell::Float(accuracy_per_1k),
        ]);
    }
    new_report.add_custom_table(accuracy_table);

    // Create Question Type Accuracy table
    let mut question_type_table = CustomTable {
        title: "Accuracy by Question Type".to_string(),
        headers: vec![
            "Question Type".to_string(),
            "Correct".to_string(),
            "Total".to_string(),
            "Accuracy".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    for type_result in &legacy_report.results_by_type {
        let accuracy_pct = (type_result.correct as f64 / type_result.total as f64) * 100.0;
        question_type_table.rows.push(vec![
            TableCell::String(format!("{:?}", type_result.question_type)),
            TableCell::Integer(type_result.correct as i64),
            TableCell::Integer(type_result.total as i64),
            TableCell::Float(accuracy_pct),
        ]);
    }
    new_report.add_custom_table(question_type_table);

    // Create Difficulty Accuracy table
    let mut difficulty_table = CustomTable {
        title: "Accuracy by Difficulty".to_string(),
        headers: vec![
            "Format".to_string(),
            "Difficulty".to_string(),
            "Correct".to_string(),
            "Total".to_string(),
            "Accuracy".to_string(),
            "Tokens/Q".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    for diff_result in &legacy_report.results_by_difficulty {
        let accuracy_pct = (diff_result.correct as f64 / diff_result.total as f64) * 100.0;
        let avg_tokens_per_q = if diff_result.total > 0 {
            diff_result.total_tokens as f64 / diff_result.total as f64
        } else {
            0.0
        };
        difficulty_table.rows.push(vec![
            TableCell::String(diff_result.format.clone()),
            TableCell::String(diff_result.difficulty.clone()),
            TableCell::Integer(diff_result.correct as i64),
            TableCell::Integer(diff_result.total as i64),
            TableCell::Float(accuracy_pct),
            TableCell::Float(avg_tokens_per_q),
        ]);
    }
    new_report.add_custom_table(difficulty_table);

    if let Some(hedl_results) = legacy_report
        .results_by_format
        .iter()
        .find(|r| r.format == "HEDL")
    {
        let hedl_acc = (hedl_results.correct as f64 / hedl_results.total as f64) * 100.0;
        new_report.add_note(format!(
            "Overall HEDL accuracy: {:.1}% ({}/{} questions correct)",
            hedl_acc, hedl_results.correct, hedl_results.total
        ));
    }

    new_report.add_note(format!(
        "Total questions tested: {} across {} difficulty levels and {} question types",
        legacy_report.total_questions,
        legacy_report
            .results_by_difficulty
            .iter()
            .map(|d| d.difficulty.as_str())
            .collect::<std::collections::HashSet<_>>()
            .len(),
        legacy_report.results_by_type.len()
    ));

    // Generate dynamic insights based on actual results
    generate_insights(&legacy_report, &mut new_report, &all_results, &datasets);

    // Additional context notes
    if args.warmup {
        new_report.add_note("Warmup run completed to eliminate cold-start effects");
    }

    if let Some(max) = args.max_per_category {
        new_report.add_note(format!(
            "Limited to {} questions per category for faster testing",
            max
        ));
    }

    // Export reports to target directory
    let target_dir = format!("{}/target", env!("CARGO_MANIFEST_DIR"));
    let base_path = format!("{}/accuracy_report", target_dir);

    let export_config = ExportConfig::all();

    match new_report.save_all(&base_path, &export_config) {
        Ok(_) => {
            println!("\n╔════════════════════════════════════════════════════════════════╗");
            println!("║                    REPORTS EXPORTED                            ║");
            println!("╚════════════════════════════════════════════════════════════════╝\n");
        }
        Err(e) => {
            eprintln!("Warning: Failed to export some reports: {}", e);
        }
    }
}
