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

//! Report types and aggregation for accuracy testing

use hedl_bench::accuracy::QuestionType;
use std::collections::HashMap;

/// Results from running all accuracy tests: test results and per-run pass/fail tracking.
pub type AllTestResults = (Vec<TestResult>, HashMap<(String, String), Vec<bool>>);

/// Per-format aggregated metrics: (correct, total, total_latency_ms, total_tokens, thinking_tokens, total_retries, questions_with_retries)
type FormatAggregates = (usize, usize, u64, usize, usize, usize, usize);

/// Aggregated results for a test run
#[derive(Debug, Default)]
pub struct AccuracyReport {
    /// Model name tested.
    pub model: String,
    /// Total number of questions asked.
    pub total_questions: usize,
    /// Results aggregated by format.
    pub results_by_format: Vec<FormatResults>,
    /// Results aggregated by question type.
    pub results_by_type: Vec<TypeResults>,
    /// Results aggregated by difficulty level.
    pub results_by_difficulty: Vec<DifficultyResults>,
}

/// Aggregated accuracy results for a specific data format.
#[derive(Debug, Default)]
pub struct FormatResults {
    /// Name of the format (e.g., "HEDL", "JSON", "YAML").
    pub format: String,
    /// Number of correctly answered questions.
    pub correct: usize,
    /// Total number of questions asked.
    pub total: usize,
    /// Average latency in milliseconds per question.
    pub avg_latency_ms: f64,
    /// Total tokens used (input + output).
    pub total_tokens: usize,
    /// Total thinking/reasoning tokens.
    pub total_thinking_tokens: usize,
    /// Total number of retries across all questions.
    pub total_retries: usize,
    /// Number of questions that required at least one retry.
    pub questions_with_retries: usize,
}

/// Aggregated accuracy results for a specific question type.
#[derive(Debug, Default)]
pub struct TypeResults {
    /// Name of the question type category.
    pub question_type: String,
    /// Number of correctly answered questions.
    pub correct: usize,
    /// Total number of questions in this category.
    pub total: usize,
}

/// Aggregated results for a specific format+difficulty combination.
#[derive(Debug, Default)]
pub struct DifficultyResults {
    /// Format name (e.g., "HEDL", "JSON").
    pub format: String,
    /// Difficulty level.
    pub difficulty: String,
    /// Number of correctly answered questions.
    pub correct: usize,
    /// Total number of questions.
    pub total: usize,
    /// Total tokens used (input + output).
    pub total_tokens: usize,
}

/// Result of a single test (question + format combination).
#[derive(Debug, Clone)]
pub struct TestResult {
    pub question_type: QuestionType,
    pub format: String,
    pub expected: String,
    pub actual: String,
    pub correct: bool,
    pub latency_ms: u64,
    pub difficulty: String,
    /// Input tokens (data only, not prompt overhead)
    pub tokens_in: usize,
    /// Output tokens from LLM response
    pub tokens_out: usize,
    /// Thinking/reasoning tokens (for models that support extended thinking)
    pub tokens_thinking: usize,
    /// Number of retry attempts before getting a valid response
    pub retry_count: usize,
}

impl AccuracyReport {
    /// Generate a formatted report
    pub fn report(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("\n{}\n", "=".repeat(100)));
        out.push_str(&format!("LLM Accuracy Report - Model: {}\n", self.model));
        out.push_str(&format!("{}\n\n", "=".repeat(100)));

        out.push_str("Results by Format:\n");
        out.push_str(&format!("{:-<100}\n", ""));
        out.push_str(&format!(
            "{:<10} {:>8} {:>8} {:>10} {:>12} {:>10} {:>12} {:>10}\n",
            "Format",
            "Correct",
            "Total",
            "Accuracy",
            "Avg Latency",
            "Tokens",
            "Think Toks",
            "Retries"
        ));
        out.push_str(&format!("{:-<100}\n", ""));

        for fr in &self.results_by_format {
            let accuracy = if fr.total > 0 {
                fr.correct as f64 / fr.total as f64 * 100.0
            } else {
                0.0
            };
            let retry_pct = if fr.total > 0 {
                fr.questions_with_retries as f64 / fr.total as f64 * 100.0
            } else {
                0.0
            };
            out.push_str(&format!(
                "{:<10} {:>8} {:>8} {:>9.1}% {:>10.0}ms {:>10} {:>12} {:>7} ({:.1}%)\n",
                fr.format,
                fr.correct,
                fr.total,
                accuracy,
                fr.avg_latency_ms,
                fr.total_tokens,
                fr.total_thinking_tokens,
                fr.total_retries,
                retry_pct
            ));
        }

        out.push_str("\nResults by Question Type:\n");
        out.push_str(&format!("{:-<50}\n", ""));
        out.push_str(&format!(
            "{:<25} {:>10} {:>10} {:>12}\n",
            "Type", "Correct", "Total", "Accuracy"
        ));
        out.push_str(&format!("{:-<50}\n", ""));

        for tr in &self.results_by_type {
            let accuracy = if tr.total > 0 {
                tr.correct as f64 / tr.total as f64 * 100.0
            } else {
                0.0
            };
            out.push_str(&format!(
                "{:<25} {:>10} {:>10} {:>11.1}%\n",
                tr.question_type, tr.correct, tr.total, accuracy
            ));
        }

        out.push_str(&format!("\n{}\n", "=".repeat(100)));

        out
    }
}

/// Aggregate results into a report
pub fn aggregate_results(model: &str, results: Vec<TestResult>) -> AccuracyReport {
    // (correct, total, latency, tokens, thinking_tokens, total_retries, questions_with_retries)
    let mut by_format: HashMap<String, FormatAggregates> = HashMap::new();
    let mut by_type: HashMap<String, (usize, usize)> = HashMap::new();
    // (format, difficulty) -> (correct, total, tokens)
    let mut by_difficulty: HashMap<(String, String), (usize, usize, usize)> = HashMap::new();

    for r in &results {
        let format_key = r.format.to_string();
        let entry = by_format
            .entry(format_key.clone())
            .or_insert((0, 0, 0, 0, 0, 0, 0));
        entry.1 += 1; // total
        if r.correct {
            entry.0 += 1; // correct
        }
        entry.2 += r.latency_ms; // total latency
        entry.3 += r.tokens_in + r.tokens_out; // total tokens
        entry.4 += r.tokens_thinking; // total thinking tokens
        entry.5 += r.retry_count; // total retries
        if r.retry_count > 0 {
            entry.6 += 1; // questions with retries
        }

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
            |(
                format,
                (correct, total, latency, tokens, thinking_tokens, total_retries, with_retries),
            )| FormatResults {
                format,
                correct,
                total,
                avg_latency_ms: if total > 0 {
                    latency as f64 / total as f64
                } else {
                    0.0
                },
                total_tokens: tokens,
                total_thinking_tokens: thinking_tokens,
                total_retries,
                questions_with_retries: with_retries,
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
