// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Real-world datasets with verified ground truth for LLM accuracy benchmarks.
//!
//! These datasets are based on publicly available data from:
//! - JSONPlaceholder (typicode)
//! - DummyJSON
//! - Real-world IoT/sensor data patterns
//!
//! Each dataset has VERIFIED ground truth values that exactly match the data.

// Re-export from new accuracy module for fixture loading
pub use crate::accuracy::complexity::ComplexityLevel;
pub use crate::accuracy::fixtures::{load_default_fixtures, FixtureDataset};
use crate::accuracy::questions::{AnswerType, Question, QuestionType};

/// Generate all real-world test datasets with verified ground truth.
///
/// Returns datasets using the new `FixtureDataset` type with auto-conversion.
#[must_use]
pub fn generate_real_datasets() -> Vec<FixtureDataset> {
    let mut datasets = Vec::new();

    // ========== EASY: Simple flat data (5-10 records) ==========

    if let Ok(ds) = easy_employees_dataset() {
        datasets.push(ds);
    }

    if let Ok(ds) = easy_sensors_dataset() {
        datasets.push(ds);
    }

    // ========== MEDIUM: Moderate size with some nesting (20-30 records) ==========

    if let Ok(ds) = medium_products_dataset() {
        datasets.push(ds);
    }

    if let Ok(ds) = medium_orders_dataset() {
        datasets.push(ds);
    }

    // ========== HARD: Complex nesting and cross-references ==========

    if let Ok(ds) = hard_ecommerce_dataset() {
        datasets.push(ds);
    }

    if let Ok(ds) = hard_factory_dataset() {
        datasets.push(ds);
    }

    datasets
}

/// Load test datasets from pre-made fixture files (no conversion).
/// Each format is hand-crafted and verified, not auto-converted.
///
/// This function now uses the new accuracy module's FixtureDataset.
#[must_use]
pub fn load_fixture_datasets() -> Vec<FixtureDataset> {
    load_default_fixtures()
}

// =============================================================================
// EASY DATASETS
// =============================================================================

/// Easy: 8 employees, flat structure, simple queries
/// Ground truth verified by manual count
fn easy_employees_dataset() -> Result<FixtureDataset, String> {
    // Real data based on JSONPlaceholder style
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
employees:@Employee[id,name,email,department,salary,hire_date]
 |e001,Alice Johnson,alice.johnson@techcorp.com,Engineering,95000,2021-03-15
 |e002,Bob Smith,bob.smith@techcorp.com,Engineering,87000,2022-01-10
 |e003,Carol Williams,carol.williams@techcorp.com,Marketing,72000,2020-06-01
 |e004,David Brown,david.brown@techcorp.com,Engineering,91000,2021-08-20
 |e005,Emma Davis,emma.davis@techcorp.com,Sales,68000,2023-02-14
 |e006,Frank Miller,frank.miller@techcorp.com,Marketing,75000,2019-11-30
 |e007,Grace Lee,grace.lee@techcorp.com,Engineering,102000,2018-04-05
 |e008,Henry Wilson,henry.wilson@techcorp.com,Sales,71000,2022-09-01
"#;

    // VERIFIED GROUND TRUTH:
    // - Total employees: 8
    // - Engineering dept: e001, e002, e004, e007 = 4 employees
    // - Marketing dept: e003, e006 = 2 employees
    // - Sales dept: e005, e008 = 2 employees
    // - Salary > 90000: e001 (95000), e004 (91000), e007 (102000) = 3 employees
    // - Hired in 2021: e001, e004 = 2 employees
    // - Alice Johnson's salary: 95000
    // - Grace Lee's department: Engineering

    let questions = vec![
        Question {
            id: "easy_emp_q01".into(),
            dataset: "easy_employees".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many employees are there in total?".into(),
            ground_truth: "8".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: Some("Simple count".into()),
            complexity_level: 2,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "easy_emp_q02".into(),
            dataset: "easy_employees".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many employees work in the Engineering department?".into(),
            ground_truth: "4".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: Some("Filter and count".into()),
            complexity_level: 2,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "easy_emp_q03".into(),
            dataset: "easy_employees".into(),
            question_type: QuestionType::FieldRetrieval,
            domain: "real_datasets".into(),
            prompt: "What is Alice Johnson's salary?".into(),
            ground_truth: "95000".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: Some("Direct lookup".into()),
            complexity_level: 2,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "easy_emp_q04".into(),
            dataset: "easy_employees".into(),
            question_type: QuestionType::FieldRetrieval,
            domain: "real_datasets".into(),
            prompt: "What department does Grace Lee work in?".into(),
            ground_truth: "Engineering".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::String,
            notes: Some("Direct lookup".into()),
            complexity_level: 2,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "easy_emp_q05".into(),
            dataset: "easy_employees".into(),
            question_type: QuestionType::Filtering,
            domain: "real_datasets".into(),
            prompt: "How many employees have a salary greater than 90000?".into(),
            ground_truth: "3".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: Some("Numeric filter".into()),
            complexity_level: 2,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "easy_emp_q06".into(),
            dataset: "easy_employees".into(),
            question_type: QuestionType::FieldRetrieval,
            domain: "real_datasets".into(),
            prompt: "What is the email of employee e005?".into(),
            ground_truth: "emma.davis@techcorp.com".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::String,
            notes: Some("ID-based lookup".into()),
            complexity_level: 2,
            tags: Vec::new(),
            blind_mode: true,
        },
    ];

    FixtureDataset::from_hedl("easy_employees", hedl, questions, ComplexityLevel::L2Basic)
}

/// Easy: 10 IoT sensor readings, flat time-series
fn easy_sensors_dataset() -> Result<FixtureDataset, String> {
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
readings:@Reading[id,sensor_id,timestamp,temperature,humidity,status]
 |r001,sensor_a,2024-01-15T08:00:00Z,22.5,45.2,normal
 |r002,sensor_a,2024-01-15T09:00:00Z,23.1,44.8,normal
 |r003,sensor_b,2024-01-15T08:00:00Z,19.8,52.1,normal
 |r004,sensor_b,2024-01-15T09:00:00Z,20.2,51.5,normal
 |r005,sensor_a,2024-01-15T10:00:00Z,24.8,43.2,warning
 |r006,sensor_c,2024-01-15T08:00:00Z,18.2,58.9,normal
 |r007,sensor_c,2024-01-15T09:00:00Z,18.5,57.2,normal
 |r008,sensor_b,2024-01-15T10:00:00Z,21.5,50.1,normal
 |r009,sensor_a,2024-01-15T11:00:00Z,26.2,41.5,alert
 |r010,sensor_c,2024-01-15T10:00:00Z,19.1,55.8,normal
"#;

    // VERIFIED GROUND TRUTH:
    // - Total readings: 10
    // - sensor_a readings: r001, r002, r005, r009 = 4
    // - sensor_b readings: r003, r004, r008 = 3
    // - sensor_c readings: r006, r007, r010 = 3
    // - status=normal: r001, r002, r003, r004, r006, r007, r008, r010 = 8
    // - status=warning: r005 = 1
    // - status=alert: r009 = 1
    // - Temperature > 24: r005 (24.8), r009 (26.2) = 2
    // - r005 temperature: 24.8
    // - Humidity of r003: 52.1

    let questions = vec![
        Question {
            id: "easy_sensor_q01".into(),
            dataset: "easy_sensors".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many sensor readings are there in total?".into(),
            ground_truth: "10".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 2,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "easy_sensor_q02".into(),
            dataset: "easy_sensors".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many readings are from sensor_a?".into(),
            ground_truth: "4".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 2,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "easy_sensor_q03".into(),
            dataset: "easy_sensors".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many readings have status 'normal'?".into(),
            ground_truth: "8".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 2,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "easy_sensor_q04".into(),
            dataset: "easy_sensors".into(),
            question_type: QuestionType::FieldRetrieval,
            domain: "real_datasets".into(),
            prompt: "What is the temperature in reading r005?".into(),
            ground_truth: "24.8".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Number { decimals: 1 },
            notes: None,
            complexity_level: 2,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "easy_sensor_q05".into(),
            dataset: "easy_sensors".into(),
            question_type: QuestionType::Filtering,
            domain: "real_datasets".into(),
            prompt: "How many readings have temperature greater than 24?".into(),
            ground_truth: "2".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 2,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "easy_sensor_q06".into(),
            dataset: "easy_sensors".into(),
            question_type: QuestionType::FieldRetrieval,
            domain: "real_datasets".into(),
            prompt: "What is the humidity value for reading r003?".into(),
            ground_truth: "52.1".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Number { decimals: 1 },
            notes: None,
            complexity_level: 2,
            tags: Vec::new(),
            blind_mode: true,
        },
    ];

    FixtureDataset::from_hedl("easy_sensors", hedl, questions, ComplexityLevel::L2Basic)
}

// =============================================================================
// MEDIUM DATASETS
// =============================================================================

/// Medium: 15 products with categories and inventory
fn medium_products_dataset() -> Result<FixtureDataset, String> {
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
products:@Product[id,name,category,price,stock,rating]
 |p001,Wireless Mouse,electronics,29.99,150,4.5
 |p002,USB-C Cable,electronics,12.99,500,4.2
 |p003,Mechanical Keyboard,electronics,89.99,75,4.8
 |p004,Monitor Stand,furniture,45.00,60,4.1
 |p005,Desk Lamp,furniture,35.50,120,4.3
 |p006,Office Chair,furniture,199.99,25,4.7
 |p007,Notebook Set,stationery,8.99,300,4.0
 |p008,Pen Pack,stationery,5.99,450,3.9
 |p009,Desk Organizer,stationery,22.50,80,4.4
 |p010,Webcam HD,electronics,59.99,90,4.6
 |p011,Headphone Stand,furniture,28.00,110,4.2
 |p012,Cable Management Kit,electronics,15.99,200,4.1
 |p013,Sticky Notes,stationery,3.99,600,4.0
 |p014,Whiteboard,furniture,75.00,40,4.5
 |p015,Ergonomic Wrist Rest,electronics,19.99,180,4.3
"#;

    // VERIFIED GROUND TRUTH:
    // - Total products: 15
    // - electronics: p001, p002, p003, p010, p012, p015 = 6
    // - furniture: p004, p005, p006, p011, p014 = 5
    // - stationery: p007, p008, p009, p013 = 4
    // - Price > 50: p003 (89.99), p006 (199.99), p010 (59.99), p014 (75.00) = 4
    // - Stock < 100: p003 (75), p004 (60), p006 (25), p009 (80), p010 (90), p014 (40) = 6
    // - Rating >= 4.5: p001 (4.5), p003 (4.8), p006 (4.7), p010 (4.6), p014 (4.5) = 5
    // - Mechanical Keyboard price: 89.99
    // - p006 name: Office Chair

    let questions = vec![
        Question {
            id: "med_prod_q01".into(),
            dataset: "medium_products".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many products are there in total?".into(),
            ground_truth: "15".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_prod_q02".into(),
            dataset: "medium_products".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many products are in the electronics category?".into(),
            ground_truth: "6".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_prod_q03".into(),
            dataset: "medium_products".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many products are in the furniture category?".into(),
            ground_truth: "5".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_prod_q04".into(),
            dataset: "medium_products".into(),
            question_type: QuestionType::Filtering,
            domain: "real_datasets".into(),
            prompt: "How many products have a price greater than 50?".into(),
            ground_truth: "4".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_prod_q05".into(),
            dataset: "medium_products".into(),
            question_type: QuestionType::Filtering,
            domain: "real_datasets".into(),
            prompt: "How many products have stock less than 100?".into(),
            ground_truth: "6".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_prod_q06".into(),
            dataset: "medium_products".into(),
            question_type: QuestionType::FieldRetrieval,
            domain: "real_datasets".into(),
            prompt: "What is the price of the Mechanical Keyboard?".into(),
            ground_truth: "89.99".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Number { decimals: 2 },
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_prod_q07".into(),
            dataset: "medium_products".into(),
            question_type: QuestionType::FieldRetrieval,
            domain: "real_datasets".into(),
            prompt: "What is the name of product p006?".into(),
            ground_truth: "Office Chair".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::String,
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_prod_q08".into(),
            dataset: "medium_products".into(),
            question_type: QuestionType::Filtering,
            domain: "real_datasets".into(),
            prompt: "How many products have a rating of 4.5 or higher?".into(),
            ground_truth: "5".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
    ];

    FixtureDataset::from_hedl(
        "medium_products",
        hedl,
        questions,
        ComplexityLevel::L3Intermediate,
    )
}

/// Medium: 12 orders with status tracking
fn medium_orders_dataset() -> Result<FixtureDataset, String> {
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
orders:@Order[id,customer,total,status,items_count,order_date]
 |ord001,customer_a,125.50,delivered,3,2024-01-10
 |ord002,customer_b,89.99,delivered,1,2024-01-11
 |ord003,customer_c,234.00,shipped,5,2024-01-12
 |ord004,customer_a,45.00,delivered,2,2024-01-13
 |ord005,customer_d,178.50,processing,4,2024-01-14
 |ord006,customer_b,67.25,delivered,2,2024-01-15
 |ord007,customer_e,312.00,shipped,6,2024-01-16
 |ord008,customer_c,55.99,cancelled,1,2024-01-17
 |ord009,customer_a,199.00,processing,3,2024-01-18
 |ord010,customer_f,88.50,delivered,2,2024-01-19
 |ord011,customer_d,145.75,shipped,3,2024-01-20
 |ord012,customer_b,276.00,delivered,4,2024-01-21
"#;

    // VERIFIED GROUND TRUTH:
    // - Total orders: 12
    // - delivered: ord001, ord002, ord004, ord006, ord010, ord012 = 6
    // - shipped: ord003, ord007, ord011 = 3
    // - processing: ord005, ord009 = 2
    // - cancelled: ord008 = 1
    // - customer_a orders: ord001, ord004, ord009 = 3
    // - customer_b orders: ord002, ord006, ord012 = 3
    // - Total > 200: ord003 (234), ord007 (312), ord012 (276) = 3
    // - ord007 total: 312.00
    // - Total items: 3+1+5+2+4+2+6+1+3+2+3+4 = 36

    let questions = vec![
        Question {
            id: "med_ord_q01".into(),
            dataset: "medium_orders".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many orders are there in total?".into(),
            ground_truth: "12".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_ord_q02".into(),
            dataset: "medium_orders".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many orders have status 'delivered'?".into(),
            ground_truth: "6".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_ord_q03".into(),
            dataset: "medium_orders".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many orders have status 'shipped'?".into(),
            ground_truth: "3".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_ord_q04".into(),
            dataset: "medium_orders".into(),
            question_type: QuestionType::Filtering,
            domain: "real_datasets".into(),
            prompt: "How many orders were placed by customer_a?".into(),
            ground_truth: "3".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_ord_q05".into(),
            dataset: "medium_orders".into(),
            question_type: QuestionType::Filtering,
            domain: "real_datasets".into(),
            prompt: "How many orders have a total greater than 200?".into(),
            ground_truth: "3".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_ord_q06".into(),
            dataset: "medium_orders".into(),
            question_type: QuestionType::FieldRetrieval,
            domain: "real_datasets".into(),
            prompt: "What is the total amount for order ord007?".into(),
            ground_truth: "312.00".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Number { decimals: 2 },
            notes: None,
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "med_ord_q07".into(),
            dataset: "medium_orders".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "What is the total number of items across all orders?".into(),
            ground_truth: "36".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: Some("Sum of items_count column".into()),
            complexity_level: 3,
            tags: Vec::new(),
            blind_mode: true,
        },
    ];

    FixtureDataset::from_hedl(
        "medium_orders",
        hedl,
        questions,
        ComplexityLevel::L3Intermediate,
    )
}

// =============================================================================
// HARD DATASETS
// =============================================================================

/// Hard: E-commerce with nested products and reviews
fn hard_ecommerce_dataset() -> Result<FixtureDataset, String> {
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Category:[id,name,parent]
%S:Product:[id,name,category,price,stock,avg_rating]
%S:Review:[id,product,user,rating,date]
%N:Product>Review
---
categories:@Category
 |cat01,Electronics,~
 |cat02,Computers,@Category:cat01
 |cat03,Audio,@Category:cat01
 |cat04,Home,~
 |cat05,Furniture,@Category:cat04

products:@Product
 |prod01,Laptop Pro 15,@Category:cat02,1299.99,45,4.6
  @Review#3:
  |rev01,prod01,user_a,5,2024-01-05
  |rev02,prod01,user_b,4,2024-01-08
  |rev03,prod01,user_c,5,2024-01-12
 |prod02,Wireless Headphones,@Category:cat03,149.99,120,4.3
  @Review#2:
  |rev04,prod02,user_d,4,2024-01-06
  |rev05,prod02,user_e,5,2024-01-10
 |prod03,Standing Desk,@Category:cat05,549.00,30,4.7
  @Review#3:
  |rev06,prod03,user_a,5,2024-01-07
  |rev07,prod03,user_f,4,2024-01-11
  |rev08,prod03,user_g,5,2024-01-15
 |prod04,USB-C Hub,@Category:cat02,79.99,200,4.1
  @Review#1:
  |rev09,prod04,user_h,4,2024-01-09
 |prod05,Office Chair,@Category:cat05,399.00,55,4.5
  @Review#2:
  |rev10,prod05,user_b,5,2024-01-13
  |rev11,prod05,user_i,4,2024-01-16
"#;

    // VERIFIED GROUND TRUTH:
    // - Total categories: 5
    // - Total products: 5
    // - Total reviews: 11
    // - Products in cat02 (Computers): prod01, prod04 = 2
    // - Products in cat05 (Furniture): prod03, prod05 = 2
    // - Reviews for prod01: rev01, rev02, rev03 = 3
    // - Reviews for prod03: rev06, rev07, rev08 = 3
    // - 5-star reviews: rev01, rev03, rev05, rev06, rev08, rev10 = 6
    // - 4-star reviews: rev02, rev04, rev07, rev09, rev11 = 5
    // - Laptop Pro 15 price: 1299.99
    // - Parent of cat02: cat01 (Electronics)
    // - Products with rating >= 4.5: prod01 (4.6), prod03 (4.7), prod05 (4.5) = 3

    let questions = vec![
        Question {
            id: "hard_ecom_q01".into(),
            dataset: "hard_ecommerce".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many products are there in total?".into(),
            ground_truth: "5".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_ecom_q02".into(),
            dataset: "hard_ecommerce".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many reviews are there in total?".into(),
            ground_truth: "11".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: Some("Count nested reviews".into()),
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_ecom_q03".into(),
            dataset: "hard_ecommerce".into(),
            question_type: QuestionType::StructureAwareness,
            domain: "real_datasets".into(),
            prompt: "How many reviews does the Laptop Pro 15 have?".into(),
            ground_truth: "3".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: Some("Nested child counting".into()),
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_ecom_q04".into(),
            dataset: "hard_ecommerce".into(),
            question_type: QuestionType::Filtering,
            domain: "real_datasets".into(),
            prompt: "How many reviews have a rating of 5?".into(),
            ground_truth: "6".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_ecom_q05".into(),
            dataset: "hard_ecommerce".into(),
            question_type: QuestionType::FieldRetrieval,
            domain: "real_datasets".into(),
            prompt: "What is the price of the Laptop Pro 15?".into(),
            ground_truth: "1299.99".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Number { decimals: 2 },
            notes: None,
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_ecom_q06".into(),
            dataset: "hard_ecommerce".into(),
            question_type: QuestionType::StructureAwareness,
            domain: "real_datasets".into(),
            prompt: "What is the parent category of Computers?".into(),
            ground_truth: "Electronics".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::String,
            notes: Some("Cross-reference resolution".into()),
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_ecom_q07".into(),
            dataset: "hard_ecommerce".into(),
            question_type: QuestionType::Filtering,
            domain: "real_datasets".into(),
            prompt: "How many products have an average rating of 4.5 or higher?".into(),
            ground_truth: "3".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_ecom_q08".into(),
            dataset: "hard_ecommerce".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many categories are there in total?".into(),
            ground_truth: "5".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
    ];

    FixtureDataset::from_hedl(
        "hard_ecommerce",
        hedl,
        questions,
        ComplexityLevel::L4Advanced,
    )
}

/// Hard: Factory with equipment, sensors, and maintenance logs
fn hard_factory_dataset() -> Result<FixtureDataset, String> {
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Zone:[id,name,type,capacity]
%S:Equipment:[id,name,zone,status,install_date]
%S:Sensor:[id,equipment,type,last_reading,unit]
%S:MaintenanceLog:[id,equipment,type,date,cost]
%N:Zone>Equipment
%N:Equipment>Sensor
---
zones:@Zone
 |z01,Assembly Line A,production,50
  @Equipment#3:
  |eq001,CNC Machine 1,@Zone:z01,operational,2022-03-15
   @Sensor#2:
   |s001,eq001,temperature,45.2,celsius
   |s002,eq001,vibration,0.12,mm/s
  |eq002,Robot Arm Alpha,@Zone:z01,operational,2021-08-20
   @Sensor#2:
   |s003,eq002,temperature,38.5,celsius
   |s004,eq002,current,12.5,amps
  |eq003,Conveyor Belt 1,@Zone:z01,maintenance,2020-01-10
   @Sensor#1:
   |s005,eq003,speed,0.0,m/s
 |z02,Assembly Line B,production,40
  @Equipment#2:
  |eq004,CNC Machine 2,@Zone:z02,operational,2023-01-05
   @Sensor#2:
   |s006,eq004,temperature,42.8,celsius
   |s007,eq004,vibration,0.08,mm/s
  |eq005,Robot Arm Beta,@Zone:z02,operational,2022-06-12
   @Sensor#1:
   |s008,eq005,temperature,36.2,celsius
 |z03,Warehouse,storage,200
  @Equipment#2:
  |eq006,Forklift 1,@Zone:z03,operational,2021-11-30
   @Sensor#1:
   |s009,eq006,battery,78,percent
  |eq007,Forklift 2,@Zone:z03,idle,2022-04-18
   @Sensor#1:
   |s010,eq007,battery,95,percent

maintenance_logs:@MaintenanceLog
 |m001,eq001,preventive,2024-01-05,250.00
 |m002,eq003,repair,2024-01-10,1200.00
 |m003,eq002,preventive,2024-01-12,180.00
 |m004,eq006,preventive,2024-01-15,120.00
 |m005,eq003,inspection,2024-01-18,50.00
"#;

    // VERIFIED GROUND TRUTH:
    // - Total zones: 3
    // - Total equipment: 7
    // - Total sensors: 10
    // - Total maintenance logs: 5
    // - Equipment in z01: eq001, eq002, eq003 = 3
    // - Equipment in z02: eq004, eq005 = 2
    // - Equipment in z03: eq006, eq007 = 2
    // - Operational equipment: eq001, eq002, eq004, eq005, eq006 = 5
    // - Temperature sensors: s001, s003, s006, s008 = 4
    // - Maintenance cost sum: 250 + 1200 + 180 + 120 + 50 = 1800
    // - Preventive maintenance count: m001, m003, m004 = 3
    // - eq003 status: maintenance
    // - Sensors on eq001: s001, s002 = 2

    let questions = vec![
        Question {
            id: "hard_factory_q01".into(),
            dataset: "hard_factory".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many pieces of equipment are there in total?".into(),
            ground_truth: "7".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_factory_q02".into(),
            dataset: "hard_factory".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many sensors are there in total?".into(),
            ground_truth: "10".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: Some("Count deeply nested sensors".into()),
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_factory_q03".into(),
            dataset: "hard_factory".into(),
            question_type: QuestionType::StructureAwareness,
            domain: "real_datasets".into(),
            prompt: "How many pieces of equipment are in Assembly Line A (zone z01)?".into(),
            ground_truth: "3".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_factory_q04".into(),
            dataset: "hard_factory".into(),
            question_type: QuestionType::Filtering,
            domain: "real_datasets".into(),
            prompt: "How many pieces of equipment have status 'operational'?".into(),
            ground_truth: "5".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_factory_q05".into(),
            dataset: "hard_factory".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many temperature sensors are there?".into(),
            ground_truth: "4".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_factory_q06".into(),
            dataset: "hard_factory".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "What is the total cost of all maintenance logs?".into(),
            ground_truth: "1800".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: Some("Sum of cost column".into()),
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_factory_q07".into(),
            dataset: "hard_factory".into(),
            question_type: QuestionType::Filtering,
            domain: "real_datasets".into(),
            prompt: "How many maintenance logs are of type 'preventive'?".into(),
            ground_truth: "3".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_factory_q08".into(),
            dataset: "hard_factory".into(),
            question_type: QuestionType::FieldRetrieval,
            domain: "real_datasets".into(),
            prompt: "What is the status of equipment eq003?".into(),
            ground_truth: "maintenance".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::String,
            notes: None,
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_factory_q09".into(),
            dataset: "hard_factory".into(),
            question_type: QuestionType::StructureAwareness,
            domain: "real_datasets".into(),
            prompt: "How many sensors are attached to equipment eq001?".into(),
            ground_truth: "2".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: Some("Count nested children".into()),
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
        Question {
            id: "hard_factory_q10".into(),
            dataset: "hard_factory".into(),
            question_type: QuestionType::Aggregation,
            domain: "real_datasets".into(),
            prompt: "How many zones are there?".into(),
            ground_truth: "3".into(),
            ground_truth_by_format: None,
            answer_type: AnswerType::Integer,
            notes: None,
            complexity_level: 4,
            tags: Vec::new(),
            blind_mode: true,
        },
    ];

    FixtureDataset::from_hedl("hard_factory", hedl, questions, ComplexityLevel::L4Advanced)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_datasets_generate() {
        let datasets = generate_real_datasets();
        assert_eq!(datasets.len(), 6, "Should have 6 real datasets");
    }

    #[test]
    fn test_easy_datasets_have_questions() {
        let datasets = generate_real_datasets();
        for ds in datasets
            .iter()
            .filter(|d| d.complexity == ComplexityLevel::L2Basic)
        {
            assert!(
                !ds.questions.is_empty(),
                "Easy dataset {} should have questions",
                ds.name
            );
            assert!(
                ds.questions.len() >= 5,
                "Easy dataset {} should have at least 5 questions",
                ds.name
            );
        }
    }

    #[test]
    fn test_hard_datasets_have_questions() {
        let datasets = generate_real_datasets();
        for ds in datasets
            .iter()
            .filter(|d| d.complexity == ComplexityLevel::L4Advanced)
        {
            assert!(
                !ds.questions.is_empty(),
                "Hard dataset {} should have questions",
                ds.name
            );
            assert!(
                ds.questions.len() >= 8,
                "Hard dataset {} should have at least 8 questions",
                ds.name
            );
        }
    }

    #[test]
    fn test_all_formats_generated() {
        let datasets = generate_real_datasets();
        for ds in &datasets {
            assert!(
                !ds.hedl_data.is_empty(),
                "HEDL should not be empty for {}",
                ds.name
            );
            assert!(
                ds.json_data.as_ref().is_some_and(|s| !s.is_empty()),
                "JSON should not be empty for {}",
                ds.name
            );
            assert!(
                ds.yaml_data.as_ref().is_some_and(|s| !s.is_empty()),
                "YAML should not be empty for {}",
                ds.name
            );
            assert!(
                ds.toon_data.as_ref().is_some_and(|s| !s.is_empty()),
                "TOON should not be empty for {}",
                ds.name
            );
        }
    }
}
