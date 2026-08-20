// ==============================================================================
// LLM Integration Module
// ==============================================================================
//
// Provides prompt generation, multi-provider API abstractions (Gemini, OpenAI,
// Ollama, mock testing), and response cleaning.

pub mod cleaner;
pub mod client;
pub mod prompt;
pub mod providers;

pub use cleaner::clean_commit_message;
pub use client::LlmClient;
pub use prompt::build_prompt;
pub use providers::{GeminiProvider, LlmProvider, MockProvider, OpenAiProvider, create_provider};
