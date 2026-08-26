# ADR-002: Staged Diff Sanitization and Token Bounds

## Status
Accepted

## Date
2026-08-20

## Context
When developers stage large dependency lockfiles (`Cargo.lock`, `package-lock.json`, `pnpm-lock.yaml`, `go.sum`, etc.) or binary files, the raw unified diff can easily exceed tens of thousands of lines. Sending these large diffs directly to LLMs causes:
1. Context window exhaustion and excessive API token consumption.
2. Distortion of LLM attention away from the actual business logic changes.
3. Errors on binary patches that cannot be processed meaningfully by text models.

## Decision
Implement a pre-prompt sanitization layer in `src/git/diff.rs` that:
1. Parses unified diff headers (`diff --git a/... b/...`).
2. Identifies and strips known noisy lockfiles and binary diff hunks from the text sent to the LLM.
3. Appends an `# Omitted files summary` to inform the LLM that lockfiles were updated without flooding the prompt with raw hash changes.
4. Truncates diffs exceeding the character limit (default 32,000 chars) strictly at newline boundaries, appending an explicit truncation notice.

## Alternatives Considered

### Unfiltered Raw Diff
- **Pros:** Trivial implementation.
- **Cons:** Triggers token limit errors on routine dependency updates and degrades commit message quality.
- **Rejected:** Lockfiles add noise with minimal semantic intent.

### Git Pathspec Exclusion at CLI Invocation (`git diff -- . ':(exclude)Cargo.lock'`)
- **Pros:** Git filters the diff before stdout.
- **Cons:** Prevents the tool from knowing which lockfiles were updated and reporting them in the prompt metadata.
- **Rejected:** In-memory parsing allows generating an omitted files summary so the LLM knows a lockfile changed without reading the full diff.

## Consequences
- Clean, focused prompts for the LLM.
- Fast performance with zero external dependencies.
- Preservation of line and UTF-8 integrity during truncation.
