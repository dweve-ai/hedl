// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! LLM Provider Support for Accuracy Benchmarks
//!
//! Supports 8 providers to far exceed TOON's 4-model testing:
//! - DeepSeek (deepseek-chat, deepseek-reasoner)
//! - Mistral (mistral-large-latest, devstral)
//! - OpenAI (gpt-4o, o1, o3, gpt-5)
//! - Anthropic (claude-3.5-sonnet, claude-3-opus, claude-4)
//! - Google (gemini-1.5-pro, gemini-2.0-flash)
//! - Meta (llama-3.3-70b via various endpoints)
//! - GLM (Zhipu AI: glm-4.7, glm-4.6, glm-4.5)
//! - KIMI (Moonshot AI: kimi-k2, moonshot-v1)
//! - NVIDIA (via build.nvidia.com: z-ai/glm4.7, meta/llama-3.3-70b-instruct)

use std::time::Duration;

/// Supported LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LlmProvider {
    /// DeepSeek API (api.deepseek.com)
    DeepSeek,
    /// Mistral API (api.mistral.ai)
    Mistral,
    /// OpenAI API (api.openai.com)
    OpenAI,
    /// Anthropic API (api.anthropic.com)
    Anthropic,
    /// Google AI API (generativelanguage.googleapis.com)
    Google,
    /// Meta Llama (via various endpoints: together.ai, fireworks.ai, groq.com)
    Meta,
    /// GLM / Zhipu AI API (open.bigmodel.cn or api.z.ai)
    Glm,
    /// KIMI / Moonshot AI API (api.moonshot.ai)
    Kimi,
    /// NVIDIA Build API (integrate.api.nvidia.com)
    Nvidia,
}

impl LlmProvider {
    /// All providers for iteration
    pub const ALL: [LlmProvider; 9] = [
        LlmProvider::DeepSeek,
        LlmProvider::Mistral,
        LlmProvider::OpenAI,
        LlmProvider::Anthropic,
        LlmProvider::Google,
        LlmProvider::Meta,
        LlmProvider::Glm,
        LlmProvider::Kimi,
        LlmProvider::Nvidia,
    ];

    /// Human-readable name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            LlmProvider::DeepSeek => "DeepSeek",
            LlmProvider::Mistral => "Mistral",
            LlmProvider::OpenAI => "OpenAI",
            LlmProvider::Anthropic => "Anthropic",
            LlmProvider::Google => "Google",
            LlmProvider::Meta => "Meta",
            LlmProvider::Glm => "GLM",
            LlmProvider::Kimi => "KIMI",
            LlmProvider::Nvidia => "NVIDIA",
        }
    }

    /// Base API URL
    #[must_use]
    pub fn api_base(&self) -> &'static str {
        match self {
            LlmProvider::DeepSeek => "https://api.deepseek.com/v1",
            LlmProvider::Mistral => "https://api.mistral.ai/v1",
            LlmProvider::OpenAI => "https://api.openai.com/v1",
            LlmProvider::Anthropic => "https://api.anthropic.com/v1",
            LlmProvider::Google => "https://generativelanguage.googleapis.com/v1beta",
            LlmProvider::Meta => "https://api.together.xyz/v1", // Default to Together.ai
            LlmProvider::Glm => "https://api.z.ai/api/paas/v4", // Zhipu AI (z.ai global endpoint)
            LlmProvider::Kimi => "https://api.moonshot.ai/v1",  // Moonshot AI
            LlmProvider::Nvidia => "https://integrate.api.nvidia.com/v1", // NVIDIA Build API
        }
    }

    /// Environment variable for API key
    #[must_use]
    pub fn env_var(&self) -> &'static str {
        match self {
            LlmProvider::DeepSeek => "DEEPSEEK_API_KEY",
            LlmProvider::Mistral => "MISTRAL_API_KEY",
            LlmProvider::OpenAI => "OPENAI_API_KEY",
            LlmProvider::Anthropic => "ANTHROPIC_API_KEY",
            LlmProvider::Google => "GOOGLE_API_KEY",
            LlmProvider::Meta => "TOGETHER_API_KEY",
            LlmProvider::Glm => "GLM_API_KEY",
            LlmProvider::Kimi => "KIMI_API_KEY",
            LlmProvider::Nvidia => "NVIDIA_API_KEY",
        }
    }

    /// Default model for this provider
    #[must_use]
    pub fn default_model(&self) -> &'static str {
        match self {
            LlmProvider::DeepSeek => "deepseek-chat",
            LlmProvider::Mistral => "mistral-large-latest",
            LlmProvider::OpenAI => "gpt-4o",
            LlmProvider::Anthropic => "claude-sonnet-4-20250514",
            LlmProvider::Google => "gemini-1.5-pro",
            LlmProvider::Meta => "meta-llama/Meta-Llama-3.3-70B-Instruct-Turbo",
            LlmProvider::Glm => "glm-4.7",
            LlmProvider::Kimi => "kimi-k2",
            LlmProvider::Nvidia => "z-ai/glm4.7",
        }
    }

    /// Available models for this provider
    #[must_use]
    pub fn available_models(&self) -> &'static [&'static str] {
        match self {
            LlmProvider::DeepSeek => &["deepseek-chat", "deepseek-reasoner"],
            LlmProvider::Mistral => &[
                "mistral-large-latest",
                "magistral-medium",
                "devstral-small-latest",
                "ministral-3b",
            ],
            LlmProvider::OpenAI => &[
                "gpt-4o",
                "gpt-4-turbo",
                "gpt-4o-mini",
                "o1",
                "o1-mini",
                "o3",
                "o3-mini",
                "gpt-5.1",
                "gpt-5",
            ],
            LlmProvider::Anthropic => &[
                "claude-sonnet-4-20250514",
                "claude-opus-4-20250514",
                "claude-3-5-sonnet-20241022",
                "claude-3-5-haiku-20241022",
                "claude-3-opus-20240229",
            ],
            LlmProvider::Google => &[
                "gemini-1.5-pro",
                "gemini-1.5-flash",
                "gemini-2.0-flash-exp",
                "gemini-2.0-flash-thinking-exp",
            ],
            LlmProvider::Meta => &[
                "meta-llama/Meta-Llama-3.3-70B-Instruct-Turbo",
                "meta-llama/Meta-Llama-3.1-405B-Instruct-Turbo",
                "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo",
                "meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo",
            ],
            LlmProvider::Glm => &["glm-4.7", "glm-4.6", "glm-4.5", "glm-4.5-air"],
            LlmProvider::Kimi => &[
                "kimi-k2",
                "kimi-k2-0905-preview",
                "moonshot-v1-8k",
                "moonshot-v1-32k",
                "moonshot-v1-128k",
            ],
            LlmProvider::Nvidia => &[
                "z-ai/glm4.7",
                "meta/llama-3.3-70b-instruct",
                "nvidia/llama-3.1-nemotron-70b-instruct",
            ],
        }
    }

    /// Whether this provider uses OpenAI-compatible API format
    #[must_use]
    pub fn is_openai_compatible(&self) -> bool {
        matches!(
            self,
            LlmProvider::DeepSeek
                | LlmProvider::Mistral
                | LlmProvider::OpenAI
                | LlmProvider::Meta
                | LlmProvider::Glm
                | LlmProvider::Kimi
                | LlmProvider::Nvidia
        )
    }

    /// Chat completion endpoint path
    #[must_use]
    pub fn chat_endpoint(&self) -> &'static str {
        match self {
            LlmProvider::Anthropic => "/messages",
            LlmProvider::Google => "/models/{model}:generateContent",
            _ => "/chat/completions",
        }
    }

    /// Recommended rate limit delay between requests
    #[must_use]
    pub fn rate_limit_delay(&self) -> Duration {
        match self {
            LlmProvider::DeepSeek => Duration::from_millis(100),
            LlmProvider::Mistral => Duration::from_millis(200),
            LlmProvider::OpenAI => Duration::from_millis(100),
            LlmProvider::Anthropic => Duration::from_millis(200),
            LlmProvider::Google => Duration::from_millis(100),
            LlmProvider::Meta => Duration::from_millis(50),
            LlmProvider::Glm => Duration::from_millis(150),
            LlmProvider::Kimi => Duration::from_millis(100),
            LlmProvider::Nvidia => Duration::from_millis(1500), // 40 req/min limit
        }
    }

    /// Default timeout for requests
    #[must_use]
    pub fn timeout(&self) -> Duration {
        match self {
            LlmProvider::OpenAI => Duration::from_secs(120), // o1/o3 can be slow
            LlmProvider::Anthropic => Duration::from_secs(90),
            LlmProvider::Glm => Duration::from_secs(90), // GLM-4.7 thinking can be slow
            LlmProvider::Kimi => Duration::from_secs(90), // Kimi K2 can be slow
            _ => Duration::from_secs(60),
        }
    }

    /// Parse provider from string name
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "deepseek" => Some(LlmProvider::DeepSeek),
            "mistral" => Some(LlmProvider::Mistral),
            "openai" => Some(LlmProvider::OpenAI),
            "anthropic" | "claude" => Some(LlmProvider::Anthropic),
            "google" | "gemini" => Some(LlmProvider::Google),
            "meta" | "llama" | "together" => Some(LlmProvider::Meta),
            "glm" | "zhipu" | "chatglm" => Some(LlmProvider::Glm),
            "kimi" | "moonshot" => Some(LlmProvider::Kimi),
            "nvidia" | "nim" => Some(LlmProvider::Nvidia),
            _ => None,
        }
    }
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Configuration for a specific provider
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// The provider
    pub provider: LlmProvider,
    /// Model to use
    pub model: String,
    /// API key (from env or explicit)
    pub api_key: Option<String>,
    /// Custom API base URL (for proxies/self-hosted)
    pub api_base: Option<String>,
    /// Request timeout
    pub timeout: Duration,
    /// Rate limit delay between requests
    pub rate_limit_delay: Duration,
    /// Maximum retries on failure
    pub max_retries: u8,
    /// Temperature (0.0 for deterministic)
    pub temperature: f32,
    /// Maximum tokens in response
    pub max_tokens: u32,
}

impl ProviderConfig {
    /// Create config with default settings for a provider
    #[must_use]
    pub fn new(provider: LlmProvider) -> Self {
        Self {
            provider,
            model: provider.default_model().to_string(),
            api_key: None,
            api_base: None,
            timeout: provider.timeout(),
            rate_limit_delay: provider.rate_limit_delay(),
            max_retries: 3,
            temperature: 0.0, // Deterministic for benchmarks
            max_tokens: 256,
        }
    }

    /// Set model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set API key explicitly
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set custom API base
    pub fn with_api_base(mut self, base: impl Into<String>) -> Self {
        self.api_base = Some(base.into());
        self
    }

    /// Set timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set rate limit delay
    #[must_use]
    pub fn with_rate_limit(mut self, delay: Duration) -> Self {
        self.rate_limit_delay = delay;
        self
    }

    /// Set max retries
    #[must_use]
    pub fn with_retries(mut self, retries: u8) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set temperature
    #[must_use]
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp.clamp(0.0, 2.0);
        self
    }

    /// Set max tokens
    #[must_use]
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = tokens;
        self
    }

    /// Get the effective API base URL
    #[must_use]
    pub fn effective_api_base(&self) -> &str {
        self.api_base
            .as_deref()
            .unwrap_or_else(|| self.provider.api_base())
    }

    /// Get the effective API key (from config or environment)
    #[must_use]
    pub fn effective_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var(self.provider.env_var()).ok())
    }

    /// Validate configuration
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.effective_api_key().is_none() {
            issues.push(format!(
                "No API key found for {}. Set {} environment variable or provide explicitly.",
                self.provider.name(),
                self.provider.env_var()
            ));
        }

        if !self
            .provider
            .available_models()
            .contains(&self.model.as_str())
        {
            issues.push(format!(
                "Model '{}' may not be available for {}. Known models: {:?}",
                self.model,
                self.provider.name(),
                self.provider.available_models()
            ));
        }

        issues
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self::new(LlmProvider::DeepSeek)
    }
}

/// Model-specific request configuration
#[derive(Debug, Clone)]
pub struct ModelConfig {
    /// Whether model uses max_completion_tokens instead of max_tokens
    pub uses_completion_tokens: bool,
    /// Whether model supports system messages
    pub supports_system_message: bool,
    /// Whether model supports temperature
    pub supports_temperature: bool,
    /// Maximum context window
    pub context_window: u32,
    /// Cost per 1M input tokens (USD)
    pub cost_per_1m_input: f64,
    /// Cost per 1M output tokens (USD)
    pub cost_per_1m_output: f64,
}

impl ModelConfig {
    /// Get configuration for a specific model
    #[must_use]
    pub fn for_model(provider: LlmProvider, model: &str) -> Self {
        match provider {
            LlmProvider::OpenAI => {
                let uses_completion_tokens = model.starts_with("o1")
                    || model.starts_with("o3")
                    || model.starts_with("gpt-5");
                let supports_temperature = !model.starts_with("o1") && !model.starts_with("o3");

                let (context, cost_in, cost_out) = match model {
                    "gpt-4o" => (128_000, 2.50, 10.00),
                    "gpt-4o-mini" => (128_000, 0.15, 0.60),
                    "gpt-4-turbo" => (128_000, 10.00, 30.00),
                    "o1" => (200_000, 15.00, 60.00),
                    "o1-mini" => (128_000, 3.00, 12.00),
                    "o3" | "o3-mini" => (200_000, 15.00, 60.00),
                    _ => (128_000, 5.00, 15.00),
                };

                Self {
                    uses_completion_tokens,
                    supports_system_message: true,
                    supports_temperature,
                    context_window: context,
                    cost_per_1m_input: cost_in,
                    cost_per_1m_output: cost_out,
                }
            }
            LlmProvider::Anthropic => {
                let (context, cost_in, cost_out) = match model {
                    "claude-3-opus-20240229" | "claude-opus-4-20250514" => (200_000, 15.00, 75.00),
                    "claude-3-5-sonnet-20241022" | "claude-sonnet-4-20250514" => {
                        (200_000, 3.00, 15.00)
                    }
                    "claude-3-5-haiku-20241022" => (200_000, 0.25, 1.25),
                    _ => (200_000, 3.00, 15.00),
                };

                Self {
                    uses_completion_tokens: false,
                    supports_system_message: true,
                    supports_temperature: true,
                    context_window: context,
                    cost_per_1m_input: cost_in,
                    cost_per_1m_output: cost_out,
                }
            }
            LlmProvider::Google => Self {
                uses_completion_tokens: false,
                supports_system_message: true,
                supports_temperature: true,
                context_window: 1_000_000, // Gemini 1.5 Pro
                cost_per_1m_input: 1.25,
                cost_per_1m_output: 5.00,
            },
            LlmProvider::DeepSeek => Self {
                uses_completion_tokens: false,
                supports_system_message: true,
                supports_temperature: true,
                context_window: 64_000,
                cost_per_1m_input: 0.14,
                cost_per_1m_output: 0.28,
            },
            LlmProvider::Mistral => Self {
                uses_completion_tokens: false,
                supports_system_message: true,
                supports_temperature: true,
                context_window: 128_000,
                cost_per_1m_input: 2.00,
                cost_per_1m_output: 6.00,
            },
            LlmProvider::Meta => Self {
                uses_completion_tokens: false,
                supports_system_message: true,
                supports_temperature: true,
                context_window: 128_000,
                cost_per_1m_input: 0.88, // Via Together.ai
                cost_per_1m_output: 0.88,
            },
            LlmProvider::Glm => {
                // GLM-4.6 has 200K context, GLM-4.5 has 128K
                let context = if model.contains("4.6") || model.contains("4.7") {
                    200_000
                } else {
                    128_000
                };
                Self {
                    uses_completion_tokens: false,
                    supports_system_message: true,
                    supports_temperature: true,
                    context_window: context,
                    cost_per_1m_input: 0.50, // Approximate pricing
                    cost_per_1m_output: 0.50,
                }
            }
            LlmProvider::Kimi => {
                // Kimi K2 has 256K context
                let context = if model.contains("k2") {
                    256_000
                } else if model.contains("128k") {
                    128_000
                } else if model.contains("32k") {
                    32_000
                } else {
                    8_000
                };
                Self {
                    uses_completion_tokens: false,
                    supports_system_message: true,
                    supports_temperature: true,
                    context_window: context,
                    cost_per_1m_input: 0.60, // Approximate pricing
                    cost_per_1m_output: 0.60,
                }
            }
            LlmProvider::Nvidia => {
                // NVIDIA Build API - free tier with credits
                Self {
                    uses_completion_tokens: false,
                    supports_system_message: true,
                    supports_temperature: true,
                    context_window: 128_000,
                    cost_per_1m_input: 0.0, // Free tier credits
                    cost_per_1m_output: 0.0,
                }
            }
        }
    }
}

/// Response from an LLM API call
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// The response text
    pub text: String,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Input tokens used
    pub tokens_in: usize,
    /// Output tokens used
    pub tokens_out: usize,
    /// Model used
    pub model: String,
    /// Provider used
    pub provider: LlmProvider,
    /// Whether this was a cached response
    pub cached: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl LlmResponse {
    /// Check if response is successful
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }

    /// Total tokens used
    #[must_use]
    pub fn total_tokens(&self) -> usize {
        self.tokens_in + self.tokens_out
    }

    /// Estimated cost in USD
    #[must_use]
    pub fn estimated_cost(&self, config: &ModelConfig) -> f64 {
        let input_cost = self.tokens_in as f64 / 1_000_000.0 * config.cost_per_1m_input;
        let output_cost = self.tokens_out as f64 / 1_000_000.0 * config.cost_per_1m_output;
        input_cost + output_cost
    }
}

/// Multi-provider benchmark configuration
#[derive(Debug, Clone)]
pub struct MultiProviderConfig {
    /// Configurations for each provider to test
    pub providers: Vec<ProviderConfig>,
    /// Run providers in parallel
    pub parallel: bool,
    /// Compare results across providers
    pub cross_compare: bool,
}

impl MultiProviderConfig {
    /// Create with a single provider
    #[must_use]
    pub fn single(config: ProviderConfig) -> Self {
        Self {
            providers: vec![config],
            parallel: false,
            cross_compare: false,
        }
    }

    /// Create with multiple providers for comparison
    #[must_use]
    pub fn compare(configs: Vec<ProviderConfig>) -> Self {
        Self {
            providers: configs,
            parallel: true,
            cross_compare: true,
        }
    }

    /// Create default comparison across all providers
    #[must_use]
    pub fn all_providers() -> Self {
        Self::compare(
            LlmProvider::ALL
                .iter()
                .map(|p| ProviderConfig::new(*p))
                .collect(),
        )
    }

    /// Get only providers with valid API keys
    #[must_use]
    pub fn available_providers(&self) -> Vec<&ProviderConfig> {
        self.providers
            .iter()
            .filter(|c| c.effective_api_key().is_some())
            .collect()
    }
}

impl Default for MultiProviderConfig {
    fn default() -> Self {
        Self::single(ProviderConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_providers() {
        assert_eq!(LlmProvider::ALL.len(), 9);
        for provider in LlmProvider::ALL {
            assert!(!provider.name().is_empty());
            assert!(!provider.api_base().is_empty());
            assert!(!provider.env_var().is_empty());
            assert!(!provider.default_model().is_empty());
            assert!(!provider.available_models().is_empty());
        }
    }

    #[test]
    fn test_provider_parsing() {
        assert_eq!(LlmProvider::parse("deepseek"), Some(LlmProvider::DeepSeek));
        assert_eq!(LlmProvider::parse("claude"), Some(LlmProvider::Anthropic));
        assert_eq!(LlmProvider::parse("gemini"), Some(LlmProvider::Google));
        assert_eq!(LlmProvider::parse("llama"), Some(LlmProvider::Meta));
        assert_eq!(LlmProvider::parse("glm"), Some(LlmProvider::Glm));
        assert_eq!(LlmProvider::parse("zhipu"), Some(LlmProvider::Glm));
        assert_eq!(LlmProvider::parse("kimi"), Some(LlmProvider::Kimi));
        assert_eq!(LlmProvider::parse("moonshot"), Some(LlmProvider::Kimi));
        assert_eq!(LlmProvider::parse("unknown"), None);
    }

    #[test]
    fn test_provider_config() {
        let config = ProviderConfig::new(LlmProvider::OpenAI)
            .with_model("gpt-4o")
            .with_temperature(0.0)
            .with_max_tokens(512);

        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.temperature, 0.0);
        assert_eq!(config.max_tokens, 512);
    }

    #[test]
    fn test_model_config() {
        let o1_config = ModelConfig::for_model(LlmProvider::OpenAI, "o1");
        assert!(o1_config.uses_completion_tokens);
        assert!(!o1_config.supports_temperature);

        let gpt4o_config = ModelConfig::for_model(LlmProvider::OpenAI, "gpt-4o");
        assert!(!gpt4o_config.uses_completion_tokens);
        assert!(gpt4o_config.supports_temperature);
    }

    #[test]
    fn test_openai_compatible() {
        assert!(LlmProvider::DeepSeek.is_openai_compatible());
        assert!(LlmProvider::OpenAI.is_openai_compatible());
        assert!(!LlmProvider::Anthropic.is_openai_compatible());
        assert!(!LlmProvider::Google.is_openai_compatible());
    }
}
