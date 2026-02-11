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

//! Command line argument parsing for accuracy testing

use hedl_bench::accuracy::{DataFormat, LlmProvider};
use std::env;

/// Command line arguments
pub struct Args {
    pub provider: LlmProvider,
    pub model: Option<String>,
    pub formats: Vec<DataFormat>,
    pub max_per_category: Option<usize>,
    pub dry_run: bool,
    pub verbose: bool,
    /// Number of runs per question for statistical significance
    pub runs: usize,
    /// Whether to run a warmup iteration (discarded)
    pub warmup: bool,
    /// Delay between API calls in milliseconds (default: 1500 for rate limiting)
    pub delay_ms: u64,
    /// Filter to specific fixture(s) by name
    pub fixtures: Vec<String>,
    /// Filter to specific question type(s)
    pub question_types: Vec<String>,
    /// Filter to specific question ID(s)
    pub question_ids: Vec<String>,
    /// List available fixtures and exit
    pub list_fixtures: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            provider: LlmProvider::DeepSeek,
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
            delay_ms: 1500,       // Default 1.5s delay to respect rate limits (1 RPS)
            fixtures: Vec::new(), // Empty = all fixtures
            question_types: Vec::new(), // Empty = all types
            question_ids: Vec::new(), // Empty = all questions
            list_fixtures: false,
        }
    }
}

/// Parse command-line arguments for the accuracy benchmark.
pub fn parse_args() -> Args {
    let mut args = Args::default();
    let mut argv: Vec<String> = env::args().skip(1).collect();

    while !argv.is_empty() {
        let arg = argv.remove(0);
        match arg.as_str() {
            "--provider" | "-p" => {
                if let Some(val) = argv.first() {
                    args.provider = match val.to_lowercase().as_str() {
                        "deepseek" => LlmProvider::DeepSeek,
                        "mistral" => LlmProvider::Mistral,
                        "openai" => LlmProvider::OpenAI,
                        "anthropic" | "claude" => LlmProvider::Anthropic,
                        "glm" | "zhipu" | "chatglm" => LlmProvider::Glm,
                        "kimi" | "moonshot" => LlmProvider::Kimi,
                        "nvidia" | "nim" => LlmProvider::Nvidia,
                        _ => {
                            eprintln!(
                                "Unknown provider: {val}. Use 'deepseek', 'mistral', 'openai', 'anthropic', 'glm', 'kimi', or 'nvidia'"
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
                            eprintln!("Unknown format: {val}. Use hedl/toon/json/yaml/xml/csv");
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
            "--delay" => {
                if let Some(val) = argv.first() {
                    args.delay_ms = val.parse().unwrap_or(1500);
                    if args.delay_ms < 100 {
                        args.delay_ms = 100; // Minimum 100ms
                    }
                    argv.remove(0);
                }
            }
            "--fixture" | "-F" => {
                if let Some(val) = argv.first() {
                    args.fixtures.push(val.clone());
                    argv.remove(0);
                }
            }
            "--type" | "-t" => {
                if let Some(val) = argv.first() {
                    args.question_types.push(val.to_lowercase());
                    argv.remove(0);
                }
            }
            "--question" | "-q" => {
                if let Some(val) = argv.first() {
                    args.question_ids.push(val.clone());
                    argv.remove(0);
                }
            }
            "--list" | "-l" => {
                args.list_fixtures = true;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {arg}");
                print_help();
                std::process::exit(1);
            }
        }
    }

    args
}

/// Print usage help text and available options.
pub fn print_help() {
    println!(
        r"HEDL LLM Accuracy Testing

USAGE:
    cargo run --package hedl-bench --bin accuracy [OPTIONS]

OPTIONS:
    -p, --provider <PROVIDER>     LLM provider [default: deepseek]
                                  Providers: deepseek, mistral, openai, anthropic, glm, kimi, nvidia
    -m, --model <MODEL>           Model name [default: provider's default]
    -f, --format <FORMAT>         Format to test (can repeat): hedl, toon, json, yaml, xml, csv
    -F, --fixture <NAME>          Filter to specific fixture(s) (can repeat)
                                  Fixtures: ecommerce_orders, blog_platform, financial_transactions,
                                           healthcare_records, iot_sensors, ml_training_logs, sports_statistics
    -t, --type <TYPE>             Filter to specific question type(s) (can repeat)
                                  Types: field_retrieval, aggregation, filtering, hierarchical,
                                        comparison, temporal, validation
    -n, --max-per-category <N>    Max questions per category
    -q, --question <ID>           Run specific question(s) by ID (can repeat)
    -r, --runs <N>                Number of runs per question [default: 3]
    -w, --warmup                  Run one warmup iteration (discarded)
    --delay <MS>                  Delay between API calls in ms [default: 1500]
    -d, --dry-run                 Don't call API, just show what would be tested
    -v, --verbose                 Show each question and answer
    -l, --list                    List available fixtures and exit
    -h, --help                    Print help

ENVIRONMENT:
    DEEPSEEK_API_KEY              Required for DeepSeek provider
    MISTRAL_API_KEY               Required for Mistral provider
    OPENAI_API_KEY                Required for OpenAI provider
    ANTHROPIC_API_KEY             Required for Anthropic provider
    GLM_API_KEY                   Required for GLM/Zhipu provider
    KIMI_API_KEY                  Required for KIMI/Moonshot provider
    NVIDIA_API_KEY                Required for NVIDIA Build API

EXAMPLES:
    # Full test with DeepSeek (3 runs per question, default)
    cargo run --package hedl-bench --bin accuracy

    # Test specific fixture with Mistral
    cargo run --package hedl-bench --bin accuracy -- -p mistral -F ecommerce_orders

    # Test HEDL vs TOON only on one fixture
    cargo run --package hedl-bench --bin accuracy -- -F ecommerce_orders -f hedl -f toon -v -r 1

    # Test with Anthropic Claude
    cargo run --package hedl-bench --bin accuracy -- -p anthropic

    # Single run (faster, less statistically robust)
    cargo run --package hedl-bench --bin accuracy -- -r 1

    # Quick test with 3 questions per category
    cargo run --package hedl-bench --bin accuracy -- -n 3

    # List available fixtures
    cargo run --package hedl-bench --bin accuracy -- -l

    # Dry run to see test plan
    cargo run --package hedl-bench --bin accuracy -- --dry-run
"
    );
}
