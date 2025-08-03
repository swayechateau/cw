use std::process::{Command};

/// Pushes the current branch to the specified remote.
pub fn git_push(remote: &str, branch: &str) -> anyhow::Result<()> {
    git_push_with(remote, branch, run_git_push)
}

pub fn git_push_with<F>(remote: &str, branch: &str, runner: F) -> anyhow::Result<()>
where
    F: Fn(&str, &str) -> std::io::Result<std::process::ExitStatus>,
{
    let status = runner(remote, branch)?;

    if status.success() {
        println!("✅ Pushed to {} {}", remote, branch);
        Ok(())
    } else {
        eprintln!("❌ Push failed.");
        anyhow::bail!("❌ Push failed.")
    }
}

/// Executes a git push and returns its exit status.
fn run_git_push(remote: &str, branch: &str) -> std::io::Result<std::process::ExitStatus> {
    Command::new("git")
        .args(["push", remote, branch])
        .status()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::ExitStatus;
    use std::os::unix::process::ExitStatusExt; // For from_raw on Unix

    fn mock_success(_: &str, _: &str) -> std::io::Result<ExitStatus> {
        Ok(ExitStatus::from_raw(0)) // Exit code 0 = success
    }

    fn mock_failure(_: &str, _: &str) -> std::io::Result<ExitStatus> {
        Ok(ExitStatus::from_raw(1 << 8)) // Exit code 1 = failure
    }

    #[test]
    fn test_git_push_successful() {
        let result = git_push_with("origin", "main", mock_success);
        assert!(result.is_ok(), "Expected git_push_with to succeed");
    }

    #[test]
    fn test_git_push_failure_returns_error() {
        let result = git_push_with("origin", "main", mock_failure);
        assert!(result.is_err(), "Expected git_push_with to return error on failure");
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "❌ Push failed.",
            "Error message should match"
        );
    }
}
