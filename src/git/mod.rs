// ==============================================================================
// Git Bridge Module
// ==============================================================================
//
// Provides safe, high-level interfaces to invoke git commands for inspecting
// staged changes and committing generated messages while respecting pre-commit
// hooks and GPG signing.

pub mod branch_context;
pub mod commit;
pub mod diff;
pub mod hooks;

pub use branch_context::{
    checkout_new_branch, get_branch_commits, get_branch_diff, get_default_base_branch,
    get_working_tree_diff,
};
pub use commit::execute_commit;
pub use diff::{
    StagedDiff, get_staged_diff, get_staged_files, has_staged_changes, is_git_repository,
};
pub use hooks::{
    install_prepare_commit_msg_hook, is_prepare_commit_msg_hook_installed,
    uninstall_prepare_commit_msg_hook,
};
