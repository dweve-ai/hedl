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

//! LLM API calling with retry logic

use hedl_bench::accuracy::LlmProvider;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::{Duration, Instant};

// P0 OPTIMIZATION: Reusable HTTP agent for connection pooling (1.08x speedup)
// CRITICAL FIX (P0): Use Arc instead of Mutex - ureq::Agent is already thread-safe
// Previous issue: Mutex serialized all HTTP calls, blocking parallel requests
static HTTP_AGENT: Lazy<Arc<ureq::Agent>> = Lazy::new(|| {
    Arc::new(
        ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(60)))
            // Force IPv4 - WSL2 and many environments have broken IPv6
            .ip_family(ureq::config::IpFamily::Ipv4Only)
            .build()
            .into(),
    )
});

/// LLM response with token usage metrics.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// Content of the response from the LLM.
    pub content: String,
    /// Latency of the request in milliseconds.
    pub latency_ms: u64,
    /// Output tokens from LLM response.
    pub tokens_out: usize,
    /// Reasoning/thinking tokens (for models like o1, o3, deepseek-reasoner).
    pub tokens_thinking: usize,
}

/// Make an LLM API call using the specified provider.
///
/// Uses a connection-pooled HTTP agent for performance. The agent is thread-safe
/// and cached globally via `Lazy<Arc<ureq::Agent>>` for efficient concurrent requests.
///
/// # Arguments
/// * `provider` - The LLM provider (`DeepSeek`, Mistral, `OpenAI`)
/// * `model` - Model identifier
/// * `api_key` - API authentication key
/// * `prompt` - The prompt to send to the LLM
///
/// # Returns
/// * `Ok(LlmResponse)` - Successful response containing:
/// * `Err(String)` - Error message if request fails
///
/// # Performance
/// - Connection pooling provides ~1.08x speedup for sequential requests
/// - Thread-safe Arc design enables parallel requests without mutex contention
/// - 60-second timeout prevents indefinite hangs
pub fn call_llm(
    provider: &LlmProvider,
    model: &str,
    api_key: &str,
    prompt: &str,
) -> Result<LlmResponse, String> {
    let start = Instant::now();

    // Anthropic uses a different API format
    if *provider == LlmProvider::Anthropic {
        return call_anthropic(model, api_key, prompt, start);
    }

    let url = format!("{}/chat/completions", provider.api_base());

    // OpenAI newer models (o-series, gpt-5.x) require max_completion_tokens
    // Older models (gpt-4o, gpt-4-turbo) use max_tokens
    let needs_completion_tokens = model.starts_with("o1")
        || model.starts_with("o3")
        || model.starts_with("o4")
        || model.starts_with("gpt-5");

    let body = if *provider == LlmProvider::OpenAI && needs_completion_tokens {
        serde_json::json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.0,
            "max_completion_tokens": 256
        })
    } else if *provider == LlmProvider::Nvidia {
        // NVIDIA GLM-4.7 uses thinking mode and needs more tokens
        serde_json::json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.0,
            "max_tokens": 1024
        })
    } else {
        serde_json::json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.0,
            "max_tokens": 256
        })
    };

    // P0 OPTIMIZATION: Use connection-pooled agent (no lock needed - Arc only)
    let mut response = HTTP_AGENT
        .post(&url)
        .header("Authorization", &format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .send_json(&body)
        .map_err(|e| format!("HTTP error: {e}"))?;

    let json: serde_json::Value = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("JSON parse error: {e}"))?;

    let latency = start.elapsed().as_millis() as u64;

    // Try content first, fall back to reasoning_content for thinking models (NVIDIA GLM-4.7)
    let raw_content = json["choices"][0]["message"]["content"]
        .as_str()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            // For thinking models, try to extract from reasoning_content if content is empty
            // Look for the last line or sentence which typically contains the answer
            json["choices"][0]["message"]["reasoning_content"]
                .as_str()
                .and_then(|r| r.lines().last())
        })
        .unwrap_or("");

    // Strip training data leakage from Mistral-large (and potentially other models)
    // Pattern: model answers correctly, then appends memorized benchmark data after "+++++ path/to/file"
    let content = if let Some(pos) = raw_content.find("+++++") {
        raw_content[..pos].trim().to_string()
    } else if let Some(pos) = raw_content.find("\n\n\n") {
        // Also strip if there's excessive newlines followed by reasoning
        let before = raw_content[..pos].trim();
        if before.lines().count() <= 2 {
            before.to_string()
        } else {
            raw_content.trim().to_string()
        }
    } else {
        raw_content.trim().to_string()
    };

    let tokens_out = json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as usize;

    // Extract reasoning/thinking tokens (varies by provider)
    // - DeepSeek: reasoning_tokens in usage
    // - OpenAI o1/o3: completion_tokens_details.reasoning_tokens
    let tokens_thinking = json["usage"]["reasoning_tokens"]
        .as_u64()
        .or_else(|| json["usage"]["completion_tokens_details"]["reasoning_tokens"].as_u64())
        .unwrap_or(0) as usize;

    Ok(LlmResponse {
        content,
        latency_ms: latency,
        tokens_out,
        tokens_thinking,
    })
}

/// Call Anthropic's Messages API (different format from OpenAI-compatible)
fn call_anthropic(
    model: &str,
    api_key: &str,
    prompt: &str,
    start: Instant,
) -> Result<LlmResponse, String> {
    let url = "https://api.anthropic.com/v1/messages";

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 256,
        "messages": [
            {"role": "user", "content": prompt}
        ]
    });

    let mut response = HTTP_AGENT
        .post(url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .send_json(&body)
        .map_err(|e| format!("HTTP error: {e}"))?;

    let json: serde_json::Value = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("JSON parse error: {e}"))?;

    let latency = start.elapsed().as_millis() as u64;

    // Anthropic returns content as an array of content blocks
    let content = json["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();

    let tokens_out = json["usage"]["output_tokens"].as_u64().unwrap_or(0) as usize;

    // Anthropic extended thinking returns thinking content in a separate content block
    // For now, we don't have a way to count thinking tokens without extended thinking enabled
    // The cache_creation_input_tokens and cache_read_input_tokens are for prompt caching, not thinking
    let tokens_thinking = 0;

    Ok(LlmResponse {
        content,
        latency_ms: latency,
        tokens_out,
        tokens_thinking,
    })
}

/// Maximum number of retries for transient API errors
pub const MAX_RETRIES: usize = 3;

/// Initial backoff delay in milliseconds for retries
const INITIAL_BACKOFF_MS: u64 = 1000;

/// Call LLM with retry logic for transient errors
///
/// Returns the response and the number of retries needed (0 if first attempt succeeded).
pub fn call_llm_with_retry(
    provider: &LlmProvider,
    model: &str,
    api_key: &str,
    prompt: &str,
) -> Result<(LlmResponse, usize), String> {
    let mut last_error = String::new();
    let mut retry_count = 0;

    for attempt in 0..=MAX_RETRIES {
        match call_llm(provider, model, api_key, prompt) {
            Ok(response) => {
                return Ok((response, retry_count));
            }
            Err(e) => {
                last_error = e.clone();

                // Check if error is retryable (transient network/rate limit errors)
                let is_retryable = e.contains("rate limit")
                    || e.contains("429")
                    || e.contains("503")
                    || e.contains("502")
                    || e.contains("timeout")
                    || e.contains("connection")
                    || e.contains("temporarily unavailable");

                if !is_retryable || attempt == MAX_RETRIES {
                    break;
                }

                retry_count = attempt + 1;

                // Exponential backoff: 1s, 2s, 4s
                let backoff_ms = INITIAL_BACKOFF_MS * (1 << attempt);
                std::thread::sleep(Duration::from_millis(backoff_ms));
            }
        }
    }

    Err(last_error)
}
