// ==============================================================================
// Google Gemini Provider Backend
// ==============================================================================
//
// Implements `LlmProvider` for Google's native Gemini REST API (v1beta generateContent).

use crate::llm::cleaner::clean_commit_message;
use crate::llm::providers::LlmProvider;
use anyhow::{Context, Result};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

/// Provider backend for Google Gemini models.
#[derive(Debug, Clone)]
pub struct GeminiProvider {
    name: String,
    base_url: String,
    model: String,
    api_key: Option<String>,
}

impl GeminiProvider {
    pub fn new(
        name: &str,
        base_url: Option<String>,
        model: String,
        api_key: Option<String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            base_url: base_url
                .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string()),
            model,
            api_key,
        }
    }
}

#[derive(Debug, Serialize)]
struct GeminiRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent<'a>>,
    contents: Vec<GeminiContent<'a>>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenConfig,
}

#[derive(Debug, Serialize)]
struct GeminiContent<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<&'a str>,
    parts: Vec<GeminiPart<'a>>,
}

#[derive(Debug, Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

#[derive(Debug, Serialize)]
struct GeminiGenConfig {
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    error: Option<GeminiApiError>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiResponseContent>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Option<Vec<GeminiResponsePart>>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiApiError {
    message: String,
    code: Option<i32>,
}

impl LlmProvider for GeminiProvider {
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
            let clean_base = self.base_url.trim_end_matches('/');
            let url = format!("{}/models/{}:generateContent", clean_base, self.model);

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            if let Some(ref api_key) = self.api_key {
                let key = api_key.trim();
                if !key.is_empty()
                    && let Ok(val) = HeaderValue::from_str(key)
                {
                    headers.insert("x-goog-api-key", val);
                }
            } else {
                anyhow::bail!(
                    "GEMINI_API_KEY is required for Gemini provider. Please set the environment variable or configure in ~/.config/cogit/config.toml"
                );
            }

            let payload = GeminiRequest {
                system_instruction: Some(GeminiContent {
                    role: None,
                    parts: vec![GeminiPart {
                        text: system_prompt,
                    }],
                }),
                contents: vec![GeminiContent {
                    role: Some("user"),
                    parts: vec![GeminiPart { text: user_prompt }],
                }],
                generation_config: GeminiGenConfig { temperature: 0.2 },
            };

            let response = client
                .post(&url)
                .headers(headers)
                .json(&payload)
                .send()
                .await
                .with_context(|| format!("send HTTP request to Gemini endpoint at {}", url))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unable to decode error body".to_string());
                anyhow::bail!(
                    "Gemini API request failed with status {}: {}",
                    status,
                    error_body
                );
            }

            let gemini_resp: GeminiResponse = response
                .json()
                .await
                .with_context(|| format!("parse JSON response from Gemini at {}", url))?;

            if let Some(err) = gemini_resp.error {
                anyhow::bail!("Gemini API error (code {:?}): {}", err.code, err.message);
            }

            let raw_text = gemini_resp
                .candidates
                .and_then(|c| c.into_iter().next())
                .and_then(|c| c.content)
                .and_then(|c| c.parts)
                .and_then(|p| p.into_iter().next())
                .and_then(|p| p.text)
                .unwrap_or_default();

            if raw_text.trim().is_empty() {
                anyhow::bail!("Gemini returned an empty commit message");
            }

            Ok(clean_commit_message(&raw_text))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gemini_mock_response() {
        let sample_json = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "text": "feat(core): implement Gemini integration\n\n- Add REST client\n- Parse candidate parts"
                            }
                        ],
                        "role": "model"
                    }
                }
            ]
        }"#;

        let parsed: GeminiResponse =
            serde_json::from_str(sample_json).expect("parse sample Gemini JSON");
        let text = parsed
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts)
            .and_then(|p| p.into_iter().next())
            .and_then(|p| p.text)
            .expect("extract candidate text");

        assert_eq!(
            clean_commit_message(&text),
            "feat(core): implement Gemini integration\n\n- Add REST client\n- Parse candidate parts"
        );
    }
}
