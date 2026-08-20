# Contributing to Cogit

Thank you for your interest in contributing to **Cogit**! We welcome bug reports, feature suggestions, documentation improvements, and pull requests.

---

## Code of Conduct

Please be respectful, constructive, and collaborative in all issues, pull requests, and discussions.

---

## How to Contribute

### 1. Reporting Bugs & Feature Requests
- Check the [Issues](../../issues) tab to see if the issue or idea has already been reported.
- Use our [Bug Report](.github/ISSUE_TEMPLATE/bug_report.md) or [Feature Request](.github/ISSUE_TEMPLATE/feature_request.md) templates to submit clear and reproducible reports.

### 2. Submitting Pull Requests
- Fork the repository and create a feature branch (`git checkout -b feat/your-feature-name`).
- Keep PRs focused on a single change or fix.
- Ensure all tests pass and your code conforms to the project's formatting and linting standards before opening a PR.

---

## Development Setup

### Prerequisites
- **Rust:** Stable compiler (edition 2024 support). Install via [rustup.rs](https://rustup.rs).
- **Git:** Installed and available in your `PATH`.

### Building & Running

```bash
# Build debug binary
cargo build

# Run with staged changes
git add <files>
cargo run -- [flags]
```

### Quality & Verification Checks

Before submitting a PR, verify that all quality gates pass:

```bash
# 1. Run unit tests
cargo test

# 2. Check code formatting
cargo fmt --check

# 3. Run Clippy with strict warnings
cargo clippy --all-targets -- -D warnings
```

---

## Commit Guidelines

We practice what we preach! Please write commit messages following the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` A new feature
- `fix:` A bug fix
- `docs:` Documentation changes
- `refactor:` Code refactoring without behavioral change
- `perf:` Performance improvements
- `test:` Adding or updating tests
- `chore:` Build/dependency updates or maintenance

*Tip: You can use `cogit` itself to generate your commit messages!*

---

## License

By contributing to Cogit, you agree that your contributions will be licensed under the [MIT License](LICENSE).
