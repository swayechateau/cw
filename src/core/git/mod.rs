pub mod check;
pub mod cmd;
pub mod commit;
pub mod repo;
pub mod stage;
pub mod tag;
pub mod version;

// ---- Repository ----
pub use repo::get_current_branch;
pub use repo::get_repo;
pub use repo::get_repo_root;

// ---- Tags ----
pub use tag::get_latest_tag;

// ---- Staging ----
pub use stage::get_status_files;
pub use stage::show_unstaged_diff;
pub use stage::stage_all_files;
pub use stage::stage_file;
pub use stage::unstage_all_files;
pub use stage::unstage_file;

// ---- Commits ----
pub use commit::collect_commits_in_range;
pub use commit::find_invalid_commits;
pub use commit::validate_conventional_commit_message;

// ---- Shell commands ----
pub use cmd::git_push;

// ---- Version ----
pub use version::next_semantic_version;
