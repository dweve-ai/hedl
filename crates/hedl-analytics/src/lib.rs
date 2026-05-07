// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Analytics and tracking for the HEDL ecosystem.
//!
//! Provides system-wide token usage tracking, performance metrics,
//! and savings analysis across all HEDL operations.

pub mod metrics;
pub mod storage;
pub mod tracker;

pub use metrics::{CommandMetrics, FormatMetrics, TokenMetrics};
pub use storage::AnalyticsStorage;
pub use tracker::{track_command, track_format_conversion, get_analytics_report, AnalyticsReport};
