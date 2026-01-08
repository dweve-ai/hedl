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


//! LLM Accuracy Testing Harness
//!
//! Tests how well LLMs can comprehend and extract information from HEDL documents
//! compared to equivalent JSON/YAML representations.
//!
//! Supported providers (December 2025):
//! - DeepSeek: deepseek-chat, deepseek-reasoner (V3.2)
//! - Mistral: mistral-large-latest, magistral-medium, devstral-2
//!
//! Usage:
//! ```bash
//! # Run with DeepSeek API
//! DEEPSEEK_API_KEY=... cargo run --bin accuracy -- --provider deepseek --model deepseek-chat
//!
//! # Run with Mistral API
//! MISTRAL_API_KEY=... cargo run --bin accuracy -- --provider mistral --model mistral-large-latest
//!
//! # Dry run (no API calls, just print prompts)
//! cargo run --bin accuracy -- --dry-run
//! ```

use super::questions::{AnswerType, Question, QuestionType};

/// Result of a single accuracy test
#[derive(Debug, Clone)]
pub struct TestResult {
    pub question: String,
    pub question_type: QuestionType,
    pub expected: String,
    pub actual: String,
    pub correct: bool,
    pub format: DataFormat,
    pub difficulty: Difficulty,
    pub model: String,
    pub latency_ms: u64,
    pub tokens_in: usize,
    pub tokens_out: usize,
}

/// Data format being tested for LLM accuracy comparison.
///
/// Represents different serialization formats that HEDL documents can be
/// converted to for testing LLM comprehension across formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFormat {
    Hedl,
    Json,
    Yaml,
    Xml,
    Toon,
    Csv,
}

impl std::fmt::Display for DataFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataFormat::Hedl => write!(f, "HEDL"),
            DataFormat::Json => write!(f, "JSON"),
            DataFormat::Yaml => write!(f, "YAML"),
            DataFormat::Xml => write!(f, "XML"),
            DataFormat::Toon => write!(f, "TOON"),
            DataFormat::Csv => write!(f, "CSV"),
        }
    }
}

/// Dataset difficulty level for comprehensive accuracy testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Difficulty {
    /// Easy: 5-10 entities, simple flat structure, basic field retrieval
    Easy,
    /// Medium: 50-100 entities, some nesting, aggregation queries
    Medium,
    /// Hard: 200+ entities, deep nesting, cross-references, complex queries
    Hard,
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Difficulty::Easy => write!(f, "Easy"),
            Difficulty::Medium => write!(f, "Medium"),
            Difficulty::Hard => write!(f, "Hard"),
        }
    }
}

/// Aggregated results for a test run
#[derive(Debug, Default)]
pub struct AccuracyReport {
    pub model: String,
    pub total_questions: usize,
    pub results_by_format: Vec<FormatResults>,
    pub results_by_type: Vec<TypeResults>,
    pub results_by_difficulty: Vec<DifficultyResults>,
}

/// Aggregated accuracy results for a specific data format.
///
/// Contains metrics about how well an LLM performed on questions
/// when the data was presented in a particular format (HEDL, JSON, etc.).
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
}

/// Aggregated accuracy results for a specific question type.
///
/// Contains metrics about how well an LLM performed on different
/// categories of questions (field retrieval, aggregation, filtering, etc.).
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
///
/// Used for comparing token efficiency across difficulty levels.
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

impl AccuracyReport {
    /// Generate a formatted report
    pub fn report(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("\n{}\n", "=".repeat(70)));
        out.push_str(&format!("LLM Accuracy Report - Model: {}\n", self.model));
        out.push_str(&format!("{}\n\n", "=".repeat(70)));

        out.push_str("Results by Format:\n");
        out.push_str(&format!("{:-<50}\n", ""));
        out.push_str(&format!(
            "{:<10} {:>10} {:>10} {:>12} {:>12}\n",
            "Format", "Correct", "Total", "Accuracy", "Avg Latency"
        ));
        out.push_str(&format!("{:-<50}\n", ""));

        for fr in &self.results_by_format {
            let accuracy = if fr.total > 0 {
                fr.correct as f64 / fr.total as f64 * 100.0
            } else {
                0.0
            };
            out.push_str(&format!(
                "{:<10} {:>10} {:>10} {:>11.1}% {:>10.0}ms\n",
                fr.format, fr.correct, fr.total, accuracy, fr.avg_latency_ms
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

        out.push_str(&format!("\n{}\n", "=".repeat(70)));

        out
    }
}

/// LLM API Provider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// DeepSeek API (api.deepseek.com)
    /// Models: deepseek-chat, deepseek-reasoner
    DeepSeek,
    /// Mistral API (api.mistral.ai)
    /// Models: mistral-large-latest, magistral-medium, devstral-2, ministral-3b
    Mistral,
    /// OpenAI API (api.openai.com)
    /// Models: gpt-5.1, gpt-5.1-codex, gpt-5, gpt-4.1, gpt-4o, o1, o3
    OpenAI,
}

impl Provider {
    pub fn api_base(&self) -> &'static str {
        match self {
            Provider::DeepSeek => "https://api.deepseek.com/v1",
            Provider::Mistral => "https://api.mistral.ai/v1",
            Provider::OpenAI => "https://api.openai.com/v1",
        }
    }

    pub fn env_var(&self) -> &'static str {
        match self {
            Provider::DeepSeek => "DEEPSEEK_API_KEY",
            Provider::Mistral => "MISTRAL_API_KEY",
            Provider::OpenAI => "OPENAI_API_KEY",
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            Provider::DeepSeek => "deepseek-chat",
            Provider::Mistral => "mistral-large-latest",
            Provider::OpenAI => "gpt-5.1",
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::DeepSeek => write!(f, "DeepSeek"),
            Provider::Mistral => write!(f, "Mistral"),
            Provider::OpenAI => write!(f, "OpenAI"),
        }
    }
}

/// Configuration for accuracy testing
#[derive(Debug, Clone)]
pub struct AccuracyConfig {
    /// API provider
    pub provider: Provider,
    /// Model to test (e.g., "deepseek-chat", "mistral-large-latest")
    pub model: String,
    /// Formats to test
    pub formats: Vec<DataFormat>,
    /// Question types to include
    pub question_types: Option<Vec<QuestionType>>,
    /// Difficulty levels to include
    pub difficulties: Option<Vec<Difficulty>>,
    /// Maximum questions per category (for faster testing)
    pub max_per_category: Option<usize>,
    /// Dry run mode (don't call API)
    pub dry_run: bool,
}

impl Default for AccuracyConfig {
    fn default() -> Self {
        Self {
            provider: Provider::DeepSeek,
            model: "deepseek-chat".to_string(),
            formats: vec![
                DataFormat::Hedl,
                DataFormat::Toon,
                DataFormat::Json,
                DataFormat::Yaml,
                DataFormat::Xml,
                DataFormat::Csv,
            ],
            question_types: None,
            difficulties: None,
            max_per_category: None,
            dry_run: false,
        }
    }
}

/// Test dataset with HEDL source and questions
#[derive(Debug, Clone)]
pub struct TestDataset {
    pub name: String,
    pub difficulty: Difficulty,
    pub hedl: String,
    pub json: String,
    pub yaml: String,
    pub xml: String,
    pub toon: String,
    pub csv: String,
    pub questions: Vec<Question>,
}

impl TestDataset {
    /// Create a test dataset from HEDL source
    pub fn from_hedl(name: &str, hedl: &str, questions: Vec<Question>) -> Result<Self, String> {
        Self::from_hedl_with_difficulty(name, hedl, questions, Difficulty::Medium)
    }

    /// Create a test dataset from HEDL source with specified difficulty
    pub fn from_hedl_with_difficulty(
        name: &str,
        hedl: &str,
        questions: Vec<Question>,
        difficulty: Difficulty,
    ) -> Result<Self, String> {
        let doc = hedl_core::parse(hedl.as_bytes())
            .map_err(|e| format!("Failed to parse HEDL: {}", e))?;

        let json = hedl_json::to_json(&doc, &hedl_json::ToJsonConfig::default())
            .map_err(|e| format!("Failed to convert to JSON: {}", e))?;

        let yaml = hedl_yaml::to_yaml(&doc, &hedl_yaml::ToYamlConfig::default())
            .map_err(|e| format!("Failed to convert to YAML: {}", e))?;

        let xml = hedl_xml::to_xml(&doc, &hedl_xml::ToXmlConfig::default())
            .map_err(|e| format!("Failed to convert to XML: {}", e))?;

        // Generate TOON using proper hedl-toon crate
        let toon = hedl_toon::hedl_to_toon(&doc)
            .map_err(|e| format!("Failed to convert to TOON: {}", e))?;

        // Generate CSV (may fail for nested/complex structures - that's OK)
        let csv = hedl_csv::to_csv(&doc).unwrap_or_else(|_| {
            // CSV can't represent nested structures - provide explanation
            "[CSV format cannot represent this nested/complex data structure]".to_string()
        });

        Ok(Self {
            name: name.to_string(),
            difficulty,
            hedl: hedl.to_string(), // Use real HEDL as-is
            json,
            yaml,
            xml,
            toon,
            csv,
            questions,
        })
    }

    /// Get data in specified format
    pub fn data(&self, format: DataFormat) -> &str {
        match format {
            DataFormat::Hedl => &self.hedl,
            DataFormat::Json => &self.json,
            DataFormat::Yaml => &self.yaml,
            DataFormat::Xml => &self.xml,
            DataFormat::Toon => &self.toon,
            DataFormat::Csv => &self.csv,
        }
    }
}

/// Build the prompt for LLM testing
pub fn build_prompt(data: &str, format: DataFormat, question: &Question) -> String {
    let format_desc = match format {
        DataFormat::Hedl => "HEDL (Human-Efficient Data Language)",
        DataFormat::Json => "JSON",
        DataFormat::Yaml => "YAML",
        DataFormat::Xml => "XML",
        DataFormat::Toon => "TOON (Token-Oriented Object Notation)",
        DataFormat::Csv => "CSV (Comma-Separated Values)",
    };

    let answer_instruction = match question.answer_type {
        AnswerType::String => "Respond with just the string value, no quotes.".to_string(),
        AnswerType::Integer => {
            // For counting questions, add explicit instructions
            if question.prompt.to_lowercase().contains("how many") {
                "IMPORTANT: Go through EVERY SINGLE record in the data from start to end. Count each matching record. Do not estimate or sample. After counting all records, respond with ONLY the final count as a single number.".to_string()
            } else {
                "Respond with just the number, no formatting.".to_string()
            }
        }
        AnswerType::Number { decimals } => {
            if decimals == 0 {
                "Respond with just the number.".to_string()
            } else {
                "Respond with just the decimal number.".to_string()
            }
        }
        AnswerType::Boolean => "Respond with 'true' or 'false'.".to_string(),
        AnswerType::Date => "Respond with the date in YYYY-MM-DD format.".to_string(),
        AnswerType::CsvListOrdered => "Respond with a comma-separated list in order.".to_string(),
        AnswerType::CsvListUnordered => {
            "Respond with a comma-separated list of unique values only (order doesn't matter). Do not include duplicates or additional context.".to_string()
        }
    };

    // Format hints - ~10 lines each for fairness. Not counted in token metrics.
    let format_hint = match format {
        DataFormat::Hedl => {
            r#"
HEDL FORMAT GUIDE:
- %STRUCT: Type: [col1,col2,col3] declares columns for entity type
- | rows contain values matching column positions
- |[N] indicates N children follow, indented under this row
- Indentation (2 spaces) shows nesting hierarchy
- @Type:id references another entity
- Count hints (N) in %STRUCT show expected entity count
- --- separates header from data section"#
        }
        DataFormat::Toon => {
            r#"
TOON FORMAT GUIDE:
- List header: key: [count]{col1,col2,...} declares N items with columns
- Each following line is a data row: val1,val2,...
- First value in each row is the ID
- Indentation (2 spaces) shows nested children
- Scalars: key: value for simple key-value pairs
- Objects: key: {} followed by indented children
- Strings with special chars are quoted
- Numbers, booleans, null are unquoted
- References appear as @Type:id strings"#
        }
        DataFormat::Json => {
            r#"
JSON FORMAT GUIDE:
- Objects: {"key": value, ...} with quoted keys
- Arrays: [value1, value2, ...]
- Strings must be double-quoted
- Numbers, booleans (true/false), null are unquoted
- Nested structures use indentation for readability
- No trailing commas allowed
- References appear as {"$ref": "Type:id"} objects
- null represents missing values
- All keys are strings"#
        }
        DataFormat::Yaml => {
            r#"
YAML FORMAT GUIDE:
- Key-value: key: value (space after colon required)
- Lists: - item (dash prefix) or [item1, item2]
- Nested structures use indentation (2 spaces)
- Strings usually unquoted unless special chars
- Numbers and booleans parsed automatically
- null or ~ for missing values
- References appear as strings @Type:id
- Multi-line strings use | or >
- Comments start with #"#
        }
        DataFormat::Xml => {
            r#"
XML FORMAT GUIDE:
- Elements: <tag>content</tag> or <tag attr="val"/>
- Root element wraps all data
- Nested elements show hierarchy
- Attributes for metadata, elements for data
- Text content between open/close tags
- Empty elements: <tag/> or <tag></tag>
- Special chars escaped: &lt; &gt; &amp; &quot;
- CDATA for unescaped content: <![CDATA[...]]>
- References as @Type:id in text content"#
        }
        DataFormat::Csv => {
            r#"
CSV FORMAT GUIDE:
- First row contains column headers
- Each subsequent row is one data record
- Values separated by commas
- Quoted strings for values with commas/quotes
- Double quotes escaped as ""
- Each row should have same number of columns
- Empty values represented as empty between commas
- No nested structures - flat tabular data only
- Line breaks separate rows"#
        }
    };

    format!(
        r#"You are analyzing data in {} format. Answer the question based solely on the data provided.{}

DATA:
```
{}
```

QUESTION: {}

INSTRUCTIONS: {}

ANSWER:"#,
        format_desc, format_hint, data, question.prompt, answer_instruction
    )
}

/// Generate comprehensive test datasets with varying difficulty levels
/// - Easy: 5-10 entities, flat structure, simple queries
/// - Medium: 50-100 entities, some nesting, aggregation
/// - Hard: 200+ entities, deep nesting, cross-references, complex queries
pub fn generate_test_datasets() -> Vec<TestDataset> {
    let mut datasets = Vec::new();

    // ========== EASY DATASETS (5-10 entities, flat, simple queries) ==========

    // Easy 1: Simple flat user list (5 users)
    let easy_users_hedl = crate::datasets::generate_users(5);
    let easy_user_questions = generate_easy_user_questions();
    if let Ok(ds) = TestDataset::from_hedl_with_difficulty(
        "easy_users_5",
        &easy_users_hedl,
        easy_user_questions,
        Difficulty::Easy,
    ) {
        datasets.push(ds);
    }

    // Easy 2: Simple product catalog (8 products)
    let easy_products_hedl = crate::datasets::generate_products(8);
    let easy_product_questions = generate_easy_product_questions();
    if let Ok(ds) = TestDataset::from_hedl_with_difficulty(
        "easy_products_8",
        &easy_products_hedl,
        easy_product_questions,
        Difficulty::Easy,
    ) {
        datasets.push(ds);
    }

    // ========== MEDIUM DATASETS (50-100 entities, some nesting) ==========

    // Medium 1: Users with varied roles (100 users)
    let users_hedl = crate::datasets::generate_users(100);
    let user_questions = generate_large_user_questions();
    if let Ok(ds) = TestDataset::from_hedl_with_difficulty(
        "medium_users_100",
        &users_hedl,
        user_questions,
        Difficulty::Medium,
    ) {
        datasets.push(ds);
    }

    // Medium 2: Events log (150 entries) - matches question ground_truth
    let events_hedl = crate::datasets::generate_events(150);
    let event_questions = generate_large_event_questions();
    if let Ok(ds) = TestDataset::from_hedl_with_difficulty(
        "medium_events_150",
        &events_hedl,
        event_questions,
        Difficulty::Medium,
    ) {
        datasets.push(ds);
    }

    // Medium 3: Analytics metrics (100 metrics with tensors)
    let analytics_hedl = crate::datasets::generate_analytics(100);
    let analytics_questions = generate_large_analytics_questions();
    if let Ok(ds) = TestDataset::from_hedl_with_difficulty(
        "medium_analytics_100",
        &analytics_hedl,
        analytics_questions,
        Difficulty::Medium,
    ) {
        datasets.push(ds);
    }

    // ========== HARD DATASETS (200+ entities, deep nesting, cross-refs) ==========

    // Hard 1: Blog with deep nesting (50 posts, 4 comments each = 200 comments)
    let blog_hedl = crate::datasets::generate_blog(50, 4);
    let blog_questions = generate_large_blog_questions();
    if let Ok(ds) = TestDataset::from_hedl_with_difficulty(
        "hard_blog_nested",
        &blog_hedl,
        blog_questions,
        Difficulty::Hard,
    ) {
        datasets.push(ds);
    }

    // Hard 2: Orders with nested items (80 orders)
    let orders_hedl = crate::datasets::generate_orders(80);
    let order_questions = generate_order_questions();
    if let Ok(ds) = TestDataset::from_hedl_with_difficulty(
        "hard_orders_80",
        &orders_hedl,
        order_questions,
        Difficulty::Hard,
    ) {
        datasets.push(ds);
    }

    // Hard 3: Large events (300 log entries)
    let large_events_hedl = crate::datasets::generate_events(300);
    let large_event_questions = generate_hard_event_questions();
    if let Ok(ds) = TestDataset::from_hedl_with_difficulty(
        "hard_events_300",
        &large_events_hedl,
        large_event_questions,
        Difficulty::Hard,
    ) {
        datasets.push(ds);
    }

    // ========== STRUCTURALLY COMPLEX DATASETS (showcase HEDL features) ==========

    // NOTE: Ditto-heavy dataset excluded from LLM accuracy benchmarks.
    // Ditto (^) saves ~10% tokens but reduces LLM comprehension accuracy.
    // For repeated values, %ALIAS is recommended for LLM contexts.

    // Hard 4: Reference-heavy dataset (tests @Type:id cross-references)
    let refs_hedl = crate::datasets::generate_reference_heavy(10);
    let refs_questions = generate_reference_questions();
    if let Ok(ds) = TestDataset::from_hedl_with_difficulty(
        "reference_heavy_10",
        &refs_hedl,
        refs_questions,
        Difficulty::Hard,
    ) {
        datasets.push(ds);
    }

    // Hard 6: Deep hierarchy (tests 5-level %NEST structure)
    let deep_hedl = crate::datasets::generate_deep_hierarchy(3);
    let deep_questions = generate_deep_hierarchy_questions();
    if let Ok(ds) = TestDataset::from_hedl_with_difficulty(
        "deep_hierarchy_3div",
        &deep_hedl,
        deep_questions,
        Difficulty::Hard,
    ) {
        datasets.push(ds);
    }

    // 6. DEEP ORG: Smaller but deeply nested organization (no aliases/dittos for LLM accuracy)
    let org_hedl = r#"%VERSION: 1.0
%STRUCT: Company: [id,name,founded,industry,status]
%STRUCT: Department: [id,name,budget,head,location]
%STRUCT: Team: [id,name,focus,lead,members]
%STRUCT: Employee: [id,name,role,salary,status,work_mode]
%NEST: Company > Department
%NEST: Department > Team
%NEST: Team > Employee
---
companies: @Company
  |acme,Acme Corp,1985,Technology,active
    |eng,Engineering,2500000,@Employee:e001,San Francisco HQ Building A
      |platform,Platform Team,Infrastructure,@Employee:e002,[e002, e003, e004, e005]
        |e001,Alice Chen,VP Engineering,185000,active,onsite
        |e002,Bob Kumar,Staff Engineer,165000,active,remote
        |e003,Carol Smith,Senior Engineer,145000,active,remote
        |e004,David Lee,Senior Engineer,142000,active,onsite
        |e005,Eve Wilson,Engineer,125000,active,onsite
      |frontend,Frontend Team,User Experience,@Employee:e006,[e006, e007, e008]
        |e006,Frank Zhang,Tech Lead,155000,active,remote
        |e007,Grace Park,Senior Engineer,140000,active,remote
        |e008,Henry Brown,Engineer,120000,active,onsite
      |ml,ML Team,AI/ML Products,@Employee:e009,[e009, e010, e011, e012]
        |e009,Iris Johnson,ML Lead,175000,active,remote
        |e010,Jack Davis,Senior ML Engineer,160000,active,remote
        |e011,Kate Miller,ML Engineer,135000,active,remote
        |e012,Leo Garcia,Data Scientist,130000,active,onsite
    |sales,Sales,1800000,@Employee:e013,San Francisco HQ Building B
      |enterprise,Enterprise Sales,Fortune 500,@Employee:e013,[e013, e014, e015]
        |e013,Maria Rodriguez,Sales Director,150000,active,onsite
        |e014,Nathan White,Account Exec,95000,active,remote
        |e015,Olivia Taylor,Account Exec,92000,active,remote
      |smb,SMB Sales,Small Business,@Employee:e016,[e016, e017]
        |e016,Peter Kim,Sales Manager,110000,active,remote
        |e017,Quinn Adams,Sales Rep,75000,active,onsite
    |ops,Operations,950000,@Employee:e018,San Francisco HQ Building C
      |devops,DevOps,Cloud Infrastructure,@Employee:e018,[e018, e019, e020]
        |e018,Rachel Green,DevOps Lead,145000,active,remote
        |e019,Sam Turner,Site Reliability Engineer,130000,active,remote
        |e020,Tina Black,Cloud Engineer,125000,active,onsite
"#;

    let org_questions = generate_organization_questions();
    if let Ok(ds) = TestDataset::from_hedl("organization", org_hedl, org_questions) {
        datasets.push(ds);
    }

    // 2. E-COMMERCE: Products with tensors, ratings, inventory tracking
    // Schema optimized: frequently-queried fields (name, status, ratings) placed early
    let ecommerce_hedl = r#"%VERSION: 1.0
%STRUCT: Category: [id,name,parent,margin]
%STRUCT: Product: [id,name,status,ratings,price,cost,stock,category,tags]
%STRUCT: Review: [id,author,rating,helpful_votes,text]
%NEST: Product > Review
---
categories: @Category
  |electronics,Electronics,~,0.25
  |computers,Computers,@Category:electronics,0.22
  |laptops,Laptops,@Category:computers,0.20
  |accessories,Accessories,@Category:electronics,0.35
  |audio,Audio,@Category:electronics,0.30
  |home,Home & Office,~,0.40
  |furniture,Furniture,@Category:home,0.45
products: @Product
  |p001,ProBook X1 Laptop 15in,in_stock,[5, 4, 5, 5, 4, 5, 4],1299.99,980.00,45,@Category:laptops,[laptop, professional, 15inch]
    |r001,john_doe,5,23,Excellent performance for development work!
    |r002,tech_reviewer,4,15,Great but keyboard could be better
    |r003,power_user,5,31,Best laptop I have owned
  |p002,Wireless Ergonomic Mouse,in_stock,[4, 4, 5, 4, 3, 4, 5],79.99,32.00,230,@Category:accessories,[mouse, ergonomic, wireless]
    |r004,office_worker,4,12,Comfortable for long hours
    |r005,gamer123,3,8,Good for work but not gaming
  |p003,USB-C Hub 7in1,low_stock,[5, 5, 4, 5, 5, 4, 5],59.99,22.00,12,@Category:accessories,[hub, usbc, multiport]
    |r006,road_warrior,5,45,Essential for travel!
  |p004,Noise-Canceling Headphones,out_of_stock,[5, 5, 5, 5, 4, 5, 5],299.99,150.00,0,@Category:audio,[headphones, anc, premium]
    |r007,audiophile,5,67,Studio-quality sound
    |r008,commuter,5,52,ANC is incredible on flights
    |r009,music_lover,4,28,Amazing but pricey
  |p005,Standing Desk Pro,in_stock,[4, 5, 4, 4, 5, 3, 4],599.99,320.00,28,@Category:furniture,[desk, standing, adjustable]
    |r010,wfh_pro,5,89,Worth every penny for my back
    |r011,office_mgr,4,34,Solid build quality
  |p006,Mechanical Keyboard TKL,in_stock,[5, 5, 5, 4, 5, 5, 4],149.99,65.00,85,@Category:accessories,[keyboard, mechanical, tkl]
    |r012,typist,5,56,Best typing experience ever!
  |p007,4K Monitor 32in,low_stock,[4, 5, 4, 4, 5, 4, 5],449.99,280.00,6,@Category:computers,[monitor, 4k, 32inch]
    |r013,designer,5,41,Color accuracy is perfect
    |r014,developer,4,29,Great for coding split-screen
"#;

    let ecommerce_questions = generate_ecommerce_questions();
    if let Ok(ds) = TestDataset::from_hedl("ecommerce", ecommerce_hedl, ecommerce_questions) {
        datasets.push(ds);
    }

    // 3. ANALYTICS: Time-series with complex aggregations
    // Flat table - uses inline schema and compact format
    let analytics_hedl = r#"%VERSION: 1.0
---
metrics: @Metric[id,metric_name,value,source,timestamp,dimensions,percentiles]
  |m001,page_load_time,1250,web,2024-12-01T00:00:00Z,[homepage, us-east],[850, 1100, 1400, 2100, 3500]
  |m002,api_latency,45,web,2024-12-01T00:00:00Z,[auth, us-east],[12, 28, 52, 89, 145]
  |m003,app_startup,890,mobile,2024-12-01T00:00:00Z,[ios, us-east],[620, 780, 920, 1200, 1800]
  |m004,page_load_time,1180,web,2024-12-01T01:00:00Z,[homepage, us-west],[800, 1050, 1350, 2000, 3200]
  |m005,api_latency,52,web,2024-12-01T01:00:00Z,[checkout, us-west],[15, 32, 58, 95, 160]
  |m006,app_startup,920,mobile,2024-12-01T01:00:00Z,[android, us-west],[650, 810, 980, 1300, 1950]
  |m007,request_count,15420,api,2024-12-01T02:00:00Z,[orders, eu-central],~
  |m008,request_count,12850,api,2024-12-01T02:00:00Z,[users, eu-central],~
  |m009,request_count,8930,api,2024-12-01T02:00:00Z,[products, eu-central],~
  |m010,error_rate,0.023,web,2024-12-01T03:00:00Z,[checkout, us-east],~
  |m011,error_rate,0.015,web,2024-12-01T03:00:00Z,[search, us-east],~
  |m012,crash_rate,0.0045,mobile,2024-12-01T03:00:00Z,[ios, us-east],~
  |m013,crash_rate,0.0082,mobile,2024-12-01T03:00:00Z,[android, us-east],~
  |m014,throughput,2340,api,2024-12-01T04:00:00Z,[orders, us-east],~
  |m015,throughput,1890,api,2024-12-01T04:00:00Z,[orders, eu-west],~
  |m016,throughput,3210,api,2024-12-01T04:00:00Z,[orders, ap-south],~
"#;

    let analytics_questions = generate_analytics_questions();
    if let Ok(ds) = TestDataset::from_hedl("analytics", analytics_hedl, analytics_questions) {
        datasets.push(ds);
    }

    // 4. KNOWLEDGE GRAPH: Complex cross-references
    let knowledge_hedl = r#"%VERSION: 1.0
%STRUCT: Person: [id,name,birth_year,occupation,known_for]
%STRUCT: Organization: [id,name,founded,type,headquarters]
%STRUCT: Publication: [id,title,year,authors,citations,topics]
%STRUCT: Affiliation: [id,person,org,role,start_year,end_year]
---
people: @Person
  |turing,Alan Turing,1912,Mathematician,[TuringMachine, Enigma, AIFoundations]
  |shannon,Claude Shannon,1916,Engineer,[InformationTheory, BooleanAlgebra]
  |hopper,Grace Hopper,1906,Computer Scientist,[COBOL, Compilers, BugTerm]
  |dijkstra,Edsger Dijkstra,1930,Computer Scientist,[ShortestPath, StructuredProgramming]
  |knuth,Donald Knuth,1938,Computer Scientist,[TeX, TAOCP, AlgorithmAnalysis]
  |lovelace,Ada Lovelace,1815,Mathematician,[FirstProgrammer, AnalyticalEngine]
  |mccarthy,John McCarthy,1927,Computer Scientist,[LISP, AITerm, TimeSharing]
  |kay,Alan Kay,1940,Computer Scientist,[OOP, Smalltalk, Dynabook]
orgs: @Organization
  |mit,MIT,1861,University,Cambridge MA
  |stanford,Stanford University,1885,University,Stanford CA
  |bell,Bell Labs,1925,Research Lab,Murray Hill NJ
  |bletchley,Bletchley Park,1939,Government,Milton Keynes UK
  |ibm,IBM,1911,Corporation,Armonk NY
publications: @Publication
  |pub001,On Computable Numbers,1936,[@Person:turing],12500,[computation, decidability, TuringMachine]
  |pub002,A Mathematical Theory of Communication,1948,[@Person:shannon],143000,[information, entropy, coding]
  |pub003,A Note on Two Problems in Connexion with Graphs,1959,[@Person:dijkstra],8900,[algorithms, shortestPath, graphTheory]
  |pub004,The Art of Computer Programming Vol 1,1968,[@Person:knuth],25000,[algorithms, dataStructures, analysis]
  |pub005,Recursive Functions of Symbolic Expressions,1960,[@Person:mccarthy],7200,[LISP, functionalProgramming, AI]
affiliations: @Affiliation
  |aff01,@Person:turing,@Organization:bletchley,Codebreaker,1939,1945
  |aff02,@Person:shannon,@Organization:bell,Research Mathematician,1941,1972
  |aff03,@Person:hopper,@Organization:ibm,Senior Consultant,1967,1971
  |aff04,@Person:dijkstra,@Organization:mit,Visiting Professor,1973,1984
  |aff05,@Person:knuth,@Organization:stanford,Professor Emeritus,1968,~
  |aff06,@Person:mccarthy,@Organization:mit,Professor,1958,1962
  |aff07,@Person:mccarthy,@Organization:stanford,Professor,1962,2011
  |aff08,@Person:kay,@Organization:stanford,Adjunct Professor,1984,~
"#;

    let knowledge_questions = generate_knowledge_questions();
    if let Ok(ds) = TestDataset::from_hedl("knowledge", knowledge_hedl, knowledge_questions) {
        datasets.push(ds);
    }

    datasets
}

/// Questions for 100-user dataset
fn generate_large_user_questions() -> Vec<Question> {
    vec![
        Question {
            id: "users100_q01".into(),
            dataset: "users_100".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many users have the role 'admin'?".into(),
            ground_truth: "23".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests role counting".into()),
        },
        Question {
            id: "users100_q02".into(),
            dataset: "users_100".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many total users are in the dataset?".into(),
            ground_truth: "100".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "users100_q03".into(),
            dataset: "users_100".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is the email of the user with id 'user50'?".into(),
            ground_truth: "cole.kihn@example.com".into(),
            answer_type: AnswerType::String,
            notes: Some("Tests mid-list lookup".into()),
        },
        Question {
            id: "users100_q04".into(),
            dataset: "users_100".into(),
            question_type: QuestionType::Filtering,
            prompt: "How many users were created in 2023?".into(),
            ground_truth: "21".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
    ]
}

/// Questions for nested blog dataset (50 posts, 200 comments)
fn generate_large_blog_questions() -> Vec<Question> {
    vec![
        Question {
            id: "blog_q01".into(),
            dataset: "blog_nested".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many total comments are in the blog?".into(),
            ground_truth: "200".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests nested child counting".into()),
        },
        Question {
            id: "blog_q02".into(),
            dataset: "blog_nested".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many posts are in the blog?".into(),
            ground_truth: "50".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "blog_q03".into(),
            dataset: "blog_nested".into(),
            question_type: QuestionType::StructureAwareness,
            prompt: "How many comments does post1 have?".into(),
            ground_truth: "4".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests nested child counting".into()),
        },
    ]
}

/// Questions for 150-event log dataset
fn generate_large_event_questions() -> Vec<Question> {
    vec![
        Question {
            id: "events_q01".into(),
            dataset: "events_150".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many ERROR level events are there?".into(),
            ground_truth: "31".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests level counting".into()),
        },
        Question {
            id: "events_q02".into(),
            dataset: "events_150".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many total events are logged?".into(),
            ground_truth: "150".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "events_q03".into(),
            dataset: "events_150".into(),
            question_type: QuestionType::Filtering,
            prompt: "How many events are from the 'auth' service?".into(),
            ground_truth: "24".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests service filtering".into()),
        },
    ]
}

/// Questions for nested orders dataset
fn generate_order_questions() -> Vec<Question> {
    vec![
        Question {
            id: "orders_q01".into(),
            dataset: "orders_nested".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many orders are in the dataset?".into(),
            ground_truth: "80".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "orders_q02".into(),
            dataset: "orders_nested".into(),
            question_type: QuestionType::StructureAwareness,
            prompt: "How many items are in order 'ord1'?".into(),
            ground_truth: "1".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests nested item counting".into()),
        },
        Question {
            id: "orders_q03".into(),
            dataset: "orders_nested".into(),
            question_type: QuestionType::Filtering,
            prompt: "How many orders have status 'delivered'?".into(),
            ground_truth: "14".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
    ]
}

/// Questions for 100-metric analytics dataset
fn generate_large_analytics_questions() -> Vec<Question> {
    vec![
        Question {
            id: "analytics_q01".into(),
            dataset: "analytics_100".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many metrics are recorded?".into(),
            ground_truth: "100".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "analytics_q02".into(),
            dataset: "analytics_100".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is the value of metric 'm10'?".into(),
            ground_truth: "55.51".into(),
            answer_type: AnswerType::String,
            notes: Some("Tests specific metric lookup".into()),
        },
        Question {
            id: "analytics_q03".into(),
            dataset: "analytics_100".into(),
            question_type: QuestionType::Filtering,
            prompt: "How many metrics are from the 'us-east' region?".into(),
            ground_truth: "31".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
    ]
}

/// Generate questions for organization dataset (deep nesting, references)
fn generate_organization_questions() -> Vec<Question> {
    vec![
        // Field retrieval
        Question {
            id: "org_q01".into(),
            dataset: "organization".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is Bob Kumar's salary?".into(),
            ground_truth: "165000".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "org_q02".into(),
            dataset: "organization".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is the budget for the Engineering department?".into(),
            ground_truth: "2500000".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "org_q03".into(),
            dataset: "organization".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is Bob Kumar's role?".into(),
            ground_truth: "Staff Engineer".into(),
            answer_type: AnswerType::String,
            notes: None,
        },
        // Aggregation
        Question {
            id: "org_q04".into(),
            dataset: "organization".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many employees are in the Platform Team?".into(),
            ground_truth: "5".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests nested counting".into()),
        },
        Question {
            id: "org_q05".into(),
            dataset: "organization".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many teams are in the Engineering department?".into(),
            ground_truth: "3".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests deep nesting".into()),
        },
        Question {
            id: "org_q06".into(),
            dataset: "organization".into(),
            question_type: QuestionType::Aggregation,
            prompt: "What is the total salary of all employees in the ML Team?".into(),
            ground_truth: "600000".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        // Filtering
        Question {
            id: "org_q07".into(),
            dataset: "organization".into(),
            question_type: QuestionType::Filtering,
            prompt: "How many employees work remotely?".into(),
            ground_truth: "12".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "org_q08".into(),
            dataset: "organization".into(),
            question_type: QuestionType::Filtering,
            prompt: "List the names of all employees with salary above 150000.".into(),
            ground_truth: "Alice Chen, Bob Kumar, Iris Johnson, Frank Zhang, Maria Rodriguez"
                .into(),
            answer_type: AnswerType::CsvListUnordered,
            notes: None,
        },
        // Structure awareness
        Question {
            id: "org_q09".into(),
            dataset: "organization".into(),
            question_type: QuestionType::StructureAwareness,
            prompt: "What is the parent department of the Frontend Team?".into(),
            ground_truth: "Engineering".into(),
            answer_type: AnswerType::String,
            notes: Some("Tests NEST understanding".into()),
        },
        Question {
            id: "org_q10".into(),
            dataset: "organization".into(),
            question_type: QuestionType::StructureAwareness,
            prompt: "Who is the head of the department that contains the DevOps team?".into(),
            ground_truth: "Rachel Green".into(),
            answer_type: AnswerType::String,
            notes: Some("Tests cross-reference resolution".into()),
        },
    ]
}

/// Generate questions for e-commerce dataset (tensors, reviews, categories)
fn generate_ecommerce_questions() -> Vec<Question> {
    vec![
        Question {
            id: "ecom_q01".into(),
            dataset: "ecommerce".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is the average rating for the ProBook X1 Laptop 15in?".into(),
            ground_truth: "4.57".into(),
            answer_type: AnswerType::Number { decimals: 2 },
            notes: Some("Tests tensor average calculation".into()),
        },
        Question {
            id: "ecom_q02".into(),
            dataset: "ecommerce".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "How many reviews does the Noise-Canceling Headphones have?".into(),
            ground_truth: "3".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests nested review counting".into()),
        },
        Question {
            id: "ecom_q03".into(),
            dataset: "ecommerce".into(),
            question_type: QuestionType::Aggregation,
            prompt: "What is the total helpful votes across all reviews?".into(),
            ground_truth: "530".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "ecom_q04".into(),
            dataset: "ecommerce".into(),
            question_type: QuestionType::Filtering,
            prompt: "Which products are low on stock or out of stock?".into(),
            ground_truth: "USB-C Hub 7in1, Noise-Canceling Headphones, 4K Monitor 32in".into(),
            answer_type: AnswerType::CsvListUnordered,
            notes: Some("Tests status filtering".into()),
        },
        Question {
            id: "ecom_q05".into(),
            dataset: "ecommerce".into(),
            question_type: QuestionType::Aggregation,
            prompt: "What is the profit margin percentage for the Standing Desk Pro?".into(),
            ground_truth: "46.67".into(),
            answer_type: AnswerType::Number { decimals: 2 },
            notes: Some("(599.99-320)/599.99*100".into()),
        },
        Question {
            id: "ecom_q06".into(),
            dataset: "ecommerce".into(),
            question_type: QuestionType::StructureAwareness,
            prompt: "What is the parent category of Laptops?".into(),
            ground_truth: "Computers".into(),
            answer_type: AnswerType::String,
            notes: Some("Tests category hierarchy".into()),
        },
        Question {
            id: "ecom_q07".into(),
            dataset: "ecommerce".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is the 5th rating value (last in array) for the ProBook X1 Laptop 15in?"
                .into(),
            ground_truth: "4".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests tensor indexing".into()),
        },
    ]
}

/// Generate questions for analytics dataset (time-series, tensors)
fn generate_analytics_questions() -> Vec<Question> {
    vec![
        Question {
            id: "analytics_q01".into(),
            dataset: "analytics".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is the page load time for us-west homepage at 01:00?".into(),
            ground_truth: "1180".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "analytics_q02".into(),
            dataset: "analytics".into(),
            question_type: QuestionType::Aggregation,
            prompt: "What is the total request count from the api source at 02:00?".into(),
            ground_truth: "37200".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests source field counting".into()),
        },
        Question {
            id: "analytics_q03".into(),
            dataset: "analytics".into(),
            question_type: QuestionType::Filtering,
            prompt: "Which metrics have error or crash rates?".into(),
            ground_truth: "error_rate, crash_rate".into(),
            answer_type: AnswerType::CsvListUnordered,
            notes: None,
        },
        Question {
            id: "analytics_q04".into(),
            dataset: "analytics".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt:
                "What is the p50 (median, 3rd percentile value) for web page_load_time at 00:00?"
                    .into(),
            ground_truth: "1400".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests tensor extraction".into()),
        },
        Question {
            id: "analytics_q05".into(),
            dataset: "analytics".into(),
            question_type: QuestionType::Aggregation,
            prompt: "What is the total throughput for orders across all regions at 04:00?".into(),
            ground_truth: "7440".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests filtering by source".into()),
        },
    ]
}

/// Generate questions for knowledge graph dataset (complex references)
fn generate_knowledge_questions() -> Vec<Question> {
    vec![
        Question {
            id: "know_q01".into(),
            dataset: "knowledge".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "In what year was Claude Shannon born?".into(),
            ground_truth: "1916".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "know_q02".into(),
            dataset: "knowledge".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "How many citations does 'A Mathematical Theory of Communication' have?".into(),
            ground_truth: "143000".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "know_q03".into(),
            dataset: "knowledge".into(),
            question_type: QuestionType::StructureAwareness,
            prompt: "Who authored 'On Computable Numbers'?".into(),
            ground_truth: "Alan Turing".into(),
            answer_type: AnswerType::String,
            notes: Some("Tests reference resolution".into()),
        },
        Question {
            id: "know_q04".into(),
            dataset: "knowledge".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many different organizations did John McCarthy work at?".into(),
            ground_truth: "2".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests multi-affiliation counting".into()),
        },
        Question {
            id: "know_q05".into(),
            dataset: "knowledge".into(),
            question_type: QuestionType::Filtering,
            prompt: "Which people have affiliations that are still active (no end year)?".into(),
            ground_truth: "Donald Knuth, Alan Kay".into(),
            answer_type: AnswerType::CsvListUnordered,
            notes: Some("Tests null filtering on references".into()),
        },
        Question {
            id: "know_q06".into(),
            dataset: "knowledge".into(),
            question_type: QuestionType::StructureAwareness,
            prompt: "What organization is referenced as the affiliation for Alan Turing?".into(),
            ground_truth: "Bletchley Park".into(),
            answer_type: AnswerType::String,
            notes: Some("Tests cross-type reference resolution".into()),
        },
        Question {
            id: "know_q07".into(),
            dataset: "knowledge".into(),
            question_type: QuestionType::Aggregation,
            prompt: "What is the total number of citations across all publications?".into(),
            ground_truth: "196600".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
    ]
}

/// Easy questions for 5-user dataset (simple field retrieval)
fn generate_easy_user_questions() -> Vec<Question> {
    vec![
        Question {
            id: "easy_users_q01".into(),
            dataset: "easy_users_5".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is the name of user1?".into(),
            ground_truth: "Tomas Tillman".into(),
            answer_type: AnswerType::String,
            notes: Some("Simple first-record lookup".into()),
        },
        Question {
            id: "easy_users_q02".into(),
            dataset: "easy_users_5".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many users are in the dataset?".into(),
            ground_truth: "5".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Simple count".into()),
        },
        Question {
            id: "easy_users_q03".into(),
            dataset: "easy_users_5".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is the email of user3?".into(),
            ground_truth: "merritt.farrell@example.com".into(),
            answer_type: AnswerType::String,
            notes: None,
        },
    ]
}

/// Easy questions for 8-product dataset
fn generate_easy_product_questions() -> Vec<Question> {
    vec![
        Question {
            id: "easy_prod_q01".into(),
            dataset: "easy_products_8".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is the price of product prod1?".into(),
            ground_truth: "634.12".into(),
            answer_type: AnswerType::Number { decimals: 2 },
            notes: Some("Simple price lookup".into()),
        },
        Question {
            id: "easy_prod_q02".into(),
            dataset: "easy_products_8".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many products are in the catalog?".into(),
            ground_truth: "8".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
    ]
}

/// Hard questions for 300-event dataset
fn generate_hard_event_questions() -> Vec<Question> {
    vec![
        Question {
            id: "hard_events_q01".into(),
            dataset: "hard_events_300".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many ERROR level events are there?".into(),
            ground_truth: "65".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests counting in large dataset".into()),
        },
        Question {
            id: "hard_events_q02".into(),
            dataset: "hard_events_300".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many total events are logged?".into(),
            ground_truth: "300".into(),
            answer_type: AnswerType::Integer,
            notes: None,
        },
        Question {
            id: "hard_events_q03".into(),
            dataset: "hard_events_300".into(),
            question_type: QuestionType::Filtering,
            prompt: "How many events are from the 'auth' service?".into(),
            ground_truth: "50".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests service filtering in large dataset".into()),
        },
        Question {
            id: "hard_events_q04".into(),
            dataset: "hard_events_300".into(),
            question_type: QuestionType::Filtering,
            prompt: "How many WARN events are from the 'api' service?".into(),
            ground_truth: "16".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests multi-condition filtering".into()),
        },
    ]
}

// Ditto (^) reduces LLM comprehension accuracy. Use %ALIAS for repeated values instead.

/// Questions for reference-heavy dataset (tests @Type:id with nested structure)
fn generate_reference_questions() -> Vec<Question> {
    vec![
        Question {
            id: "refs_q01".into(),
            dataset: "reference_heavy_10".into(),
            question_type: QuestionType::StructureAwareness,
            prompt: "How many milestones does proj001 have?".into(),
            ground_truth: "3".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests nested milestone counting".into()),
        },
        Question {
            id: "refs_q02".into(),
            dataset: "reference_heavy_10".into(),
            question_type: QuestionType::StructureAwareness,
            prompt: "What project does proj006 depend on? (Look at the depends_on reference)"
                .into(),
            ground_truth: "proj001".into(),
            answer_type: AnswerType::String,
            notes: Some("Tests cross-reference resolution".into()),
        },
        Question {
            id: "refs_q03".into(),
            dataset: "reference_heavy_10".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many total tasks are in the dataset?".into(),
            ground_truth: "87".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests deep nested counting".into()),
        },
        Question {
            id: "refs_q04".into(),
            dataset: "reference_heavy_10".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is the name of person p001?".into(),
            ground_truth: "Raphaelle Jenkins".into(),
            answer_type: AnswerType::String,
            notes: Some("Simple lookup in complex structure".into()),
        },
        Question {
            id: "refs_q05".into(),
            dataset: "reference_heavy_10".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many projects have dependencies (depends_on is not ~)?".into(),
            ground_truth: "3".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests cross-reference counting".into()),
        },
    ]
}

/// Questions for deep hierarchy dataset (tests 5-level %NEST)
fn generate_deep_hierarchy_questions() -> Vec<Question> {
    vec![
        Question {
            id: "deep_q01".into(),
            dataset: "deep_hierarchy_3div".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many divisions does MegaCorp International have?".into(),
            ground_truth: "3".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests top-level counting".into()),
        },
        Question {
            id: "deep_q02".into(),
            dataset: "deep_hierarchy_3div".into(),
            question_type: QuestionType::StructureAwareness,
            prompt: "Which division contains Dept 22?".into(),
            ground_truth: "Division B".into(),
            answer_type: AnswerType::String,
            notes: Some("Tests parent lookup in hierarchy".into()),
        },
        Question {
            id: "deep_q03".into(),
            dataset: "deep_hierarchy_3div".into(),
            question_type: QuestionType::Aggregation,
            prompt: "How many total employees are in the company?".into(),
            ground_truth: "36".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests deep nested counting".into()),
        },
        Question {
            id: "deep_q04".into(),
            dataset: "deep_hierarchy_3div".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is the budget for Division A?".into(),
            ground_truth: "8000000".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Field lookup in nested structure".into()),
        },
        Question {
            id: "deep_q05".into(),
            dataset: "deep_hierarchy_3div".into(),
            question_type: QuestionType::StructureAwareness,
            prompt: "How many teams are in Division B?".into(),
            ground_truth: "3".into(),
            answer_type: AnswerType::Integer,
            notes: Some("Tests intermediate level counting".into()),
        },
        Question {
            id: "deep_q06".into(),
            dataset: "deep_hierarchy_3div".into(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "Who is the head of Division C?".into(),
            ground_truth: "Marcus Monahan".into(),
            answer_type: AnswerType::String,
            notes: Some("Tests field lookup in hierarchy".into()),
        },
    ]
}

/// Simulate an accuracy test (for testing the harness without API calls)
pub fn simulate_test(dataset: &TestDataset, format: DataFormat, question: &Question) -> TestResult {
    let prompt = build_prompt(dataset.data(format), format, question);

    // In dry run, just return expected as actual (100% accuracy)
    TestResult {
        question: question.prompt.clone(),
        question_type: question.question_type,
        expected: question.ground_truth.clone(),
        actual: question.ground_truth.clone(), // Simulated correct answer
        correct: true,
        format,
        difficulty: dataset.difficulty,
        model: "dry-run".to_string(),
        latency_ms: 0,
        tokens_in: prompt.len() / 4, // Rough estimate
        tokens_out: question.ground_truth.len() / 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_datasets() {
        let datasets = generate_test_datasets();
        assert!(!datasets.is_empty());

        for ds in &datasets {
            assert!(!ds.hedl.is_empty());
            assert!(!ds.json.is_empty());
            assert!(!ds.yaml.is_empty());
            assert!(!ds.toon.is_empty());
            assert!(!ds.questions.is_empty());
        }
    }

    #[test]
    fn test_build_prompt() {
        let question = Question {
            id: "q1".to_string(),
            dataset: "test".to_string(),
            question_type: QuestionType::FieldRetrieval,
            prompt: "What is Alice's salary?".to_string(),
            ground_truth: "125000".to_string(),
            answer_type: AnswerType::Integer,
            notes: None,
        };

        let prompt = build_prompt("test data", DataFormat::Hedl, &question);
        assert!(prompt.contains("HEDL"));
        assert!(prompt.contains("What is Alice's salary?"));
        assert!(prompt.contains("test data"));
    }

    #[test]
    fn test_accuracy_report() {
        let report = AccuracyReport {
            model: "test-model".to_string(),
            total_questions: 10,
            results_by_format: vec![
                FormatResults {
                    format: "HEDL".to_string(),
                    correct: 8,
                    total: 10,
                    avg_latency_ms: 150.0,
                    total_tokens: 1000,
                },
                FormatResults {
                    format: "JSON".to_string(),
                    correct: 7,
                    total: 10,
                    avg_latency_ms: 180.0,
                    total_tokens: 1500,
                },
            ],
            results_by_type: vec![
                TypeResults {
                    question_type: "FieldRetrieval".to_string(),
                    correct: 5,
                    total: 5,
                },
                TypeResults {
                    question_type: "Aggregation".to_string(),
                    correct: 3,
                    total: 5,
                },
            ],
            results_by_difficulty: vec![],
        };

        let output = report.report();
        assert!(output.contains("test-model"));
        assert!(output.contains("80.0%")); // HEDL accuracy
        assert!(output.contains("70.0%")); // JSON accuracy
    }
}
