// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! 5-Level Progressive Complexity System
//!
//! Matches and exceeds TOON's complexity progression with clearer definitions.

/// Complexity level for questions and datasets (L1-L5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ComplexityLevel {
    /// L1: Single field extraction, flat data, 5-10 entities
    L1Trivial = 1,
    /// L2: Multi-field extraction, simple aggregation, 20-50 entities
    L2Basic = 2,
    /// L3: Conditional filtering, some nesting, 50-100 entities
    L3Intermediate = 3,
    /// L4: Cross-record aggregation, deep nesting, 100-500 entities
    L4Advanced = 4,
    /// L5: Complex queries, references, calculations, 500+ entities
    L5Expert = 5,
}

impl ComplexityLevel {
    /// All complexity levels for iteration
    pub const ALL: [ComplexityLevel; 5] = [
        ComplexityLevel::L1Trivial,
        ComplexityLevel::L2Basic,
        ComplexityLevel::L3Intermediate,
        ComplexityLevel::L4Advanced,
        ComplexityLevel::L5Expert,
    ];

    /// Create from u8 value
    #[must_use]
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(ComplexityLevel::L1Trivial),
            2 => Some(ComplexityLevel::L2Basic),
            3 => Some(ComplexityLevel::L3Intermediate),
            4 => Some(ComplexityLevel::L4Advanced),
            5 => Some(ComplexityLevel::L5Expert),
            _ => None,
        }
    }

    /// Human-readable name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            ComplexityLevel::L1Trivial => "L1: Trivial",
            ComplexityLevel::L2Basic => "L2: Basic",
            ComplexityLevel::L3Intermediate => "L3: Intermediate",
            ComplexityLevel::L4Advanced => "L4: Advanced",
            ComplexityLevel::L5Expert => "L5: Expert",
        }
    }

    /// Short code (L1-L5)
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            ComplexityLevel::L1Trivial => "L1",
            ComplexityLevel::L2Basic => "L2",
            ComplexityLevel::L3Intermediate => "L3",
            ComplexityLevel::L4Advanced => "L4",
            ComplexityLevel::L5Expert => "L5",
        }
    }

    /// Detailed description
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            ComplexityLevel::L1Trivial => {
                "Single field extraction from flat structures with 5-10 entities. \
                 No nesting, no references, no calculations."
            }
            ComplexityLevel::L2Basic => {
                "Multi-field extraction, simple counting/averaging across 20-50 entities. \
                 Minimal nesting (1 level), basic string matching."
            }
            ComplexityLevel::L3Intermediate => {
                "Conditional filtering with AND/OR logic across 50-100 entities. \
                 Moderate nesting (2-3 levels), date filtering, basic aggregation."
            }
            ComplexityLevel::L4Advanced => {
                "Cross-record aggregation, deep nesting (4+ levels), 100-500 entities. \
                 Reference resolution, temporal queries, pattern matching."
            }
            ComplexityLevel::L5Expert => {
                "Complex multi-step queries, graph traversal, calculations, 500+ entities. \
                 Requires understanding format-specific features, edge case handling."
            }
        }
    }

    /// Expected entity count range
    #[must_use]
    pub fn entity_range(&self) -> (usize, usize) {
        match self {
            ComplexityLevel::L1Trivial => (5, 10),
            ComplexityLevel::L2Basic => (20, 50),
            ComplexityLevel::L3Intermediate => (50, 100),
            ComplexityLevel::L4Advanced => (100, 500),
            ComplexityLevel::L5Expert => (500, 5000),
        }
    }

    /// Expected nesting depth range
    #[must_use]
    pub fn nesting_depth(&self) -> (usize, usize) {
        match self {
            ComplexityLevel::L1Trivial => (0, 0),
            ComplexityLevel::L2Basic => (0, 1),
            ComplexityLevel::L3Intermediate => (2, 0),
            ComplexityLevel::L4Advanced => (3, 5),
            ComplexityLevel::L5Expert => (4, 10),
        }
    }

    /// Numeric value (1-5)
    #[must_use]
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

impl std::fmt::Display for ComplexityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Complexity profile for a dataset or question set
#[derive(Debug, Clone)]
pub struct ComplexityProfile {
    /// Primary complexity level
    pub level: ComplexityLevel,
    /// Entity count in the dataset
    pub entity_count: usize,
    /// Maximum nesting depth
    pub max_nesting: usize,
    /// Number of entity types/schemas
    pub type_count: usize,
    /// Whether dataset has cross-references
    pub has_references: bool,
    /// Whether dataset has temporal data
    pub has_temporal: bool,
    /// Whether dataset has tensor/array data
    pub has_tensors: bool,
    /// Whether dataset has sparse/null values
    pub has_sparse_data: bool,
    /// Cognitive load score (1-100)
    pub cognitive_load: u8,
}

impl ComplexityProfile {
    /// Create a new complexity profile
    #[must_use]
    pub fn new(level: ComplexityLevel) -> Self {
        let (min_entities, max_entities) = level.entity_range();
        let (_, max_nesting) = level.nesting_depth();

        Self {
            level,
            entity_count: (min_entities + max_entities) / 2,
            max_nesting,
            type_count: level.value() as usize + 1,
            has_references: level.value() >= 3,
            has_temporal: level.value() >= 2,
            has_tensors: level.value() >= 3,
            has_sparse_data: level.value() >= 2,
            cognitive_load: level.value() * 20,
        }
    }

    /// Create profile with specific entity count
    #[must_use]
    pub fn with_entities(mut self, count: usize) -> Self {
        self.entity_count = count;
        self.recalculate_cognitive_load();
        self
    }

    /// Create profile with specific nesting depth
    #[must_use]
    pub fn with_nesting(mut self, depth: usize) -> Self {
        self.max_nesting = depth;
        self.recalculate_cognitive_load();
        self
    }

    /// Enable references
    #[must_use]
    pub fn with_references(mut self) -> Self {
        self.has_references = true;
        self.recalculate_cognitive_load();
        self
    }

    /// Enable temporal data
    #[must_use]
    pub fn with_temporal(mut self) -> Self {
        self.has_temporal = true;
        self.recalculate_cognitive_load();
        self
    }

    /// Enable tensors
    #[must_use]
    pub fn with_tensors(mut self) -> Self {
        self.has_tensors = true;
        self.recalculate_cognitive_load();
        self
    }

    /// Enable sparse data
    #[must_use]
    pub fn with_sparse(mut self) -> Self {
        self.has_sparse_data = true;
        self.recalculate_cognitive_load();
        self
    }

    /// Recalculate cognitive load based on features
    fn recalculate_cognitive_load(&mut self) {
        let mut load = self.level.value() * 15;

        // Entity count factor
        load = load.saturating_add(match self.entity_count {
            0..=10 => 5,
            11..=50 => 10,
            51..=100 => 15,
            101..=500 => 20,
            _ => 25,
        });

        // Nesting factor
        load = load.saturating_add((self.max_nesting as u8).saturating_mul(5));

        // Feature factors
        if self.has_references {
            load = load.saturating_add(10);
        }
        if self.has_temporal {
            load = load.saturating_add(5);
        }
        if self.has_tensors {
            load = load.saturating_add(8);
        }
        if self.has_sparse_data {
            load = load.saturating_add(5);
        }

        self.cognitive_load = load.min(100);
    }

    /// Validate that profile matches declared level
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let (min_entities, max_entities) = self.level.entity_range();
        let (_, max_nesting) = self.level.nesting_depth();

        if self.entity_count < min_entities {
            issues.push(format!(
                "Entity count {} below minimum {} for {}",
                self.entity_count,
                min_entities,
                self.level.code()
            ));
        }

        if self.entity_count > max_entities * 2 {
            issues.push(format!(
                "Entity count {} significantly exceeds maximum {} for {}",
                self.entity_count,
                max_entities,
                self.level.code()
            ));
        }

        if self.max_nesting > max_nesting + 2 {
            issues.push(format!(
                "Nesting depth {} exceeds expected {} for {}",
                self.max_nesting,
                max_nesting,
                self.level.code()
            ));
        }

        issues
    }

    /// Suggest the appropriate complexity level based on features
    #[must_use]
    pub fn suggest_level(&self) -> ComplexityLevel {
        // Score based on features
        let mut score = 0;

        // Entity count scoring
        score += match self.entity_count {
            0..=10 => 1,
            11..=50 => 2,
            51..=100 => 3,
            101..=500 => 4,
            _ => 5,
        };

        // Nesting scoring
        score += match self.max_nesting {
            0 => 0,
            1 => 1,
            2..=3 => 2,
            4..=5 => 3,
            _ => 4,
        };

        // Feature scoring
        if self.has_references {
            score += 2;
        }
        if self.has_tensors {
            score += 1;
        }
        if self.has_sparse_data {
            score += 1;
        }

        // Map to level
        match score {
            0..=2 => ComplexityLevel::L1Trivial,
            3..=5 => ComplexityLevel::L2Basic,
            6..=8 => ComplexityLevel::L3Intermediate,
            9..=11 => ComplexityLevel::L4Advanced,
            _ => ComplexityLevel::L5Expert,
        }
    }
}

impl Default for ComplexityProfile {
    fn default() -> Self {
        Self::new(ComplexityLevel::L3Intermediate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_levels() {
        assert_eq!(ComplexityLevel::L1Trivial.value(), 1);
        assert_eq!(ComplexityLevel::L5Expert.value(), 5);
        assert_eq!(ComplexityLevel::ALL.len(), 5);
    }

    #[test]
    fn test_complexity_ordering() {
        assert!(ComplexityLevel::L1Trivial < ComplexityLevel::L2Basic);
        assert!(ComplexityLevel::L4Advanced < ComplexityLevel::L5Expert);
    }

    #[test]
    fn test_profile_suggestion() {
        let profile = ComplexityProfile {
            level: ComplexityLevel::L1Trivial,
            entity_count: 500,
            max_nesting: 5,
            type_count: 10,
            has_references: true,
            has_temporal: true,
            has_tensors: true,
            has_sparse_data: true,
            cognitive_load: 0,
        };

        let suggested = profile.suggest_level();
        assert!(suggested >= ComplexityLevel::L4Advanced);
    }

    #[test]
    fn test_entity_ranges() {
        for level in ComplexityLevel::ALL {
            let (min, max) = level.entity_range();
            assert!(min <= max);
        }
    }

    #[test]
    fn test_cognitive_load_calculation() {
        let simple = ComplexityProfile::new(ComplexityLevel::L1Trivial);
        let complex = ComplexityProfile::new(ComplexityLevel::L5Expert)
            .with_references()
            .with_tensors()
            .with_entities(1000);

        assert!(simple.cognitive_load < complex.cognitive_load);
    }
}
