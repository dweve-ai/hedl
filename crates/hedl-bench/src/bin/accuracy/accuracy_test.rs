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

//! Test execution logic

use crate::accuracy_llm::{call_llm_with_retry, MAX_RETRIES};
use crate::accuracy_types::TestResult;
use hedl_bench::accuracy::{
    build_prompt, compare, DataFormat, FixtureDataset, LlmProvider, Question,
};
use hedl_bench::token_counter::count_tokens;

/// Run a single accuracy test
pub fn run_test(
    provider: &LlmProvider,
    model: &str,
    api_key: &str,
    dataset: &FixtureDataset,
    format: DataFormat,
    question: &Question,
    verbose: bool,
) -> TestResult {
    let format_str = format!("{:?}", format).to_lowercase();
    let data = dataset.data_for_format(&format_str).unwrap_or("");
    let prompt = build_prompt(data, format, question);

    // Count tokens on DATA ONLY (not the full prompt with instructions)
    // This measures true format efficiency
    let data_tokens = count_tokens(data);

    // Get format-specific ground truth (falls back to default if not specified)
    let format_str = format.to_string().to_lowercase();
    let expected = question.ground_truth_for_format(&format_str);

    match call_llm_with_retry(provider, model, api_key, &prompt) {
        Ok((response, retry_count)) => {
            let correct =
                compare(expected, &response.content, &question.answer_type).unwrap_or(false);

            if verbose {
                let status = if correct { "✓" } else { "✗" };
                let retry_info = if retry_count > 0 {
                    format!(" (retries: {})", retry_count)
                } else {
                    String::new()
                };
                println!(
                    "  {} [{}] {} -> {} (expected: {}){}",
                    status, format, question.prompt, response.content, expected, retry_info
                );
            }

            TestResult {
                question_type: question.question_type,
                expected: expected.to_string(),
                actual: response.content,
                correct,
                format: format.to_string(),
                difficulty: format!("{:?}", dataset.complexity),
                latency_ms: response.latency_ms,
                tokens_in: data_tokens, // DATA tokens only!
                tokens_out: response.tokens_out,
                tokens_thinking: response.tokens_thinking,
                retry_count,
            }
        }
        Err(e) => {
            if verbose {
                println!("  ✗ [{}] {} -> ERROR: {}", format, question.prompt, e);
            }

            TestResult {
                question_type: question.question_type,
                expected: expected.to_string(),
                actual: format!("ERROR: {e}"),
                correct: false,
                format: format.to_string(),
                difficulty: format!("{:?}", dataset.complexity),
                latency_ms: 0,
                tokens_in: data_tokens, // Still count data tokens
                tokens_out: 0,
                tokens_thinking: 0,
                retry_count: MAX_RETRIES, // All retries exhausted
            }
        }
    }
}
