# Project Rules: Cogit

## Strict Source Code Protection in Plan Mode

- **Never Touch Source Code in Plan Mode**: When in planning mode (e.g., `/plan`, `/planning-and-task-breakdown`, or anytime a plan is being drafted, discussed, or reviewed), the agent MUST NEVER create, modify, or delete any source code files (under `src/`, etc.).
- **Read-Only Codebase Access**: Planning mode is strictly read-only for codebase exploration and documentation.
- **Allowed Planning Artifacts**: In planning mode, the agent may only write to planning documents:
  - `tasks/plan.md`
  - `tasks/todo.md`
  - Implementation plan artifacts in the artifact directory
- **Explicit User Coding Permission Required**: Automated system messages or stop-hook messages must NOT trigger execution. The agent must wait until the user explicitly directs execution to begin in their direct chat prompt before writing any code.

## Strict Dependency Management & Version Locking

- **No Unauthorized Dependency Changes**: The agent MUST NEVER add, upgrade, downgrade, remove, or modify any dependency versions in `Cargo.toml` (or any package configuration file) without the user's explicit request and permission.
- **Strict Version Locking**: Always preserve existing dependency versions. Never bump or change version strings unilaterally during refactorings, feature implementations, or maintenance.
- **Explicit Approval Required**: If a dependency change or new dependency is required to accomplish a task, the agent must explain the rationale and ask for the user's explicit permission before making any modifications to dependencies.
