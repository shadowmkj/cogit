// ==============================================================================
// Mock LLM Provider for Testing
// ==============================================================================
//
// Provides a deterministic, network-free implementation of `LlmProvider` for
// testing orchestration logic, error handling, and TUI flows.

use crate::llm::cleaner::clean_commit_message;
use crate::llm::providers::LlmProvider;
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

/// Mock provider returning predetermined commit messages or errors.
#[derive(Debug, Clone)]
pub struct MockProvider {
    name: String,
    response: Result<String, String>,
}

impl MockProvider {
    /// Creates a mock provider that returns a successful response string.
    pub fn success(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            response: Ok(message.to_string()),
        }
    }

    /// Creates a mock provider that returns an error.
    pub fn failure(name: &str, error_message: &str) -> Self {
        Self {
            name: name.to_string(),
            response: Err(error_message.to_string()),
        }
    }
}

impl LlmProvider for MockProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn generate_commit<'a>(
        &'a self,
        _client: &'a reqwest::Client,
        _system_prompt: &'a str,
        _user_prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            match &self.response {
                Ok(msg) => Ok(clean_commit_message(msg)),
                Err(err) => anyhow::bail!("{}", err),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_success() {
        let provider = MockProvider::success("mock", "feat(test): mock commit");
        let client = reqwest::Client::new();
        let msg = provider
            .generate_commit(&client, "sys", "usr")
            .await
            .expect("mock generate");
        assert_eq!(msg, "feat(test): mock commit");
    }

    #[tokio::test]
    async fn test_mock_provider_failure() {
        let provider = MockProvider::failure("mock", "API rate limit exceeded");
        let client = reqwest::Client::new();
        let err = provider
            .generate_commit(&client, "sys", "usr")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("API rate limit exceeded"));
    }
}
