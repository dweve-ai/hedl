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

//! Comprehensive comparison report generator for accuracy benchmarks.
//!
//! Generates detailed reports comparing HEDL against other formats across:
//! - Question types (12 categories)
//! - Complexity levels (L1-L5)
//! - Domains (8 specialized areas)
//! - LLM providers (6 models)
//! - Edge cases (10 adversarial categories)
//!
//! Output formats: Console, JSON, Markdown, HTML

use std::collections::HashMap;

use crate::accuracy::{
    complexity::ComplexityLevel,
    domains::Domain,
    providers::LlmProvider,
    questions::QuestionType,
    statistics::{ConfidenceInterval, EffectSize},
};

/// Supported data format for benchmarking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataFormat {
    /// HEDL (Hierarchical Entity Data Language)
    Hedl,
    /// TOON (Token-Oriented Object Notation)
    Toon,
    /// JSON (JavaScript Object Notation)
    Json,
    /// YAML (YAML Ain't Markup Language)
    Yaml,
    /// XML (eXtensible Markup Language)
    Xml,
    /// CSV (Comma-Separated Values)
    Csv,
    /// TOML (Tom's Obvious Minimal Language)
    Toml,
    /// Markdown tables
    Markdown,
}

impl DataFormat {
    /// All formats for benchmarking
    pub fn all() -> &'static [DataFormat] {
        &[
            DataFormat::Hedl,
            DataFormat::Toon,
            DataFormat::Json,
            DataFormat::Yaml,
            DataFormat::Xml,
            DataFormat::Csv,
            DataFormat::Toml,
            DataFormat::Markdown,
        ]
    }

    /// Format name for display
    pub fn name(&self) -> &'static str {
        match self {
            DataFormat::Hedl => "HEDL",
            DataFormat::Toon => "TOON",
            DataFormat::Json => "JSON",
            DataFormat::Yaml => "YAML",
            DataFormat::Xml => "XML",
            DataFormat::Csv => "CSV",
            DataFormat::Toml => "TOML",
            DataFormat::Markdown => "Markdown",
        }
    }
}

impl std::fmt::Display for DataFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Result for a single question evaluation
#[derive(Debug, Clone)]
pub struct QuestionResult {
    /// Question identifier
    pub question_id: String,
    /// Question type category
    pub question_type: QuestionType,
    /// Complexity level
    pub complexity: ComplexityLevel,
    /// Domain if applicable
    pub domain: Option<Domain>,
    /// Data format used
    pub format: DataFormat,
    /// LLM provider
    pub provider: LlmProvider,
    /// Whether answer was correct
    pub correct: bool,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Token count for input
    pub input_tokens: u32,
    /// Token count for output
    pub output_tokens: u32,
}

/// Aggregated results for a specific dimension
#[derive(Debug, Clone)]
pub struct DimensionResult {
    /// Name of this dimension value (e.g., "FieldRetrieval", "L1")
    pub name: String,
    /// Correct count
    pub correct: u32,
    /// Total count
    pub total: u32,
    /// Accuracy as fraction
    pub accuracy: f64,
    /// 95% confidence interval
    pub confidence_interval: ConfidenceInterval,
    /// Mean latency in ms
    pub mean_latency_ms: f64,
    /// Mean input tokens
    pub mean_input_tokens: f64,
}

impl DimensionResult {
    /// Create from a slice of results
    pub fn from_results(name: &str, results: &[&QuestionResult]) -> Self {
        let total = results.len() as u32;
        let correct = results.iter().filter(|r| r.correct).count() as u32;
        let accuracy = if total > 0 {
            correct as f64 / total as f64
        } else {
            0.0
        };

        let mean_latency_ms = if !results.is_empty() {
            results.iter().map(|r| r.latency_ms as f64).sum::<f64>() / results.len() as f64
        } else {
            0.0
        };

        let mean_input_tokens = if !results.is_empty() {
            results.iter().map(|r| r.input_tokens as f64).sum::<f64>() / results.len() as f64
        } else {
            0.0
        };

        DimensionResult {
            name: name.to_string(),
            correct,
            total,
            accuracy,
            confidence_interval: ConfidenceInterval::wilson_score(
                correct as usize,
                total as usize,
                0.95,
            ),
            mean_latency_ms,
            mean_input_tokens,
        }
    }
}

/// Comparison between HEDL and another format
#[derive(Debug, Clone)]
pub struct FormatComparison {
    /// Format being compared to HEDL
    pub other_format: DataFormat,
    /// HEDL accuracy
    pub hedl_accuracy: f64,
    /// Other format accuracy
    pub other_accuracy: f64,
    /// Absolute improvement (HEDL - other)
    pub absolute_improvement: f64,
    /// Relative improvement percentage
    pub relative_improvement_pct: f64,
    /// Effect size (Cohen's d)
    pub effect_size: EffectSize,
    /// Statistical significance (p-value)
    pub p_value: f64,
    /// Whether difference is statistically significant (p < 0.05)
    pub significant: bool,
}

/// Complete benchmark report
#[derive(Debug, Clone)]
pub struct AccuracyReport {
    /// Report title
    pub title: String,
    /// Total questions evaluated
    pub total_questions: u32,
    /// Overall results by format
    pub by_format: HashMap<DataFormat, DimensionResult>,
    /// Results by question type and format
    pub by_question_type: HashMap<QuestionType, HashMap<DataFormat, DimensionResult>>,
    /// Results by complexity level and format
    pub by_complexity: HashMap<ComplexityLevel, HashMap<DataFormat, DimensionResult>>,
    /// Results by domain and format
    pub by_domain: HashMap<Domain, HashMap<DataFormat, DimensionResult>>,
    /// Results by provider and format
    pub by_provider: HashMap<LlmProvider, HashMap<DataFormat, DimensionResult>>,
    /// HEDL vs other format comparisons
    pub hedl_comparisons: Vec<FormatComparison>,
    /// Key insights generated from the data
    pub insights: Vec<String>,
}

impl AccuracyReport {
    /// Create report from raw results
    pub fn from_results(title: &str, results: &[QuestionResult]) -> Self {
        let total_questions = results.len() as u32;

        // Group by format
        let by_format = Self::aggregate_by_format(results);

        // Group by question type
        let by_question_type = Self::aggregate_by_question_type(results);

        // Group by complexity
        let by_complexity = Self::aggregate_by_complexity(results);

        // Group by domain
        let by_domain = Self::aggregate_by_domain(results);

        // Group by provider
        let by_provider = Self::aggregate_by_provider(results);

        // Generate HEDL comparisons
        let hedl_comparisons = Self::generate_comparisons(results, &by_format);

        // Generate insights
        let insights = Self::generate_insights(&by_format, &hedl_comparisons);

        AccuracyReport {
            title: title.to_string(),
            total_questions,
            by_format,
            by_question_type,
            by_complexity,
            by_domain,
            by_provider,
            hedl_comparisons,
            insights,
        }
    }

    fn aggregate_by_format(results: &[QuestionResult]) -> HashMap<DataFormat, DimensionResult> {
        let mut grouped: HashMap<DataFormat, Vec<&QuestionResult>> = HashMap::new();
        for r in results {
            grouped.entry(r.format).or_default().push(r);
        }

        grouped
            .into_iter()
            .map(|(format, res)| (format, DimensionResult::from_results(format.name(), &res)))
            .collect()
    }

    fn aggregate_by_question_type(
        results: &[QuestionResult],
    ) -> HashMap<QuestionType, HashMap<DataFormat, DimensionResult>> {
        let mut by_type: HashMap<QuestionType, Vec<&QuestionResult>> = HashMap::new();
        for r in results {
            by_type.entry(r.question_type).or_default().push(r);
        }

        by_type
            .into_iter()
            .map(|(qt, type_results)| {
                let mut by_format: HashMap<DataFormat, Vec<&QuestionResult>> = HashMap::new();
                for r in type_results {
                    by_format.entry(r.format).or_default().push(r);
                }

                let format_results = by_format
                    .into_iter()
                    .map(|(format, res)| {
                        (format, DimensionResult::from_results(format.name(), &res))
                    })
                    .collect();

                (qt, format_results)
            })
            .collect()
    }

    fn aggregate_by_complexity(
        results: &[QuestionResult],
    ) -> HashMap<ComplexityLevel, HashMap<DataFormat, DimensionResult>> {
        let mut by_level: HashMap<ComplexityLevel, Vec<&QuestionResult>> = HashMap::new();
        for r in results {
            by_level.entry(r.complexity).or_default().push(r);
        }

        by_level
            .into_iter()
            .map(|(level, level_results)| {
                let mut by_format: HashMap<DataFormat, Vec<&QuestionResult>> = HashMap::new();
                for r in level_results {
                    by_format.entry(r.format).or_default().push(r);
                }

                let format_results = by_format
                    .into_iter()
                    .map(|(format, res)| {
                        (format, DimensionResult::from_results(format.name(), &res))
                    })
                    .collect();

                (level, format_results)
            })
            .collect()
    }

    fn aggregate_by_domain(
        results: &[QuestionResult],
    ) -> HashMap<Domain, HashMap<DataFormat, DimensionResult>> {
        let mut by_domain: HashMap<Domain, Vec<&QuestionResult>> = HashMap::new();
        for r in results {
            if let Some(domain) = r.domain {
                by_domain.entry(domain).or_default().push(r);
            }
        }

        by_domain
            .into_iter()
            .map(|(domain, domain_results)| {
                let mut by_format: HashMap<DataFormat, Vec<&QuestionResult>> = HashMap::new();
                for r in domain_results {
                    by_format.entry(r.format).or_default().push(r);
                }

                let format_results = by_format
                    .into_iter()
                    .map(|(format, res)| {
                        (format, DimensionResult::from_results(format.name(), &res))
                    })
                    .collect();

                (domain, format_results)
            })
            .collect()
    }

    fn aggregate_by_provider(
        results: &[QuestionResult],
    ) -> HashMap<LlmProvider, HashMap<DataFormat, DimensionResult>> {
        let mut by_provider: HashMap<LlmProvider, Vec<&QuestionResult>> = HashMap::new();
        for r in results {
            by_provider.entry(r.provider).or_default().push(r);
        }

        by_provider
            .into_iter()
            .map(|(provider, provider_results)| {
                let mut by_format: HashMap<DataFormat, Vec<&QuestionResult>> = HashMap::new();
                for r in provider_results {
                    by_format.entry(r.format).or_default().push(r);
                }

                let format_results = by_format
                    .into_iter()
                    .map(|(format, res)| {
                        (format, DimensionResult::from_results(format.name(), &res))
                    })
                    .collect();

                (provider, format_results)
            })
            .collect()
    }

    fn generate_comparisons(
        results: &[QuestionResult],
        by_format: &HashMap<DataFormat, DimensionResult>,
    ) -> Vec<FormatComparison> {
        let hedl_result = match by_format.get(&DataFormat::Hedl) {
            Some(r) => r,
            None => return vec![],
        };

        let other_formats: Vec<DataFormat> = DataFormat::all()
            .iter()
            .filter(|f| **f != DataFormat::Hedl)
            .copied()
            .collect();

        other_formats
            .into_iter()
            .filter_map(|format| {
                let other_result = by_format.get(&format)?;

                let absolute_improvement = hedl_result.accuracy - other_result.accuracy;
                let relative_improvement_pct = if other_result.accuracy > 0.0 {
                    (absolute_improvement / other_result.accuracy) * 100.0
                } else {
                    0.0
                };

                // Calculate effect size using binary outcomes (1.0=correct, 0.0=incorrect)
                let hedl_scores: Vec<f64> = results
                    .iter()
                    .filter(|r| r.format == DataFormat::Hedl)
                    .map(|r| if r.correct { 1.0 } else { 0.0 })
                    .collect();

                let other_scores: Vec<f64> = results
                    .iter()
                    .filter(|r| r.format == format)
                    .map(|r| if r.correct { 1.0 } else { 0.0 })
                    .collect();

                let effect_size = EffectSize::from_groups(&hedl_scores, &other_scores);

                // Calculate p-value using chi-square test approximation
                let p_value = Self::chi_square_p_value(
                    hedl_result.correct,
                    hedl_result.total,
                    other_result.correct,
                    other_result.total,
                );

                Some(FormatComparison {
                    other_format: format,
                    hedl_accuracy: hedl_result.accuracy,
                    other_accuracy: other_result.accuracy,
                    absolute_improvement,
                    relative_improvement_pct,
                    effect_size,
                    p_value,
                    significant: p_value < 0.05,
                })
            })
            .collect()
    }

    fn chi_square_p_value(correct1: u32, total1: u32, correct2: u32, total2: u32) -> f64 {
        if total1 == 0 || total2 == 0 {
            return 1.0;
        }

        let p1 = correct1 as f64 / total1 as f64;
        let p2 = correct2 as f64 / total2 as f64;
        let p_pooled = (correct1 + correct2) as f64 / (total1 + total2) as f64;

        if p_pooled == 0.0 || p_pooled == 1.0 {
            return 1.0;
        }

        let se = (p_pooled * (1.0 - p_pooled) * (1.0 / total1 as f64 + 1.0 / total2 as f64)).sqrt();

        if se == 0.0 {
            return 1.0;
        }

        let z = (p1 - p2).abs() / se;

        // Approximate p-value from z-score (two-tailed)
        2.0 * (1.0 - Self::normal_cdf(z))
    }

    fn normal_cdf(z: f64) -> f64 {
        // Approximation of standard normal CDF
        0.5 * (1.0 + Self::erf(z / std::f64::consts::SQRT_2))
    }

    fn erf(x: f64) -> f64 {
        // Approximation of error function
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;

        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();

        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

        sign * y
    }

    fn generate_insights(
        by_format: &HashMap<DataFormat, DimensionResult>,
        comparisons: &[FormatComparison],
    ) -> Vec<String> {
        let mut insights = Vec::new();

        // Overall HEDL performance
        if let Some(hedl) = by_format.get(&DataFormat::Hedl) {
            insights.push(format!(
                "HEDL achieved {:.1}% overall accuracy (95% CI: {:.1}%-{:.1}%)",
                hedl.accuracy * 100.0,
                hedl.confidence_interval.lower * 100.0,
                hedl.confidence_interval.upper * 100.0
            ));
        }

        // Best comparison
        if let Some(best) = comparisons
            .iter()
            .filter(|c| c.significant && c.absolute_improvement > 0.0)
            .max_by(|a, b| {
                a.absolute_improvement
                    .partial_cmp(&b.absolute_improvement)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            insights.push(format!(
                "HEDL outperforms {} by {:.1} percentage points (p < {:.4})",
                best.other_format.name(),
                best.absolute_improvement * 100.0,
                best.p_value
            ));
        }

        // Significant improvements
        let sig_improvements: Vec<_> = comparisons
            .iter()
            .filter(|c| c.significant && c.absolute_improvement > 0.0)
            .collect();

        if !sig_improvements.is_empty() {
            insights.push(format!(
                "HEDL shows statistically significant improvement over {} of {} formats",
                sig_improvements.len(),
                comparisons.len()
            ));
        }

        // Token efficiency insight
        if let Some(hedl) = by_format.get(&DataFormat::Hedl) {
            if let Some(json) = by_format.get(&DataFormat::Json) {
                if hedl.mean_input_tokens < json.mean_input_tokens {
                    let reduction = (1.0 - hedl.mean_input_tokens / json.mean_input_tokens) * 100.0;
                    insights.push(format!(
                        "HEDL uses {:.0}% fewer input tokens than JSON while achieving higher accuracy",
                        reduction
                    ));
                }
            }
        }

        insights
    }
}

/// Report output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Console (ASCII art)
    Console,
    /// JSON
    Json,
    /// Markdown
    Markdown,
    /// HTML
    Html,
}

/// Report generator
pub struct ReportGenerator {
    /// Output format
    format: OutputFormat,
    /// Include detailed breakdowns
    detailed: bool,
    /// Include ASCII visualizations in console mode
    visualizations: bool,
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportGenerator {
    /// Create new report generator
    pub fn new() -> Self {
        ReportGenerator {
            format: OutputFormat::Console,
            detailed: true,
            visualizations: true,
        }
    }

    /// Set output format
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Enable/disable detailed breakdowns
    pub fn detailed(mut self, detailed: bool) -> Self {
        self.detailed = detailed;
        self
    }

    /// Enable/disable visualizations
    pub fn visualizations(mut self, visualizations: bool) -> Self {
        self.visualizations = visualizations;
        self
    }

    /// Generate report string
    pub fn generate(&self, report: &AccuracyReport) -> String {
        match self.format {
            OutputFormat::Console => self.generate_console(report),
            OutputFormat::Json => self.generate_json(report),
            OutputFormat::Markdown => self.generate_markdown(report),
            OutputFormat::Html => self.generate_html(report),
        }
    }

    fn generate_console(&self, report: &AccuracyReport) -> String {
        let mut output = String::new();

        // Title
        output.push_str(&self.console_header(&report.title));
        output.push('\n');

        // Summary
        output.push_str(&format!("Total Questions: {}\n\n", report.total_questions));

        // Overall accuracy table
        output.push_str(&self.console_section("Overall Accuracy by Format"));
        output.push_str(&self.console_accuracy_table(&report.by_format));
        output.push('\n');

        // Visualizations
        if self.visualizations {
            output.push_str(&self.console_section("Accuracy Comparison"));
            output.push_str(&self.console_bar_chart(&report.by_format));
            output.push('\n');
        }

        // HEDL comparisons
        output.push_str(&self.console_section("HEDL vs Other Formats"));
        output.push_str(&self.console_comparison_table(&report.hedl_comparisons));
        output.push('\n');

        // Detailed breakdowns
        if self.detailed {
            // By question type
            output.push_str(&self.console_section("Accuracy by Question Type"));
            output.push_str(
                &self.console_dimension_table(&report.by_question_type, |qt| format!("{:?}", qt)),
            );
            output.push('\n');

            // By complexity
            output.push_str(&self.console_section("Accuracy by Complexity Level"));
            output.push_str(
                &self.console_dimension_table(&report.by_complexity, |cl| cl.name().to_string()),
            );
            output.push('\n');

            // By domain
            if !report.by_domain.is_empty() {
                output.push_str(&self.console_section("Accuracy by Domain"));
                output.push_str(
                    &self.console_dimension_table(&report.by_domain, |d| d.name().to_string()),
                );
                output.push('\n');
            }

            // By provider
            output.push_str(&self.console_section("Accuracy by LLM Provider"));
            output.push_str(
                &self.console_dimension_table(&report.by_provider, |p| format!("{:?}", p)),
            );
            output.push('\n');
        }

        // Insights
        output.push_str(&self.console_section("Key Insights"));
        for insight in &report.insights {
            output.push_str(&format!("  * {}\n", insight));
        }

        output
    }

    fn console_header(&self, title: &str) -> String {
        let border = "=".repeat(72);
        format!("{}\n  {}\n{}\n", border, title, border)
    }

    fn console_section(&self, title: &str) -> String {
        let border = "-".repeat(72);
        format!("\n{}\n  {}\n{}\n", border, title, border)
    }

    fn console_accuracy_table(&self, by_format: &HashMap<DataFormat, DimensionResult>) -> String {
        let mut output = String::new();
        output.push_str("  Format     | Accuracy | 95% CI        | n     | Latency\n");
        output.push_str("  -----------|----------|---------------|-------|--------\n");

        let mut formats: Vec<_> = by_format.iter().collect();
        formats.sort_by(|a, b| {
            b.1.accuracy
                .partial_cmp(&a.1.accuracy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (format, result) in formats {
            output.push_str(&format!(
                "  {:10} | {:6.1}% | {:5.1}%-{:5.1}% | {:5} | {:6.0}ms\n",
                format.name(),
                result.accuracy * 100.0,
                result.confidence_interval.lower * 100.0,
                result.confidence_interval.upper * 100.0,
                result.total,
                result.mean_latency_ms
            ));
        }

        output
    }

    fn console_bar_chart(&self, by_format: &HashMap<DataFormat, DimensionResult>) -> String {
        let mut output = String::new();
        let bar_width = 50;

        let mut formats: Vec<_> = by_format.iter().collect();
        formats.sort_by(|a, b| {
            b.1.accuracy
                .partial_cmp(&a.1.accuracy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (format, result) in formats {
            let filled = (result.accuracy * bar_width as f64) as usize;
            let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);
            output.push_str(&format!(
                "  {:10} |{}| {:5.1}%\n",
                format.name(),
                bar,
                result.accuracy * 100.0
            ));
        }

        output
    }

    fn console_comparison_table(&self, comparisons: &[FormatComparison]) -> String {
        let mut output = String::new();
        output.push_str("  vs Format  | HEDL   | Other  | Diff    | Effect | Sig?\n");
        output.push_str("  -----------|--------|--------|---------|--------|-----\n");

        for comp in comparisons {
            let sig_marker = if comp.significant { "***" } else { "" };
            output.push_str(&format!(
                "  {:10} | {:5.1}% | {:5.1}% | {:+5.1}pp | {:6.2} | {}\n",
                comp.other_format.name(),
                comp.hedl_accuracy * 100.0,
                comp.other_accuracy * 100.0,
                comp.absolute_improvement * 100.0,
                comp.effect_size.cohens_d,
                sig_marker
            ));
        }

        output.push_str("\n  *** = statistically significant (p < 0.05)\n");
        output
    }

    fn console_dimension_table<K, F>(
        &self,
        data: &HashMap<K, HashMap<DataFormat, DimensionResult>>,
        key_formatter: F,
    ) -> String
    where
        K: std::hash::Hash + Eq,
        F: Fn(&K) -> String,
    {
        let mut output = String::new();

        // Collect all formats across all dimensions
        let mut all_formats: Vec<DataFormat> = Vec::new();
        for format_map in data.values() {
            for format in format_map.keys() {
                if !all_formats.contains(format) {
                    all_formats.push(*format);
                }
            }
        }
        all_formats.sort_by_key(|f| f.name());

        // Header
        output.push_str("  Dimension        ");
        for format in &all_formats {
            output.push_str(&format!("| {:8} ", format.name()));
        }
        output.push('\n');

        // Separator
        output.push_str("  -----------------");
        for _ in &all_formats {
            output.push_str("|----------");
        }
        output.push('\n');

        // Data rows
        let mut rows: Vec<_> = data.iter().collect();
        rows.sort_by(|a, b| key_formatter(a.0).cmp(&key_formatter(b.0)));

        for (key, format_map) in rows {
            output.push_str(&format!("  {:16} ", key_formatter(key)));
            for format in &all_formats {
                if let Some(result) = format_map.get(format) {
                    output.push_str(&format!("| {:6.1}%  ", result.accuracy * 100.0));
                } else {
                    output.push_str("|    -     ");
                }
            }
            output.push('\n');
        }

        output
    }

    fn generate_json(&self, report: &AccuracyReport) -> String {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!("  \"title\": \"{}\",\n", report.title));
        json.push_str(&format!(
            "  \"total_questions\": {},\n",
            report.total_questions
        ));

        // Overall by format
        json.push_str("  \"by_format\": {\n");
        let formats: Vec<_> = report.by_format.iter().collect();
        for (i, (format, result)) in formats.iter().enumerate() {
            json.push_str(&format!(
                "    \"{}\": {{ \"accuracy\": {:.4}, \"correct\": {}, \"total\": {}, \"ci_lower\": {:.4}, \"ci_upper\": {:.4} }}",
                format.name(),
                result.accuracy,
                result.correct,
                result.total,
                result.confidence_interval.lower,
                result.confidence_interval.upper
            ));
            if i < formats.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("  },\n");

        // Comparisons
        json.push_str("  \"hedl_comparisons\": [\n");
        for (i, comp) in report.hedl_comparisons.iter().enumerate() {
            json.push_str(&format!(
                "    {{ \"vs\": \"{}\", \"hedl_accuracy\": {:.4}, \"other_accuracy\": {:.4}, \"improvement\": {:.4}, \"effect_size\": {:.4}, \"p_value\": {:.6}, \"significant\": {} }}",
                comp.other_format.name(),
                comp.hedl_accuracy,
                comp.other_accuracy,
                comp.absolute_improvement,
                comp.effect_size.cohens_d,
                comp.p_value,
                comp.significant
            ));
            if i < report.hedl_comparisons.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("  ],\n");

        // Insights
        json.push_str("  \"insights\": [\n");
        for (i, insight) in report.insights.iter().enumerate() {
            json.push_str(&format!("    \"{}\"", insight.replace('"', "\\\"")));
            if i < report.insights.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("  ]\n");

        json.push_str("}\n");
        json
    }

    fn generate_markdown(&self, report: &AccuracyReport) -> String {
        let mut md = String::new();

        // Title
        md.push_str(&format!("# {}\n\n", report.title));
        md.push_str(&format!(
            "**Total Questions Evaluated:** {}\n\n",
            report.total_questions
        ));

        // Summary table
        md.push_str("## Overall Accuracy by Format\n\n");
        md.push_str("| Format | Accuracy | 95% CI | n | Mean Latency |\n");
        md.push_str("|--------|----------|--------|---|-------------|\n");

        let mut formats: Vec<_> = report.by_format.iter().collect();
        formats.sort_by(|a, b| {
            b.1.accuracy
                .partial_cmp(&a.1.accuracy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (format, result) in &formats {
            md.push_str(&format!(
                "| {} | {:.1}% | {:.1}%-{:.1}% | {} | {:.0}ms |\n",
                format.name(),
                result.accuracy * 100.0,
                result.confidence_interval.lower * 100.0,
                result.confidence_interval.upper * 100.0,
                result.total,
                result.mean_latency_ms
            ));
        }
        md.push('\n');

        // HEDL comparisons
        md.push_str("## HEDL vs Other Formats\n\n");
        md.push_str("| Format | HEDL | Other | Diff | Effect Size | Significant |\n");
        md.push_str("|--------|------|-------|------|-------------|-------------|\n");

        for comp in &report.hedl_comparisons {
            let sig = if comp.significant {
                "Yes (p<0.05)"
            } else {
                "No"
            };
            md.push_str(&format!(
                "| {} | {:.1}% | {:.1}% | {:+.1}pp | {:.2} | {} |\n",
                comp.other_format.name(),
                comp.hedl_accuracy * 100.0,
                comp.other_accuracy * 100.0,
                comp.absolute_improvement * 100.0,
                comp.effect_size.cohens_d,
                sig
            ));
        }
        md.push('\n');

        // Insights
        md.push_str("## Key Insights\n\n");
        for insight in &report.insights {
            md.push_str(&format!("- {}\n", insight));
        }

        md
    }

    fn generate_html(&self, report: &AccuracyReport) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str(&format!("<title>{}</title>\n", report.title));
        html.push_str("<style>\n");
        html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 1200px; margin: 0 auto; padding: 20px; }\n");
        html.push_str("h1 { color: #1a1a2e; }\n");
        html.push_str(
            "h2 { color: #16213e; border-bottom: 2px solid #e94560; padding-bottom: 5px; }\n",
        );
        html.push_str("table { border-collapse: collapse; width: 100%; margin: 20px 0; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }\n");
        html.push_str("th { background-color: #1a1a2e; color: white; }\n");
        html.push_str("tr:nth-child(even) { background-color: #f2f2f2; }\n");
        html.push_str(".bar { height: 20px; background-color: #e94560; border-radius: 4px; }\n");
        html.push_str(
            ".bar-container { width: 200px; background-color: #eee; border-radius: 4px; }\n",
        );
        html.push_str(".significant { color: #27ae60; font-weight: bold; }\n");
        html.push_str(".not-significant { color: #7f8c8d; }\n");
        html.push_str(".insight { background-color: #f8f9fa; padding: 15px; margin: 10px 0; border-left: 4px solid #e94560; }\n");
        html.push_str("</style>\n</head>\n<body>\n");

        // Title
        html.push_str(&format!("<h1>{}</h1>\n", report.title));
        html.push_str(&format!(
            "<p><strong>Total Questions:</strong> {}</p>\n",
            report.total_questions
        ));

        // Overall accuracy table
        html.push_str("<h2>Overall Accuracy by Format</h2>\n");
        html.push_str("<table>\n<tr><th>Format</th><th>Accuracy</th><th>Visual</th><th>95% CI</th><th>n</th><th>Latency</th></tr>\n");

        let mut formats: Vec<_> = report.by_format.iter().collect();
        formats.sort_by(|a, b| {
            b.1.accuracy
                .partial_cmp(&a.1.accuracy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (format, result) in &formats {
            let bar_width = (result.accuracy * 200.0) as u32;
            html.push_str(&format!(
                "<tr><td>{}</td><td>{:.1}%</td><td><div class=\"bar-container\"><div class=\"bar\" style=\"width:{}px\"></div></div></td><td>{:.1}%-{:.1}%</td><td>{}</td><td>{:.0}ms</td></tr>\n",
                format.name(),
                result.accuracy * 100.0,
                bar_width,
                result.confidence_interval.lower * 100.0,
                result.confidence_interval.upper * 100.0,
                result.total,
                result.mean_latency_ms
            ));
        }
        html.push_str("</table>\n");

        // Comparisons
        html.push_str("<h2>HEDL vs Other Formats</h2>\n");
        html.push_str("<table>\n<tr><th>vs Format</th><th>HEDL</th><th>Other</th><th>Difference</th><th>Effect Size</th><th>Significant?</th></tr>\n");

        for comp in &report.hedl_comparisons {
            let sig_class = if comp.significant {
                "significant"
            } else {
                "not-significant"
            };
            let sig_text = if comp.significant {
                "Yes (p<0.05)"
            } else {
                "No"
            };
            html.push_str(&format!(
                "<tr><td>{}</td><td>{:.1}%</td><td>{:.1}%</td><td>{:+.1}pp</td><td>{:.2}</td><td class=\"{}\">{}</td></tr>\n",
                comp.other_format.name(),
                comp.hedl_accuracy * 100.0,
                comp.other_accuracy * 100.0,
                comp.absolute_improvement * 100.0,
                comp.effect_size.cohens_d,
                sig_class,
                sig_text
            ));
        }
        html.push_str("</table>\n");

        // Insights
        html.push_str("<h2>Key Insights</h2>\n");
        for insight in &report.insights {
            html.push_str(&format!("<div class=\"insight\">{}</div>\n", insight));
        }

        html.push_str("</body>\n</html>\n");
        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_results() -> Vec<QuestionResult> {
        vec![
            QuestionResult {
                question_id: "q1".to_string(),
                question_type: QuestionType::FieldRetrieval,
                complexity: ComplexityLevel::L1Trivial,
                domain: Some(Domain::Finance),
                format: DataFormat::Hedl,
                provider: LlmProvider::OpenAI,
                correct: true,
                latency_ms: 150,
                input_tokens: 100,
                output_tokens: 20,
            },
            QuestionResult {
                question_id: "q1".to_string(),
                question_type: QuestionType::FieldRetrieval,
                complexity: ComplexityLevel::L1Trivial,
                domain: Some(Domain::Finance),
                format: DataFormat::Json,
                provider: LlmProvider::OpenAI,
                correct: false,
                latency_ms: 180,
                input_tokens: 150,
                output_tokens: 25,
            },
            QuestionResult {
                question_id: "q2".to_string(),
                question_type: QuestionType::Aggregation,
                complexity: ComplexityLevel::L2Basic,
                domain: Some(Domain::Finance),
                format: DataFormat::Hedl,
                provider: LlmProvider::OpenAI,
                correct: true,
                latency_ms: 200,
                input_tokens: 120,
                output_tokens: 30,
            },
            QuestionResult {
                question_id: "q2".to_string(),
                question_type: QuestionType::Aggregation,
                complexity: ComplexityLevel::L2Basic,
                domain: Some(Domain::Finance),
                format: DataFormat::Json,
                provider: LlmProvider::OpenAI,
                correct: true,
                latency_ms: 220,
                input_tokens: 180,
                output_tokens: 35,
            },
        ]
    }

    #[test]
    fn test_dimension_result_from_results() {
        let results = sample_results();
        let refs: Vec<_> = results.iter().collect();
        let dim = DimensionResult::from_results("test", &refs);

        assert_eq!(dim.total, 4);
        assert_eq!(dim.correct, 3);
        assert!((dim.accuracy - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_accuracy_report_from_results() {
        let results = sample_results();
        let report = AccuracyReport::from_results("Test Report", &results);

        assert_eq!(report.total_questions, 4);
        assert!(report.by_format.contains_key(&DataFormat::Hedl));
        assert!(report.by_format.contains_key(&DataFormat::Json));
    }

    #[test]
    fn test_console_report_generation() {
        let results = sample_results();
        let report = AccuracyReport::from_results("Test Report", &results);

        let generator = ReportGenerator::new().format(OutputFormat::Console);
        let output = generator.generate(&report);

        assert!(output.contains("Test Report"));
        assert!(output.contains("HEDL"));
        assert!(output.contains("JSON"));
    }

    #[test]
    fn test_markdown_report_generation() {
        let results = sample_results();
        let report = AccuracyReport::from_results("Test Report", &results);

        let generator = ReportGenerator::new().format(OutputFormat::Markdown);
        let output = generator.generate(&report);

        assert!(output.contains("# Test Report"));
        assert!(output.contains("| Format | Accuracy |"));
    }

    #[test]
    fn test_json_report_generation() {
        let results = sample_results();
        let report = AccuracyReport::from_results("Test Report", &results);

        let generator = ReportGenerator::new().format(OutputFormat::Json);
        let output = generator.generate(&report);

        assert!(output.contains("\"title\": \"Test Report\""));
        assert!(output.contains("\"by_format\""));
    }

    #[test]
    fn test_html_report_generation() {
        let results = sample_results();
        let report = AccuracyReport::from_results("Test Report", &results);

        let generator = ReportGenerator::new().format(OutputFormat::Html);
        let output = generator.generate(&report);

        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("<h1>Test Report</h1>"));
    }

    #[test]
    fn test_format_comparison_calculation() {
        let results = sample_results();
        let report = AccuracyReport::from_results("Test Report", &results);

        // HEDL should have higher accuracy than JSON in sample
        let json_comp = report
            .hedl_comparisons
            .iter()
            .find(|c| c.other_format == DataFormat::Json);

        assert!(json_comp.is_some());
        let comp = json_comp.unwrap();
        assert!(comp.absolute_improvement > 0.0);
    }

    #[test]
    fn test_insights_generation() {
        let results = sample_results();
        let report = AccuracyReport::from_results("Test Report", &results);

        assert!(!report.insights.is_empty());
        assert!(report.insights.iter().any(|i| i.contains("HEDL")));
    }
}
