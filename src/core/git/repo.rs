use git2::Repository;
use std::path::PathBuf;

/// Gets the current Git repository.
pub fn get_repo() -> Repository {
    Repository::discover(".").expect("Failed to discover git repository")
}

/// Gets the root directory of the current Git repository.
pub fn get_repo_root() -> Option<PathBuf> {
    let repo = get_repo();
    repo.workdir().map(PathBuf::from)
}

/// Get the current branch name from the Git repository.
pub fn get_current_branch() -> anyhow::Result<String> {
    let repo = get_repo();
    let head = repo.head()?;
    if head.is_branch() {
        head.shorthand()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Failed to get branch name"))
    } else {
        anyhow::bail!("HEAD is not pointing to a branch");
    }
}
