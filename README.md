# Cogit

Fast, lightweight CLI and TUI tool written in Rust that inspects staged Git diffs and generates **Conventional Commits** using AI (Google Gemini, OpenAI, Grok, Groq, and Ollama).

## Features

- **Conventional Commits**: Produces compliant messages (`feat:`, `fix:`, `refactor:`, `chore:`, etc.).
- **Detailed Mode (`-d`)**: Generates a subject line with bulleted rationale.
- **Multiple Providers**: First-class support for Google Gemini, OpenAI, Grok (xAI), Groq, and Ollama.
- **Interactive Review & TUI**: Lightweight CLI menu (`inquire`), full visual dual-pane TUI (`ratatui`), and external `$EDITOR` support.
- **Safe Commits**: Preserves pre-commit hooks and GPG/SSH commit signing.

## Quick Start

### Build & Install

```bash
cargo install --path .
```

### Usage

Stage your changes and run `cogit`:

```bash
git add .
cogit
```

#### Flags

- `cogit -d, --detailed`: Generate structured commit with bulleted details.
- `cogit --tui`: Open full-screen dual-pane Terminal UI.
- `cogit --dry-run`: Print commit message to stdout without committing.
- `cogit -p, --provider <NAME>`: Override active provider (`gemini`, `openai`, `grok`, `groq`, `ollama`).
- `cogit -m, --model <MODEL>`: Override model name (e.g. `gemini-3.5-flash-lite`, `grok-2-latest`, `llama-3.3-70b-versatile`).
- `cogit --prompt <HINT>`: Pass extra context or instructions.
- `cogit --init-config`: Generate default `~/.config/cogit/config.toml`.

## Configuration

Initialize the default configuration:

```bash
cogit --init-config
```

Location: `~/.config/cogit/config.toml`

```toml
default_provider = "gemini"

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
```

## License

MIT OR Apache-2.0
