// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Fixture Dataset Loading for Accuracy Benchmarks
//!
//! Loads pre-prepared fixture files from the fixtures/accuracy directory.
//! Each fixture consists of multiple format files (hedl, json, yaml, xml, toon, csv)
//! and a corresponding .questions.json file.

use std::collections::HashMap;
use std::path::Path;

use crate::accuracy::complexity::ComplexityLevel;
use crate::accuracy::questions::{AnswerType, Question, QuestionType};

/// A dataset loaded from fixture files.
///
/// Contains all format variations and associated questions for benchmarking.
#[derive(Debug, Clone)]
pub struct FixtureDataset {
    /// Dataset name identifier
    pub name: String,
    /// HEDL format data
    pub hedl_data: String,
    /// JSON format data (optional)
    pub json_data: Option<String>,
    /// YAML format data (optional)
    pub yaml_data: Option<String>,
    /// XML format data (optional)
    pub xml_data: Option<String>,
    /// TOON format data (optional)
    pub toon_data: Option<String>,
    /// CSV format data (optional)
    pub csv_data: Option<String>,
    /// Questions to ask about this dataset
    pub questions: Vec<Question>,
    /// Complexity level (parsed from questions file)
    pub complexity: ComplexityLevel,
}

impl FixtureDataset {
    /// Load a fixture dataset from a directory.
    ///
    /// # Arguments
    /// * `fixture_dir` - Path to the fixtures/accuracy directory
    /// * `name` - Name of the fixture (e.g., "ecommerce_orders")
    ///
    /// # File Structure
    /// Expects the following files:
    /// - `{name}.hedl` (required)
    /// - `{name}.json` (optional)
    /// - `{name}.yaml` (optional)
    /// - `{name}.xml` (optional)
    /// - `{name}.toon` (optional)
    /// - `{name}.csv` (optional)
    /// - `{name}.questions.json` (required)
    ///
    /// # Errors
    /// Returns an error if required files are missing or cannot be parsed.
    pub fn from_fixtures(fixture_dir: &Path, name: &str) -> Result<Self, String> {
        use std::fs;

        let base = fixture_dir.join(name);

        // Load HEDL (required)
        let hedl_data = fs::read_to_string(base.with_extension("hedl"))
            .map_err(|e| format!("Failed to read {name}.hedl: {e}"))?;

        // Load optional format files
        let json_data = fs::read_to_string(base.with_extension("json")).ok();
        let yaml_data = fs::read_to_string(base.with_extension("yaml")).ok();
        let xml_data = fs::read_to_string(base.with_extension("xml")).ok();
        let toon_data = fs::read_to_string(base.with_extension("toon")).ok();
        let csv_data = fs::read_to_string(base.with_extension("csv")).ok();

        // Load questions from JSON file (required)
        let questions_path = fixture_dir.join(format!("{name}.questions.json"));
        let questions_json = fs::read_to_string(&questions_path)
            .map_err(|e| format!("Failed to read {name}.questions.json: {e}"))?;

        let questions_data: serde_json::Value = serde_json::from_str(&questions_json)
            .map_err(|e| format!("Failed to parse questions JSON: {e}"))?;

        // Parse complexity level from difficulty field
        let complexity = parse_complexity(&questions_data)?;

        // Parse questions array
        let questions = parse_questions(&questions_data, name)?;

        Ok(Self {
            name: name.to_string(),
            hedl_data,
            json_data,
            yaml_data,
            xml_data,
            toon_data,
            csv_data,
            questions,
            complexity,
        })
    }

    /// Get data for a specific format.
    ///
    /// # Arguments
    /// * `format` - Format name (hedl, json, yaml, xml, toon, csv)
    ///
    /// # Returns
    /// Some(data) if the format is available, None otherwise.
    #[must_use]
    pub fn data_for_format(&self, format: &str) -> Option<&str> {
        match format.to_lowercase().as_str() {
            "hedl" => Some(&self.hedl_data),
            "json" => self.json_data.as_deref(),
            "yaml" => self.yaml_data.as_deref(),
            "xml" => self.xml_data.as_deref(),
            "toon" => self.toon_data.as_deref(),
            "csv" => self.csv_data.as_deref(),
            _ => None,
        }
    }

    /// Get all available formats for this dataset.
    #[must_use]
    pub fn available_formats(&self) -> Vec<&'static str> {
        let mut formats = vec!["hedl"];
        if self.json_data.is_some() {
            formats.push("json");
        }
        if self.yaml_data.is_some() {
            formats.push("yaml");
        }
        if self.xml_data.is_some() {
            formats.push("xml");
        }
        if self.toon_data.is_some() {
            formats.push("toon");
        }
        if self.csv_data.is_some() {
            formats.push("csv");
        }
        formats
    }

    /// Get question count.
    #[must_use]
    pub fn question_count(&self) -> usize {
        self.questions.len()
    }

    /// Get questions by type.
    #[must_use]
    pub fn questions_by_type(&self, qtype: QuestionType) -> Vec<&Question> {
        self.questions
            .iter()
            .filter(|q| q.question_type == qtype)
            .collect()
    }

    /// Create a fixture dataset from inline HEDL source with auto-conversion to other formats.
    ///
    /// Parses the HEDL source and converts it to JSON, YAML, XML, TOON, and CSV.
    /// CSV conversion may fail for nested structures (returns None in that case).
    ///
    /// # Arguments
    /// * `name` - Dataset name identifier
    /// * `hedl` - HEDL source text
    /// * `questions` - Questions for this dataset
    /// * `complexity` - Complexity level of the dataset
    ///
    /// # Errors
    /// Returns an error if HEDL parsing or format conversion fails.
    pub fn from_hedl(
        name: &str,
        hedl: &str,
        questions: Vec<Question>,
        complexity: ComplexityLevel,
    ) -> Result<Self, String> {
        let doc =
            hedl_core::parse(hedl.as_bytes()).map_err(|e| format!("Failed to parse HEDL: {e}"))?;

        let json_data = Some(
            hedl_json::to_json(&doc, &hedl_json::ToJsonConfig::default())
                .map_err(|e| format!("Failed to convert to JSON: {e}"))?,
        );

        let yaml_data = Some(
            hedl_yaml::to_yaml(&doc, &hedl_yaml::ToYamlConfig::default())
                .map_err(|e| format!("Failed to convert to YAML: {e}"))?,
        );

        let xml_data = Some(
            hedl_xml::to_xml(&doc, &hedl_xml::ToXmlConfig::default())
                .map_err(|e| format!("Failed to convert to XML: {e}"))?,
        );

        let toon_data = Some(
            hedl_toon::hedl_to_toon(&doc).map_err(|e| format!("Failed to convert to TOON: {e}"))?,
        );

        // CSV can't represent nested structures
        let csv_data = hedl_csv::to_csv(&doc).ok();

        Ok(Self {
            name: name.to_string(),
            hedl_data: hedl.to_string(),
            json_data,
            yaml_data,
            xml_data,
            toon_data,
            csv_data,
            questions,
            complexity,
        })
    }
}

/// Parse complexity level from questions JSON.
fn parse_complexity(data: &serde_json::Value) -> Result<ComplexityLevel, String> {
    let difficulty = data["difficulty"].as_str().unwrap_or("medium");

    match difficulty.to_lowercase().as_str() {
        "trivial" | "l1" => Ok(ComplexityLevel::L1Trivial),
        "easy" | "basic" | "l2" => Ok(ComplexityLevel::L2Basic),
        "medium" | "intermediate" | "l3" => Ok(ComplexityLevel::L3Intermediate),
        "hard" | "advanced" | "l4" => Ok(ComplexityLevel::L4Advanced),
        "expert" | "l5" => Ok(ComplexityLevel::L5Expert),
        _ => Ok(ComplexityLevel::L3Intermediate),
    }
}

/// Parse questions array from JSON.
fn parse_questions(data: &serde_json::Value, dataset_name: &str) -> Result<Vec<Question>, String> {
    let questions_array = data["questions"]
        .as_array()
        .ok_or("questions field must be an array")?;

    let mut questions = Vec::new();

    for q in questions_array {
        // Parse format-specific ground truths if present
        let ground_truth_by_format = q["ground_truth_by_format"].as_object().map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.to_lowercase(), s.to_string())))
                .collect::<HashMap<String, String>>()
        });

        // Parse question type
        let question_type = parse_question_type(q["type"].as_str().unwrap_or("FieldRetrieval"))?;

        // Parse answer type
        let answer_type = parse_answer_type(&q["answer_type"])?;

        // Extract complexity level from question ID or default to 3
        let complexity_level = extract_complexity_from_id(q["id"].as_str().unwrap_or(""));

        // Parse ground_truth which can be string, int, or float
        let ground_truth = if let Some(s) = q["ground_truth"].as_str() {
            s.to_string()
        } else if let Some(i) = q["ground_truth"].as_i64() {
            i.to_string()
        } else if let Some(f) = q["ground_truth"].as_f64() {
            f.to_string()
        } else {
            String::new()
        };

        // Build the question
        let question = Question {
            id: q["id"].as_str().unwrap_or("").to_string(),
            prompt: q["prompt"].as_str().unwrap_or("").to_string(),
            ground_truth,
            ground_truth_by_format,
            question_type,
            domain: dataset_name.to_string(),
            dataset: dataset_name.to_string(),
            answer_type,
            complexity_level,
            notes: q["notes"].as_str().map(|s| s.to_string()),
            tags: Vec::new(),
            blind_mode: false,
        };

        questions.push(question);
    }

    Ok(questions)
}

/// Parse question type from string.
fn parse_question_type(type_str: &str) -> Result<QuestionType, String> {
    match type_str {
        "Aggregation" => Ok(QuestionType::Aggregation),
        "FieldRetrieval" => Ok(QuestionType::FieldRetrieval),
        "Filtering" => Ok(QuestionType::Filtering),
        "StructureAwareness" => Ok(QuestionType::StructureAwareness),
        "StructuralValidation" => Ok(QuestionType::StructuralValidation),
        "Comparison" => Ok(QuestionType::Comparison),
        "NestedNavigation" | "Hierarchical" => Ok(QuestionType::NestedNavigation),
        "ReferenceResolution" => Ok(QuestionType::ReferenceResolution),
        "TemporalQuery" => Ok(QuestionType::TemporalQuery),
        "RelationshipTraversal" => Ok(QuestionType::RelationshipTraversal),
        "MathematicalOperation" => Ok(QuestionType::MathematicalOperation),
        "PatternMatching" => Ok(QuestionType::PatternMatching),
        _ => Err(format!("Unknown question type: {type_str}")),
    }
}

/// Parse answer type from JSON value.
fn parse_answer_type(value: &serde_json::Value) -> Result<AnswerType, String> {
    let type_str = value.as_str().unwrap_or("String");

    match type_str {
        "String" => Ok(AnswerType::String),
        "StringCaseSensitive" => Ok(AnswerType::StringCaseSensitive),
        "Integer" => Ok(AnswerType::Integer),
        "Number" => {
            // Try to get decimals from config, default to 2
            Ok(AnswerType::Number { decimals: 2 })
        }
        "Boolean" => Ok(AnswerType::Boolean),
        "Date" => Ok(AnswerType::Date),
        "DateTime" => Ok(AnswerType::DateTime),
        "ListOrdered" => Ok(AnswerType::ListOrdered),
        "ListUnordered" => Ok(AnswerType::ListUnordered),
        "JsonObject" => Ok(AnswerType::JsonObject),
        _ => Ok(AnswerType::String),
    }
}

/// Extract complexity level from question ID.
///
/// Looks for patterns like "L3" or "_q3_" in the ID.
/// Defaults to 3 if no pattern is found.
fn extract_complexity_from_id(id: &str) -> u8 {
    // Try to find L1-L5 pattern
    if id.contains("L1") || id.contains("_l1_") {
        return 1;
    }
    if id.contains("L2") || id.contains("_l2_") {
        return 2;
    }
    if id.contains("L3") || id.contains("_l3_") {
        return 3;
    }
    if id.contains("L4") || id.contains("_l4_") {
        return 4;
    }
    if id.contains("L5") || id.contains("_l5_") {
        return 5;
    }

    // Default to medium complexity
    3
}

/// Load all available fixture datasets from a directory.
///
/// # Arguments
/// * `fixture_dir` - Path to the fixtures/accuracy directory
///
/// # Returns
/// A vector of successfully loaded datasets. Failed loads are logged but skipped.
#[must_use]
pub fn load_all_fixtures(fixture_dir: &Path) -> Vec<FixtureDataset> {
    let dataset_names = [
        "ml_training_logs",
        "iot_sensors",
        "ecommerce_orders",
        "blog_platform",
        "sports_statistics",
        "healthcare_records",
        "financial_transactions",
    ];

    let mut datasets = Vec::new();
    for name in dataset_names {
        match FixtureDataset::from_fixtures(fixture_dir, name) {
            Ok(ds) => {
                datasets.push(ds);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load fixture {name}: {e}");
            }
        }
    }

    datasets
}

/// Load all fixtures from the default location (relative to CARGO_MANIFEST_DIR).
#[must_use]
pub fn load_default_fixtures() -> Vec<FixtureDataset> {
    use std::path::PathBuf;
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/accuracy");
    load_all_fixtures(&fixture_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/accuracy")
    }

    #[test]
    fn test_load_ecommerce_fixture() {
        let dir = fixture_dir();
        let dataset = FixtureDataset::from_fixtures(&dir, "ecommerce_orders")
            .expect("Failed to load ecommerce_orders fixture");

        assert_eq!(dataset.name, "ecommerce_orders");
        assert!(!dataset.hedl_data.is_empty());
        assert!(!dataset.questions.is_empty());

        // Should have multiple formats
        assert!(dataset.json_data.is_some());
        assert!(dataset.yaml_data.is_some());

        // Check complexity was parsed
        assert!(matches!(dataset.complexity, ComplexityLevel::L4Advanced));
    }

    #[test]
    fn test_available_formats() {
        let dir = fixture_dir();
        let dataset = FixtureDataset::from_fixtures(&dir, "ecommerce_orders")
            .expect("Failed to load fixture");

        let formats = dataset.available_formats();
        assert!(formats.contains(&"hedl"));
        assert!(formats.len() > 1); // Should have multiple formats
    }

    #[test]
    fn test_data_for_format() {
        let dir = fixture_dir();
        let dataset = FixtureDataset::from_fixtures(&dir, "ecommerce_orders")
            .expect("Failed to load fixture");

        // HEDL should always be available
        assert!(dataset.data_for_format("hedl").is_some());
        assert!(dataset.data_for_format("HEDL").is_some()); // Case insensitive

        // JSON should be available for this fixture
        assert!(dataset.data_for_format("json").is_some());
    }

    #[test]
    fn test_questions_parsed() {
        let dir = fixture_dir();
        let dataset = FixtureDataset::from_fixtures(&dir, "ecommerce_orders")
            .expect("Failed to load fixture");

        assert!(!dataset.questions.is_empty());

        // Check first question structure
        let first = &dataset.questions[0];
        assert!(!first.id.is_empty());
        assert!(!first.prompt.is_empty());
        assert!(!first.ground_truth.is_empty());
        assert_eq!(first.dataset, "ecommerce_orders");
    }

    #[test]
    fn test_format_specific_ground_truth() {
        let dir = fixture_dir();
        let dataset = FixtureDataset::from_fixtures(&dir, "ecommerce_orders")
            .expect("Failed to load fixture");

        // Find a question with format-specific ground truth
        let question_with_format_specific = dataset
            .questions
            .iter()
            .find(|q| q.has_format_specific_answers());

        assert!(
            question_with_format_specific.is_some(),
            "Should have at least one question with format-specific answers"
        );
    }

    #[test]
    fn test_load_all_fixtures() {
        let dir = fixture_dir();
        let datasets = load_all_fixtures(&dir);

        // Should load multiple datasets
        assert!(!datasets.is_empty());

        // All should have valid data
        for ds in &datasets {
            assert!(!ds.name.is_empty());
            assert!(!ds.hedl_data.is_empty());
            assert!(!ds.questions.is_empty());
        }
    }

    #[test]
    fn test_questions_by_type() {
        let dir = fixture_dir();
        let dataset = FixtureDataset::from_fixtures(&dir, "ecommerce_orders")
            .expect("Failed to load fixture");

        let aggregation_questions = dataset.questions_by_type(QuestionType::Aggregation);
        assert!(!aggregation_questions.is_empty());

        let field_retrieval_questions = dataset.questions_by_type(QuestionType::FieldRetrieval);
        assert!(!field_retrieval_questions.is_empty());
    }

    #[test]
    fn test_complexity_parsing() {
        assert_eq!(
            parse_complexity(&serde_json::json!({"difficulty": "easy"})).unwrap(),
            ComplexityLevel::L2Basic
        );
        assert_eq!(
            parse_complexity(&serde_json::json!({"difficulty": "hard"})).unwrap(),
            ComplexityLevel::L4Advanced
        );
        assert_eq!(
            parse_complexity(&serde_json::json!({"difficulty": "expert"})).unwrap(),
            ComplexityLevel::L5Expert
        );
    }
}
