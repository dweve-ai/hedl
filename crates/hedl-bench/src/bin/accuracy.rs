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

#[path = "accuracy/accuracy_cli.rs"]
mod accuracy_cli;
#[path = "accuracy/accuracy_insights.rs"]
mod accuracy_insights;
#[path = "accuracy/accuracy_llm.rs"]
mod accuracy_llm;
#[path = "accuracy/accuracy_tables.rs"]
mod accuracy_tables;
#[path = "accuracy/accuracy_test.rs"]
mod accuracy_test;
#[path = "accuracy/accuracy_types.rs"]
mod accuracy_types;

use accuracy_cli::{parse_args, Args};
use accuracy_insights::generate_insights;
use accuracy_test::run_test;
use accuracy_types::{aggregate_results, DifficultyResults};
use hedl_bench::accuracy::{build_prompt, Question, QuestionType};
use hedl_bench::real_datasets::load_fixture_datasets;
use hedl_bench::{BenchmarkReport, CustomTable, ExportConfig, TableCell};
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::time::Duration;

fn main() {
    let args = parse_args();

    let model = args
        .model
        .clone()
        .unwrap_or_else(|| args.provider.default_model().to_string());

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           HEDL LLM Accuracy Testing Framework                  ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ Provider: {:<54}║", args.provider);
    println!("║ Model:    {model:<54}║");
    println!(
        "║ Formats:  {:<54}║",
        args.formats
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );
    let runs_info = if args.warmup {
        format!("{} runs + warmup", args.runs)
    } else {
        format!("{} runs", args.runs)
    };
    println!("║ Runs:     {runs_info:<54}║");
    if let Some(max) = args.max_per_category {
        println!("║ Max/Cat:  {max:<54}║");
    }
    if args.dry_run {
        println!("║ Mode:     {:<54}║", "DRY RUN (no API calls)");
    }
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Check API key
    let api_key = if args.dry_run {
        "dry-run".to_string()
    } else if let Ok(key) = env::var(args.provider.env_var()) {
        key
    } else {
        eprintln!(
            "ERROR: {} environment variable not set.\n\
             Set it or use --dry-run to test without API calls.",
            args.provider.env_var()
        );
        std::process::exit(1);
    };

    // Load pre-made fixture datasets (no conversion, all formats hand-crafted)
    println!("Loading fixture datasets (hand-crafted, no conversion)...");
    let mut datasets = load_fixture_datasets();
    println!("  Loaded {} datasets", datasets.len());

    // Handle --list option
    if args.list_fixtures {
        println!("\nAvailable fixtures:");
        for ds in &datasets {
            println!("  - {} ({} questions)", ds.name, ds.questions.len());
        }
        return;
    }

    // Filter datasets by fixture name if specified
    if !args.fixtures.is_empty() {
        datasets.retain(|ds| {
            args.fixtures
                .iter()
                .any(|f| ds.name.to_lowercase().contains(&f.to_lowercase()))
        });
        println!(
            "  Filtered to {} datasets: {:?}",
            datasets.len(),
            args.fixtures
        );
    }
    println!();

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
            format!(" + {warmup_tests} warmup")
        } else {
            String::new()
        }
    );

    if args.dry_run {
        print_dry_run_info(&args, &datasets);
        return;
    }

    // Run warmup if enabled
    if args.warmup {
        run_warmup(&args, &model, &api_key, &datasets);
    }

    // Run actual tests with multiple runs per question
    let (all_results, run_results) = run_all_tests(&args, &model, &api_key, &datasets);

    // Calculate per-format statistics
    let format_stats = calculate_format_stats(&run_results);

    // Generate and print legacy report
    let legacy_report = aggregate_results(&model, all_results.clone());
    println!("{}", legacy_report.report());

    // Print summary comparison
    print_summary(&args, &legacy_report, &format_stats);

    // Generate and export modern report
    export_report(&args, &model, &legacy_report, &all_results, &datasets);
}

fn print_dry_run_info(args: &Args, datasets: &[hedl_bench::accuracy::FixtureDataset]) {
    println!("DRY RUN - Showing test plan:\n");
    for ds in datasets {
        println!("Dataset: {}", ds.name);
        println!("  Questions: {}", ds.questions.len());
        println!("  HEDL size: {} bytes", ds.hedl_data.len());
        let toon_len = ds.toon_data.as_ref().map(|t| t.len()).unwrap_or(0);
        let json_len = ds.json_data.as_ref().map(|j| j.len()).unwrap_or(1); // avoid div by zero
        println!(
            "  TOON size: {} bytes ({:.1}% of JSON)",
            toon_len,
            toon_len as f64 / json_len as f64 * 100.0
        );
        println!(
            "  JSON size: {} bytes",
            ds.json_data.as_ref().map(|j| j.len()).unwrap_or(0)
        );
        println!(
            "  YAML size: {} bytes",
            ds.yaml_data.as_ref().map(|y| y.len()).unwrap_or(0)
        );
        println!(
            "  XML size:  {} bytes",
            ds.xml_data.as_ref().map(|x| x.len()).unwrap_or(0)
        );
        println!(
            "  CSV size:  {} bytes",
            ds.csv_data.as_ref().map(|c| c.len()).unwrap_or(0)
        );
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
                let format_str = format!("{:?}", format).to_lowercase();
                let data = ds.data_for_format(&format_str).unwrap_or("");
                println!("=== {format} PROMPT ===");
                println!("{}", build_prompt(data, *format, q));
                println!();
            }
        }
    }
}

fn run_warmup(
    args: &Args,
    model: &str,
    api_key: &str,
    datasets: &[hedl_bench::accuracy::FixtureDataset],
) {
    println!("Running warmup iteration (results discarded)...");
    for ds in datasets.iter().take(1) {
        for format in args.formats.iter().take(1) {
            if let Some(q) = ds.questions.first() {
                let _ = run_test(&args.provider, model, api_key, ds, *format, q, false);
                std::thread::sleep(Duration::from_millis(args.delay_ms));
            }
        }
    }
    println!("Warmup complete.\n");
}

fn run_all_tests(
    args: &Args,
    model: &str,
    api_key: &str,
    datasets: &[hedl_bench::accuracy::FixtureDataset],
) -> accuracy_types::AllTestResults {
    let mut run_results: HashMap<(String, String), Vec<bool>> = HashMap::new();
    let mut all_results = Vec::new();

    for ds in datasets {
        println!("Testing dataset: {}", ds.name);

        // Start with all questions
        let mut questions: Vec<&Question> = ds.questions.iter().collect();

        // Filter by question type if specified
        if !args.question_types.is_empty() {
            questions.retain(|q| {
                let q_type = format!("{:?}", q.question_type).to_lowercase();
                args.question_types.iter().any(|t| q_type.contains(t))
            });
        }

        // Filter by specific question IDs if specified
        if !args.question_ids.is_empty() {
            questions.retain(|q| {
                args.question_ids
                    .iter()
                    .any(|id| q.id.to_lowercase().contains(&id.to_lowercase()))
            });
        }

        // Apply max per category limit
        let questions: Vec<&Question> = if let Some(max) = args.max_per_category {
            // Group by type and take max per type
            let mut by_type: HashMap<QuestionType, Vec<&Question>> = HashMap::new();
            for q in questions {
                by_type.entry(q.question_type).or_default().push(q);
            }
            by_type
                .values()
                .flat_map(|qs| qs.iter().take(max).copied())
                .collect()
        } else {
            questions
        };

        if questions.is_empty() {
            println!("  No questions match filters, skipping...\n");
            continue;
        }

        for format in &args.formats {
            print!("  [{format}] ");
            let _ = io::stdout().flush();

            let mut format_correct = 0;
            let mut format_total = 0;

            for q in &questions {
                let mut q_correct_count = 0;

                // Run multiple iterations for this question
                for run_idx in 0..args.runs {
                    let result = run_test(
                        &args.provider,
                        model,
                        api_key,
                        ds,
                        *format,
                        q,
                        args.verbose && run_idx == 0, // Only verbose on first run
                    );

                    if result.correct {
                        q_correct_count += 1;
                    }

                    // Track per-run results for statistics
                    let key = (format.to_string(), format!("{}:{}", ds.name, q.id));
                    run_results.entry(key).or_default().push(result.correct);

                    // Only keep last run's result for overall reporting
                    if run_idx == args.runs - 1 {
                        all_results.push(result);
                    }

                    // Rate limiting between calls to avoid 429 errors
                    std::thread::sleep(Duration::from_millis(args.delay_ms));
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
                let _ = io::stdout().flush();

                format_total += 1;
            }

            let accuracy = f64::from(format_correct) / f64::from(format_total) * 100.0;
            println!(" {format_correct}/{format_total} ({accuracy:.1}%)");
        }
        println!();
    }

    (all_results, run_results)
}

fn calculate_format_stats(
    run_results: &HashMap<(String, String), Vec<bool>>,
) -> HashMap<String, (f64, f64, usize)> {
    let mut format_stats: HashMap<String, (f64, f64, usize)> = HashMap::new();

    // First, organize by format
    let mut format_questions: HashMap<String, Vec<(String, Vec<bool>)>> = HashMap::new();
    for (key, results) in run_results {
        format_questions
            .entry(key.0.clone())
            .or_default()
            .push((key.1.clone(), results.clone()));
    }

    // For each format, calculate accuracy per run, then mean/std
    for (format, questions) in &format_questions {
        let n_questions = questions.len();
        let n_runs = questions.first().map_or(0, |(_, r)| r.len());

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

    format_stats
}

fn print_summary(
    args: &Args,
    legacy_report: &accuracy_types::AccuracyReport,
    format_stats: &HashMap<String, (f64, f64, usize)>,
) {
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
            let acc_per_1k = (r.correct as f64 / r.total as f64) / (tokens_per_q / 1000.0);

            if args.runs > 1 {
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
                        "{name}     {acc:>5.1}% ± N/A           {tokens_per_q:>5.0}       {acc_per_1k:.2}"
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

    // Print format comparisons
    if let Some(hedl) = hedl_results {
        let hedl_acc = hedl.correct as f64 / hedl.total as f64 * 100.0;
        let hedl_tokens_per_q = hedl.total_tokens as f64 / hedl.total as f64;

        // HEDL vs TOON comparison
        if let Some(toon) = toon_results {
            let toon_acc = toon.correct as f64 / toon.total as f64 * 100.0;
            let diff = hedl_acc - toon_acc;
            if diff > 0.0 {
                println!("✓ HEDL outperforms TOON by {diff:.1} percentage points");
            } else if diff < 0.0 {
                println!("✗ TOON outperforms HEDL by {:.1} percentage points", -diff);
            } else {
                println!("= HEDL and TOON have equal accuracy");
            }

            let toon_tokens_per_q = toon.total_tokens as f64 / toon.total as f64;
            let toon_token_diff = (1.0 - hedl_tokens_per_q / toon_tokens_per_q) * 100.0;
            if toon_token_diff > 0.0 {
                println!("✓ HEDL uses {toon_token_diff:.1}% fewer tokens than TOON");
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
                println!("✓ HEDL outperforms JSON by {diff:.1} percentage points");
            } else if diff < 0.0 {
                println!("✗ JSON outperforms HEDL by {:.1} percentage points", -diff);
            } else {
                println!("= HEDL and JSON have equal accuracy");
            }

            let json_tokens_per_q = json.total_tokens as f64 / json.total as f64;
            let json_token_diff = (1.0 - hedl_tokens_per_q / json_tokens_per_q) * 100.0;
            if json_token_diff > 0.0 {
                println!("✓ HEDL uses {json_token_diff:.1}% fewer tokens than JSON");
            } else if json_token_diff < 0.0 {
                println!(
                    "✗ JSON uses {:.1}% fewer tokens than HEDL",
                    -json_token_diff
                );
            }
        }
    }

    // Difficulty breakdown
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

        println!("{difficulty} datasets:");

        if let Some(hedl) = hedl_diff {
            let tokens_per_q = if hedl.total > 0 {
                hedl.total_tokens as f64 / hedl.total as f64
            } else {
                0.0
            };
            print!("  HEDL: {tokens_per_q:.0} tokens/q");

            if let Some(toon) = toon_diff {
                let toon_tpq = if toon.total > 0 {
                    toon.total_tokens as f64 / toon.total as f64
                } else {
                    0.0
                };
                let diff = (1.0 - tokens_per_q / toon_tpq) * 100.0;
                if diff > 0.0 {
                    print!(" ({diff:.1}% less than TOON)");
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
            println!("  TOON: {tokens_per_q:.0} tokens/q");
        }

        if let Some(json) = json_diff {
            let tokens_per_q = if json.total > 0 {
                json.total_tokens as f64 / json.total as f64
            } else {
                0.0
            };
            println!("  JSON: {tokens_per_q:.0} tokens/q");
        }
        println!();
    }

    println!("═══════════════════════════════════════════════════════════════════\n");
}

fn export_report(
    args: &Args,
    model: &str,
    legacy_report: &accuracy_types::AccuracyReport,
    all_results: &[accuracy_types::TestResult],
    datasets: &[hedl_bench::accuracy::FixtureDataset],
) {
    let mut new_report = BenchmarkReport::new(format!("HEDL LLM Accuracy Testing - {model} Model"));
    new_report.set_timestamp();

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

    // Create Token Usage and Reasoning table
    let mut token_reasoning_table = CustomTable {
        title: "Token Usage and Reasoning Analysis".to_string(),
        headers: vec![
            "Format".to_string(),
            "Total Tokens".to_string(),
            "Thinking Tokens".to_string(),
            "Think/Total %".to_string(),
            "Avg Think/Q".to_string(),
            "Retries".to_string(),
            "Retry Rate %".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    for format_result in &legacy_report.results_by_format {
        let think_pct = if format_result.total_tokens > 0 {
            (format_result.total_thinking_tokens as f64 / format_result.total_tokens as f64) * 100.0
        } else {
            0.0
        };
        let avg_think_per_q = if format_result.total > 0 {
            format_result.total_thinking_tokens as f64 / format_result.total as f64
        } else {
            0.0
        };
        let retry_rate = if format_result.total > 0 {
            (format_result.questions_with_retries as f64 / format_result.total as f64) * 100.0
        } else {
            0.0
        };

        token_reasoning_table.rows.push(vec![
            TableCell::String(format_result.format.clone()),
            TableCell::Integer(format_result.total_tokens as i64),
            TableCell::Integer(format_result.total_thinking_tokens as i64),
            TableCell::Float(think_pct),
            TableCell::Float(avg_think_per_q),
            TableCell::Integer(format_result.total_retries as i64),
            TableCell::Float(retry_rate),
        ]);
    }
    new_report.add_custom_table(token_reasoning_table);

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

    generate_insights(legacy_report, &mut new_report, all_results, datasets);

    if args.warmup {
        new_report.add_note("Warmup run completed to eliminate cold-start effects");
    }

    if let Some(max) = args.max_per_category {
        new_report.add_note(format!(
            "Limited to {max} questions per category for faster testing"
        ));
    }

    // Export reports
    let target_dir = format!("{}/target", env!("CARGO_MANIFEST_DIR"));
    let base_path = format!("{target_dir}/accuracy_report");

    let export_config = ExportConfig::all();

    match new_report.save_all(&base_path, &export_config) {
        Ok(()) => {
            println!("\n╔════════════════════════════════════════════════════════════════╗");
            println!("║                    REPORTS EXPORTED                            ║");
            println!("╚════════════════════════════════════════════════════════════════╝\n");
        }
        Err(e) => {
            eprintln!("Warning: Failed to export some reports: {e}");
        }
    }
}
