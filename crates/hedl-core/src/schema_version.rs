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

//! Schema versioning for HEDL types.
//!
//! This module provides version tracking for schemas, enabling schema evolution
//! and compatibility checking across different versions of HEDL documents.
//!
//! # Schema Evolution
//!
//! Schema versioning allows HEDL documents to specify which version of a schema
//! they conform to. This enables:
//!
//! - Forward compatibility: Older parsers can recognize newer schemas
//! - Backward compatibility: Newer parsers can read older documents
//! - Migration tooling: Automated schema migrations between versions
//!
//! # Version Format
//!
//! Versions follow semantic versioning: `major.minor.patch`
//!
//! - `major`: Breaking changes (field removals, type changes)
//! - `minor`: Backward-compatible additions (new optional fields)
//! - `patch`: Bug fixes to schema definitions

use std::fmt;

/// Schema version identifier.
///
/// Represents the version of a type's schema using semantic versioning.
///
/// # Examples
///
/// ```
/// use hedl_core::schema_version::SchemaVersion;
///
/// // Create a version
/// let v1 = SchemaVersion::new(1, 0, 0);
/// let v2 = SchemaVersion::new(1, 1, 0);
///
/// // Compare versions
/// assert!(v2 > v1);
/// assert!(v2.is_compatible_with(&v1));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaVersion {
    /// Major version (breaking changes)
    pub major: u32,
    /// Minor version (backward-compatible additions)
    pub minor: u32,
    /// Patch version (bug fixes)
    pub patch: u32,
}

impl SchemaVersion {
    /// Create a new schema version.
    ///
    /// # Arguments
    ///
    /// * `major` - Major version number
    /// * `minor` - Minor version number
    /// * `patch` - Patch version number
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::schema_version::SchemaVersion;
    ///
    /// let version = SchemaVersion::new(1, 2, 3);
    /// assert_eq!(version.major, 1);
    /// assert_eq!(version.minor, 2);
    /// assert_eq!(version.patch, 3);
    /// ```
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Create version 1.0.0 (common default).
    pub const fn v1() -> Self {
        Self::new(1, 0, 0)
    }

    /// Check if this version is compatible with another version.
    ///
    /// Compatibility rules:
    /// - Same major version required
    /// - This version's minor must be >= other's minor
    ///
    /// # Arguments
    ///
    /// * `other` - The version to check compatibility with
    ///
    /// # Returns
    ///
    /// `true` if this schema can read data written for `other`
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::schema_version::SchemaVersion;
    ///
    /// let v1_0 = SchemaVersion::new(1, 0, 0);
    /// let v1_1 = SchemaVersion::new(1, 1, 0);
    /// let v2_0 = SchemaVersion::new(2, 0, 0);
    ///
    /// // v1.1 can read v1.0 data (backward compatible)
    /// assert!(v1_1.is_compatible_with(&v1_0));
    ///
    /// // v1.0 cannot read v1.1 data (missing new fields)
    /// assert!(!v1_0.is_compatible_with(&v1_1));
    ///
    /// // Different major versions are not compatible
    /// assert!(!v2_0.is_compatible_with(&v1_0));
    /// ```
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // Same major version required
        if self.major != other.major {
            return false;
        }

        // This version must be >= other's minor version
        // (can read data with same or fewer features)
        self.minor >= other.minor
    }

    /// Check if this is a breaking change from another version.
    ///
    /// A breaking change occurs when the major version increases.
    pub fn is_breaking_from(&self, other: &Self) -> bool {
        self.major != other.major
    }

    /// Parse a version string in "major.minor.patch" format.
    ///
    /// # Arguments
    ///
    /// * `s` - Version string to parse
    ///
    /// # Returns
    ///
    /// `Some(SchemaVersion)` if parsing succeeds, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::schema_version::SchemaVersion;
    ///
    /// let v = SchemaVersion::parse("1.2.3").unwrap();
    /// assert_eq!(v, SchemaVersion::new(1, 2, 3));
    ///
    /// // Short forms also work
    /// let v = SchemaVersion::parse("1.2").unwrap();
    /// assert_eq!(v, SchemaVersion::new(1, 2, 0));
    ///
    /// let v = SchemaVersion::parse("1").unwrap();
    /// assert_eq!(v, SchemaVersion::new(1, 0, 0));
    /// ```
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.trim().split('.').collect();

        if parts.is_empty() || parts.len() > 3 {
            return None;
        }

        let major = parts.first()?.parse().ok()?;
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

        Some(Self::new(major, minor, patch))
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::v1()
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Parse a schema version from a string.
impl std::str::FromStr for SchemaVersion {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or("invalid version format")
    }
}

/// Field definition with optional default value.
///
/// Represents a field in a schema with its name, whether it's optional,
/// and an optional default value.
///
/// # Examples
///
/// ```
/// use hedl_core::schema_version::FieldDef;
/// use hedl_core::Value;
///
/// // Required field
/// let id_field = FieldDef {
///     name: "id".to_string(),
///     optional: false,
///     default: None,
/// };
///
/// // Optional field with default
/// let status_field = FieldDef {
///     name: "status".to_string(),
///     optional: true,
///     default: Some(Value::String("active".to_string().into())),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    /// The field name.
    pub name: String,
    /// Whether the field is optional.
    pub optional: bool,
    /// Default value if the field is omitted.
    pub default: Option<crate::Value>,
}

impl FieldDef {
    /// Create a new required field definition.
    ///
    /// # Arguments
    ///
    /// * `name` - The field name
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::schema_version::FieldDef;
    ///
    /// let field = FieldDef::required("id");
    /// assert!(!field.optional);
    /// assert!(field.default.is_none());
    /// ```
    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            optional: false,
            default: None,
        }
    }

    /// Create a new optional field definition.
    ///
    /// # Arguments
    ///
    /// * `name` - The field name
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::schema_version::FieldDef;
    ///
    /// let field = FieldDef::optional("description");
    /// assert!(field.optional);
    /// assert!(field.default.is_none());
    /// ```
    pub fn optional(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            optional: true,
            default: None,
        }
    }

    /// Create a new optional field with a default value.
    ///
    /// # Arguments
    ///
    /// * `name` - The field name
    /// * `default` - The default value
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::schema_version::FieldDef;
    /// use hedl_core::Value;
    ///
    /// let field = FieldDef::with_default("active", Value::Bool(true));
    /// assert!(field.optional);
    /// assert_eq!(field.default, Some(Value::Bool(true)));
    /// ```
    pub fn with_default(name: impl Into<String>, default: crate::Value) -> Self {
        Self {
            name: name.into(),
            optional: true,
            default: Some(default),
        }
    }
}

/// Schema definition with versioning.
///
/// Represents a complete schema for a type, including version information
/// and field definitions.
///
/// # Examples
///
/// ```
/// use hedl_core::schema_version::{Schema, SchemaVersion, FieldDef};
///
/// let mut schema = Schema::new(SchemaVersion::new(1, 0, 0));
/// schema.add_type("User", vec![
///     FieldDef::required("id"),
///     FieldDef::required("name"),
///     FieldDef::optional("email"),
/// ]);
/// ```
#[derive(Debug, Clone)]
pub struct Schema {
    /// Schema version.
    pub version: SchemaVersion,
    /// Type definitions: type name -> field definitions.
    pub types: std::collections::BTreeMap<String, Vec<FieldDef>>,
}

impl Schema {
    /// Create a new schema with the specified version.
    ///
    /// # Arguments
    ///
    /// * `version` - The schema version
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::schema_version::{Schema, SchemaVersion};
    ///
    /// let schema = Schema::new(SchemaVersion::new(1, 0, 0));
    /// assert_eq!(schema.version, SchemaVersion::new(1, 0, 0));
    /// assert!(schema.types.is_empty());
    /// ```
    pub fn new(version: SchemaVersion) -> Self {
        Self {
            version,
            types: std::collections::BTreeMap::new(),
        }
    }

    /// Add a type definition to the schema.
    ///
    /// # Arguments
    ///
    /// * `name` - The type name
    /// * `fields` - Field definitions for the type
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::schema_version::{Schema, SchemaVersion, FieldDef};
    ///
    /// let mut schema = Schema::new(SchemaVersion::new(1, 0, 0));
    /// schema.add_type("User", vec![
    ///     FieldDef::required("id"),
    ///     FieldDef::required("name"),
    /// ]);
    /// assert!(schema.types.contains_key("User"));
    /// ```
    pub fn add_type(&mut self, name: &str, fields: Vec<FieldDef>) {
        self.types.insert(name.to_string(), fields);
    }

    /// Get field definitions for a type.
    ///
    /// # Arguments
    ///
    /// * `type_name` - The type name to look up
    ///
    /// # Returns
    ///
    /// `Some(&Vec<FieldDef>)` if the type exists, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::schema_version::{Schema, SchemaVersion, FieldDef};
    ///
    /// let mut schema = Schema::new(SchemaVersion::new(1, 0, 0));
    /// schema.add_type("User", vec![FieldDef::required("id")]);
    ///
    /// assert!(schema.get_fields("User").is_some());
    /// assert!(schema.get_fields("Post").is_none());
    /// ```
    pub fn get_fields(&self, type_name: &str) -> Option<&Vec<FieldDef>> {
        self.types.get(type_name)
    }

    /// Check if this schema is compatible with another schema.
    ///
    /// Schemas are compatible if their versions are compatible.
    ///
    /// # Arguments
    ///
    /// * `other` - The schema to check compatibility with
    ///
    /// # Returns
    ///
    /// `true` if this schema can read data written for `other`
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::schema_version::{Schema, SchemaVersion};
    ///
    /// let v1 = Schema::new(SchemaVersion::new(1, 0, 0));
    /// let v1_1 = Schema::new(SchemaVersion::new(1, 1, 0));
    /// let v2 = Schema::new(SchemaVersion::new(2, 0, 0));
    ///
    /// assert!(v1_1.is_compatible_with(&v1));
    /// assert!(!v1.is_compatible_with(&v1_1));
    /// assert!(!v2.is_compatible_with(&v1));
    /// ```
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.version.is_compatible_with(&other.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Construction tests ====================

    #[test]
    fn test_new() {
        let v = SchemaVersion::new(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_v1() {
        let v = SchemaVersion::v1();
        assert_eq!(v, SchemaVersion::new(1, 0, 0));
    }

    #[test]
    fn test_default() {
        let v = SchemaVersion::default();
        assert_eq!(v, SchemaVersion::v1());
    }

    // ==================== Parsing tests ====================

    #[test]
    fn test_parse_full() {
        let v = SchemaVersion::parse("1.2.3").unwrap();
        assert_eq!(v, SchemaVersion::new(1, 2, 3));
    }

    #[test]
    fn test_parse_major_minor() {
        let v = SchemaVersion::parse("1.2").unwrap();
        assert_eq!(v, SchemaVersion::new(1, 2, 0));
    }

    #[test]
    fn test_parse_major_only() {
        let v = SchemaVersion::parse("1").unwrap();
        assert_eq!(v, SchemaVersion::new(1, 0, 0));
    }

    #[test]
    fn test_parse_with_whitespace() {
        let v = SchemaVersion::parse("  1.2.3  ").unwrap();
        assert_eq!(v, SchemaVersion::new(1, 2, 3));
    }

    #[test]
    fn test_parse_invalid_empty() {
        assert!(SchemaVersion::parse("").is_none());
    }

    #[test]
    fn test_parse_invalid_non_numeric() {
        assert!(SchemaVersion::parse("a.b.c").is_none());
    }

    #[test]
    fn test_parse_invalid_too_many_parts() {
        assert!(SchemaVersion::parse("1.2.3.4").is_none());
    }

    #[test]
    fn test_from_str() {
        let v: SchemaVersion = "1.2.3".parse().unwrap();
        assert_eq!(v, SchemaVersion::new(1, 2, 3));
    }

    #[test]
    fn test_from_str_invalid() {
        let result: Result<SchemaVersion, _> = "invalid".parse();
        assert!(result.is_err());
    }

    // ==================== Display tests ====================

    #[test]
    fn test_display() {
        let v = SchemaVersion::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_display_zeros() {
        let v = SchemaVersion::new(1, 0, 0);
        assert_eq!(v.to_string(), "1.0.0");
    }

    // ==================== Comparison tests ====================

    #[test]
    fn test_equality() {
        let v1 = SchemaVersion::new(1, 2, 3);
        let v2 = SchemaVersion::new(1, 2, 3);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_inequality() {
        let v1 = SchemaVersion::new(1, 2, 3);
        let v2 = SchemaVersion::new(1, 2, 4);
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_ordering_major() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(2, 0, 0);
        assert!(v2 > v1);
    }

    #[test]
    fn test_ordering_minor() {
        let v1 = SchemaVersion::new(1, 1, 0);
        let v2 = SchemaVersion::new(1, 2, 0);
        assert!(v2 > v1);
    }

    #[test]
    fn test_ordering_patch() {
        let v1 = SchemaVersion::new(1, 0, 1);
        let v2 = SchemaVersion::new(1, 0, 2);
        assert!(v2 > v1);
    }

    // ==================== Compatibility tests ====================

    #[test]
    fn test_compatible_same_version() {
        let v = SchemaVersion::new(1, 2, 3);
        assert!(v.is_compatible_with(&v));
    }

    #[test]
    fn test_compatible_higher_minor() {
        let v1_0 = SchemaVersion::new(1, 0, 0);
        let v1_1 = SchemaVersion::new(1, 1, 0);
        // v1.1 can read v1.0 data
        assert!(v1_1.is_compatible_with(&v1_0));
    }

    #[test]
    fn test_incompatible_lower_minor() {
        let v1_0 = SchemaVersion::new(1, 0, 0);
        let v1_1 = SchemaVersion::new(1, 1, 0);
        // v1.0 cannot read v1.1 data
        assert!(!v1_0.is_compatible_with(&v1_1));
    }

    #[test]
    fn test_incompatible_different_major() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(2, 0, 0);
        assert!(!v2.is_compatible_with(&v1));
        assert!(!v1.is_compatible_with(&v2));
    }

    #[test]
    fn test_compatible_different_patch() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(1, 0, 5);
        // Patch versions don't affect compatibility
        assert!(v1.is_compatible_with(&v2));
        assert!(v2.is_compatible_with(&v1));
    }

    // ==================== Breaking change tests ====================

    #[test]
    fn test_breaking_change_true() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(2, 0, 0);
        assert!(v2.is_breaking_from(&v1));
    }

    #[test]
    fn test_breaking_change_false_same_major() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(1, 5, 0);
        assert!(!v2.is_breaking_from(&v1));
    }

    #[test]
    fn test_breaking_change_false_same_version() {
        let v = SchemaVersion::new(1, 2, 3);
        assert!(!v.is_breaking_from(&v));
    }

    // ==================== Hash tests ====================

    #[test]
    fn test_hash_consistency() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(SchemaVersion::new(1, 0, 0));
        set.insert(SchemaVersion::new(1, 0, 0)); // duplicate

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_hash_different_versions() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(SchemaVersion::new(1, 0, 0));
        set.insert(SchemaVersion::new(1, 1, 0));
        set.insert(SchemaVersion::new(2, 0, 0));

        assert_eq!(set.len(), 3);
    }

    // ==================== Clone and Copy tests ====================

    #[test]
    fn test_clone() {
        let v1 = SchemaVersion::new(1, 2, 3);
        let v2 = v1;
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_copy() {
        let v1 = SchemaVersion::new(1, 2, 3);
        let v2 = v1; // Copy, not move
        assert_eq!(v1, v2);
    }

    // ==================== Debug tests ====================

    #[test]
    fn test_debug() {
        let v = SchemaVersion::new(1, 2, 3);
        let debug = format!("{:?}", v);
        assert!(debug.contains("SchemaVersion"));
        assert!(debug.contains("major: 1"));
        assert!(debug.contains("minor: 2"));
        assert!(debug.contains("patch: 3"));
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_max_values() {
        let v = SchemaVersion::new(u32::MAX, u32::MAX, u32::MAX);
        assert_eq!(v.major, u32::MAX);
        assert_eq!(v.minor, u32::MAX);
        assert_eq!(v.patch, u32::MAX);
    }

    #[test]
    fn test_zero_version() {
        let v = SchemaVersion::new(0, 0, 0);
        assert_eq!(v.to_string(), "0.0.0");
    }

    // ==================== FieldDef tests ====================

    #[test]
    fn test_field_def_required() {
        let field = FieldDef::required("id");
        assert_eq!(field.name, "id");
        assert!(!field.optional);
        assert!(field.default.is_none());
    }

    #[test]
    fn test_field_def_optional() {
        let field = FieldDef::optional("description");
        assert_eq!(field.name, "description");
        assert!(field.optional);
        assert!(field.default.is_none());
    }

    #[test]
    fn test_field_def_with_default() {
        use crate::Value;
        let field = FieldDef::with_default("active", Value::Bool(true));
        assert_eq!(field.name, "active");
        assert!(field.optional);
        assert_eq!(field.default, Some(Value::Bool(true)));
    }

    #[test]
    fn test_field_def_with_default_string() {
        use crate::Value;
        let field = FieldDef::with_default("status", Value::String("pending".to_string().into()));
        assert_eq!(field.name, "status");
        assert!(field.optional);
        assert_eq!(
            field.default,
            Some(Value::String("pending".to_string().into()))
        );
    }

    #[test]
    fn test_field_def_with_default_int() {
        use crate::Value;
        let field = FieldDef::with_default("count", Value::Int(0));
        assert_eq!(field.name, "count");
        assert_eq!(field.default, Some(Value::Int(0)));
    }

    #[test]
    fn test_field_def_equality() {
        let a = FieldDef::required("id");
        let b = FieldDef::required("id");
        assert_eq!(a, b);
    }

    #[test]
    fn test_field_def_inequality() {
        let a = FieldDef::required("id");
        let b = FieldDef::optional("id");
        assert_ne!(a, b);
    }

    #[test]
    fn test_field_def_clone() {
        use crate::Value;
        let original = FieldDef::with_default("test", Value::Int(42));
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_field_def_debug() {
        let field = FieldDef::required("id");
        let debug = format!("{:?}", field);
        assert!(debug.contains("FieldDef"));
        assert!(debug.contains("id"));
    }

    // ==================== Schema tests ====================

    #[test]
    fn test_schema_new() {
        let schema = Schema::new(SchemaVersion::new(1, 0, 0));
        assert_eq!(schema.version, SchemaVersion::new(1, 0, 0));
        assert!(schema.types.is_empty());
    }

    #[test]
    fn test_schema_add_type() {
        let mut schema = Schema::new(SchemaVersion::new(1, 0, 0));
        schema.add_type(
            "User",
            vec![FieldDef::required("id"), FieldDef::required("name")],
        );
        assert!(schema.types.contains_key("User"));
        assert_eq!(schema.types["User"].len(), 2);
    }

    #[test]
    fn test_schema_add_multiple_types() {
        let mut schema = Schema::new(SchemaVersion::new(1, 0, 0));
        schema.add_type("User", vec![FieldDef::required("id")]);
        schema.add_type("Post", vec![FieldDef::required("id")]);
        assert_eq!(schema.types.len(), 2);
        assert!(schema.types.contains_key("User"));
        assert!(schema.types.contains_key("Post"));
    }

    #[test]
    fn test_schema_get_fields() {
        let mut schema = Schema::new(SchemaVersion::new(1, 0, 0));
        schema.add_type(
            "User",
            vec![FieldDef::required("id"), FieldDef::optional("email")],
        );

        let fields = schema.get_fields("User").unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "id");
        assert_eq!(fields[1].name, "email");
    }

    #[test]
    fn test_schema_get_fields_missing() {
        let schema = Schema::new(SchemaVersion::new(1, 0, 0));
        assert!(schema.get_fields("MissingType").is_none());
    }

    #[test]
    fn test_schema_is_compatible_with() {
        let v1 = Schema::new(SchemaVersion::new(1, 0, 0));
        let v1_1 = Schema::new(SchemaVersion::new(1, 1, 0));
        let v2 = Schema::new(SchemaVersion::new(2, 0, 0));

        assert!(v1_1.is_compatible_with(&v1));
        assert!(!v1.is_compatible_with(&v1_1));
        assert!(!v2.is_compatible_with(&v1));
    }

    #[test]
    fn test_schema_replace_type() {
        let mut schema = Schema::new(SchemaVersion::new(1, 0, 0));
        schema.add_type("User", vec![FieldDef::required("id")]);
        schema.add_type(
            "User",
            vec![FieldDef::required("id"), FieldDef::required("name")],
        );

        let fields = schema.get_fields("User").unwrap();
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn test_schema_clone() {
        let mut schema = Schema::new(SchemaVersion::new(1, 0, 0));
        schema.add_type("User", vec![FieldDef::required("id")]);
        let cloned = schema.clone();
        assert_eq!(schema.version, cloned.version);
        assert_eq!(schema.types.len(), cloned.types.len());
    }

    #[test]
    fn test_schema_debug() {
        let schema = Schema::new(SchemaVersion::new(1, 0, 0));
        let debug = format!("{:?}", schema);
        assert!(debug.contains("Schema"));
        assert!(debug.contains("version"));
    }

    #[test]
    fn test_schema_with_optional_and_default_fields() {
        use crate::Value;
        let mut schema = Schema::new(SchemaVersion::new(1, 0, 0));
        schema.add_type(
            "User",
            vec![
                FieldDef::required("id"),
                FieldDef::optional("name"),
                FieldDef::with_default("active", Value::Bool(true)),
                FieldDef::with_default("role", Value::String("user".to_string().into())),
            ],
        );

        let fields = schema.get_fields("User").unwrap();
        assert_eq!(fields.len(), 4);
        assert!(!fields[0].optional); // id is required
        assert!(fields[1].optional); // name is optional
        assert!(fields[2].optional); // active has default
        assert!(fields[3].optional); // role has default
        assert!(fields[2].default.is_some());
        assert!(fields[3].default.is_some());
    }

    // ==================== Integration tests ====================

    #[test]
    fn test_schema_evolution_scenario() {
        use crate::Value;

        // Version 1.0.0: Initial schema
        let mut v1 = Schema::new(SchemaVersion::new(1, 0, 0));
        v1.add_type(
            "User",
            vec![FieldDef::required("id"), FieldDef::required("name")],
        );

        // Version 1.1.0: Add optional field (backward compatible)
        let mut v1_1 = Schema::new(SchemaVersion::new(1, 1, 0));
        v1_1.add_type(
            "User",
            vec![
                FieldDef::required("id"),
                FieldDef::required("name"),
                FieldDef::optional("email"),
            ],
        );

        // Version 1.2.0: Add field with default (backward compatible)
        let mut v1_2 = Schema::new(SchemaVersion::new(1, 2, 0));
        v1_2.add_type(
            "User",
            vec![
                FieldDef::required("id"),
                FieldDef::required("name"),
                FieldDef::optional("email"),
                FieldDef::with_default("active", Value::Bool(true)),
            ],
        );

        // Verify compatibility
        assert!(v1_1.is_compatible_with(&v1)); // v1.1 can read v1.0 data
        assert!(v1_2.is_compatible_with(&v1_1)); // v1.2 can read v1.1 data
        assert!(v1_2.is_compatible_with(&v1)); // v1.2 can read v1.0 data

        // Older versions cannot read newer data
        assert!(!v1.is_compatible_with(&v1_1));
        assert!(!v1_1.is_compatible_with(&v1_2));
    }

    #[test]
    fn test_breaking_schema_change() {
        // Version 1.0.0
        let mut v1 = Schema::new(SchemaVersion::new(1, 0, 0));
        v1.add_type("User", vec![FieldDef::required("id")]);

        // Version 2.0.0: Breaking change
        let mut v2 = Schema::new(SchemaVersion::new(2, 0, 0));
        v2.add_type(
            "User",
            vec![
                FieldDef::required("user_id"), // Changed field name
                FieldDef::required("name"),
            ],
        );

        // Breaking changes are not compatible
        assert!(!v2.is_compatible_with(&v1));
        assert!(!v1.is_compatible_with(&v2));
    }

    #[test]
    fn test_field_def_with_null_default() {
        use crate::Value;
        let field = FieldDef::with_default("optional_value", Value::Null);
        assert_eq!(field.default, Some(Value::Null));
    }

    #[test]
    fn test_empty_schema() {
        let schema = Schema::new(SchemaVersion::new(0, 0, 1));
        assert!(schema.types.is_empty());
        assert!(schema.get_fields("AnyType").is_none());
    }

    #[test]
    fn test_schema_with_empty_field_list() {
        let mut schema = Schema::new(SchemaVersion::new(1, 0, 0));
        schema.add_type("EmptyType", vec![]);
        let fields = schema.get_fields("EmptyType").unwrap();
        assert!(fields.is_empty());
    }
}
