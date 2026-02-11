// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Anchor caching and cycle detection infrastructure

use hedl_core::Item;

/// Internal representation for items that may be shared via reference counting
/// or owned independently.
#[derive(Clone)]
pub(crate) enum ItemOrRef {
    /// Owned item (default case, no aliasing)
    Owned(Item),
}

impl ItemOrRef {
    /// Convert to owned Item
    pub(crate) fn into_item(self) -> Item {
        match self {
            ItemOrRef::Owned(item) => item,
        }
    }
}

/// Cache for resolved YAML anchors to avoid redundant processing.
/// Lives for the duration of a single `from_yaml()` call.
///
/// Currently a placeholder -- serde_yaml resolves anchors before we see the
/// values, so there is nothing to cache yet. The type is kept so the
/// conversion pipeline can be extended with a different YAML backend later.
pub(crate) struct AnchorCache;

impl AnchorCache {
    pub(crate) fn new() -> Self {
        Self
    }
}

/// Detects and prevents circular alias references.
///
/// Currently a placeholder -- cycle detection happens during the
/// pre-parse scanning phase in `anchors.rs`. The type is kept so the
/// conversion pipeline can be extended with a different YAML backend later.
pub(crate) struct CycleDetector;

impl CycleDetector {
    pub(crate) fn new() -> Self {
        Self
    }
}
