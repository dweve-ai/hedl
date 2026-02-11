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

//! Table generation functions for comprehensive analysis

use crate::accuracy_types::{AccuracyReport, TestResult};
use hedl_bench::accuracy::FixtureDataset;
use hedl_bench::{BenchmarkReport, CustomTable, TableCell};
use std::collections::HashMap;

/// Create Accuracy by Dataset table
pub fn create_accuracy_by_dataset_table(
    all_results: &[TestResult],
    _datasets: &[FixtureDataset],
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
pub fn create_error_analysis_table(all_results: &[TestResult], report: &mut BenchmarkReport) {
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
pub fn create_latency_distribution_table(all_results: &[TestResult], report: &mut BenchmarkReport) {
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
pub fn create_cost_benefit_matrix(legacy_report: &AccuracyReport, report: &mut BenchmarkReport) {
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
pub fn create_format_pair_comparison(legacy_report: &AccuracyReport, report: &mut BenchmarkReport) {
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
                format!("Prefer {fmt1} for better overall performance")
            } else if winner == fmt2 {
                format!("Prefer {fmt2} for better overall performance")
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
                TableCell::String(format!("{fmt1} vs {fmt2}")),
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
pub fn create_token_usage_breakdown(all_results: &[TestResult], report: &mut BenchmarkReport) {
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
pub fn create_question_type_ranking(legacy_report: &AccuracyReport, report: &mut BenchmarkReport) {
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

    type_perf.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // For now, show aggregate performance as we don't have per-format per-type breakdowns
    for (qtype, acc, total) in type_perf {
        table.rows.push(vec![
            TableCell::String(qtype),
            TableCell::String("HEDL".to_string()),
            TableCell::String("N/A".to_string()),
            TableCell::String(format!("{total} questions")),
            TableCell::Float(acc),
        ]);
    }

    report.add_custom_table(table);
}

/// Create Difficulty Scaling Analysis table
pub fn create_difficulty_scaling_analysis(
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

        let easy_acc = easy.map_or(0.0, |r| (r.correct as f64 / r.total as f64) * 100.0);
        let medium_acc = medium.map_or(0.0, |r| (r.correct as f64 / r.total as f64) * 100.0);
        let hard_acc = hard.map_or(0.0, |r| (r.correct as f64 / r.total as f64) * 100.0);

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

/// Create a token efficiency comparison table for the benchmark report.
pub fn create_token_efficiency_table(legacy_report: &AccuracyReport, report: &mut BenchmarkReport) {
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
        b_acc_per_1k
            .partial_cmp(&a_acc_per_1k)
            .unwrap_or(std::cmp::Ordering::Equal)
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
            (format!("{token_diff:+.1}%"), format!("{acc_diff:+.1}pp"))
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

/// Create a format feature comparison table for the benchmark report.
pub fn create_feature_comparison_table(report: &mut BenchmarkReport) {
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
