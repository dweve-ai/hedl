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

//! Prompt generation for LLM accuracy evaluation.
//!
//! Builds prompts that combine data in various formats with questions,
//! format-specific hints, and answer instructions for LLM evaluation.

use crate::accuracy::{
    questions::{AnswerType, Question},
    reports::DataFormat,
};

/// Build a prompt for LLM testing.
///
/// Combines the provided data, format-specific guidance, and question into
/// a complete prompt string suitable for sending to an LLM for evaluation.
///
/// # Arguments
///
/// * `data` - The data content in the specified format
/// * `format` - The format of the data (HEDL, JSON, YAML, etc.)
/// * `question` - The question to ask about the data
///
/// # Returns
///
/// A formatted prompt string ready for LLM evaluation
///
/// # Format Hints
///
/// Each format receives ~10 lines of format-specific guidance to ensure
/// fair comparison. These hints help the LLM understand format-specific
/// features like HEDL's `%STRUCT` and caret repetition syntax.
///
/// # Answer Instructions
///
/// Instructions are customized based on the question's answer type to
/// guide the LLM to produce responses in the expected format.
///
/// # Examples
///
/// ```
/// use hedl_bench::accuracy::{
///     prompts::build_prompt,
///     reports::DataFormat,
///     questions::{Question, AnswerType, QuestionType},
/// };
///
/// let question = Question::builder("test_001", "What is the product name?")
///     .ground_truth("Widget Pro")
///     .answer_type(AnswerType::String)
///     .question_type(QuestionType::FieldRetrieval)
///     .build();
///
/// let data = "products:\n- name: Widget Pro\n  price: 29.99";
/// let prompt = build_prompt(data, DataFormat::Yaml, &question);
///
/// assert!(prompt.contains("YAML FORMAT GUIDE"));
/// assert!(prompt.contains("What is the product name?"));
/// ```
#[must_use]
pub fn build_prompt(data: &str, format: DataFormat, question: &Question) -> String {
    let format_desc = match format {
        DataFormat::Hedl => "HEDL (Human-Efficient Data Language)",
        DataFormat::Json => "JSON",
        DataFormat::Yaml => "YAML",
        DataFormat::Xml => "XML",
        DataFormat::Toon => "TOON (Token-Oriented Object Notation)",
        DataFormat::Csv => "CSV (Comma-Separated Values)",
        DataFormat::Toml => "TOML",
        DataFormat::Markdown => "Markdown",
    };

    let answer_instruction = match &question.answer_type {
        AnswerType::String | AnswerType::StringCaseSensitive => {
            "Give the exact value only.".to_string()
        }
        AnswerType::Integer => "Give just the number.".to_string(),
        AnswerType::Number { decimals: _ } => "Give just the number.".to_string(),
        AnswerType::Boolean => "Answer: true or false".to_string(),
        AnswerType::Date => "Give the date as YYYY-MM-DD.".to_string(),
        AnswerType::DateTime => "Give the datetime as YYYY-MM-DDTHH:MM:SSZ.".to_string(),
        AnswerType::ListOrdered => "Give a comma-separated list.".to_string(),
        AnswerType::ListUnordered => "Give a comma-separated list.".to_string(),
        AnswerType::JsonObject => "Give a JSON object.".to_string(),
        AnswerType::NumericRange { min, max } => {
            format!("Give a number between {} and {}.", min, max)
        }
        AnswerType::MultipleValid(options) => {
            format!("Answer: {}", options.join(" or "))
        }
    };

    // Format hints - ~10 lines each for fairness. Not counted in token metrics.
    let format_hint = match format {
        DataFormat::Hedl => {
            r#"
HEDL FORMAT GUIDE:
Header (before ---):
- %S:Type:[col1,col2,...] defines struct with columns
- %C:Type.total=N gives TOTAL COUNT - USE THIS, don't count!
- %C:Type.field:val1=N,val2=M gives counts per value - USE THESE!
- %N:Parent>Child declares nesting relationship
- %NULL:~ and %QUOTE:" define null/quote chars

Data (after ---):
- name:@Type starts a collection of that type
- |val1,val2,... is a record (columns match %S order)
- Tensors [1,2,3] and lists (a,b,c) are atomic
- "quoted" for values with , or |
- Escapes: \" = quote, \n = newline, \\ = backslash
- ~ is null, "~" is literal tilde
- @Type#N:|a|b|c = N inline children (pipe-separated)
- @Type#N: on own line = block children below
- Indentation = nesting (1 space = 1 level)

COUNTING: Always use %C values. NEVER count records manually!"#
        }
        DataFormat::Toon => {
            r"
TOON FORMAT GUIDE:
- List header: key: [count]{col1,col2,...} declares N items with columns
- Each following line is a data row: val1,val2,...
- First value in each row is the ID
- Indentation (2 spaces) shows nested children
- Scalars: key: value for simple key-value pairs
- Objects: key: {} followed by indented children
- Strings with special chars are quoted
- Numbers, booleans, null are unquoted
- References appear as @Type:id strings"
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
            r"
YAML FORMAT GUIDE:
- Key-value: key: value (space after colon required)
- Lists: - item (dash prefix) or [item1, item2]
- Nested structures use indentation (2 spaces)
- Strings usually unquoted unless special chars
- Numbers and booleans parsed automatically
- null or ~ for missing values
- References appear as strings @Type:id
- Multi-line strings use | or >
- Comments start with #"
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
        DataFormat::Toml => {
            r#"
TOML FORMAT GUIDE:
- Key-value: key = "value" (strings quoted)
- Numbers and booleans unquoted
- Tables: [table_name] defines sections
- Nested tables: [parent.child]
- Arrays: key = [val1, val2, ...]
- Inline tables: key = { sub1 = val1, sub2 = val2 }
- Dates use ISO 8601: 2025-01-24T10:30:00Z
- Comments start with #"#
        }
        DataFormat::Markdown => {
            r#"
MARKDOWN TABLE GUIDE:
- Header row with column names
- Separator row with dashes (---)
- Data rows with cells separated by pipes (|)
- Optional leading/trailing pipes
- Alignment using colons in separator
- Cell content can contain inline markdown
- No nested structures - flat tabular data
- Empty cells shown as blank spaces"#
        }
    };

    format!(
        r"You are analyzing data in {} format.{}

DATA:
{}

QUESTION: {}

{} DO NOT explain your reasoning. Give ONLY the answer value.",
        format_desc, format_hint, data, question.prompt, answer_instruction
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accuracy::questions::QuestionType;

    #[test]
    fn test_build_prompt_basic() {
        let question = Question::builder("test_001", "What is the product name?")
            .ground_truth("Widget Pro")
            .answer_type(AnswerType::String)
            .question_type(QuestionType::FieldRetrieval)
            .build();

        let data = r#"{"product": {"name": "Widget Pro", "price": 29.99}}"#;
        let prompt = build_prompt(data, DataFormat::Json, &question);

        assert!(prompt.contains("JSON"));
        assert!(prompt.contains("What is the product name?"));
        assert!(prompt.contains(data));
        assert!(prompt.contains("DO NOT explain"));
    }

    #[test]
    fn test_build_prompt_with_hedl_format() {
        let question = Question::builder("test_002", "How many orders are there?")
            .ground_truth("5")
            .answer_type(AnswerType::Integer)
            .question_type(QuestionType::Aggregation)
            .build();

        let data = r#"%S:Order:[id,total]
---
orders:@Order
|ord-001,99.99
|ord-002,149.50"#;
        let prompt = build_prompt(data, DataFormat::Hedl, &question);

        assert!(prompt.contains("HEDL FORMAT GUIDE"));
        assert!(prompt.contains("%S:Type"));
        assert!(prompt.contains("%C:"));
        assert!(prompt.contains("How many orders are there?"));
        assert!(prompt.contains("Give just the number"));
    }

    #[test]
    fn test_build_prompt_with_counting_question() {
        let question = Question::builder("test_003", "How many items have status 'active'?")
            .ground_truth("3")
            .answer_type(AnswerType::Integer)
            .question_type(QuestionType::Filtering)
            .build();

        let data = "id,status\n1,active\n2,inactive\n3,active";
        let prompt = build_prompt(data, DataFormat::Csv, &question);

        assert!(prompt.contains("Give just the number"));
        assert!(prompt.contains("DO NOT explain"));
    }

    #[test]
    fn test_build_prompt_with_number_answer() {
        let question = Question::builder("test_004", "What is the average price?")
            .ground_truth("45.50")
            .answer_type(AnswerType::Number { decimals: 2 })
            .question_type(QuestionType::Aggregation)
            .build();

        let data = "prices: [29.99, 45.50, 61.01]";
        let prompt = build_prompt(data, DataFormat::Yaml, &question);

        assert!(prompt.contains("Give just the number"));
    }

    #[test]
    fn test_build_prompt_with_list_answer() {
        let question = Question::builder("test_005", "List all product names")
            .ground_truth("Widget,Gadget,Tool")
            .answer_type(AnswerType::ListUnordered)
            .question_type(QuestionType::FieldRetrieval)
            .build();

        let data = "<products><product>Widget</product><product>Gadget</product></products>";
        let prompt = build_prompt(data, DataFormat::Xml, &question);

        assert!(prompt.contains("comma-separated list"));
    }

    #[test]
    fn test_build_prompt_with_boolean_answer() {
        let question = Question::builder("test_006", "Is the service active?")
            .ground_truth("true")
            .answer_type(AnswerType::Boolean)
            .question_type(QuestionType::FieldRetrieval)
            .build();

        let data = "service_status = true";
        let prompt = build_prompt(data, DataFormat::Toml, &question);

        assert!(prompt.contains("true or false"));
    }

    #[test]
    fn test_build_prompt_with_date_answer() {
        let question = Question::builder("test_007", "What is the start date?")
            .ground_truth("2025-01-24")
            .answer_type(AnswerType::Date)
            .question_type(QuestionType::FieldRetrieval)
            .build();

        let data = "| Start Date |\n|------------|\n| 2025-01-24 |";
        let prompt = build_prompt(data, DataFormat::Markdown, &question);

        assert!(prompt.contains("YYYY-MM-DD"));
    }

    #[test]
    fn test_all_formats_have_hints() {
        let question = Question::builder("test_008", "Test")
            .ground_truth("test")
            .build();

        let formats = [
            DataFormat::Hedl,
            DataFormat::Json,
            DataFormat::Yaml,
            DataFormat::Xml,
            DataFormat::Toon,
            DataFormat::Csv,
            DataFormat::Toml,
            DataFormat::Markdown,
        ];

        for format in formats {
            let prompt = build_prompt("test data", format, &question);
            // All formats should have either "FORMAT GUIDE" or "GUIDE" in their hints
            assert!(
                prompt.contains("FORMAT GUIDE") || prompt.contains("GUIDE"),
                "Format {:?} missing guide text",
                format
            );
            assert!(prompt.len() > 100, "Prompt for {:?} is too short", format);
        }
    }

    #[test]
    fn test_numeric_range_answer_type() {
        let question = Question::builder("test_009", "What is the temperature?")
            .ground_truth("72.5")
            .answer_type(AnswerType::NumericRange {
                min: 32.0,
                max: 212.0,
            })
            .question_type(QuestionType::FieldRetrieval)
            .build();

        let prompt = build_prompt("temp: 72.5", DataFormat::Yaml, &question);

        assert!(prompt.contains("between 32 and 212"));
    }

    #[test]
    fn test_multiple_valid_answer_type() {
        let question = Question::builder("test_010", "What is the color?")
            .ground_truth("red")
            .answer_type(AnswerType::MultipleValid(vec![
                "red".to_string(),
                "green".to_string(),
                "blue".to_string(),
            ]))
            .question_type(QuestionType::FieldRetrieval)
            .build();

        let prompt = build_prompt("color: red", DataFormat::Yaml, &question);

        assert!(prompt.contains("red or green or blue"));
    }
}
