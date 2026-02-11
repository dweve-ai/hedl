// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Configuration types for YAML import

/// Default maximum document size: 500 MB
///
/// YAML documents can be very large, especially for configurations like Kubernetes manifests,
/// large datasets, or complex application configurations. This high default allows processing
/// substantial YAML files while still providing `DoS` protection.
pub const DEFAULT_MAX_DOCUMENT_SIZE: usize = 500 * 1024 * 1024; // 500 MB

/// Default maximum array length: 10 million elements
///
/// Large YAML arrays are common in data processing, configuration management, and
/// infrastructure-as-code scenarios. This high limit accommodates real-world use cases
/// while preventing unbounded memory allocation.
pub const DEFAULT_MAX_ARRAY_LENGTH: usize = 10_000_000; // 10 million elements

/// Default maximum nesting depth: 10,000 levels
///
/// Deep nesting can occur in complex hierarchical configurations, particularly in
/// infrastructure definitions and data transformations. This high limit supports
/// realistic nesting while preventing stack overflow attacks.
pub const DEFAULT_MAX_NESTING_DEPTH: usize = 10_000; // 10,000 levels

/// Configuration for YAML import
///
/// # Security Considerations
///
/// This configuration includes resource limits to prevent Denial of Service (`DoS`) attacks
/// through maliciously crafted YAML documents:
///
/// - `max_document_size`: Prevents memory exhaustion from extremely large documents
/// - `max_array_length`: Prevents excessive memory allocation from huge arrays
/// - `max_nesting_depth`: Prevents stack overflow from deeply nested structures
///
/// These limits are enforced during parsing and conversion. Exceeding any limit will
/// result in an error.
///
/// # Default Limits
///
/// The default limits are intentionally high to accommodate real-world YAML files:
/// - Document size: 500 MB (large K8s manifests, datasets)
/// - Array length: 10 million elements (large data arrays)
/// - Nesting depth: 10,000 levels (complex hierarchies)
///
/// # Examples
///
/// ## Using Default Configuration
///
/// ```rust
/// use hedl_yaml::FromYamlConfig;
///
/// let config = FromYamlConfig::default();
/// // Uses high default limits: 500MB / 10M / 10K
/// ```
///
/// ## Customizing Limits with Builder Pattern
///
/// ```rust
/// use hedl_yaml::FromYamlConfig;
///
/// let config = FromYamlConfig::builder()
///     .max_document_size(100 * 1024 * 1024) // 100 MB
///     .max_array_length(1_000_000)           // 1 million
///     .max_nesting_depth(1000)               // 1000 levels
///     .build();
/// ```
///
/// ## Using Struct Initialization
///
/// ```rust
/// use hedl_yaml::FromYamlConfig;
///
/// let config = FromYamlConfig {
///     max_document_size: 200 * 1024 * 1024, // 200 MB
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct FromYamlConfig {
    /// Default type name for arrays without metadata
    pub default_type_name: String,
    /// HEDL version to use
    pub version: (u32, u32),
    /// Maximum allowed document size in bytes (default: 500 MB)
    pub max_document_size: usize,
    /// Maximum allowed array length (default: 10 million elements)
    pub max_array_length: usize,
    /// Maximum allowed nesting depth (default: 10,000 levels)
    pub max_nesting_depth: usize,
}

impl Default for FromYamlConfig {
    fn default() -> Self {
        Self {
            default_type_name: "Item".to_string(),
            version: (2, 0),
            max_document_size: DEFAULT_MAX_DOCUMENT_SIZE,
            max_array_length: DEFAULT_MAX_ARRAY_LENGTH,
            max_nesting_depth: DEFAULT_MAX_NESTING_DEPTH,
        }
    }
}

impl FromYamlConfig {
    /// Creates a new builder for `FromYamlConfig`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_yaml::FromYamlConfig;
    ///
    /// let config = FromYamlConfig::builder()
    ///     .max_document_size(100 * 1024 * 1024)
    ///     .max_array_length(5_000_000)
    ///     .max_nesting_depth(5000)
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder() -> FromYamlConfigBuilder {
        FromYamlConfigBuilder::new()
    }
}

/// Builder for `FromYamlConfig` providing an ergonomic way to customize configuration.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use hedl_yaml::FromYamlConfig;
///
/// let config = FromYamlConfig::builder()
///     .max_document_size(100 * 1024 * 1024)  // 100 MB
///     .build();
/// ```
///
/// ## Customizing All Limits
///
/// ```rust
/// use hedl_yaml::FromYamlConfig;
///
/// let config = FromYamlConfig::builder()
///     .default_type_name("Entity")
///     .version(2, 0)
///     .max_document_size(200 * 1024 * 1024)  // 200 MB
///     .max_array_length(20_000_000)          // 20 million
///     .max_nesting_depth(20_000)             // 20K levels
///     .build();
/// ```
///
/// ## Conservative Limits for Untrusted Input
///
/// ```rust
/// use hedl_yaml::FromYamlConfig;
///
/// // For processing untrusted YAML from external sources
/// let config = FromYamlConfig::builder()
///     .max_document_size(10 * 1024 * 1024)  // 10 MB only
///     .max_array_length(100_000)             // 100K elements
///     .max_nesting_depth(100)                // 100 levels
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct FromYamlConfigBuilder {
    default_type_name: String,
    version: (u32, u32),
    max_document_size: usize,
    max_array_length: usize,
    max_nesting_depth: usize,
}

impl FromYamlConfigBuilder {
    /// Creates a new builder with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            default_type_name: "Item".to_string(),
            version: (2, 0),
            max_document_size: DEFAULT_MAX_DOCUMENT_SIZE,
            max_array_length: DEFAULT_MAX_ARRAY_LENGTH,
            max_nesting_depth: DEFAULT_MAX_NESTING_DEPTH,
        }
    }

    /// Sets the default type name for arrays without metadata.
    pub fn default_type_name(mut self, name: impl Into<String>) -> Self {
        self.default_type_name = name.into();
        self
    }

    /// Sets the HEDL version to use.
    #[must_use]
    pub fn version(mut self, major: u32, minor: u32) -> Self {
        self.version = (major, minor);
        self
    }

    /// Sets the maximum allowed document size in bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_yaml::FromYamlConfig;
    ///
    /// // Set to 100 MB
    /// let config = FromYamlConfig::builder()
    ///     .max_document_size(100 * 1024 * 1024)
    ///     .build();
    /// ```
    #[must_use]
    pub fn max_document_size(mut self, size: usize) -> Self {
        self.max_document_size = size;
        self
    }

    /// Sets the maximum allowed array length.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_yaml::FromYamlConfig;
    ///
    /// // Set to 5 million elements
    /// let config = FromYamlConfig::builder()
    ///     .max_array_length(5_000_000)
    ///     .build();
    /// ```
    #[must_use]
    pub fn max_array_length(mut self, length: usize) -> Self {
        self.max_array_length = length;
        self
    }

    /// Sets the maximum allowed nesting depth.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_yaml::FromYamlConfig;
    ///
    /// // Set to 5000 levels
    /// let config = FromYamlConfig::builder()
    ///     .max_nesting_depth(5000)
    ///     .build();
    /// ```
    #[must_use]
    pub fn max_nesting_depth(mut self, depth: usize) -> Self {
        self.max_nesting_depth = depth;
        self
    }

    /// Builds the `FromYamlConfig`.
    #[must_use]
    pub fn build(self) -> FromYamlConfig {
        FromYamlConfig {
            default_type_name: self.default_type_name,
            version: self.version,
            max_document_size: self.max_document_size,
            max_array_length: self.max_array_length,
            max_nesting_depth: self.max_nesting_depth,
        }
    }
}

impl Default for FromYamlConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl hedl_core::convert::ImportConfig for FromYamlConfig {
    fn default_type_name(&self) -> &str {
        &self.default_type_name
    }

    fn version(&self) -> (u32, u32) {
        self.version
    }
}
