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

//! Generator configuration and complexity levels.
//!
//! Provides configuration structures and enums for controlling document
//! generation parameters across all generator types.

use crate::sizes;

/// Complexity levels for benchmark documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityLevel {
    /// Flat tabular data (users, products, events, analytics).
    Flat,
    /// Moderate nesting (blog posts with comments, orders with items).
    ModerateNesting,
    /// Heavy use of ditto markers for repeated values.
    DittoHeavy,
    /// Cross-references and graph structures.
    ReferenceHeavy,
    /// Deep hierarchical nesting (5+ levels).
    DeepHierarchy,
}

impl ComplexityLevel {
    /// Returns a human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Flat => "Flat",
            Self::ModerateNesting => "Moderate Nesting",
            Self::DittoHeavy => "Ditto-Heavy",
            Self::ReferenceHeavy => "Reference-Heavy",
            Self::DeepHierarchy => "Deep Hierarchy",
        }
    }

    /// Returns a short identifier.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::ModerateNesting => "nested",
            Self::DittoHeavy => "ditto",
            Self::ReferenceHeavy => "refs",
            Self::DeepHierarchy => "deep",
        }
    }

    /// Returns all complexity levels.
    pub fn all() -> &'static [ComplexityLevel] {
        &[
            Self::Flat,
            Self::ModerateNesting,
            Self::DittoHeavy,
            Self::ReferenceHeavy,
            Self::DeepHierarchy,
        ]
    }
}

/// Size distribution for generated data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeDistribution {
    /// Uniform size across all entities.
    Uniform,
    /// Small variation in sizes.
    LowVariance,
    /// Large variation in sizes.
    HighVariance,
}

/// Generator configuration with builder pattern.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Number of primary entities.
    pub count: usize,
    /// Size distribution pattern.
    pub size_distribution: SizeDistribution,
    /// Complexity level.
    pub complexity: ComplexityLevel,
    /// Include cross-references.
    pub include_references: bool,
    /// Nesting depth for hierarchical structures.
    pub depth: usize,
    /// Width/breadth parameter (edges, comments, etc).
    pub width: usize,
}

impl GeneratorConfig {
    /// Creates a new configuration with default values.
    pub fn new(count: usize) -> Self {
        Self {
            count,
            size_distribution: SizeDistribution::Uniform,
            complexity: ComplexityLevel::Flat,
            include_references: false,
            depth: 3,
            width: 3,
        }
    }

    /// Sets the size distribution.
    pub fn with_size_distribution(mut self, dist: SizeDistribution) -> Self {
        self.size_distribution = dist;
        self
    }

    /// Sets the complexity level.
    pub fn with_complexity(mut self, complexity: ComplexityLevel) -> Self {
        self.complexity = complexity;
        self
    }

    /// Sets whether to include references.
    pub fn with_references(mut self, include: bool) -> Self {
        self.include_references = include;
        self
    }

    /// Sets the depth parameter.
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Sets the width parameter.
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Creates a small dataset configuration.
    pub fn small() -> Self {
        Self::new(sizes::SMALL)
    }

    /// Creates a medium dataset configuration.
    pub fn medium() -> Self {
        Self::new(sizes::MEDIUM)
    }

    /// Creates a large dataset configuration.
    pub fn large() -> Self {
        Self::new(sizes::LARGE)
    }

    /// Creates a stress test configuration.
    pub fn stress() -> Self {
        Self::new(sizes::STRESS)
    }

    /// Creates an extreme test configuration.
    pub fn extreme() -> Self {
        Self::new(sizes::EXTREME)
    }
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self::new(sizes::MEDIUM)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_levels() {
        assert_eq!(ComplexityLevel::Flat.name(), "Flat");
        assert_eq!(ComplexityLevel::Flat.id(), "flat");
        assert_eq!(ComplexityLevel::all().len(), 5);
    }

    #[test]
    fn test_generator_config() {
        let config = GeneratorConfig::new(100)
            .with_complexity(ComplexityLevel::Flat)
            .with_depth(5)
            .with_width(3);

        assert_eq!(config.count, 100);
        assert_eq!(config.depth, 5);
        assert_eq!(config.width, 3);
    }

    #[test]
    fn test_size_presets() {
        let small = GeneratorConfig::small();
        assert_eq!(small.count, sizes::SMALL);

        let large = GeneratorConfig::large();
        assert_eq!(large.count, sizes::LARGE);
    }
}
