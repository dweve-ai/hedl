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

//! Canonicalization configuration.
//!
//! This module defines configuration options for controlling HEDL canonical output format.

/// Quoting strategy for string values.
///
/// Controls when string values are enclosed in double quotes in the output.
/// This affects both key-value context and matrix cell context.
///
/// # Examples
///
/// ```
/// use hedl_c14n::{QuotingStrategy, CanonicalConfig};
///
/// // Minimal quoting (default) - quotes only when necessary
/// let config = CanonicalConfig::new().with_quoting(QuotingStrategy::Minimal);
///
/// // Always quote all strings
/// let config = CanonicalConfig::new().with_quoting(QuotingStrategy::Always);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum QuotingStrategy {
    /// Only quote when necessary to prevent ambiguity.
    ///
    /// Quotes are added when strings:
    /// - Are empty
    /// - Have leading/trailing whitespace
    /// - Contain special characters (`#`, `"`, `,`, `|`, etc.)
    /// - Would be parsed as other types (`true`, `false`, numbers, etc.)
    /// - Start with type prefixes (`~`, `@`, `$`, `%`, `^`, `[`)
    ///
    /// This produces the most compact output while maintaining correctness.
    #[default]
    Minimal,

    /// Quote all string values unconditionally.
    ///
    /// Produces more verbose output but makes string types immediately visible.
    /// Useful for debugging or when working with parsers that expect quoted strings.
    Always,
}

/// Configuration for canonical output format.
///
/// Controls various aspects of how HEDL documents are serialized to canonical form.
/// All options follow SPEC.md Section 13.2 requirements while allowing customization.
///
/// # Examples
///
/// ```
/// use hedl_c14n::{CanonicalConfig, QuotingStrategy};
///
/// // Default configuration (SPEC-compliant canonical form)
/// let config = CanonicalConfig::default();
/// assert_eq!(config.quoting, QuotingStrategy::Minimal);
/// assert!(config.use_ditto);
/// assert!(config.sort_keys);
/// assert!(!config.inline_schemas);
///
/// // Custom configuration for compact output
/// let config = CanonicalConfig::new()
///     .with_quoting(QuotingStrategy::Always)
///     .with_ditto(false)
///     .with_sort_keys(true)
///     .with_inline_schemas(true);
/// ```
///
/// # Field Descriptions
///
/// - `quoting`: Controls when strings are quoted (minimal or always)
/// - `use_ditto`: Enables `^` ditto markers for repeated values in matrix rows
/// - `sort_keys`: Sort object keys alphabetically (note: BTreeMap is already sorted)
/// - `inline_schemas`: Use inline schemas `@Type[cols]` vs header `%STRUCT` directives
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CanonicalConfig {
    /// Quoting strategy for string values.
    ///
    /// Determines when double quotes are added around string values.
    /// Default: [`QuotingStrategy::Minimal`]
    pub quoting: QuotingStrategy,

    /// Use ditto optimization in matrix rows.
    ///
    /// When enabled, repeated values in consecutive rows use `^` marker instead
    /// of the full value. Never applied to first column (ID) or first row.
    ///
    /// Example with ditto enabled:
    /// ```text
    /// |alice,engineer,NYC
    /// |bob,^,^           # engineer,NYC repeated
    /// ```
    ///
    /// Default: `true`
    pub use_ditto: bool,

    /// Sort object keys alphabetically.
    ///
    /// Note: This field is currently redundant since BTreeMap inherently maintains
    /// sorted order. Kept for API compatibility and future HashMap support.
    ///
    /// Default: `true`
    pub sort_keys: bool,

    /// Use inline schemas instead of header %STRUCT directives.
    ///
    /// When `false` (canonical form per SPEC.md Section 13.2):
    /// ```text
    /// %STRUCT: User: [id,name,role]
    /// ---
    /// users: @User
    /// ```
    ///
    /// When `true` (inline schemas):
    /// ```text
    /// ---
    /// users: @User[id,name,role]
    /// ```
    ///
    /// Default: `false` (use header directives for canonical form)
    pub inline_schemas: bool,
}

impl Default for CanonicalConfig {
    fn default() -> Self {
        Self {
            quoting: QuotingStrategy::Minimal,
            use_ditto: true,
            sort_keys: true,
            // Per SPEC Section 13.2: canonical form includes %STRUCT directives
            inline_schemas: false,
        }
    }
}

impl CanonicalConfig {
    /// Create a new configuration with all default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new builder for constructing a `CanonicalConfig`.
    ///
    /// The builder provides a chainable API for setting configuration options
    /// and has a `build()` method to finalize the configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_c14n::{CanonicalConfig, QuotingStrategy};
    ///
    /// let config = CanonicalConfig::builder()
    ///     .use_ditto(true)
    ///     .sort_keys(true)
    ///     .quoting(QuotingStrategy::Minimal)
    ///     .build();
    /// ```
    pub fn builder() -> CanonicalConfigBuilder {
        CanonicalConfigBuilder::new()
    }

    /// Set the quoting strategy.
    pub fn with_quoting(mut self, quoting: QuotingStrategy) -> Self {
        self.quoting = quoting;
        self
    }

    /// Set whether to use ditto optimization.
    pub fn with_ditto(mut self, use_ditto: bool) -> Self {
        self.use_ditto = use_ditto;
        self
    }

    /// Set whether to sort keys.
    pub fn with_sort_keys(mut self, sort_keys: bool) -> Self {
        self.sort_keys = sort_keys;
        self
    }

    /// Set whether to use inline schemas.
    pub fn with_inline_schemas(mut self, inline_schemas: bool) -> Self {
        self.inline_schemas = inline_schemas;
        self
    }
}

/// Builder for constructing a `CanonicalConfig` with a chainable API.
///
/// This builder provides an ergonomic way to construct `CanonicalConfig` instances
/// with custom settings. All configuration options are optional and default to
/// the standard canonical form per SPEC.md Section 13.2.
///
/// # Examples
///
/// ```
/// use hedl_c14n::{CanonicalConfig, QuotingStrategy};
///
/// // Simple builder usage
/// let config = CanonicalConfig::builder()
///     .use_ditto(true)
///     .sort_keys(true)
///     .build();
///
/// // Builder with all options
/// let config = CanonicalConfig::builder()
///     .quoting(QuotingStrategy::Always)
///     .use_ditto(false)
///     .sort_keys(true)
///     .inline_schemas(true)
///     .build();
///
/// // Default configuration (same as CanonicalConfig::default())
/// let config = CanonicalConfig::builder().build();
/// ```
#[derive(Debug, Clone)]
pub struct CanonicalConfigBuilder {
    quoting: QuotingStrategy,
    use_ditto: bool,
    sort_keys: bool,
    inline_schemas: bool,
}

impl Default for CanonicalConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonicalConfigBuilder {
    /// Create a new builder with default configuration values.
    ///
    /// All settings are initialized to their defaults (SPEC-compliant canonical form):
    /// - `quoting`: `Minimal`
    /// - `use_ditto`: `true`
    /// - `sort_keys`: `true`
    /// - `inline_schemas`: `false`
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_c14n::CanonicalConfig;
    ///
    /// let builder = CanonicalConfig::builder();
    /// let config = builder.build();
    /// assert_eq!(config, CanonicalConfig::default());
    /// ```
    pub fn new() -> Self {
        Self {
            quoting: QuotingStrategy::Minimal,
            use_ditto: true,
            sort_keys: true,
            inline_schemas: false,
        }
    }

    /// Set the quoting strategy for string values.
    ///
    /// # Arguments
    ///
    /// * `quoting` - The [`QuotingStrategy`] to use (Minimal or Always)
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_c14n::{CanonicalConfig, QuotingStrategy};
    ///
    /// let config = CanonicalConfig::builder()
    ///     .quoting(QuotingStrategy::Always)
    ///     .build();
    /// assert_eq!(config.quoting, QuotingStrategy::Always);
    /// ```
    pub fn quoting(mut self, quoting: QuotingStrategy) -> Self {
        self.quoting = quoting;
        self
    }

    /// Set whether to use ditto optimization in matrix rows.
    ///
    /// When enabled, repeated values in consecutive rows use `^` marker instead
    /// of the full value. Never applied to first column (ID) or first row.
    ///
    /// # Arguments
    ///
    /// * `use_ditto` - Whether to enable ditto optimization
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_c14n::CanonicalConfig;
    ///
    /// let config = CanonicalConfig::builder()
    ///     .use_ditto(false)
    ///     .build();
    /// assert!(!config.use_ditto);
    /// ```
    pub fn use_ditto(mut self, use_ditto: bool) -> Self {
        self.use_ditto = use_ditto;
        self
    }

    /// Set whether to sort object keys alphabetically.
    ///
    /// Note: This field is currently redundant since BTreeMap inherently maintains
    /// sorted order. Kept for API compatibility and future HashMap support.
    ///
    /// # Arguments
    ///
    /// * `sort_keys` - Whether to sort keys alphabetically
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_c14n::CanonicalConfig;
    ///
    /// let config = CanonicalConfig::builder()
    ///     .sort_keys(false)
    ///     .build();
    /// assert!(!config.sort_keys);
    /// ```
    pub fn sort_keys(mut self, sort_keys: bool) -> Self {
        self.sort_keys = sort_keys;
        self
    }

    /// Set whether to use inline schemas instead of header %STRUCT directives.
    ///
    /// When `false` (canonical form per SPEC.md Section 13.2):
    /// ```text
    /// %STRUCT: User: [id,name,role]
    /// ---
    /// users: @User
    /// ```
    ///
    /// When `true` (inline schemas):
    /// ```text
    /// ---
    /// users: @User[id,name,role]
    /// ```
    ///
    /// # Arguments
    ///
    /// * `inline_schemas` - Whether to use inline schemas
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_c14n::CanonicalConfig;
    ///
    /// let config = CanonicalConfig::builder()
    ///     .inline_schemas(true)
    ///     .build();
    /// assert!(config.inline_schemas);
    /// ```
    pub fn inline_schemas(mut self, inline_schemas: bool) -> Self {
        self.inline_schemas = inline_schemas;
        self
    }

    /// Build the `CanonicalConfig` from this builder.
    ///
    /// Consumes the builder and returns a configured `CanonicalConfig` instance
    /// with all specified settings applied.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_c14n::{CanonicalConfig, QuotingStrategy};
    ///
    /// let config = CanonicalConfig::builder()
    ///     .quoting(QuotingStrategy::Always)
    ///     .use_ditto(false)
    ///     .build();
    ///
    /// assert_eq!(config.quoting, QuotingStrategy::Always);
    /// assert!(!config.use_ditto);
    /// assert!(config.sort_keys);        // Still default
    /// assert!(!config.inline_schemas);  // Still default
    /// ```
    pub fn build(self) -> CanonicalConfig {
        CanonicalConfig {
            quoting: self.quoting,
            use_ditto: self.use_ditto,
            sort_keys: self.sort_keys,
            inline_schemas: self.inline_schemas,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== QuotingStrategy tests ====================

    #[test]
    fn test_quoting_strategy_default() {
        let strategy = QuotingStrategy::default();
        assert_eq!(strategy, QuotingStrategy::Minimal);
    }

    #[test]
    fn test_quoting_strategy_debug() {
        let minimal = QuotingStrategy::Minimal;
        let always = QuotingStrategy::Always;
        assert!(format!("{:?}", minimal).contains("Minimal"));
        assert!(format!("{:?}", always).contains("Always"));
    }

    #[test]
    fn test_quoting_strategy_copy_always() {
        let strategy = QuotingStrategy::Always;
        let copied = strategy; // Copy, not clone
        assert_eq!(strategy, copied);
    }

    #[test]
    fn test_quoting_strategy_copy_minimal() {
        let strategy = QuotingStrategy::Minimal;
        let copied: QuotingStrategy = strategy;
        assert_eq!(strategy, copied);
    }

    #[test]
    fn test_quoting_strategy_eq() {
        assert_eq!(QuotingStrategy::Minimal, QuotingStrategy::Minimal);
        assert_eq!(QuotingStrategy::Always, QuotingStrategy::Always);
        assert_ne!(QuotingStrategy::Minimal, QuotingStrategy::Always);
    }

    // ==================== CanonicalConfig tests ====================

    #[test]
    fn test_canonical_config_default() {
        let config = CanonicalConfig::default();
        assert_eq!(config.quoting, QuotingStrategy::Minimal);
        assert!(config.use_ditto);
        assert!(config.sort_keys);
        assert!(!config.inline_schemas);
    }

    #[test]
    fn test_canonical_config_debug() {
        let config = CanonicalConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("CanonicalConfig"));
        assert!(debug.contains("quoting"));
        assert!(debug.contains("use_ditto"));
        assert!(debug.contains("sort_keys"));
        assert!(debug.contains("inline_schemas"));
    }

    #[test]
    fn test_canonical_config_clone() {
        let config = CanonicalConfig {
            quoting: QuotingStrategy::Always,
            use_ditto: false,
            sort_keys: false,
            inline_schemas: true,
        };
        let cloned = config.clone();
        assert_eq!(cloned.quoting, QuotingStrategy::Always);
        assert!(!cloned.use_ditto);
        assert!(!cloned.sort_keys);
        assert!(cloned.inline_schemas);
    }

    #[test]
    fn test_canonical_config_custom() {
        let config = CanonicalConfig {
            quoting: QuotingStrategy::Always,
            use_ditto: false,
            sort_keys: true,
            inline_schemas: true,
        };
        assert_eq!(config.quoting, QuotingStrategy::Always);
        assert!(!config.use_ditto);
        assert!(config.sort_keys);
        assert!(config.inline_schemas);
    }

    #[test]
    fn test_canonical_config_all_false() {
        let config = CanonicalConfig {
            quoting: QuotingStrategy::Minimal,
            use_ditto: false,
            sort_keys: false,
            inline_schemas: false,
        };
        assert!(!config.use_ditto);
        assert!(!config.sort_keys);
        assert!(!config.inline_schemas);
    }

    #[test]
    fn test_canonical_config_all_true() {
        let config = CanonicalConfig {
            quoting: QuotingStrategy::Always,
            use_ditto: true,
            sort_keys: true,
            inline_schemas: true,
        };
        assert!(config.use_ditto);
        assert!(config.sort_keys);
        assert!(config.inline_schemas);
    }

    #[test]
    fn test_canonical_config_field_independence() {
        // Changing one field shouldn't affect others
        let config = CanonicalConfig {
            use_ditto: false,
            ..Default::default()
        };
        assert!(config.sort_keys); // Still true
        assert!(!config.inline_schemas); // Still false
    }

    // ==================== CanonicalConfigBuilder tests ====================

    #[test]
    fn test_builder_default() {
        let builder = CanonicalConfigBuilder::new();
        let config = builder.build();
        assert_eq!(config.quoting, QuotingStrategy::Minimal);
        assert!(config.use_ditto);
        assert!(config.sort_keys);
        assert!(!config.inline_schemas);
    }

    #[test]
    fn test_builder_via_canonical_config_builder_method() {
        let config = CanonicalConfig::builder().build();
        assert_eq!(config, CanonicalConfig::default());
    }

    #[test]
    fn test_builder_single_option() {
        let config = CanonicalConfig::builder()
            .use_ditto(false)
            .build();
        assert!(!config.use_ditto);
        assert!(config.sort_keys);
        assert_eq!(config.quoting, QuotingStrategy::Minimal);
        assert!(!config.inline_schemas);
    }

    #[test]
    fn test_builder_multiple_options() {
        let config = CanonicalConfig::builder()
            .quoting(QuotingStrategy::Always)
            .use_ditto(false)
            .sort_keys(false)
            .build();
        assert_eq!(config.quoting, QuotingStrategy::Always);
        assert!(!config.use_ditto);
        assert!(!config.sort_keys);
        assert!(!config.inline_schemas); // Still default
    }

    #[test]
    fn test_builder_all_options() {
        let config = CanonicalConfig::builder()
            .quoting(QuotingStrategy::Always)
            .use_ditto(false)
            .sort_keys(false)
            .inline_schemas(true)
            .build();
        assert_eq!(config.quoting, QuotingStrategy::Always);
        assert!(!config.use_ditto);
        assert!(!config.sort_keys);
        assert!(config.inline_schemas);
    }

    #[test]
    fn test_builder_chainable() {
        // Verify that each method returns Self for chaining
        let config = CanonicalConfig::builder()
            .quoting(QuotingStrategy::Minimal)
            .use_ditto(true)
            .sort_keys(true)
            .inline_schemas(false)
            .build();
        assert_eq!(config.quoting, QuotingStrategy::Minimal);
        assert!(config.use_ditto);
        assert!(config.sort_keys);
        assert!(!config.inline_schemas);
    }

    #[test]
    fn test_builder_quoting_option() {
        let config_minimal = CanonicalConfig::builder()
            .quoting(QuotingStrategy::Minimal)
            .build();
        assert_eq!(config_minimal.quoting, QuotingStrategy::Minimal);

        let config_always = CanonicalConfig::builder()
            .quoting(QuotingStrategy::Always)
            .build();
        assert_eq!(config_always.quoting, QuotingStrategy::Always);
    }

    #[test]
    fn test_builder_use_ditto_option() {
        let config_true = CanonicalConfig::builder()
            .use_ditto(true)
            .build();
        assert!(config_true.use_ditto);

        let config_false = CanonicalConfig::builder()
            .use_ditto(false)
            .build();
        assert!(!config_false.use_ditto);
    }

    #[test]
    fn test_builder_sort_keys_option() {
        let config_true = CanonicalConfig::builder()
            .sort_keys(true)
            .build();
        assert!(config_true.sort_keys);

        let config_false = CanonicalConfig::builder()
            .sort_keys(false)
            .build();
        assert!(!config_false.sort_keys);
    }

    #[test]
    fn test_builder_inline_schemas_option() {
        let config_true = CanonicalConfig::builder()
            .inline_schemas(true)
            .build();
        assert!(config_true.inline_schemas);

        let config_false = CanonicalConfig::builder()
            .inline_schemas(false)
            .build();
        assert!(!config_false.inline_schemas);
    }

    #[test]
    fn test_builder_overwrite_previous() {
        // Later calls should overwrite earlier ones
        let config = CanonicalConfig::builder()
            .use_ditto(true)
            .use_ditto(false)
            .use_ditto(true)
            .build();
        assert!(config.use_ditto);
    }

    #[test]
    fn test_builder_overwrite_all() {
        let config = CanonicalConfig::builder()
            .quoting(QuotingStrategy::Minimal)
            .quoting(QuotingStrategy::Always)
            .use_ditto(true)
            .use_ditto(false)
            .sort_keys(true)
            .sort_keys(false)
            .inline_schemas(true)
            .inline_schemas(false)
            .build();
        assert_eq!(config.quoting, QuotingStrategy::Always);
        assert!(!config.use_ditto);
        assert!(!config.sort_keys);
        assert!(!config.inline_schemas);
    }

    #[test]
    fn test_builder_default_impl() {
        let builder1 = CanonicalConfigBuilder::new();
        let builder2 = CanonicalConfigBuilder::default();
        assert_eq!(builder1.build(), builder2.build());
    }

    #[test]
    fn test_builder_clone() {
        let builder1 = CanonicalConfig::builder()
            .quoting(QuotingStrategy::Always)
            .use_ditto(false);
        let builder2 = builder1.clone();
        let config1 = builder1.build();
        let config2 = builder2.build();
        assert_eq!(config1.quoting, config2.quoting);
        assert_eq!(config1.use_ditto, config2.use_ditto);
    }

    #[test]
    fn test_builder_debug() {
        let builder = CanonicalConfig::builder();
        let debug = format!("{:?}", builder);
        assert!(debug.contains("CanonicalConfigBuilder"));
        assert!(debug.contains("quoting"));
        assert!(debug.contains("use_ditto"));
        assert!(debug.contains("sort_keys"));
        assert!(debug.contains("inline_schemas"));
    }

    #[test]
    fn test_builder_equals_fluent_api() {
        // Builder pattern should produce same result as fluent API
        let config_builder = CanonicalConfig::builder()
            .quoting(QuotingStrategy::Always)
            .use_ditto(false)
            .sort_keys(false)
            .inline_schemas(true)
            .build();

        let config_fluent = CanonicalConfig::new()
            .with_quoting(QuotingStrategy::Always)
            .with_ditto(false)
            .with_sort_keys(false)
            .with_inline_schemas(true);

        assert_eq!(config_builder.quoting, config_fluent.quoting);
        assert_eq!(config_builder.use_ditto, config_fluent.use_ditto);
        assert_eq!(config_builder.sort_keys, config_fluent.sort_keys);
        assert_eq!(config_builder.inline_schemas, config_fluent.inline_schemas);
    }

    #[test]
    fn test_builder_partial_customization() {
        // Builder with partial customization should keep defaults for unset options
        let config = CanonicalConfig::builder()
            .quoting(QuotingStrategy::Always)
            .build();

        assert_eq!(config.quoting, QuotingStrategy::Always); // Changed
        assert!(config.use_ditto); // Still default
        assert!(config.sort_keys); // Still default
        assert!(!config.inline_schemas); // Still default
    }

    #[test]
    fn test_builder_canonical_form() {
        // Builder should produce canonical form by default
        let config = CanonicalConfig::builder().build();
        assert_eq!(config.quoting, QuotingStrategy::Minimal);
        assert!(config.use_ditto); // Canonical: enabled
        assert!(config.sort_keys); // Canonical: enabled
        assert!(!config.inline_schemas); // Canonical: disabled (use header directives)
    }

    #[test]
    fn test_builder_compact_form() {
        // Builder for compact form (minimal output)
        let config = CanonicalConfig::builder()
            .use_ditto(true)
            .sort_keys(true)
            .quoting(QuotingStrategy::Minimal)
            .build();

        assert_eq!(config.quoting, QuotingStrategy::Minimal);
        assert!(config.use_ditto);
        assert!(config.sort_keys);
    }

    #[test]
    fn test_builder_verbose_form() {
        // Builder for verbose form (maximal clarity)
        let config = CanonicalConfig::builder()
            .quoting(QuotingStrategy::Always)
            .use_ditto(false)
            .sort_keys(true)
            .inline_schemas(true)
            .build();

        assert_eq!(config.quoting, QuotingStrategy::Always);
        assert!(!config.use_ditto);
        assert!(config.sort_keys);
        assert!(config.inline_schemas);
    }

    #[test]
    fn test_builder_consistency() {
        // Multiple builds with same settings should be identical
        let builder = CanonicalConfig::builder()
            .quoting(QuotingStrategy::Always)
            .use_ditto(false);

        let config1 = builder.clone().build();
        let config2 = builder.clone().build();

        assert_eq!(config1.quoting, config2.quoting);
        assert_eq!(config1.use_ditto, config2.use_ditto);
        assert_eq!(config1.sort_keys, config2.sort_keys);
        assert_eq!(config1.inline_schemas, config2.inline_schemas);
    }
}
