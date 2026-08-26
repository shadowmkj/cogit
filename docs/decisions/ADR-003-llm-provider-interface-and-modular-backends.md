# ADR-003: LLM Provider Interface and Modular Backends

## Status
Accepted

## Date
2026-08-20

## Context
As `cogit` expands its support to multiple model families (Google Gemini, OpenAI, Ollama, Groq, Mistral, and local/mock test environments), hardcoding API request logic into a single monolithic client causes:
1. Tight coupling to specific API request/response structures.
2. Difficulty writing deterministic unit and integration tests without network access.
3. Code duplication when adding future backends (e.g. Anthropic, AWS Bedrock, Ollama native).

## Decision
1. Group all provider implementations into a dedicated `src/llm/providers/` folder.
2. Define a common trait `LlmProvider`:
   ```rust
   pub trait LlmProvider: Send + Sync {
       fn name(&self) -> &str;
       fn generate_commit<'a>(
           &'a self,
           client: &'a reqwest::Client,
           system_prompt: &'a str,
           user_prompt: &'a str,
       ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>>;
   }
   ```
3. Implement `GeminiProvider` for Google Gemini native REST API (`v1beta/generateContent`).
4. Implement `OpenAiProvider` for OpenAI-compatible `/v1/chat/completions` endpoints.
5. Implement `MockProvider` for deterministic testing.
6. Provide a factory function `create_provider(name, config) -> Box<dyn LlmProvider>` for runtime polymorphism.

## Alternatives Considered

### Direct Enum Matching in `LlmClient`
- **Pros:** No dynamic dispatch / `Box<dyn LlmProvider>`.
- **Cons:** Adding a new provider requires modifying central match arms in `client.rs`, creating high coupling and making mock test injection harder.
- **Rejected:** Trait-based polymorphism allows plug-and-play addition of new providers and easy test mocking.

## Consequences
- Modular, decoupled codebase where each provider lives in its own file (`src/llm/providers/<provider>.rs`).
- `MockProvider` allows fast, deterministic testing without live API keys or network latency.
- Future providers (Anthropic, Bedrock, etc.) can be added by implementing `LlmProvider` without touching existing provider logic.
