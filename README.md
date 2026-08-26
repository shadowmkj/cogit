# Cogit

<p align="center">
  <a href="https://crates.io/crates/cogit"><img src="https://img.shields.io/crates/v/cogit.svg?style=flat-square&logo=rust" alt="Crates.io Version"></a>
  <a href="https://github.com/shadowmkj/cogit/actions"><img src="https://img.shields.io/github/actions/workflow/status/shadowmkj/cogit/ci.yml?branch=main&style=flat-square&logo=github" alt="CI Build Status"></a>
  <a href="https://codecov.io/gh/shadowmkj/cogit"><img src="https://img.shields.io/codecov/c/github/shadowmkj/cogit?style=flat-square&logo=codecov" alt="Codecov Coverage"></a>
  <a href="https://docs.rs/cogit"><img src="https://img.shields.io/docsrs/cogit?style=flat-square&logo=docs.rs" alt="Documentation"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
</p>

Fast, lightweight CLI and TUI tool written in Rust that inspects staged Git diffs and generates compliant **Conventional Commits** using AI.

---

## ✨ Features

- **Conventional Commits**: Produces structured commits (`feat:`, `fix:`, `refactor:`, `chore:`, etc.).
- **Detailed Mode (`-d`)**: Generates a concise subject with a bulleted rationale.
- **Multiple Providers**: Out-of-the-box support for Google Gemini, OpenAI, Grok (xAI), Groq, and Ollama.
- **Interactive Review**: Lightweight CLI menu (`inquire`), full-screen dual-pane TUI (`ratatui`), and `$EDITOR` support.
- **Diff Sanitization**: Strips noisy lockfiles and binary blobs while truncating cleanly at newline boundaries.
- **Safe Execution**: Preserves Git pre-commit hooks, GPG/SSH commit signatures, and terminal raw mode.

---

## 🚀 Quick Start

### Installation

**Via Shell Script (macOS / Linux):**
```bash
curl -proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/shadowmkj/cogit/main/scripts/install.sh | sh
```

**Via Homebrew (macOS / Linux):**
```bash
brew tap shadowmkj/tap
brew install cogit
```

**Via PowerShell (Windows):**
```powershell
irm https://raw.githubusercontent.com/shadowmkj/cogit/main/scripts/install.ps1 | iex
```

**Via Cargo:**
```bash
cargo install cogit
```

### Usage

Stage your changes and invoke `cogit`:

```bash
git add .
cogit
```

#### Common Options

| Option | Description |
| :--- | :--- |
| `-d, --detailed` | Generate structured commit with bulleted details |
| `-e, --edit` | Open message in `$EDITOR` before committing |
| `--tui` | Launch full-screen interactive dual-pane TUI |
| `--dry-run` | Print generated commit message to stdout without committing |
| `-p, --provider <NAME>` | Select active provider (`gemini`, `openai`, `grok`, `groq`, `ollama`) |
| `-m, --model <MODEL>` | Override LLM model name |
| `--prompt <HINT>` | Supply additional guidance or context for message generation |
| `--init-config` | Initialize default `~/.config/cogit/config.toml` |

#### Workflow Automation Subcommands

##### 1. Git Hook Integration (`cogit hook`)
Install `prepare-commit-msg` to automatically populate commit message drafts when using standard `git commit`:

```bash
cogit hook install     # Install hook in current git repository
cogit hook uninstall   # Remove hook from current git repository
```

##### 2. Pull Request Description Generator (`cogit pr`)
Generate structured Markdown PR titles, summaries, and testing checklists by comparing your branch against the base branch:

```bash
cogit pr               # Generate PR description against base branch (main/master)
cogit pr --base dev    # Compare against a custom base branch
cogit pr --copy        # Copy generated PR description to system clipboard
cogit pr --create      # Directly create PR on GitHub via `gh pr create`
```

##### 3. Conventional Branch Name Generator (`cogit branch`)
Generate clean, kebab-case conventional branch names based on your working diff or task description:

```bash
cogit branch                              # Suggest branch names from working tree changes
cogit branch "implement oauth2 flow"      # Suggest branch names from task description
cogit branch "fix token leak" -c          # Interactively pick and checkout the branch
```

---

## 🔌 Supported Providers

| Provider | Default Model | Authentication |
| :--- | :--- | :--- |
| **Google Gemini** | `gemini-3.5-flash-lite` | `GEMINI_API_KEY` |
| **OpenAI** | `gpt-4o-mini` | `OPENAI_API_KEY` |
| **Grok (xAI)** | `grok-2-latest` | `XAI_API_KEY` |
| **Groq** | `qwen3.6-27b` | `GROQ_API_KEY` |
| **Ollama** | `qwen2.5-coder:7b` | Local (`http://localhost:11434/v1`) |

---

## ⚙️ Configuration

Initialize the starter configuration file:

```bash
cogit --init-config
```

Location: `~/.config/cogit/config.toml`

```toml
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
```

---

## 📄 License

Licensed under the [MIT License](LICENSE).
