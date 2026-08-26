# Project Guidelines & Rules: Cogit

## 1. Strict Source Code Protection in Plan Mode
- Never create, edit, or delete any source code files while in planning mode.
- Planning mode is strictly read-only for source code.
- Only planning documents (`tasks/plan.md`, `tasks/todo.md`, and planning artifacts) may be written during planning.
- Execution requires explicit user confirmation.

## 2. Strict Dependency Version Protection
- **Never change dependency versions without permission**: The agent MUST NEVER add, upgrade, downgrade, remove, or modify any dependency versions in `Cargo.toml` or `Cargo.lock` without explicit knowledge and permission from the user.
- **Preserve exact pinned versions**: Always retain existing dependencies and version strings as-is.
- **Explicit Request Required**: Propose any required dependency changes first and await explicit permission before touching `Cargo.toml`.
