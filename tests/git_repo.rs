use std::env;
use std::path::PathBuf;
use cw::git::repo::{get_repo_root, get_current_branch};

#[test]
fn test_get_repo_root_returns_correct_path() {
    let expected = find_git_root_dir().expect("this should be a git repo");
    let actual = get_repo_root().expect("should return repo root");
    assert_eq!(actual, expected);
}

#[test]
fn test_get_current_branch_returns_a_branch() {
    let branch = get_current_branch();
    assert!(
        branch.is_ok(),
        "Expected to retrieve current branch, got error: {:?}",
        branch.err()
    );

    let name = branch.unwrap();
    assert!(
        !name.is_empty(),
        "Branch name should not be empty"
    );
}

/// Helper: finds the real Git root by traversing up to `.git`
fn find_git_root_dir() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    loop {
        if dir.join(".git").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}
