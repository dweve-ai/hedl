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

//! # HEDL Accuracy Benchmark Framework v2.0
//!
//! A comprehensive LLM accuracy testing framework that surpasses TOON's methodology:
//!
//! ## Improvements Over TOON
//!
//! | Feature | TOON | HEDL v2 |
//! |---------|------|---------|
//! | Questions | 209 | 500+ |
//! | Question Types | 5 | 12 |
//! | Complexity Levels | 5 | 5 |
//! | LLM Providers | 4 | 8 |
//! | Domains | 1 | 8 |
//! | Statistical Rigor | Basic | Full CI/Effect Size |
//! | Blind Evaluation | No | Yes |
//! | Reproducibility | Unknown | Seeded RNG |
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                     Accuracy Benchmark Framework                     │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐ │
//! │  │  Questions  │  │  Datasets   │  │  Providers  │  │ Statistics │ │
//! │  │   (500+)    │  │ (8 domains) │  │  (6 LLMs)   │  │  (full CI) │ │
//! │  └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘ │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │                           Execution Engine                          │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐ │
//! │  │   Blind     │  │   Batch     │  │   Cache     │  │   Report   │ │
//! │  │  Evaluator  │  │  Executor   │  │   Manager   │  │  Generator │ │
//! │  └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘ │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

/// 5-Level Progressive Complexity System.
pub mod complexity;
/// 8 Domain-Specific Dataset Generators.
pub mod domains;
/// Edge Case and Adversarial Test Suites.
pub mod edge_cases;
/// Fixture Dataset Loading for Accuracy Benchmarks.
pub mod fixtures;
/// Answer normalization for type-aware comparison.
pub mod normalize;
/// Prompt generation for LLM accuracy evaluation.
pub mod prompts;
/// LLM Provider Support for Accuracy Benchmarks.
pub mod providers;
/// Question Types and Corpus for LLM Accuracy Benchmarks.
pub mod questions;
/// Comprehensive comparison report generator for accuracy benchmarks.
pub mod reports;
/// Statistical Analysis for Accuracy Benchmarks.
pub mod statistics;

pub use complexity::{ComplexityLevel, ComplexityProfile};
pub use domains::{Domain, DomainDataset};
pub use edge_cases::{EdgeCaseCategory, EdgeCaseGenerator};
pub use fixtures::{load_all_fixtures, load_default_fixtures, FixtureDataset};
pub use normalize::{compare, normalize};
pub use prompts::build_prompt;
pub use providers::{LlmProvider, ProviderConfig};
pub use questions::{Question, QuestionCorpus, QuestionType};
pub use reports::{AccuracyReport, DataFormat, OutputFormat, QuestionResult, ReportGenerator};
pub use statistics::{ConfidenceInterval, EffectSize, StatisticalResult};
