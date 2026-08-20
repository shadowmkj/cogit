// ==============================================================================
// Unified LLM Client
// ==============================================================================
//
// Manages the shared HTTP client and dispatches commit generation requests
// polymorphically through the `LlmProvider` interface.

use crate::config::ProviderConfig;
use crate::llm::providers::{LlmProvider, create_provider};
use anyhow::Result;
use std::time::Duration;

/// Unified client managing HTTP transport and provider dispatch.
#[derive(Debug, Clone)]
pub struct LlmClient {
    client: reqwest::Client,
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmClient {
    /// Creates a new HTTP client with sensible connection timeouts.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("initialize standard reqwest client");

        Self { client }
    }

    /// Access to the underlying HTTP client for custom provider invocations.
    pub fn http_client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Dispatches prompt generation to a dynamic `LlmProvider` instance.
    pub async fn generate_with_provider(
        &self,
        provider: &dyn LlmProvider,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        provider
            .generate_commit(&self.client, system_prompt, user_prompt)
            .await
    }

    /// Helper that instantiates the configured provider and generates a commit message.
    pub async fn generate_commit(
        &self,
        provider_name: &str,
        provider_config: &ProviderConfig,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        let provider = create_provider(provider_name, provider_config);
        self.generate_with_provider(provider.as_ref(), system_prompt, user_prompt)
            .await
    }
}
