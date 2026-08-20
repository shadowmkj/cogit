// ==============================================================================
// LLM Provider Interface and Abstractions
// ==============================================================================
//
// Defines the common trait `LlmProvider` for all LLM backends (Gemini, OpenAI,
// Ollama, mock testing providers) enabling easy testing, interchangeability,
// and scalable addition of future providers (e.g. Anthropic, Bedrock, Groq).

pub mod gemini;
pub mod mock;
pub mod openai;

pub use gemini::GeminiProvider;
pub use mock::MockProvider;
pub use openai::OpenAiProvider;

use crate::config::{ProviderConfig, ProviderKind};
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

/// Common interface that all LLM provider backends must implement.
pub trait LlmProvider: Send + Sync {
    /// Identifier or display name of the provider.
    fn name(&self) -> &str;

    /// Generates a commit message given the HTTP client and prompts.
    fn generate_commit<'a>(
        &'a self,
        client: &'a reqwest::Client,
        system_prompt: &'a str,
        user_prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>>;
}

/// Factory function to instantiate the appropriate `LlmProvider` implementation
/// based on the configured `ProviderConfig`.
pub fn create_provider(provider_name: &str, config: &ProviderConfig) -> Box<dyn LlmProvider> {
    match config.resolve_kind(provider_name) {
        ProviderKind::Gemini => Box::new(GeminiProvider::new(
            provider_name,
            config.base_url.clone(),
            config.model.clone(),
            config.api_key.clone(),
        )),
        ProviderKind::OpenAi => Box::new(OpenAiProvider::new(
            provider_name,
            config.base_url.clone(),
            config.model.clone(),
            config.api_key.clone(),
        )),
    }
}
