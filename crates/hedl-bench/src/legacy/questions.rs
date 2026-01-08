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

//! Question generators for LLM accuracy benchmarks.
//!
//! Generates deterministic questions for testing LLM comprehension of HEDL data.
//! Supports 5 question types:
//! - Field Retrieval: Direct field lookup
//! - Aggregation: Counts, sums, averages
//! - Filtering: Multi-condition queries
//! - Structure Awareness: Format-native structural features
//! - Structural Validation: Data integrity detection

use std::collections::HashMap;

/// Question types for LLM accuracy testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestionType {
    /// Direct field lookup from records
    FieldRetrieval,
    /// Counts, sums, averages, single-condition filters
    Aggregation,
    /// Multi-condition queries with AND/OR logic
    Filtering,
    /// Format-native structural affordances
    StructureAwareness,
    /// Data integrity detection
    StructuralValidation,
}

/// Answer types for type-aware comparison
#[derive(Debug, Clone, PartialEq)]
pub enum AnswerType {
    /// Exact string match (with optional case-insensitivity)
    String,
    /// Integer with tolerance for parsing variations
    Integer,
    /// Float with configurable decimal precision
    Number { decimals: usize },
    /// Boolean (yes/no/true/false/y/n/1/0)
    Boolean,
    /// Date in ISO format (YYYY-MM-DD)
    Date,
    /// Comma-separated list, order matters
    CsvListOrdered,
    /// Comma-separated list, order doesn't matter
    CsvListUnordered,
}

/// A benchmark question
#[derive(Debug, Clone)]
pub struct Question {
    /// Unique question ID
    pub id: String,
    /// The question text to ask the LLM
    pub prompt: String,
    /// Expected ground truth answer
    pub ground_truth: String,
    /// Type of question for categorization
    pub question_type: QuestionType,
    /// Dataset this question applies to
    pub dataset: String,
    /// Answer type for comparison
    pub answer_type: AnswerType,
    /// Optional notes about the question
    pub notes: Option<String>,
}

impl Question {
    /// Creates a new QuestionBuilder (builder pattern)
    pub fn builder(id: impl Into<String>, prompt: impl Into<String>) -> QuestionBuilder {
        QuestionBuilder {
            id: id.into(),
            prompt: prompt.into(),
            ground_truth: None,
            question_type: QuestionType::FieldRetrieval,
            dataset: "unknown".to_string(),
            answer_type: AnswerType::String,
            notes: None,
        }
    }
}

/// Builder for creating questions
pub struct QuestionBuilder {
    id: String,
    prompt: String,
    ground_truth: Option<String>,
    question_type: QuestionType,
    dataset: String,
    answer_type: AnswerType,
    notes: Option<String>,
}

impl QuestionBuilder {
    pub fn ground_truth(mut self, value: impl Into<String>) -> Self {
        self.ground_truth = Some(value.into());
        self
    }

    pub fn question_type(mut self, qt: QuestionType) -> Self {
        self.question_type = qt;
        self
    }

    pub fn dataset(mut self, ds: impl Into<String>) -> Self {
        self.dataset = ds.into();
        self
    }

    pub fn answer_type(mut self, at: AnswerType) -> Self {
        self.answer_type = at;
        self
    }

    pub fn notes(mut self, n: impl Into<String>) -> Self {
        self.notes = Some(n.into());
        self
    }

    pub fn build(self) -> Question {
        Question {
            id: self.id,
            prompt: self.prompt,
            ground_truth: self.ground_truth.expect("ground_truth is required"),
            question_type: self.question_type,
            dataset: self.dataset,
            answer_type: self.answer_type,
            notes: self.notes,
        }
    }
}

/// Generate questions for the users dataset
pub fn generate_user_questions() -> Vec<Question> {
    vec![
        // Field Retrieval (12 questions)
        Question::builder("users_fr_1", "What is the email of user1?")
            .ground_truth("john.smith@example.com")
            .question_type(QuestionType::FieldRetrieval)
            .dataset("users")
            .answer_type(AnswerType::String)
            .build(),
        Question::builder("users_fr_2", "What is the role of user5?")
            .ground_truth("analyst")
            .question_type(QuestionType::FieldRetrieval)
            .dataset("users")
            .answer_type(AnswerType::String)
            .build(),
        Question::builder("users_fr_3", "What is the name of user10?")
            .ground_truth("Jane Doe")
            .question_type(QuestionType::FieldRetrieval)
            .dataset("users")
            .answer_type(AnswerType::String)
            .build(),
        // Aggregation (8 questions)
        Question::builder("users_agg_1", "How many users are there in total?")
            .ground_truth("100")
            .question_type(QuestionType::Aggregation)
            .dataset("users")
            .answer_type(AnswerType::Integer)
            .build(),
        Question::builder("users_agg_2", "How many users have the role 'admin'?")
            .ground_truth("20")
            .question_type(QuestionType::Aggregation)
            .dataset("users")
            .answer_type(AnswerType::Integer)
            .build(),
        Question::builder("users_agg_3", "How many users have the role 'developer'?")
            .ground_truth("25")
            .question_type(QuestionType::Aggregation)
            .dataset("users")
            .answer_type(AnswerType::Integer)
            .build(),
        // Filtering (6 questions)
        Question::builder("users_filt_1", "How many admins were created in 2024?")
            .ground_truth("15")
            .question_type(QuestionType::Filtering)
            .dataset("users")
            .answer_type(AnswerType::Integer)
            .build(),
        Question::builder(
            "users_filt_2",
            "How many developers with @example.com emails exist?",
        )
        .ground_truth("25")
        .question_type(QuestionType::Filtering)
        .dataset("users")
        .answer_type(AnswerType::Integer)
        .build(),
        // Structure Awareness (4 questions)
        Question::builder(
            "users_struct_1",
            "What are the column names in the users list?",
        )
        .ground_truth("id,name,email,role,created_at")
        .question_type(QuestionType::StructureAwareness)
        .dataset("users")
        .answer_type(AnswerType::CsvListOrdered)
        .build(),
        Question::builder("users_struct_2", "How many columns are in the User schema?")
            .ground_truth("5")
            .question_type(QuestionType::StructureAwareness)
            .dataset("users")
            .answer_type(AnswerType::Integer)
            .build(),
        Question::builder(
            "users_struct_3",
            "What is the third column in the User schema?",
        )
        .ground_truth("email")
        .question_type(QuestionType::StructureAwareness)
        .dataset("users")
        .answer_type(AnswerType::String)
        .build(),
        Question::builder("users_struct_4", "What is the last user's ID?")
            .ground_truth("user100")
            .question_type(QuestionType::StructureAwareness)
            .dataset("users")
            .answer_type(AnswerType::String)
            .build(),
    ]
}

/// Generate questions for the products dataset
pub fn generate_product_questions() -> Vec<Question> {
    vec![
        // Field Retrieval
        Question::builder("prod_fr_1", "What is the price of prod1?")
            .ground_truth("29.99")
            .question_type(QuestionType::FieldRetrieval)
            .dataset("products")
            .answer_type(AnswerType::Number { decimals: 2 })
            .build(),
        Question::builder("prod_fr_2", "What category is prod5 in?")
            .ground_truth("electronics")
            .question_type(QuestionType::FieldRetrieval)
            .dataset("products")
            .answer_type(AnswerType::String)
            .build(),
        Question::builder("prod_fr_3", "How many units of prod10 are in stock?")
            .ground_truth("150")
            .question_type(QuestionType::FieldRetrieval)
            .dataset("products")
            .answer_type(AnswerType::Integer)
            .build(),
        // Aggregation
        Question::builder("prod_agg_1", "How many products are there?")
            .ground_truth("100")
            .question_type(QuestionType::Aggregation)
            .dataset("products")
            .answer_type(AnswerType::Integer)
            .build(),
        Question::builder(
            "prod_agg_2",
            "How many products are in the 'electronics' category?",
        )
        .ground_truth("17")
        .question_type(QuestionType::Aggregation)
        .dataset("products")
        .answer_type(AnswerType::Integer)
        .build(),
        Question::builder(
            "prod_agg_3",
            "How many products have stock greater than 500?",
        )
        .ground_truth("20")
        .question_type(QuestionType::Aggregation)
        .dataset("products")
        .answer_type(AnswerType::Integer)
        .build(),
        // Filtering
        Question::builder(
            "prod_filt_1",
            "How many electronics products cost more than $50?",
        )
        .ground_truth("8")
        .question_type(QuestionType::Filtering)
        .dataset("products")
        .answer_type(AnswerType::Integer)
        .build(),
    ]
}

/// Generate questions for the events dataset
pub fn generate_event_questions() -> Vec<Question> {
    vec![
        // Field Retrieval
        Question::builder("evt_fr_1", "What is the level of event evt1?")
            .ground_truth("INFO")
            .question_type(QuestionType::FieldRetrieval)
            .dataset("events")
            .answer_type(AnswerType::String)
            .build(),
        Question::builder("evt_fr_2", "What service generated evt5?")
            .ground_truth("auth")
            .question_type(QuestionType::FieldRetrieval)
            .dataset("events")
            .answer_type(AnswerType::String)
            .build(),
        // Aggregation
        Question::builder("evt_agg_1", "How many events are there?")
            .ground_truth("100")
            .question_type(QuestionType::Aggregation)
            .dataset("events")
            .answer_type(AnswerType::Integer)
            .build(),
        Question::builder("evt_agg_2", "How many ERROR level events exist?")
            .ground_truth("10")
            .question_type(QuestionType::Aggregation)
            .dataset("events")
            .answer_type(AnswerType::Integer)
            .build(),
        Question::builder("evt_agg_3", "How many events came from the 'api' service?")
            .ground_truth("20")
            .question_type(QuestionType::Aggregation)
            .dataset("events")
            .answer_type(AnswerType::Integer)
            .build(),
        // Filtering
        Question::builder(
            "evt_filt_1",
            "How many ERROR events came from the 'auth' service?",
        )
        .ground_truth("3")
        .question_type(QuestionType::Filtering)
        .dataset("events")
        .answer_type(AnswerType::Integer)
        .build(),
    ]
}

/// Generate questions for the blog dataset (with NEST hierarchy)
pub fn generate_blog_questions() -> Vec<Question> {
    vec![
        // Field Retrieval
        Question::builder("blog_fr_1", "What is the title of post1?")
            .ground_truth("Getting Started with HEDL")
            .question_type(QuestionType::FieldRetrieval)
            .dataset("blog")
            .answer_type(AnswerType::String)
            .build(),
        Question::builder("blog_fr_2", "Who wrote post3?")
            .ground_truth("@Author:author2")
            .question_type(QuestionType::FieldRetrieval)
            .dataset("blog")
            .answer_type(AnswerType::String)
            .build(),
        // Aggregation
        Question::builder("blog_agg_1", "How many posts are there?")
            .ground_truth("20")
            .question_type(QuestionType::Aggregation)
            .dataset("blog")
            .answer_type(AnswerType::Integer)
            .build(),
        Question::builder("blog_agg_2", "How many comments are there in total?")
            .ground_truth("100")
            .question_type(QuestionType::Aggregation)
            .dataset("blog")
            .answer_type(AnswerType::Integer)
            .build(),
        Question::builder("blog_agg_3", "How many comments does post1 have?")
            .ground_truth("5")
            .question_type(QuestionType::Aggregation)
            .dataset("blog")
            .answer_type(AnswerType::Integer)
            .build(),
        // Structure Awareness (NEST-specific)
        Question::builder("blog_struct_1", "What type is nested under Post?")
            .ground_truth("Comment")
            .question_type(QuestionType::StructureAwareness)
            .dataset("blog")
            .answer_type(AnswerType::String)
            .notes("Tests understanding of %NEST directive")
            .build(),
        Question::builder(
            "blog_struct_2",
            "What are the schemas defined in the header?",
        )
        .ground_truth("Author,Post,Comment")
        .question_type(QuestionType::StructureAwareness)
        .dataset("blog")
        .answer_type(AnswerType::CsvListUnordered)
        .build(),
    ]
}

/// Generate structural validation questions
pub fn generate_validation_questions() -> Vec<Question> {
    vec![
        Question::builder(
            "valid_1",
            "Is this data complete and valid? (Answer YES or NO)",
        )
        .ground_truth("YES")
        .question_type(QuestionType::StructuralValidation)
        .dataset("validation_control")
        .answer_type(AnswerType::Boolean)
        .notes("Control: valid data")
        .build(),
        Question::builder(
            "valid_2",
            "Is this data complete and valid? (Answer YES or NO)",
        )
        .ground_truth("NO")
        .question_type(QuestionType::StructuralValidation)
        .dataset("validation_truncated")
        .answer_type(AnswerType::Boolean)
        .notes("Truncated: missing rows")
        .build(),
        Question::builder(
            "valid_3",
            "Is this data complete and valid? (Answer YES or NO)",
        )
        .ground_truth("NO")
        .question_type(QuestionType::StructuralValidation)
        .dataset("validation_extra")
        .answer_type(AnswerType::Boolean)
        .notes("Extra rows beyond declared")
        .build(),
        Question::builder(
            "valid_4",
            "Is this data complete and valid? (Answer YES or NO)",
        )
        .ground_truth("NO")
        .question_type(QuestionType::StructuralValidation)
        .dataset("validation_width")
        .answer_type(AnswerType::Boolean)
        .notes("Width mismatch: inconsistent columns")
        .build(),
        Question::builder(
            "valid_5",
            "Is this data complete and valid? (Answer YES or NO)",
        )
        .ground_truth("NO")
        .question_type(QuestionType::StructuralValidation)
        .dataset("validation_missing")
        .answer_type(AnswerType::Boolean)
        .notes("Missing required fields")
        .build(),
    ]
}

/// Get all questions grouped by dataset
pub fn all_questions() -> HashMap<String, Vec<Question>> {
    let mut map = HashMap::new();
    map.insert("users".to_string(), generate_user_questions());
    map.insert("products".to_string(), generate_product_questions());
    map.insert("events".to_string(), generate_event_questions());
    map.insert("blog".to_string(), generate_blog_questions());
    map.insert("validation".to_string(), generate_validation_questions());
    map
}

/// Get all questions as a flat list
pub fn all_questions_flat() -> Vec<Question> {
    let mut questions = Vec::new();
    questions.extend(generate_user_questions());
    questions.extend(generate_product_questions());
    questions.extend(generate_event_questions());
    questions.extend(generate_blog_questions());
    questions.extend(generate_validation_questions());
    questions
}

/// Count questions by type
pub fn question_counts() -> HashMap<QuestionType, usize> {
    let mut counts = HashMap::new();
    for q in all_questions_flat() {
        *counts.entry(q.question_type).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_builder() {
        let q = Question::builder("test1", "What is X?")
            .ground_truth("42")
            .question_type(QuestionType::Aggregation)
            .dataset("test")
            .answer_type(AnswerType::Integer)
            .build();

        assert_eq!(q.id, "test1");
        assert_eq!(q.ground_truth, "42");
        assert_eq!(q.question_type, QuestionType::Aggregation);
    }

    #[test]
    fn test_all_questions() {
        let questions = all_questions_flat();
        assert!(!questions.is_empty());

        // Verify all questions have ground truth
        for q in &questions {
            assert!(
                !q.ground_truth.is_empty(),
                "Question {} has no ground truth",
                q.id
            );
        }
    }

    #[test]
    fn test_question_counts() {
        let counts = question_counts();
        assert!(counts.contains_key(&QuestionType::FieldRetrieval));
        assert!(counts.contains_key(&QuestionType::Aggregation));
    }
}
