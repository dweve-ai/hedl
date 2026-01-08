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

//! Fixture management module.
//!
//! Provides utilities for loading, caching, and validating test fixtures.
//!
//! - **loader**: Fixture file loading with fallbacks
//! - **cache**: In-memory fixture caching
//! - **validator**: Fixture validation utilities

pub mod cache;
pub mod loader;
pub mod validator;

// Re-export commonly used types and functions
pub use cache::FixtureCache;
pub use loader::{list_fixtures, load_all_fixtures, load_fixture};
pub use validator::{validate_all_fixtures, validate_fixture};
