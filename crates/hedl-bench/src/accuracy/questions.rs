// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Question Types and Corpus for LLM Accuracy Benchmarks
//!
//! Implements 12 question types covering comprehensive LLM comprehension testing.

use std::collections::HashMap;

/// 12 question types for comprehensive LLM accuracy testing.
///
/// Surpasses TOON's 5 types (Extraction, Counting, Comparison, Aggregation, Nested Navigation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestionType {
    // === Basic Types (similar to TOON) ===
    /// Direct field lookup from records (TOON: Extraction)
    FieldRetrieval,
    /// Counts, sums, averages (TOON: Counting + Aggregation)
    Aggregation,
    /// Multi-condition queries with AND/OR logic (TOON: Comparison subset)
    Filtering,
    /// Format-native structural features
    StructureAwareness,
    /// Data integrity detection
    StructuralValidation,

    // === Advanced Types (beyond TOON) ===
    /// Compare values between different records
    Comparison,
    /// Multi-level data traversal with path navigation
    NestedNavigation,
    /// Follow @Type:id references to resolve values
    ReferenceResolution,
    /// Time-based filtering and temporal reasoning
    TemporalQuery,
    /// Graph-like traversal across entity relationships
    RelationshipTraversal,
    /// Mathematical operations on numeric data
    MathematicalOperation,
    /// Pattern matching and regex-like queries
    PatternMatching,
}

impl QuestionType {
    /// All question types for iteration
    pub const ALL: [QuestionType; 12] = [
        QuestionType::FieldRetrieval,
        QuestionType::Aggregation,
        QuestionType::Filtering,
        QuestionType::StructureAwareness,
        QuestionType::StructuralValidation,
        QuestionType::Comparison,
        QuestionType::NestedNavigation,
        QuestionType::ReferenceResolution,
        QuestionType::TemporalQuery,
        QuestionType::RelationshipTraversal,
        QuestionType::MathematicalOperation,
        QuestionType::PatternMatching,
    ];

    /// Human-readable name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            QuestionType::FieldRetrieval => "Field Retrieval",
            QuestionType::Aggregation => "Aggregation",
            QuestionType::Filtering => "Filtering",
            QuestionType::StructureAwareness => "Structure Awareness",
            QuestionType::StructuralValidation => "Structural Validation",
            QuestionType::Comparison => "Comparison",
            QuestionType::NestedNavigation => "Nested Navigation",
            QuestionType::ReferenceResolution => "Reference Resolution",
            QuestionType::TemporalQuery => "Temporal Query",
            QuestionType::RelationshipTraversal => "Relationship Traversal",
            QuestionType::MathematicalOperation => "Mathematical Operation",
            QuestionType::PatternMatching => "Pattern Matching",
        }
    }

    /// Description of what this question type tests
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            QuestionType::FieldRetrieval => "Direct lookup of a specific field value from a record",
            QuestionType::Aggregation => {
                "Counting, summing, averaging, or finding min/max across records"
            }
            QuestionType::Filtering => "Multi-condition queries combining AND/OR logic",
            QuestionType::StructureAwareness => {
                "Understanding format-specific structural features (schemas, nesting)"
            }
            QuestionType::StructuralValidation => {
                "Detecting data integrity issues (missing fields, truncation)"
            }
            QuestionType::Comparison => "Comparing values between two or more different records",
            QuestionType::NestedNavigation => "Traversing multi-level hierarchical data structures",
            QuestionType::ReferenceResolution => {
                "Following @Type:id references to resolve linked entities"
            }
            QuestionType::TemporalQuery => "Time-based filtering and date/time reasoning",
            QuestionType::RelationshipTraversal => {
                "Graph-like traversal across entity relationships"
            }
            QuestionType::MathematicalOperation => {
                "Arithmetic operations on numeric data (percentages, ratios)"
            }
            QuestionType::PatternMatching => {
                "Finding records matching patterns (prefix, suffix, contains)"
            }
        }
    }

    /// Cognitive complexity score (1-5) for weighted analysis
    #[must_use]
    pub fn complexity_score(&self) -> u8 {
        match self {
            QuestionType::FieldRetrieval => 1,
            QuestionType::Aggregation => 2,
            QuestionType::Filtering => 3,
            QuestionType::StructureAwareness => 2,
            QuestionType::StructuralValidation => 3,
            QuestionType::Comparison => 3,
            QuestionType::NestedNavigation => 4,
            QuestionType::ReferenceResolution => 4,
            QuestionType::TemporalQuery => 3,
            QuestionType::RelationshipTraversal => 5,
            QuestionType::MathematicalOperation => 3,
            QuestionType::PatternMatching => 2,
        }
    }
}

/// Answer types for type-aware comparison
#[derive(Debug, Clone, PartialEq)]
pub enum AnswerType {
    /// Exact string match (case-insensitive by default)
    String,
    /// Case-sensitive string match
    StringCaseSensitive,
    /// Integer with tolerance for formatting variations
    Integer,
    /// Float with configurable decimal precision
    Number {
        /// Number of decimal places for comparison
        decimals: usize,
    },
    /// Boolean (yes/no/true/false/y/n/1/0)
    Boolean,
    /// Date in ISO format (YYYY-MM-DD)
    Date,
    /// DateTime in ISO format (YYYY-MM-DDTHH:MM:SSZ)
    DateTime,
    /// Comma-separated list, order matters
    ListOrdered,
    /// Comma-separated list, order doesn't matter
    ListUnordered,
    /// JSON object for complex structured answers
    JsonObject,
    /// Numeric range (value within tolerance)
    NumericRange {
        /// Minimum acceptable value
        min: f64,
        /// Maximum acceptable value
        max: f64,
    },
    /// Multiple valid answers (any match is correct)
    MultipleValid(Vec<String>),
}

/// A benchmark question with metadata
#[derive(Debug, Clone)]
pub struct Question {
    /// Unique question ID (e.g., "fin_L3_agg_001")
    pub id: String,
    /// The question text to ask the LLM
    pub prompt: String,
    /// Expected ground truth answer (default for all formats)
    pub ground_truth: String,
    /// Format-specific ground truths (overrides `ground_truth` for specific formats).
    /// Keys are lowercase format names: "hedl", "json", "yaml", "xml", "toon", "csv"
    pub ground_truth_by_format: Option<HashMap<String, String>>,
    /// Type of question for categorization
    pub question_type: QuestionType,
    /// Domain this question applies to
    pub domain: String,
    /// Dataset within the domain
    pub dataset: String,
    /// Answer type for comparison
    pub answer_type: AnswerType,
    /// Complexity level (L1-L5)
    pub complexity_level: u8,
    /// Optional notes about the question
    pub notes: Option<String>,
    /// Tags for filtering (e.g., "unicode", "edge-case", "reference")
    pub tags: Vec<String>,
    /// Whether this question is format-agnostic (blind evaluation)
    pub blind_mode: bool,
}

impl Question {
    /// Create a new question builder
    pub fn builder(id: impl Into<String>, prompt: impl Into<String>) -> QuestionBuilder {
        QuestionBuilder {
            id: id.into(),
            prompt: prompt.into(),
            ground_truth: None,
            ground_truth_by_format: None,
            question_type: QuestionType::FieldRetrieval,
            domain: "general".to_string(),
            dataset: "default".to_string(),
            answer_type: AnswerType::String,
            complexity_level: 1,
            notes: None,
            tags: Vec::new(),
            blind_mode: false,
        }
    }

    /// Gets the ground truth for a specific format.
    /// Falls back to the default `ground_truth` if no format-specific value exists.
    #[must_use]
    pub fn ground_truth_for_format(&self, format: &str) -> &str {
        if let Some(ref by_format) = self.ground_truth_by_format {
            if let Some(value) = by_format.get(&format.to_lowercase()) {
                return value;
            }
        }
        &self.ground_truth
    }

    /// Check if this question has format-specific ground truths
    #[must_use]
    pub fn has_format_specific_answers(&self) -> bool {
        self.ground_truth_by_format
            .as_ref()
            .is_some_and(|m| !m.is_empty())
    }

    /// Weighted score combining complexity and question type
    #[must_use]
    pub fn difficulty_score(&self) -> u8 {
        self.complexity_level
            .saturating_add(self.question_type.complexity_score())
            / 2
    }
}

/// Builder for creating questions
pub struct QuestionBuilder {
    id: String,
    prompt: String,
    ground_truth: Option<String>,
    ground_truth_by_format: Option<HashMap<String, String>>,
    question_type: QuestionType,
    domain: String,
    dataset: String,
    answer_type: AnswerType,
    complexity_level: u8,
    notes: Option<String>,
    tags: Vec<String>,
    blind_mode: bool,
}

impl QuestionBuilder {
    /// Sets the ground truth answer (required)
    pub fn ground_truth(mut self, value: impl Into<String>) -> Self {
        self.ground_truth = Some(value.into());
        self
    }

    /// Sets the question type
    #[must_use]
    pub fn question_type(mut self, qt: QuestionType) -> Self {
        self.question_type = qt;
        self
    }

    /// Sets the domain
    pub fn domain(mut self, d: impl Into<String>) -> Self {
        self.domain = d.into();
        self
    }

    /// Sets the dataset
    pub fn dataset(mut self, ds: impl Into<String>) -> Self {
        self.dataset = ds.into();
        self
    }

    /// Sets the answer type for comparison
    #[must_use]
    pub fn answer_type(mut self, at: AnswerType) -> Self {
        self.answer_type = at;
        self
    }

    /// Sets the complexity level (1-5)
    #[must_use]
    pub fn complexity(mut self, level: u8) -> Self {
        self.complexity_level = level.clamp(1, 5);
        self
    }

    /// Sets optional notes
    pub fn notes(mut self, n: impl Into<String>) -> Self {
        self.notes = Some(n.into());
        self
    }

    /// Adds a tag
    pub fn tag(mut self, t: impl Into<String>) -> Self {
        self.tags.push(t.into());
        self
    }

    /// Adds multiple tags
    pub fn tags(mut self, ts: &[&str]) -> Self {
        self.tags.extend(ts.iter().map(|s| (*s).to_string()));
        self
    }

    /// Enables blind evaluation mode
    #[must_use]
    pub fn blind(mut self) -> Self {
        self.blind_mode = true;
        self
    }

    /// Sets a format-specific ground truth (overrides default for that format)
    pub fn format_ground_truth(
        mut self,
        format: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.ground_truth_by_format
            .get_or_insert_with(HashMap::new)
            .insert(format.into().to_lowercase(), value.into());
        self
    }

    /// Sets multiple format-specific ground truths at once
    #[must_use]
    pub fn format_ground_truths(mut self, map: HashMap<String, String>) -> Self {
        let normalized: HashMap<String, String> = map
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect();
        self.ground_truth_by_format = Some(normalized);
        self
    }

    /// Builds the question
    #[must_use]
    pub fn build(self) -> Question {
        Question {
            id: self.id,
            prompt: self.prompt,
            ground_truth: self.ground_truth.expect("ground_truth is required"),
            ground_truth_by_format: self.ground_truth_by_format,
            question_type: self.question_type,
            domain: self.domain,
            dataset: self.dataset,
            answer_type: self.answer_type,
            complexity_level: self.complexity_level,
            notes: self.notes,
            tags: self.tags,
            blind_mode: self.blind_mode,
        }
    }
}

/// The complete question corpus for benchmarking
#[derive(Debug, Default)]
pub struct QuestionCorpus {
    /// All questions indexed by ID
    questions: HashMap<String, Question>,
    /// Questions indexed by domain
    by_domain: HashMap<String, Vec<String>>,
    /// Questions indexed by type
    by_type: HashMap<QuestionType, Vec<String>>,
    /// Questions indexed by complexity level
    by_complexity: HashMap<u8, Vec<String>>,
    /// Questions indexed by tags
    by_tag: HashMap<String, Vec<String>>,
}

impl QuestionCorpus {
    /// Create a new empty corpus
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a question to the corpus
    pub fn add(&mut self, question: Question) {
        let id = question.id.clone();

        // Index by domain
        self.by_domain
            .entry(question.domain.clone())
            .or_default()
            .push(id.clone());

        // Index by type
        self.by_type
            .entry(question.question_type)
            .or_default()
            .push(id.clone());

        // Index by complexity
        self.by_complexity
            .entry(question.complexity_level)
            .or_default()
            .push(id.clone());

        // Index by tags
        for tag in &question.tags {
            self.by_tag.entry(tag.clone()).or_default().push(id.clone());
        }

        self.questions.insert(id, question);
    }

    /// Get a question by ID
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Question> {
        self.questions.get(id)
    }

    /// Get all questions
    pub fn all(&self) -> impl Iterator<Item = &Question> {
        self.questions.values()
    }

    /// Get questions for a specific domain
    #[must_use]
    pub fn by_domain(&self, domain: &str) -> Vec<&Question> {
        self.by_domain
            .get(domain)
            .map(|ids| ids.iter().filter_map(|id| self.questions.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get questions of a specific type
    #[must_use]
    pub fn by_type(&self, qt: QuestionType) -> Vec<&Question> {
        self.by_type
            .get(&qt)
            .map(|ids| ids.iter().filter_map(|id| self.questions.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get questions at a specific complexity level
    #[must_use]
    pub fn by_complexity(&self, level: u8) -> Vec<&Question> {
        self.by_complexity
            .get(&level)
            .map(|ids| ids.iter().filter_map(|id| self.questions.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get questions with a specific tag
    #[must_use]
    pub fn by_tag(&self, tag: &str) -> Vec<&Question> {
        self.by_tag
            .get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.questions.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get questions for blind evaluation
    #[must_use]
    pub fn blind_questions(&self) -> Vec<&Question> {
        self.questions.values().filter(|q| q.blind_mode).collect()
    }

    /// Total question count
    #[must_use]
    pub fn len(&self) -> usize {
        self.questions.len()
    }

    /// Check if corpus is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.questions.is_empty()
    }

    /// Get corpus statistics
    #[must_use]
    pub fn stats(&self) -> CorpusStats {
        let mut type_counts = HashMap::new();
        let mut domain_counts = HashMap::new();
        let mut complexity_counts = HashMap::new();

        for q in self.questions.values() {
            *type_counts.entry(q.question_type).or_insert(0) += 1;
            *domain_counts.entry(q.domain.clone()).or_insert(0) += 1;
            *complexity_counts.entry(q.complexity_level).or_insert(0) += 1;
        }

        CorpusStats {
            total_questions: self.questions.len(),
            by_type: type_counts,
            by_domain: domain_counts,
            by_complexity: complexity_counts,
            blind_count: self.questions.values().filter(|q| q.blind_mode).count(),
        }
    }
}

/// Statistics about the question corpus
#[derive(Debug)]
pub struct CorpusStats {
    /// Total number of questions
    pub total_questions: usize,
    /// Questions per type
    pub by_type: HashMap<QuestionType, usize>,
    /// Questions per domain
    pub by_domain: HashMap<String, usize>,
    /// Questions per complexity level
    pub by_complexity: HashMap<u8, usize>,
    /// Questions suitable for blind evaluation
    pub blind_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_builder() {
        let q = Question::builder("test_001", "What is the value of X?")
            .ground_truth("42")
            .question_type(QuestionType::FieldRetrieval)
            .domain("test")
            .dataset("test_ds")
            .answer_type(AnswerType::Integer)
            .complexity(2)
            .tag("basic")
            .build();

        assert_eq!(q.id, "test_001");
        assert_eq!(q.ground_truth, "42");
        assert_eq!(q.complexity_level, 2);
        assert!(q.tags.contains(&"basic".to_string()));
    }

    #[test]
    fn test_corpus_indexing() {
        let mut corpus = QuestionCorpus::new();

        corpus.add(
            Question::builder("q1", "Question 1")
                .ground_truth("a")
                .domain("finance")
                .question_type(QuestionType::Aggregation)
                .complexity(3)
                .build(),
        );

        corpus.add(
            Question::builder("q2", "Question 2")
                .ground_truth("b")
                .domain("healthcare")
                .question_type(QuestionType::FieldRetrieval)
                .complexity(1)
                .build(),
        );

        assert_eq!(corpus.len(), 2);
        assert_eq!(corpus.by_domain("finance").len(), 1);
        assert_eq!(corpus.by_type(QuestionType::Aggregation).len(), 1);
        assert_eq!(corpus.by_complexity(1).len(), 1);
    }

    #[test]
    fn test_question_types_coverage() {
        assert_eq!(QuestionType::ALL.len(), 12);
        for qt in QuestionType::ALL {
            assert!(!qt.name().is_empty());
            assert!(!qt.description().is_empty());
            assert!(qt.complexity_score() >= 1 && qt.complexity_score() <= 5);
        }
    }
}
