// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Edge Case and Adversarial Test Suites
//!
//! Tests LLM robustness against challenging data patterns:
//! - Unicode and special characters
//! - Deeply nested structures
//! - Sparse/null data
//! - Boundary values
//! - Ambiguous formats
//! - Adversarial inputs

use crate::accuracy::questions::{AnswerType, Question, QuestionType};

/// Categories of edge cases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeCaseCategory {
    /// Unicode characters: emojis, RTL text, combining characters
    Unicode,
    /// Special characters: quotes, escapes, control characters
    SpecialChars,
    /// Deeply nested structures (5+ levels)
    DeepNesting,
    /// Sparse data with many nulls
    SparseData,
    /// Boundary values: zero, negative, max int, empty strings
    BoundaryValues,
    /// Ambiguous formats: dates, numbers that look like IDs
    AmbiguousFormat,
    /// Large numbers: scientific notation, precision limits
    LargeNumbers,
    /// Long strings: paragraphs, multiline content
    LongStrings,
    /// Repeated/duplicate data
    Duplicates,
    /// Circular or self-referential structures
    CircularRefs,
}

impl EdgeCaseCategory {
    /// All categories for iteration
    pub const ALL: [EdgeCaseCategory; 10] = [
        EdgeCaseCategory::Unicode,
        EdgeCaseCategory::SpecialChars,
        EdgeCaseCategory::DeepNesting,
        EdgeCaseCategory::SparseData,
        EdgeCaseCategory::BoundaryValues,
        EdgeCaseCategory::AmbiguousFormat,
        EdgeCaseCategory::LargeNumbers,
        EdgeCaseCategory::LongStrings,
        EdgeCaseCategory::Duplicates,
        EdgeCaseCategory::CircularRefs,
    ];

    /// Human-readable name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            EdgeCaseCategory::Unicode => "Unicode",
            EdgeCaseCategory::SpecialChars => "Special Characters",
            EdgeCaseCategory::DeepNesting => "Deep Nesting",
            EdgeCaseCategory::SparseData => "Sparse Data",
            EdgeCaseCategory::BoundaryValues => "Boundary Values",
            EdgeCaseCategory::AmbiguousFormat => "Ambiguous Formats",
            EdgeCaseCategory::LargeNumbers => "Large Numbers",
            EdgeCaseCategory::LongStrings => "Long Strings",
            EdgeCaseCategory::Duplicates => "Duplicates",
            EdgeCaseCategory::CircularRefs => "Circular References",
        }
    }

    /// Description of what this category tests
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            EdgeCaseCategory::Unicode => {
                "Tests handling of emojis, RTL text, combining characters, and non-ASCII content"
            }
            EdgeCaseCategory::SpecialChars => {
                "Tests proper escaping of quotes, backslashes, and control characters"
            }
            EdgeCaseCategory::DeepNesting => {
                "Tests navigation through 5+ levels of nested structures"
            }
            EdgeCaseCategory::SparseData => {
                "Tests handling of datasets with many null/missing values"
            }
            EdgeCaseCategory::BoundaryValues => {
                "Tests zero, negative, maximum integers, and empty strings"
            }
            EdgeCaseCategory::AmbiguousFormat => {
                "Tests dates that look like numbers, numbers that look like IDs"
            }
            EdgeCaseCategory::LargeNumbers => {
                "Tests scientific notation, very large integers, precision limits"
            }
            EdgeCaseCategory::LongStrings => {
                "Tests multiline content, paragraphs, whitespace handling"
            }
            EdgeCaseCategory::Duplicates => {
                "Tests detection and handling of repeated/duplicate records"
            }
            EdgeCaseCategory::CircularRefs => {
                "Tests handling of self-referential or circular data structures"
            }
        }
    }

    /// Difficulty level of this category (1-5)
    #[must_use]
    pub fn difficulty(&self) -> u8 {
        match self {
            EdgeCaseCategory::Unicode => 3,
            EdgeCaseCategory::SpecialChars => 2,
            EdgeCaseCategory::DeepNesting => 4,
            EdgeCaseCategory::SparseData => 3,
            EdgeCaseCategory::BoundaryValues => 2,
            EdgeCaseCategory::AmbiguousFormat => 4,
            EdgeCaseCategory::LargeNumbers => 3,
            EdgeCaseCategory::LongStrings => 3,
            EdgeCaseCategory::Duplicates => 2,
            EdgeCaseCategory::CircularRefs => 5,
        }
    }
}

impl std::fmt::Display for EdgeCaseCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Edge case test dataset
#[derive(Debug, Clone)]
pub struct EdgeCaseDataset {
    /// Category of edge case
    pub category: EdgeCaseCategory,
    /// Dataset name
    pub name: String,
    /// HEDL source
    pub hedl: String,
    /// Questions for this dataset
    pub questions: Vec<Question>,
}

/// Generator for edge case datasets
pub struct EdgeCaseGenerator;

impl EdgeCaseGenerator {
    /// Create a new generator
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Generate all edge case datasets
    #[must_use]
    pub fn generate_all(&self) -> Vec<EdgeCaseDataset> {
        EdgeCaseCategory::ALL
            .iter()
            .map(|category| self.generate(*category))
            .collect()
    }

    /// Generate dataset for a specific category
    #[must_use]
    pub fn generate(&self, category: EdgeCaseCategory) -> EdgeCaseDataset {
        match category {
            EdgeCaseCategory::Unicode => self.generate_unicode(),
            EdgeCaseCategory::SpecialChars => self.generate_special_chars(),
            EdgeCaseCategory::DeepNesting => self.generate_deep_nesting(),
            EdgeCaseCategory::SparseData => self.generate_sparse_data(),
            EdgeCaseCategory::BoundaryValues => self.generate_boundary_values(),
            EdgeCaseCategory::AmbiguousFormat => self.generate_ambiguous_format(),
            EdgeCaseCategory::LargeNumbers => self.generate_large_numbers(),
            EdgeCaseCategory::LongStrings => self.generate_long_strings(),
            EdgeCaseCategory::Duplicates => self.generate_duplicates(),
            EdgeCaseCategory::CircularRefs => self.generate_circular_refs(),
        }
    }

    fn generate_unicode(&self) -> EdgeCaseDataset {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,bio,location,tags]
---
users:@User
 |u001,Björk Guðmundsdóttir,Singer from Iceland 🇮🇸,Reykjavík,[music, 🎵, icelandic]
 |u002,田中太郎,日本語のテスト,東京,[日本語, テスト]
 |u003,محمد أحمد,مرحبا بالعالم,الرياض,[عربي, مرحبا]
 |u004,José García,Café ☕ con acento,México,[español, ñ, café]
 |u005,Müller Straße,Größe: 6' 2",München,[deutsch, ß, ü]
 |u006,🌟 Star User 🌟,Profile with emojis 🎉🎊,🏠 Home,[⭐, 🚀, 💯]
 |u007,Сергей Иванов,Привет мир,Москва,[русский, привет]
 |u008,김철수,안녕하세요,서울,[한국어, 인사]
 |u009,Ω Alpha Ω,Greek: αβγδε,Athens,[Ω, α, β]
 |u010,Mixed מעורב 混合,RTL and LTR text,Global,[rtl, ltr, mixed]
"#;

        let questions = vec![
            Question::builder("unicode_001", "What is the name of user u001?")
                .ground_truth("Björk Guðmundsdóttir")
                .question_type(QuestionType::FieldRetrieval)
                .domain("edge_case")
                .dataset("unicode")
                .answer_type(AnswerType::String)
                .complexity(3)
                .tag("unicode")
                .tag("icelandic")
                .build(),
            Question::builder("unicode_002", "What city is user u002 located in?")
                .ground_truth("東京")
                .question_type(QuestionType::FieldRetrieval)
                .domain("edge_case")
                .dataset("unicode")
                .answer_type(AnswerType::String)
                .complexity(3)
                .tag("unicode")
                .tag("japanese")
                .build(),
            Question::builder("unicode_003", "How many users have emoji in their tags?")
                .ground_truth("2")
                .question_type(QuestionType::Filtering)
                .domain("edge_case")
                .dataset("unicode")
                .answer_type(AnswerType::Integer)
                .complexity(4)
                .tag("unicode")
                .tag("emoji")
                .build(),
            Question::builder("unicode_004", "What is the bio of user u003?")
                .ground_truth("مرحبا بالعالم")
                .question_type(QuestionType::FieldRetrieval)
                .domain("edge_case")
                .dataset("unicode")
                .answer_type(AnswerType::String)
                .complexity(3)
                .tag("unicode")
                .tag("arabic")
                .tag("rtl")
                .build(),
            Question::builder(
                "unicode_005",
                "Which user has the location with an emoji house?",
            )
            .ground_truth("🌟 Star User 🌟")
            .question_type(QuestionType::Filtering)
            .domain("edge_case")
            .dataset("unicode")
            .answer_type(AnswerType::String)
            .complexity(4)
            .tag("unicode")
            .tag("emoji")
            .build(),
        ];

        EdgeCaseDataset {
            category: EdgeCaseCategory::Unicode,
            name: "unicode_test".to_string(),
            hedl: hedl.to_string(),
            questions,
        }
    }

    fn generate_special_chars(&self) -> EdgeCaseDataset {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Config:[id,key,value,description]
---
config:@Config
 |cfg001,path,/usr/local/bin,"Path with forward slashes"
 |cfg002,windows_path,"C:\\Users\\Admin\\Documents","Path with backslashes"
 |cfg003,quote_test,"She said ""Hello World""","Double quotes escaped"
 |cfg004,json_like,"{""key"": ""value""}","JSON-like string value"
 |cfg005,newline_test,"Line1\nLine2\nLine3","Contains newline characters"
 |cfg006,tab_test,"Col1\tCol2\tCol3","Contains tab characters"
 |cfg007,angle_brackets,"<tag>content</tag>","HTML-like content"
 |cfg008,ampersand,"Rock & Roll","Contains ampersand"
 |cfg009,pipe_test,"A|B|C","Contains pipe characters"
 |cfg010,mixed,"It's a \"test\" with 'quotes' & <special> chars","Multiple special chars"
"#;

        let questions = vec![
            Question::builder("special_001", "What is the value of config cfg002?")
                .ground_truth("C:\\Users\\Admin\\Documents")
                .question_type(QuestionType::FieldRetrieval)
                .domain("edge_case")
                .dataset("special_chars")
                .answer_type(AnswerType::String)
                .complexity(3)
                .tag("special_chars")
                .tag("backslash")
                .build(),
            Question::builder("special_002", "What is the description of cfg003?")
                .ground_truth("Double quotes escaped")
                .question_type(QuestionType::FieldRetrieval)
                .domain("edge_case")
                .dataset("special_chars")
                .answer_type(AnswerType::String)
                .complexity(2)
                .tag("special_chars")
                .tag("quotes")
                .build(),
            Question::builder("special_003", "Which config has JSON-like content?")
                .ground_truth("cfg004")
                .question_type(QuestionType::Filtering)
                .domain("edge_case")
                .dataset("special_chars")
                .answer_type(AnswerType::String)
                .complexity(3)
                .tag("special_chars")
                .tag("json")
                .build(),
            Question::builder(
                "special_004",
                "How many configs have escape sequences (\\n or \\t)?",
            )
            .ground_truth("2")
            .question_type(QuestionType::Aggregation)
            .domain("edge_case")
            .dataset("special_chars")
            .answer_type(AnswerType::Integer)
            .complexity(4)
            .tag("special_chars")
            .tag("escapes")
            .build(),
        ];

        EdgeCaseDataset {
            category: EdgeCaseCategory::SpecialChars,
            name: "special_chars_test".to_string(),
            hedl: hedl.to_string(),
            questions,
        }
    }

    fn generate_deep_nesting(&self) -> EdgeCaseDataset {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Country:[id,name,population]
%S:State:[id,name,capital]
%S:City:[id,name,population]
%S:District:[id,name,area_km2]
%S:Neighborhood:[id,name,households]
%S:Street:[id,name,length_m]
%S:Building:[id,number,floors]
%N:Country>State
%N:State>City
%N:City>District
%N:District>Neighborhood
%N:Neighborhood>Street
%N:Street>Building
---
countries:@Country
 |usa,United States,330000000
  |ca,California,Sacramento
   |sf,San Francisco,870000
    |soma,SoMa,2.5
     |mission,Mission Bay,5000
      |3rd_st,3rd Street,1200
       |bldg001,100,15
       |bldg002,102,20
       |bldg003,104,12
      |king_st,King Street,800
       |bldg004,500,25
     |potrero,Potrero Hill,3500
      |18th_st,18th Street,600
       |bldg005,200,8
    |marina,Marina District,3.2
     |chestnut,Chestnut Area,4000
      |chestnut_st,Chestnut Street,1500
       |bldg006,1800,4
       |bldg007,1802,4
  |ny,New York,Albany
   |nyc,New York City,8300000
    |manhattan,Manhattan,58.8
     |tribeca,TriBeCa,2000
      |hudson_st,Hudson Street,1000
       |bldg008,50,30
"#;

        let questions = vec![
            Question::builder("deep_001", "How many floors does building bldg004 have?")
                .ground_truth("25")
                .question_type(QuestionType::NestedNavigation)
                .domain("edge_case")
                .dataset("deep_nesting")
                .answer_type(AnswerType::Integer)
                .complexity(5)
                .tag("deep_nesting")
                .tag("6_levels")
                .build(),
            Question::builder("deep_002", "Which neighborhood is 3rd Street in?")
                .ground_truth("Mission Bay")
                .question_type(QuestionType::NestedNavigation)
                .domain("edge_case")
                .dataset("deep_nesting")
                .answer_type(AnswerType::String)
                .complexity(5)
                .tag("deep_nesting")
                .tag("parent_lookup")
                .build(),
            Question::builder(
                "deep_003",
                "How many buildings are in the SoMa district total?",
            )
            .ground_truth("5")
            .question_type(QuestionType::Aggregation)
            .domain("edge_case")
            .dataset("deep_nesting")
            .answer_type(AnswerType::Integer)
            .complexity(5)
            .tag("deep_nesting")
            .tag("recursive_count")
            .build(),
            Question::builder(
                "deep_004",
                "What is the total length in meters of all streets in Mission Bay?",
            )
            .ground_truth("2000")
            .question_type(QuestionType::MathematicalOperation)
            .domain("edge_case")
            .dataset("deep_nesting")
            .answer_type(AnswerType::Integer)
            .complexity(5)
            .tag("deep_nesting")
            .tag("sum")
            .build(),
            Question::builder("deep_005", "Which city is building bldg008 located in?")
                .ground_truth("New York City")
                .question_type(QuestionType::NestedNavigation)
                .domain("edge_case")
                .dataset("deep_nesting")
                .answer_type(AnswerType::String)
                .complexity(5)
                .tag("deep_nesting")
                .tag("ancestor_lookup")
                .build(),
        ];

        EdgeCaseDataset {
            category: EdgeCaseCategory::DeepNesting,
            name: "deep_nesting_test".to_string(),
            hedl: hedl.to_string(),
            questions,
        }
    }

    fn generate_sparse_data(&self) -> EdgeCaseDataset {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Record:[id,name,value1,value2,value3,value4,value5,status]
---
records:@Record
 |r001,Complete,100,200,300,400,500,active
 |r002,Mostly Null,~,~,~,~,42,active
 |r003,Half Null,10,~,30,~,50,~
 |r004,Single Value,~,~,~,999,~,~
 |r005,All Null,~,~,~,~,~,~
 |r006,Name Only,~,~,~,~,~,pending
 |r007,Zeros,0,0,0,0,0,active
 |r008,Negative,~,-100,~,-200,~,error
 |r009,Mixed,1,~,3,~,5,active
 |r010,Almost Complete,10,20,30,~,50,active
"#;

        let questions = vec![
            Question::builder("sparse_001", "How many records have value4 set (not null)?")
                .ground_truth("4")
                .question_type(QuestionType::Aggregation)
                .domain("edge_case")
                .dataset("sparse_data")
                .answer_type(AnswerType::Integer)
                .complexity(3)
                .tag("sparse")
                .tag("null_handling")
                .build(),
            Question::builder("sparse_002", "What is the value5 of record r002?")
                .ground_truth("42")
                .question_type(QuestionType::FieldRetrieval)
                .domain("edge_case")
                .dataset("sparse_data")
                .answer_type(AnswerType::Integer)
                .complexity(2)
                .tag("sparse")
                .build(),
            Question::builder(
                "sparse_003",
                "Which record has all values set to null (all ~)?",
            )
            .ground_truth("r005")
            .question_type(QuestionType::Filtering)
            .domain("edge_case")
            .dataset("sparse_data")
            .answer_type(AnswerType::String)
            .complexity(4)
            .tag("sparse")
            .tag("all_null")
            .build(),
            Question::builder(
                "sparse_004",
                "What is the sum of all non-null value1 values?",
            )
            .ground_truth("121")
            .question_type(QuestionType::MathematicalOperation)
            .domain("edge_case")
            .dataset("sparse_data")
            .answer_type(AnswerType::Integer)
            .complexity(4)
            .tag("sparse")
            .tag("sum_non_null")
            .build(),
            Question::builder(
                "sparse_005",
                "How many records have a status that is not null?",
            )
            .ground_truth("7")
            .question_type(QuestionType::Aggregation)
            .domain("edge_case")
            .dataset("sparse_data")
            .answer_type(AnswerType::Integer)
            .complexity(3)
            .tag("sparse")
            .build(),
        ];

        EdgeCaseDataset {
            category: EdgeCaseCategory::SparseData,
            name: "sparse_data_test".to_string(),
            hedl: hedl.to_string(),
            questions,
        }
    }

    fn generate_boundary_values(&self) -> EdgeCaseDataset {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Boundary:[id,name,int_val,float_val,str_val]
---
boundaries:@Boundary
 |b001,Zero,0,0.0,""
 |b002,Negative One,-1,-1.0,"-1"
 |b003,Max Int32,2147483647,2147483647.0,"2147483647"
 |b004,Min Int32,-2147483648,-2147483648.0,"-2147483648"
 |b005,Max Int64,9223372036854775807,9.223372036854776e18,"huge"
 |b006,Small Float,1,0.0000001,"tiny"
 |b007,Large Float,1,999999999999.99,"big"
 |b008,Empty String,0,0.0,""
 |b009,Single Char,1,1.0,"a"
 |b010,Whitespace Only,0,0.0,"   "
"#;

        let questions = vec![
            Question::builder("boundary_001", "What is the int_val of record b003?")
                .ground_truth("2147483647")
                .question_type(QuestionType::FieldRetrieval)
                .domain("edge_case")
                .dataset("boundary_values")
                .answer_type(AnswerType::Integer)
                .complexity(2)
                .tag("boundary")
                .tag("max_int32")
                .build(),
            Question::builder("boundary_002", "Which record has the minimum int_val?")
                .ground_truth("b004")
                .question_type(QuestionType::Comparison)
                .domain("edge_case")
                .dataset("boundary_values")
                .answer_type(AnswerType::String)
                .complexity(3)
                .tag("boundary")
                .tag("min")
                .build(),
            Question::builder("boundary_003", "How many records have int_val equal to 0?")
                .ground_truth("4")
                .question_type(QuestionType::Aggregation)
                .domain("edge_case")
                .dataset("boundary_values")
                .answer_type(AnswerType::Integer)
                .complexity(2)
                .tag("boundary")
                .tag("zero")
                .build(),
            Question::builder(
                "boundary_004",
                "What is the float_val of the record with the smallest positive float_val?",
            )
            .ground_truth("0.0000001")
            .question_type(QuestionType::Comparison)
            .domain("edge_case")
            .dataset("boundary_values")
            .answer_type(AnswerType::Number { decimals: 7 })
            .complexity(4)
            .tag("boundary")
            .tag("small_float")
            .build(),
            Question::builder("boundary_005", "How many records have an empty str_val?")
                .ground_truth("2")
                .question_type(QuestionType::Filtering)
                .domain("edge_case")
                .dataset("boundary_values")
                .answer_type(AnswerType::Integer)
                .complexity(3)
                .tag("boundary")
                .tag("empty_string")
                .build(),
        ];

        EdgeCaseDataset {
            category: EdgeCaseCategory::BoundaryValues,
            name: "boundary_values_test".to_string(),
            hedl: hedl.to_string(),
            questions,
        }
    }

    fn generate_ambiguous_format(&self) -> EdgeCaseDataset {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Ambiguous:[id,value,type_hint]
---
items:@Ambiguous
 |a001,2024,year
 |a002,2024-01-15,date
 |a003,20240115,date_compact
 |a004,12345,zipcode
 |a005,12345,number
 |a006,1.5,version
 |a007,1.5,number
 |a008,true,boolean
 |a009,True,string
 |a010,null,string_literal
 |a011,~,actual_null
 |a012,001234,leading_zeros
 |a013,1234,no_leading_zeros
 |a014,+1-555-0123,phone
 |a015,1555012300,numeric_phone
"#;

        let questions = vec![
            Question::builder("ambiguous_001", "How many items have type_hint 'number'?")
                .ground_truth("2")
                .question_type(QuestionType::Aggregation)
                .domain("edge_case")
                .dataset("ambiguous")
                .answer_type(AnswerType::Integer)
                .complexity(2)
                .tag("ambiguous")
                .build(),
            Question::builder(
                "ambiguous_002",
                "What is the value of the item with type_hint 'date'?",
            )
            .ground_truth("2024-01-15")
            .question_type(QuestionType::FieldRetrieval)
            .domain("edge_case")
            .dataset("ambiguous")
            .answer_type(AnswerType::String)
            .complexity(2)
            .tag("ambiguous")
            .tag("date")
            .build(),
            Question::builder(
                "ambiguous_003",
                "Which item has the string 'null' as its value (not actual null)?",
            )
            .ground_truth("a010")
            .question_type(QuestionType::Filtering)
            .domain("edge_case")
            .dataset("ambiguous")
            .answer_type(AnswerType::String)
            .complexity(4)
            .tag("ambiguous")
            .tag("null_vs_string")
            .build(),
            Question::builder(
                "ambiguous_004",
                "What is the difference between a012 and a013 values as numbers?",
            )
            .ground_truth("0")
            .question_type(QuestionType::MathematicalOperation)
            .domain("edge_case")
            .dataset("ambiguous")
            .answer_type(AnswerType::Integer)
            .complexity(3)
            .tag("ambiguous")
            .tag("leading_zeros")
            .build(),
            Question::builder(
                "ambiguous_005",
                "Which item has an actual null value (represented as ~)?",
            )
            .ground_truth("a011")
            .question_type(QuestionType::Filtering)
            .domain("edge_case")
            .dataset("ambiguous")
            .answer_type(AnswerType::String)
            .complexity(3)
            .tag("ambiguous")
            .tag("null")
            .build(),
        ];

        EdgeCaseDataset {
            category: EdgeCaseCategory::AmbiguousFormat,
            name: "ambiguous_format_test".to_string(),
            hedl: hedl.to_string(),
            questions,
        }
    }

    fn generate_large_numbers(&self) -> EdgeCaseDataset {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:LargeNum:[id,name,value,unit]
---
numbers:@LargeNum
 |n001,Avogadro,6.02214076e23,mol^-1
 |n002,Speed of Light,299792458,m/s
 |n003,Earth Mass,5.972e24,kg
 |n004,Planck Constant,6.62607015e-34,J*s
 |n005,US Debt Dollars,35000000000000,USD
 |n006,Pi Extended,3.141592653589793,dimensionless
 |n007,Euler Number,2.718281828459045,dimensionless
 |n008,Nanosecond,0.000000001,seconds
 |n009,Microsecond,0.000001,seconds
 |n010,Big Integer,123456789012345678901234567890,count
"#;

        let questions = vec![
            Question::builder("large_001", "What is the value of Avogadro's number?")
                .ground_truth("6.02214076e23")
                .question_type(QuestionType::FieldRetrieval)
                .domain("edge_case")
                .dataset("large_numbers")
                .answer_type(AnswerType::String)
                .complexity(3)
                .tag("large_numbers")
                .tag("scientific_notation")
                .build(),
            Question::builder("large_002", "Which number has the smallest positive value?")
                .ground_truth("Planck Constant")
                .question_type(QuestionType::Comparison)
                .domain("edge_case")
                .dataset("large_numbers")
                .answer_type(AnswerType::String)
                .complexity(4)
                .tag("large_numbers")
                .tag("smallest")
                .build(),
            Question::builder(
                "large_003",
                "What is Pi Extended rounded to 2 decimal places?",
            )
            .ground_truth("3.14")
            .question_type(QuestionType::MathematicalOperation)
            .domain("edge_case")
            .dataset("large_numbers")
            .answer_type(AnswerType::Number { decimals: 2 })
            .complexity(3)
            .tag("large_numbers")
            .tag("precision")
            .build(),
            Question::builder(
                "large_004",
                "How many numbers have values greater than 1e20?",
            )
            .ground_truth("3")
            .question_type(QuestionType::Aggregation)
            .domain("edge_case")
            .dataset("large_numbers")
            .answer_type(AnswerType::Integer)
            .complexity(4)
            .tag("large_numbers")
            .tag("comparison")
            .build(),
        ];

        EdgeCaseDataset {
            category: EdgeCaseCategory::LargeNumbers,
            name: "large_numbers_test".to_string(),
            hedl: hedl.to_string(),
            questions,
        }
    }

    fn generate_long_strings(&self) -> EdgeCaseDataset {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:LongText:[id,title,content,word_count]
---
texts:@LongText
 |t001,Short,Hello world,2
 |t002,Medium,"This is a medium length text with multiple words that spans across what would normally be considered a single line of content.",21
 |t003,Paragraph,"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.",56
 |t004,Repeated,"word word word word word word word word word word word word word word word word word word word word",20
 |t005,Numbers,"1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30",30
 |t006,Single Letter,a,1
 |t007,Whitespace Heavy,"  word   word    word     word      ",4
"#;

        let questions = vec![
            Question::builder("long_001", "What is the word_count of text t003?")
                .ground_truth("56")
                .question_type(QuestionType::FieldRetrieval)
                .domain("edge_case")
                .dataset("long_strings")
                .answer_type(AnswerType::Integer)
                .complexity(2)
                .tag("long_strings")
                .build(),
            Question::builder("long_002", "Which text has the highest word_count?")
                .ground_truth("t003")
                .question_type(QuestionType::Comparison)
                .domain("edge_case")
                .dataset("long_strings")
                .answer_type(AnswerType::String)
                .complexity(3)
                .tag("long_strings")
                .tag("max")
                .build(),
            Question::builder("long_003", "What is the total word_count across all texts?")
                .ground_truth("134")
                .question_type(QuestionType::MathematicalOperation)
                .domain("edge_case")
                .dataset("long_strings")
                .answer_type(AnswerType::Integer)
                .complexity(3)
                .tag("long_strings")
                .tag("sum")
                .build(),
            Question::builder("long_004", "What is the first word in text t003?")
                .ground_truth("Lorem")
                .question_type(QuestionType::PatternMatching)
                .domain("edge_case")
                .dataset("long_strings")
                .answer_type(AnswerType::String)
                .complexity(3)
                .tag("long_strings")
                .tag("extraction")
                .build(),
        ];

        EdgeCaseDataset {
            category: EdgeCaseCategory::LongStrings,
            name: "long_strings_test".to_string(),
            hedl: hedl.to_string(),
            questions,
        }
    }

    fn generate_duplicates(&self) -> EdgeCaseDataset {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Item:[id,name,value,category]
---
items:@Item
 |i001,Apple,100,fruit
 |i002,Banana,200,fruit
 |i003,Apple,150,fruit
 |i004,Carrot,50,vegetable
 |i005,Apple,100,fruit
 |i006,Banana,200,fruit
 |i007,Date,300,fruit
 |i008,Eggplant,75,vegetable
 |i009,Apple,100,fruit
 |i010,Fig,250,fruit
"#;

        let questions = vec![
            Question::builder("dup_001", "How many items are named 'Apple'?")
                .ground_truth("4")
                .question_type(QuestionType::Aggregation)
                .domain("edge_case")
                .dataset("duplicates")
                .answer_type(AnswerType::Integer)
                .complexity(2)
                .tag("duplicates")
                .build(),
            Question::builder("dup_002", "How many unique names are in the dataset?")
                .ground_truth("7")
                .question_type(QuestionType::Aggregation)
                .domain("edge_case")
                .dataset("duplicates")
                .answer_type(AnswerType::Integer)
                .complexity(3)
                .tag("duplicates")
                .tag("unique")
                .build(),
            Question::builder(
                "dup_003",
                "What is the total value of all items named 'Apple'?",
            )
            .ground_truth("450")
            .question_type(QuestionType::MathematicalOperation)
            .domain("edge_case")
            .dataset("duplicates")
            .answer_type(AnswerType::Integer)
            .complexity(3)
            .tag("duplicates")
            .tag("sum")
            .build(),
            Question::builder(
                "dup_004",
                "How many items have both name 'Apple' and value 100?",
            )
            .ground_truth("3")
            .question_type(QuestionType::Filtering)
            .domain("edge_case")
            .dataset("duplicates")
            .answer_type(AnswerType::Integer)
            .complexity(3)
            .tag("duplicates")
            .tag("exact_match")
            .build(),
        ];

        EdgeCaseDataset {
            category: EdgeCaseCategory::Duplicates,
            name: "duplicates_test".to_string(),
            hedl: hedl.to_string(),
            questions,
        }
    }

    fn generate_circular_refs(&self) -> EdgeCaseDataset {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Person:[id,name,manager,mentor,friend]
---
people:@Person
 |p001,Alice,@Person:p002,@Person:p003,@Person:p004
 |p002,Bob,@Person:p003,@Person:p001,@Person:p001
 |p003,Carol,~,@Person:p002,@Person:p002
 |p004,David,@Person:p001,@Person:p001,@Person:p003
 |p005,Eve,@Person:p004,@Person:p004,@Person:p001
"#;

        let questions = vec![
            Question::builder("circ_001", "Who is Alice's manager?")
                .ground_truth("Bob")
                .question_type(QuestionType::ReferenceResolution)
                .domain("edge_case")
                .dataset("circular_refs")
                .answer_type(AnswerType::String)
                .complexity(3)
                .tag("circular")
                .tag("reference")
                .build(),
            Question::builder("circ_002", "Who is Bob's manager's mentor?")
                .ground_truth("Bob")
                .question_type(QuestionType::RelationshipTraversal)
                .domain("edge_case")
                .dataset("circular_refs")
                .answer_type(AnswerType::String)
                .complexity(5)
                .tag("circular")
                .tag("chain")
                .build(),
            Question::builder("circ_003", "How many people have Alice as their mentor?")
                .ground_truth("2")
                .question_type(QuestionType::Aggregation)
                .domain("edge_case")
                .dataset("circular_refs")
                .answer_type(AnswerType::Integer)
                .complexity(4)
                .tag("circular")
                .tag("reverse_lookup")
                .build(),
            Question::builder("circ_004", "Who is the only person without a manager?")
                .ground_truth("Carol")
                .question_type(QuestionType::Filtering)
                .domain("edge_case")
                .dataset("circular_refs")
                .answer_type(AnswerType::String)
                .complexity(3)
                .tag("circular")
                .tag("null_ref")
                .build(),
        ];

        EdgeCaseDataset {
            category: EdgeCaseCategory::CircularRefs,
            name: "circular_refs_test".to_string(),
            hedl: hedl.to_string(),
            questions,
        }
    }
}

impl Default for EdgeCaseGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_categories() {
        assert_eq!(EdgeCaseCategory::ALL.len(), 10);
        for cat in EdgeCaseCategory::ALL {
            assert!(!cat.name().is_empty());
            assert!(!cat.description().is_empty());
            assert!(cat.difficulty() >= 1 && cat.difficulty() <= 5);
        }
    }

    #[test]
    fn test_generator() {
        let generator = EdgeCaseGenerator::new();
        let datasets = generator.generate_all();

        assert_eq!(datasets.len(), 10);
        for ds in &datasets {
            assert!(!ds.hedl.is_empty());
            assert!(!ds.questions.is_empty());
        }
    }

    #[test]
    fn test_unicode_dataset() {
        let generator = EdgeCaseGenerator::new();
        let ds = generator.generate(EdgeCaseCategory::Unicode);

        assert!(ds.hedl.contains("Björk"));
        assert!(ds.hedl.contains("🇮🇸"));
        assert!(ds.hedl.contains("日本語"));
    }

    #[test]
    fn test_deep_nesting_dataset() {
        let generator = EdgeCaseGenerator::new();
        let ds = generator.generate(EdgeCaseCategory::DeepNesting);

        // Should have 6+ levels of nesting
        assert!(ds.hedl.contains("%N:Street>Building"));
        assert!(ds.questions.len() >= 4);
    }
}
