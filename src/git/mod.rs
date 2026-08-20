// ==============================================================================
// Git Bridge Module
// ==============================================================================
//
// Provides safe, high-level interfaces to invoke git commands for inspecting
// staged changes and committing generated messages while respecting pre-commit
// hooks and GPG signing.

pub mod commit;
pub mod diff;

pub use commit::execute_commit;
pub use diff::{
    StagedDiff, get_staged_diff, get_staged_files, has_staged_changes, is_git_repository,
};
