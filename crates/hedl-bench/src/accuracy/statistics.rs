// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Statistical Analysis for Accuracy Benchmarks
//!
//! Provides comprehensive statistical rigor beyond simple accuracy percentages:
//! - Confidence intervals (95% CI)
//! - Effect sizes (Cohen's d, Cliff's delta)
//! - Statistical significance tests (t-test, chi-square, McNemar)
//! - Multi-run aggregation with proper error propagation

use std::collections::HashMap;

/// Confidence interval with level
#[derive(Debug, Clone, Copy)]
pub struct ConfidenceInterval {
    /// Lower bound
    pub lower: f64,
    /// Point estimate
    pub point: f64,
    /// Upper bound
    pub upper: f64,
    /// Confidence level (e.g., 0.95 for 95%)
    pub level: f64,
}

impl ConfidenceInterval {
    /// Create a new confidence interval
    #[must_use]
    pub fn new(lower: f64, point: f64, upper: f64, level: f64) -> Self {
        Self {
            lower,
            point,
            upper,
            level,
        }
    }

    /// Calculate 95% CI from sample mean and standard error
    #[must_use]
    pub fn from_mean_se(mean: f64, se: f64) -> Self {
        // Z-score for 95% CI
        let z = 1.96;
        Self {
            lower: mean - z * se,
            point: mean,
            upper: mean + z * se,
            level: 0.95,
        }
    }

    /// Calculate Wilson score interval for proportions (better for small samples)
    #[must_use]
    pub fn wilson_score(successes: usize, total: usize, confidence: f64) -> Self {
        if total == 0 {
            return Self::new(0.0, 0.0, 0.0, confidence);
        }

        let n = total as f64;
        let p_hat = successes as f64 / n;

        // Z-score for confidence level
        let z = match confidence {
            c if (c - 0.99).abs() < 0.01 => 2.576,
            c if (c - 0.95).abs() < 0.01 => 1.96,
            c if (c - 0.90).abs() < 0.01 => 1.645,
            _ => 1.96,
        };

        let z2 = z * z;
        let denominator = 1.0 + z2 / n;
        let center = (p_hat + z2 / (2.0 * n)) / denominator;
        let margin = z * (p_hat * (1.0 - p_hat) / n + z2 / (4.0 * n * n)).sqrt() / denominator;

        Self {
            lower: (center - margin).max(0.0),
            point: p_hat,
            upper: (center + margin).min(1.0),
            level: confidence,
        }
    }

    /// Width of the interval
    #[must_use]
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }

    /// Check if a value falls within the interval
    #[must_use]
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }

    /// Check if two intervals overlap
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.lower <= other.upper && other.lower <= self.upper
    }

    /// Format as percentage string
    #[must_use]
    pub fn as_percentage_string(&self) -> String {
        format!(
            "{:.1}% [{:.1}%, {:.1}%]",
            self.point * 100.0,
            self.lower * 100.0,
            self.upper * 100.0
        )
    }
}

/// Effect size measures for comparing groups
#[derive(Debug, Clone, Copy)]
pub struct EffectSize {
    /// Cohen's d (standardized mean difference)
    pub cohens_d: f64,
    /// Cliff's delta (non-parametric effect size, -1 to 1)
    pub cliffs_delta: f64,
    /// Common language effect size (probability of superiority)
    pub cles: f64,
}

impl EffectSize {
    /// Calculate effect sizes from two groups of measurements
    #[must_use]
    pub fn from_groups(group1: &[f64], group2: &[f64]) -> Self {
        let cohens_d = Self::calculate_cohens_d(group1, group2);
        let cliffs_delta = Self::calculate_cliffs_delta(group1, group2);
        let cles = Self::calculate_cles(cohens_d);

        Self {
            cohens_d,
            cliffs_delta,
            cles,
        }
    }

    /// Calculate Cohen's d
    fn calculate_cohens_d(group1: &[f64], group2: &[f64]) -> f64 {
        let mean1 = mean(group1);
        let mean2 = mean(group2);
        let pooled_std = pooled_std(group1, group2);

        if pooled_std == 0.0 {
            return 0.0;
        }

        (mean1 - mean2) / pooled_std
    }

    /// Calculate Cliff's delta (non-parametric)
    fn calculate_cliffs_delta(group1: &[f64], group2: &[f64]) -> f64 {
        if group1.is_empty() || group2.is_empty() {
            return 0.0;
        }

        let mut greater = 0;
        let mut less = 0;

        for &x in group1 {
            for &y in group2 {
                if x > y {
                    greater += 1;
                } else if x < y {
                    less += 1;
                }
            }
        }

        let n = (group1.len() * group2.len()) as f64;
        (greater as f64 - less as f64) / n
    }

    /// Calculate common language effect size from Cohen's d
    fn calculate_cles(d: f64) -> f64 {
        // Approximation using normal CDF
        let phi = |x: f64| -> f64 { 0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2)) };

        phi(d / std::f64::consts::SQRT_2)
    }

    /// Interpret Cohen's d magnitude
    #[must_use]
    pub fn interpret_cohens_d(&self) -> &'static str {
        let abs_d = self.cohens_d.abs();
        if abs_d < 0.2 {
            "negligible"
        } else if abs_d < 0.5 {
            "small"
        } else if abs_d < 0.8 {
            "medium"
        } else {
            "large"
        }
    }

    /// Interpret Cliff's delta magnitude
    #[must_use]
    pub fn interpret_cliffs_delta(&self) -> &'static str {
        let abs_delta = self.cliffs_delta.abs();
        if abs_delta < 0.147 {
            "negligible"
        } else if abs_delta < 0.33 {
            "small"
        } else if abs_delta < 0.474 {
            "medium"
        } else {
            "large"
        }
    }
}

/// Statistical test results
#[derive(Debug, Clone)]
pub struct StatisticalResult {
    /// Test name
    pub test_name: String,
    /// Test statistic value
    pub statistic: f64,
    /// P-value
    pub p_value: f64,
    /// Degrees of freedom (if applicable)
    pub df: Option<f64>,
    /// Whether result is significant at alpha=0.05
    pub significant_05: bool,
    /// Whether result is significant at alpha=0.01
    pub significant_01: bool,
    /// Effect size
    pub effect_size: Option<EffectSize>,
    /// Confidence interval for the difference
    pub ci: Option<ConfidenceInterval>,
}

impl StatisticalResult {
    /// Interpret significance level
    #[must_use]
    pub fn significance_stars(&self) -> &'static str {
        if self.p_value < 0.001 {
            "***"
        } else if self.p_value < 0.01 {
            "**"
        } else if self.p_value < 0.05 {
            "*"
        } else {
            "ns"
        }
    }

    /// Format p-value
    #[must_use]
    pub fn p_value_string(&self) -> String {
        if self.p_value < 0.001 {
            "p < 0.001".to_string()
        } else {
            format!("p = {:.3}", self.p_value)
        }
    }
}

/// Welch's t-test for comparing two groups (unequal variances)
#[must_use]
pub fn welch_t_test(group1: &[f64], group2: &[f64]) -> StatisticalResult {
    let n1 = group1.len() as f64;
    let n2 = group2.len() as f64;

    let mean1 = mean(group1);
    let mean2 = mean(group2);
    let var1 = variance(group1);
    let var2 = variance(group2);

    let se1 = var1 / n1;
    let se2 = var2 / n2;
    let se = (se1 + se2).sqrt();

    let t = if se > 0.0 { (mean1 - mean2) / se } else { 0.0 };

    // Welch-Satterthwaite degrees of freedom
    let df = if se1 + se2 > 0.0 {
        (se1 + se2).powi(2) / (se1.powi(2) / (n1 - 1.0) + se2.powi(2) / (n2 - 1.0))
    } else {
        n1 + n2 - 2.0
    };

    // Approximate p-value using normal distribution for large samples
    let p_value = 2.0 * (1.0 - normal_cdf(t.abs()));

    let effect_size = EffectSize::from_groups(group1, group2);
    let ci = ConfidenceInterval::from_mean_se(mean1 - mean2, se);

    StatisticalResult {
        test_name: "Welch's t-test".to_string(),
        statistic: t,
        p_value,
        df: Some(df),
        significant_05: p_value < 0.05,
        significant_01: p_value < 0.01,
        effect_size: Some(effect_size),
        ci: Some(ci),
    }
}

/// Chi-square test for comparing proportions
#[must_use]
pub fn chi_square_test(observed: &[[usize; 2]; 2]) -> StatisticalResult {
    let n = observed
        .iter()
        .map(|row| row.iter().sum::<usize>())
        .sum::<usize>() as f64;

    let row_totals: Vec<usize> = observed.iter().map(|row| row.iter().sum()).collect();
    let col_totals: Vec<usize> = (0..2)
        .map(|j| observed.iter().map(|row| row[j]).sum())
        .collect();

    let mut chi2 = 0.0;
    for (i, row) in observed.iter().enumerate() {
        for (j, &obs) in row.iter().enumerate() {
            let expected = row_totals[i] as f64 * col_totals[j] as f64 / n;
            if expected > 0.0 {
                chi2 += (obs as f64 - expected).powi(2) / expected;
            }
        }
    }

    // P-value from chi-square distribution with df=1
    let p_value = 1.0 - chi_square_cdf(chi2, 1.0);

    StatisticalResult {
        test_name: "Chi-square test".to_string(),
        statistic: chi2,
        p_value,
        df: Some(1.0),
        significant_05: p_value < 0.05,
        significant_01: p_value < 0.01,
        effect_size: None,
        ci: None,
    }
}

/// McNemar's test for paired proportions
#[must_use]
pub fn mcnemar_test(b: usize, c: usize) -> StatisticalResult {
    // b = format1 correct, format2 wrong
    // c = format1 wrong, format2 correct

    let b_f = b as f64;
    let c_f = c as f64;

    // McNemar statistic with continuity correction
    let statistic = if b + c > 0 {
        ((b_f - c_f).abs() - 1.0).max(0.0).powi(2) / (b_f + c_f)
    } else {
        0.0
    };

    let p_value = 1.0 - chi_square_cdf(statistic, 1.0);

    StatisticalResult {
        test_name: "McNemar's test".to_string(),
        statistic,
        p_value,
        df: Some(1.0),
        significant_05: p_value < 0.05,
        significant_01: p_value < 0.01,
        effect_size: None,
        ci: None,
    }
}

/// Aggregated accuracy results with statistical analysis
#[derive(Debug, Clone)]
pub struct AccuracyAnalysis {
    /// Sample size
    pub n: usize,
    /// Number of successes
    pub successes: usize,
    /// Point estimate of accuracy
    pub accuracy: f64,
    /// Standard error
    pub se: f64,
    /// 95% confidence interval (Wilson score)
    pub ci_95: ConfidenceInterval,
    /// 99% confidence interval (Wilson score)
    pub ci_99: ConfidenceInterval,
    /// Individual run accuracies (for multi-run analysis)
    pub run_accuracies: Vec<f64>,
}

impl AccuracyAnalysis {
    /// Create from counts
    #[must_use]
    pub fn from_counts(successes: usize, total: usize) -> Self {
        let accuracy = if total > 0 {
            successes as f64 / total as f64
        } else {
            0.0
        };

        let se = if total > 0 {
            (accuracy * (1.0 - accuracy) / total as f64).sqrt()
        } else {
            0.0
        };

        Self {
            n: total,
            successes,
            accuracy,
            se,
            ci_95: ConfidenceInterval::wilson_score(successes, total, 0.95),
            ci_99: ConfidenceInterval::wilson_score(successes, total, 0.99),
            run_accuracies: Vec::new(),
        }
    }

    /// Create from multiple runs
    #[must_use]
    pub fn from_runs(run_results: &[Vec<bool>]) -> Self {
        let run_accuracies: Vec<f64> = run_results
            .iter()
            .map(|run| {
                let correct = run.iter().filter(|&&b| b).count();
                correct as f64 / run.len().max(1) as f64
            })
            .collect();

        let mean_accuracy = mean(&run_accuracies);
        let se = if run_accuracies.len() > 1 {
            std_dev(&run_accuracies) / (run_accuracies.len() as f64).sqrt()
        } else {
            0.0
        };

        // Total counts across all runs
        let total_successes: usize = run_results
            .iter()
            .map(|run| run.iter().filter(|&&b| b).count())
            .sum();
        let total_n: usize = run_results.iter().map(|run| run.len()).sum();

        Self {
            n: total_n,
            successes: total_successes,
            accuracy: mean_accuracy,
            se,
            ci_95: ConfidenceInterval::from_mean_se(mean_accuracy, se * 1.96),
            ci_99: ConfidenceInterval::from_mean_se(mean_accuracy, se * 2.576),
            run_accuracies,
        }
    }

    /// Format as percentage with CI
    #[must_use]
    pub fn format_with_ci(&self) -> String {
        format!(
            "{:.1}% ± {:.1}pp (95% CI: [{:.1}%, {:.1}%])",
            self.accuracy * 100.0,
            self.se * 1.96 * 100.0,
            self.ci_95.lower * 100.0,
            self.ci_95.upper * 100.0
        )
    }

    /// Compare with another analysis
    #[must_use]
    pub fn compare(&self, other: &Self) -> FormatComparison {
        let diff = self.accuracy - other.accuracy;
        let diff_ci =
            ConfidenceInterval::from_mean_se(diff, (self.se.powi(2) + other.se.powi(2)).sqrt());

        // Effect size if we have run data
        let effect = if !self.run_accuracies.is_empty() && !other.run_accuracies.is_empty() {
            Some(EffectSize::from_groups(
                &self.run_accuracies,
                &other.run_accuracies,
            ))
        } else {
            None
        };

        // Statistical test
        let test = if !self.run_accuracies.is_empty() && !other.run_accuracies.is_empty() {
            Some(welch_t_test(&self.run_accuracies, &other.run_accuracies))
        } else {
            None
        };

        FormatComparison {
            diff,
            diff_ci,
            overlapping_ci: self.ci_95.overlaps(&other.ci_95),
            effect_size: effect,
            test_result: test,
        }
    }
}

/// Comparison between two formats
#[derive(Debug, Clone)]
pub struct FormatComparison {
    /// Difference in accuracy (format1 - format2)
    pub diff: f64,
    /// Confidence interval for the difference
    pub diff_ci: ConfidenceInterval,
    /// Whether the 95% CIs overlap
    pub overlapping_ci: bool,
    /// Effect size (if available)
    pub effect_size: Option<EffectSize>,
    /// Statistical test result (if available)
    pub test_result: Option<StatisticalResult>,
}

impl FormatComparison {
    /// Format the comparison
    #[must_use]
    pub fn summary(&self) -> String {
        let direction = if self.diff > 0.0 { "better" } else { "worse" };
        let significance = self
            .test_result
            .as_ref()
            .map(|t| t.significance_stars())
            .unwrap_or("?");
        let effect = self
            .effect_size
            .as_ref()
            .map(|e| e.interpret_cohens_d())
            .unwrap_or("unknown");

        format!(
            "{:.1}pp {} ({}) [{}]",
            self.diff.abs() * 100.0,
            direction,
            effect,
            significance
        )
    }
}

/// Comprehensive benchmark statistics
#[derive(Debug, Default)]
pub struct BenchmarkStatistics {
    /// Accuracy by format
    pub by_format: HashMap<String, AccuracyAnalysis>,
    /// Pairwise format comparisons
    pub comparisons: HashMap<(String, String), FormatComparison>,
    /// Overall accuracy
    pub overall: Option<AccuracyAnalysis>,
}

impl BenchmarkStatistics {
    /// Add results for a format
    pub fn add_format(&mut self, format: String, analysis: AccuracyAnalysis) {
        // Calculate comparisons with existing formats
        for (other_format, other_analysis) in &self.by_format {
            let comparison = analysis.compare(other_analysis);
            self.comparisons
                .insert((format.clone(), other_format.clone()), comparison.clone());

            let reverse = other_analysis.compare(&analysis);
            self.comparisons
                .insert((other_format.clone(), format.clone()), reverse);
        }

        self.by_format.insert(format, analysis);
    }

    /// Get the best performing format
    #[must_use]
    pub fn best_format(&self) -> Option<(&String, &AccuracyAnalysis)> {
        self.by_format.iter().max_by(|a, b| {
            a.1.accuracy
                .partial_cmp(&b.1.accuracy)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get formats ranked by accuracy
    #[must_use]
    pub fn ranked_formats(&self) -> Vec<(&String, &AccuracyAnalysis)> {
        let mut formats: Vec<_> = self.by_format.iter().collect();
        formats.sort_by(|a, b| {
            b.1.accuracy
                .partial_cmp(&a.1.accuracy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        formats
    }

    /// Calculate overall statistics
    pub fn calculate_overall(&mut self) {
        let total_successes: usize = self.by_format.values().map(|a| a.successes).sum();
        let total_n: usize = self.by_format.values().map(|a| a.n).sum();

        self.overall = Some(AccuracyAnalysis::from_counts(total_successes, total_n));
    }
}

// Helper functions

fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

fn variance(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let m = mean(data);
    let sum_sq: f64 = data.iter().map(|x| (x - m).powi(2)).sum();
    sum_sq / (data.len() - 1) as f64
}

fn std_dev(data: &[f64]) -> f64 {
    variance(data).sqrt()
}

fn pooled_std(group1: &[f64], group2: &[f64]) -> f64 {
    let n1 = group1.len() as f64;
    let n2 = group2.len() as f64;

    if n1 + n2 < 4.0 {
        return 0.0;
    }

    let var1 = variance(group1);
    let var2 = variance(group2);

    (((n1 - 1.0) * var1 + (n2 - 1.0) * var2) / (n1 + n2 - 2.0)).sqrt()
}

/// Error function approximation
fn erf(x: f64) -> f64 {
    // Abramowitz and Stegun approximation
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

/// Standard normal CDF
fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Chi-square CDF approximation (for df=1)
fn chi_square_cdf(x: f64, _df: f64) -> f64 {
    // For df=1, chi-square CDF = 2 * Phi(sqrt(x)) - 1
    if x <= 0.0 {
        return 0.0;
    }
    2.0 * normal_cdf(x.sqrt()) - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_interval() {
        let ci = ConfidenceInterval::wilson_score(80, 100, 0.95);
        assert!(ci.lower > 0.7);
        assert!(ci.upper < 0.9);
        assert!((ci.point - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_effect_size() {
        let group1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let group2 = vec![2.0, 3.0, 4.0, 5.0, 6.0];

        let effect = EffectSize::from_groups(&group1, &group2);
        assert!(effect.cohens_d < 0.0); // group1 is lower
        assert!(effect.cliffs_delta < 0.0);
    }

    #[test]
    fn test_welch_t_test() {
        let group1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let group2 = vec![6.0, 7.0, 8.0, 9.0, 10.0];

        let result = welch_t_test(&group1, &group2);
        assert!(result.significant_01); // Should be highly significant
        assert!(result.effect_size.is_some());
    }

    #[test]
    fn test_accuracy_analysis() {
        let analysis = AccuracyAnalysis::from_counts(90, 100);
        assert!((analysis.accuracy - 0.9).abs() < 0.001);
        assert!(analysis.ci_95.lower > 0.8);
        assert!(analysis.ci_95.upper < 1.0);
    }

    #[test]
    fn test_multi_run_analysis() {
        let runs = vec![
            vec![true, true, true, false, true],
            vec![true, true, false, true, true],
            vec![true, false, true, true, true],
        ];

        let analysis = AccuracyAnalysis::from_runs(&runs);
        assert_eq!(analysis.run_accuracies.len(), 3);
        assert!(analysis.accuracy > 0.7);
    }

    #[test]
    fn test_format_comparison() {
        let analysis1 = AccuracyAnalysis::from_counts(90, 100);
        let analysis2 = AccuracyAnalysis::from_counts(80, 100);

        let comparison = analysis1.compare(&analysis2);
        assert!(comparison.diff > 0.0); // analysis1 is better
    }
}
