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

//! Insight generation for accuracy reports

use crate::accuracy_tables::*;
use crate::accuracy_types::{AccuracyReport, TestResult};
use hedl_bench::accuracy::FixtureDataset;
use hedl_bench::{BenchmarkReport, Insight};

/// Generate dynamic insights by analyzing actual benchmark results
pub fn generate_insights(
    legacy_report: &AccuracyReport,
    report: &mut BenchmarkReport,
    all_results: &[TestResult],
    datasets: &[FixtureDataset],
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
                    "HEDL achieves {efficiency_gain:.1}x better accuracy-per-token than JSON ({hedl_acc_per_1k:.2} vs {json_acc_per_1k:.2})"
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
                title: format!("Significant Accuracy Gap ({acc_gap:.1}pp lower than JSON)"),
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
                    "With {token_savings:.1}% token savings and only {acc_gap:.1}pp accuracy loss, HEDL is ideal for high-volume applications"
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
                description: format!("The {acc_gap:.1}pp accuracy gap may be too large for production use"),
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
                .map(|(t, acc, total)| format!("{t}: {acc:.1}% ({total} questions tested)"))
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
