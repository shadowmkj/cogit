// ==============================================================================
// Configuration Management
// ==============================================================================
//
// Handles loading, parsing, environment variable expansion, and default creation
// for Cogit's user configuration stored in ~/.config/cogit/config.toml.

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Enumeration of supported LLM provider API protocols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    /// OpenAI-compatible REST API (/v1/chat/completions, e.g. OpenAI, Ollama, Grok, Groq, Mistral).
    OpenAi,
    /// Google Gemini REST API (v1beta generateContent).
    Gemini,
}

/// Top-level configuration object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Identifier of the active LLM provider (e.g., "gemini", "openai", "grok", "groq", "ollama").
    #[serde(default = "default_provider_name")]
    pub default_provider: String,

    /// Map of named provider configurations.
    #[serde(default = "default_providers_map")]
    pub providers: HashMap<String, ProviderConfig>,

    /// User workflow preferences.
    #[serde(default)]
    pub preferences: PreferencesConfig,
}

/// Provider-specific connection and model details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Explicit provider protocol kind (defaults to auto-detection from name/url if None).
    #[serde(default)]
    pub kind: Option<ProviderKind>,

    /// Base URL for the endpoint (e.g. "https://api.openai.com/v1" or "https://api.x.ai/v1").
    #[serde(default)]
    pub base_url: Option<String>,

    /// Model name to request (e.g. "gemini-3.5-flash-lite", "gpt-4o-mini", "grok-2-latest", "llama-3.3-70b-versatile").
    pub model: String,

    /// Optional API key or environment variable token (e.g. "${GEMINI_API_KEY}", "${XAI_API_KEY}", "${GROQ_API_KEY}").
    #[serde(default)]
    pub api_key: Option<String>,
}

impl ProviderConfig {
    /// Resolves the protocol kind (Gemini vs OpenAI) based on explicit configuration or heuristics.
    pub fn resolve_kind(&self, provider_name: &str) -> ProviderKind {
        if let Some(kind) = self.kind {
            return kind;
        }

        let name_lower = provider_name.to_lowercase();
        let model_lower = self.model.to_lowercase();
        let url_lower = self.base_url.as_deref().unwrap_or("").to_lowercase();

        if name_lower.contains("gemini")
            || model_lower.contains("gemini")
            || url_lower.contains("googleapis.com")
        {
            ProviderKind::Gemini
        } else {
            ProviderKind::OpenAi
        }
    }
}

/// General user preferences.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreferencesConfig {
    /// Whether to default to detailed multi-line commit messages with bullet points.
    #[serde(default)]
    pub detailed: bool,

    /// Preferred editor command (defaults to system $GIT_EDITOR, $VISUAL, or $EDITOR).
    #[serde(default)]
    pub editor: Option<String>,

    /// Maximum characters from staged diff before truncation to prevent token overflow.
    #[serde(default = "default_max_diff_chars")]
    pub max_diff_chars: usize,
}

fn default_provider_name() -> String {
    "gemini".to_string()
}

fn default_max_diff_chars() -> usize {
    32_000
}

fn default_providers_map() -> HashMap<String, ProviderConfig> {
    let mut map = HashMap::new();

    let gemini_model =
        std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.5-flash-lite".to_string());

    map.insert(
        "gemini".to_string(),
        ProviderConfig {
            kind: Some(ProviderKind::Gemini),
            base_url: None,
            model: gemini_model,
            api_key: Some("${GEMINI_API_KEY}".to_string()),
        },
    );

    map.insert(
        "openai".to_string(),
        ProviderConfig {
            kind: Some(ProviderKind::OpenAi),
            base_url: Some("https://api.openai.com/v1".to_string()),
            model: "gpt-4o-mini".to_string(),
            api_key: Some("${OPENAI_API_KEY}".to_string()),
        },
    );

    map.insert(
        "grok".to_string(),
        ProviderConfig {
            kind: Some(ProviderKind::OpenAi),
            base_url: Some("https://api.x.ai/v1".to_string()),
            model: "grok-2-latest".to_string(),
            api_key: Some("${XAI_API_KEY}".to_string()),
        },
    );

    map.insert(
        "groq".to_string(),
        ProviderConfig {
            kind: Some(ProviderKind::OpenAi),
            base_url: Some("https://api.groq.com/openai/v1".to_string()),
            model: "qwen3.6-27b".to_string(),
            api_key: Some("${GROQ_API_KEY}".to_string()),
        },
    );

    map.insert(
        "ollama".to_string(),
        ProviderConfig {
            kind: Some(ProviderKind::OpenAi),
            base_url: Some("http://localhost:11434/v1".to_string()),
            model: "qwen2.5-coder:7b".to_string(),
            api_key: Some("ollama".to_string()),
        },
    );

    map
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_provider: default_provider_name(),
            providers: default_providers_map(),
            preferences: PreferencesConfig {
                detailed: false,
                editor: None,
                max_diff_chars: default_max_diff_chars(),
            },
        }
    }
}

impl Config {
    /// Resolves the default configuration file path (~/.config/cogit/config.toml).
    pub fn default_config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "cogit")
            .context("determine system configuration directory for cogit")?;
        let config_dir = proj_dirs.config_dir();
        Ok(config_dir.join("config.toml"))
    }

    /// Loads configuration from custom path or the standard system path.
    /// If no configuration file is present, returns default configuration.
    pub fn load(custom_path: Option<&Path>) -> Result<Self> {
        let path = match custom_path {
            Some(p) => p.to_path_buf(),
            None => match Self::default_config_path() {
                Ok(p) => p,
                Err(_) => return Ok(Self::default()),
            },
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("read configuration file from {}", path.display()))?;

        let mut config: Config = toml::from_str(&content)
            .with_context(|| format!("parse TOML configuration in {}", path.display()))?;

        config.expand_env_vars();

        Ok(config)
    }

    /// Writes a default starter configuration file to the given path or default location.
    pub fn initialize_default_file(target_path: Option<&Path>) -> Result<PathBuf> {
        let path = match target_path {
            Some(p) => p.to_path_buf(),
            None => Self::default_config_path()
                .context("determine default config path for initialization")?,
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create directory {}", parent.display()))?;
        }

        let default_toml = r#"# Cogit Configuration File
# Location: ~/.config/cogit/config.toml

default_provider = "gemini" # or "openai", "grok", "groq", "ollama"

[providers.gemini]
kind = "gemini"
model = "gemini-3.5-flash-lite"
api_key = "${GEMINI_API_KEY}"

[providers.openai]
kind = "openai"
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
api_key = "${OPENAI_API_KEY}"

[providers.grok]
kind = "openai"
base_url = "https://api.x.ai/v1"
model = "grok-2-latest"
api_key = "${XAI_API_KEY}"

[providers.groq]
kind = "openai"
base_url = "https://api.groq.com/openai/v1"
model = "qwen3.6-27b"
api_key = "${GROQ_API_KEY}"

[providers.ollama]
kind = "openai"
base_url = "http://localhost:11434/v1"
model = "qwen2.5-coder:7b"
api_key = "ollama"

[preferences]
detailed = false
max_diff_chars = 32000
# editor = "nvim" # uncomment to override system $EDITOR
"#;

        fs::write(&path, default_toml)
            .with_context(|| format!("write initial config to {}", path.display()))?;

        Ok(path)
    }

    /// Replaces `${VAR_NAME}` patterns with the actual environment variable value.
    fn expand_env_vars(&mut self) {
        for provider in self.providers.values_mut() {
            if let Some(ref url) = provider.base_url {
                provider.base_url = Some(expand_string(url));
            }
            provider.model = expand_string(&provider.model);
            if let Some(ref key) = provider.api_key {
                provider.api_key = Some(expand_string(key));
            }
        }
    }

    /// Retrieves the active provider configuration according to optional overrides.
    pub fn get_active_provider(
        &self,
        provider_override: Option<&str>,
    ) -> Result<(String, ProviderConfig)> {
        let provider_name = provider_override.unwrap_or(&self.default_provider);

        let mut provider = self
            .providers
            .get(provider_name)
            .cloned()
            .or_else(|| default_providers_map().remove(provider_name))
            .with_context(|| {
                format!(
                    "find configuration for provider '{}'. Check your config.toml or specify a valid provider.",
                    provider_name
                )
            })?;

        if let Some(ref url) = provider.base_url {
            provider.base_url = Some(expand_string(url));
        }
        provider.model = expand_string(&provider.model);
        if let Some(ref key) = provider.api_key {
            provider.api_key = Some(expand_string(key));
        }

        Ok((provider_name.to_string(), provider))
    }
}

/// Helper function to interpolate `${VAR}` in a string.
fn expand_string(input: &str) -> String {
    let mut result = input.to_string();
    let prefix = "${";
    let suffix = "}";

    while let Some(start) = result.find(prefix) {
        if let Some(end) = result[start..].find(suffix) {
            let var_name = &result[start + prefix.len()..start + end];
            let val = std::env::var(var_name).unwrap_or_default();
            result.replace_range(start..=start + end, &val);
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_string_with_env_var() {
        unsafe {
            std::env::set_var("TEST_COGIT_KEY", "secret-token-123");
        }
        let expanded = expand_string("Bearer ${TEST_COGIT_KEY}");
        assert_eq!(expanded, "Bearer secret-token-123");
        unsafe {
            std::env::remove_var("TEST_COGIT_KEY");
        }
    }

    #[test]
    fn test_expand_string_missing_var_defaults_to_empty() {
        let expanded = expand_string("Bearer ${NON_EXISTENT_VAR_XYZ_123}");
        assert_eq!(expanded, "Bearer ");
    }

    #[test]
    fn test_config_parsing_from_toml() {
        let sample = r#"
            default_provider = "gemini"
            [providers.gemini]
            kind = "gemini"
            model = "gemini-3.5-flash-lite"
            api_key = "test-gemini-key"

            [providers.openai]
            kind = "openai"
            base_url = "https://api.openai.com/v1"
            model = "gpt-4o"
            api_key = "test-openai-key"

            [preferences]
            detailed = true
        "#;

        let parsed: Config = toml::from_str(sample).expect("parse sample TOML string");
        assert_eq!(parsed.default_provider, "gemini");
        let gemini = parsed
            .providers
            .get("gemini")
            .expect("gemini provider to exist");
        assert_eq!(gemini.model, "gemini-3.5-flash-lite");
        assert_eq!(gemini.resolve_kind("gemini"), ProviderKind::Gemini);

        let openai = parsed
            .providers
            .get("openai")
            .expect("openai provider to exist");
        assert_eq!(openai.model, "gpt-4o");
        assert_eq!(openai.resolve_kind("openai"), ProviderKind::OpenAi);
        assert!(parsed.preferences.detailed);
    }

    #[test]
    fn test_grok_and_groq_default_providers() {
        let providers = default_providers_map();
        assert!(providers.contains_key("grok"));
        assert!(providers.contains_key("groq"));

        let grok = providers.get("grok").unwrap();
        assert_eq!(grok.base_url.as_deref(), Some("https://api.x.ai/v1"));
        assert_eq!(grok.resolve_kind("grok"), ProviderKind::OpenAi);

        let groq = providers.get("groq").unwrap();
        assert_eq!(
            groq.base_url.as_deref(),
            Some("https://api.groq.com/openai/v1")
        );
        assert_eq!(groq.resolve_kind("groq"), ProviderKind::OpenAi);
    }
}
