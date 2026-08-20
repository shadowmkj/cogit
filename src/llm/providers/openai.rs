// ==============================================================================
// OpenAI-Compatible Provider Backend
// ==============================================================================
//
// Implements `LlmProvider` for OpenAI-compatible REST endpoints (/v1/chat/completions),
// supporting OpenAI, Ollama, Groq, Mistral, LocalAI, vLLM, etc.

use crate::llm::cleaner::clean_commit_message;
use crate::llm::providers::LlmProvider;
use anyhow::{Context, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

/// Provider backend for OpenAI and OpenAI-compatible endpoints.
#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    name: String,
    base_url: String,
    model: String,
    api_key: Option<String>,
}

impl OpenAiProvider {
    pub fn new(
        name: &str,
        base_url: Option<String>,
        model: String,
        api_key: Option<String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model,
            api_key,
        }
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Option<Vec<ChatChoice>>,
    error: Option<OpenAiApiError>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiApiError {
    message: String,
}

impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn generate_commit<'a>(
        &'a self,
        client: &'a reqwest::Client,
        system_prompt: &'a str,
        user_prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            let url = format_openai_url(&self.base_url);

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            if let Some(ref api_key) = self.api_key {
                let key = api_key.trim();
                if !key.is_empty() {
                    let auth_val = format!("Bearer {}", key);
                    if let Ok(val) = HeaderValue::from_str(&auth_val) {
                        headers.insert(AUTHORIZATION, val);
                    }
                }
            }

            let body = ChatCompletionRequest {
                model: &self.model,
                messages: vec![
                    ChatMessage {
                        role: "system",
                        content: system_prompt,
                    },
                    ChatMessage {
                        role: "user",
                        content: user_prompt,
                    },
                ],
                temperature: 0.2,
            };

            let response = client
                .post(&url)
                .headers(headers)
                .json(&body)
                .send()
                .await
                .with_context(|| format!("send HTTP request to OpenAI endpoint at {}", url))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unable to decode error body".to_string());
                anyhow::bail!(
                    "OpenAI API request failed with status {}: {}",
                    status,
                    error_body
                );
            }

            let completion: ChatCompletionResponse = response
                .json()
                .await
                .with_context(|| format!("parse JSON response from {}", url))?;

            if let Some(err) = completion.error {
                anyhow::bail!("OpenAI API error: {}", err.message);
            }

            let raw_content = completion
                .choices
                .and_then(|c| c.into_iter().next())
                .and_then(|c| c.message.content)
                .unwrap_or_default();

            if raw_content.trim().is_empty() {
                anyhow::bail!("OpenAI returned an empty commit message");
            }

            Ok(clean_commit_message(&raw_content))
        })
    }
}

/// Normalizes the base URL into a full `/chat/completions` endpoint URL.
fn format_openai_url(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else {
        format!("{}/chat/completions", trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_openai_url() {
        assert_eq!(
            format_openai_url("http://localhost:11434/v1"),
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(
            format_openai_url("https://api.openai.com/v1/"),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn test_parse_openai_mock_response() {
        let sample_json = r#"{
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "fix(git): sanitize binary diff headers"
                    }
                }
            ]
        }"#;

        let parsed: ChatCompletionResponse =
            serde_json::from_str(sample_json).expect("parse OpenAI JSON");
        let content = parsed
            .choices
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.message.content)
            .expect("extract assistant content");

        assert_eq!(
            clean_commit_message(&content),
            "fix(git): sanitize binary diff headers"
        );
    }
}
