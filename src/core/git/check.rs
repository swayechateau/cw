// core::git::check.rs
use git2::Repository;
use std::process::Command;

/// Check if git is not installed.
pub fn git_not_installed() -> bool {
    Command::new("git").arg("--version").output().is_err()
}

/// Gets the GitHub repository web URL
pub fn get_github_web_url() -> Option<String> {
    let repo = Repository::discover(".").ok()?;
    let remotes = repo.remotes().ok()?;
    let remote = remotes.iter().flatten().find(|r| *r == "origin")?;
    let remote_obj = repo.find_remote(remote).ok()?;
    let url = remote_obj.url()?.to_string();

    to_github_web_url(&url)
}

fn to_github_web_url(remote_url: &str) -> Option<String> {
    if remote_url.starts_with("git@github.com:") {
        Some(
            remote_url
                .replacen("git@github.com:", "https://github.com/", 1)
                .trim_end_matches(".git")
                .to_string(),
        )
    } else if remote_url.starts_with("https://github.com/") {
        Some(remote_url.trim_end_matches(".git").to_string())
    } else {
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_is_installed_on_most_machines() {
        // Assumes Git is available — adjust if you want to mock or skip in some environments
        assert!(!git_not_installed(), "Expected Git to be installed in the test environment.");
    }

    #[test]
    fn test_git_ssh_url_converted_to_https() {
        let ssh = "git@github.com:username/repo.git";
        let expected = "https://github.com/username/repo";
        assert_eq!(to_github_web_url(ssh).as_deref(), Some(expected));
    }

    #[test]
    fn test_git_https_url_is_trimmed() {
        let https = "https://github.com/username/repo.git";
        let expected = "https://github.com/username/repo";
        assert_eq!(to_github_web_url(https).as_deref(), Some(expected));
    }

    #[test]
    fn test_non_github_url_returns_none() {
        let custom = "https://gitlab.com/user/repo.git";
        assert_eq!(to_github_web_url(custom), None);
    }
}
