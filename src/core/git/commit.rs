use git2::{Repository, Sort};
use regex::Regex;

use crate::{config::load_config, git::repo::get_repo};

/// Performs a Git commit with the given message.
pub fn perform_commit(message: &str) -> anyhow::Result<()> {
    let repo = Repository::discover(".")?;
    let sig = repo.signature()?;
    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    let parent_commit = repo
        .head()
        .ok()
        .and_then(|h| h.target())
        .and_then(|oid| repo.find_commit(oid).ok());

    let parents = match parent_commit {
        Some(ref p) => vec![p],
        None => vec![],
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().map(|p| *p).collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)?;

    Ok(())
}

/// Validates a commit message against conventional commits.
pub fn validate_conventional_commit_message(message: &str, allowed_types: &[&str]) -> bool {
    let pattern = format!(r"^({})(\([\w\-]+\))?(!)?: .+", allowed_types.join("|"));
    Regex::new(&pattern).unwrap().is_match(message.trim())
}

/// Parses a conventional commit message.
pub fn parse_commit_message(
    msg: &str,
    allowed_types: &[&str],
) -> Option<(String, Option<String>, String)> {
    let pattern = format!(
        r"^(?P<type>{})(\((?P<scope>[^)]+)\))?(?P<bang>!)?: (?P<summary>.+)",
        allowed_types.join("|")
    );
    Regex::new(&pattern).ok()?.captures(msg).and_then(|caps| {
        Some((
            caps.name("type")?.as_str().to_string(),
            caps.name("scope").map(|s| s.as_str().to_string()),
            caps.name("summary")?.as_str().to_string(),
        ))
    })
}

/// Collects all commits in a given range.
pub fn collect_commits_in_range(
    from: Option<String>,
    to: String,
) -> anyhow::Result<Vec<(String, String)>> {
    let repo = Repository::discover(".")?;
    let to_oid = repo.revparse_single(&to)?.id();
    let from_oid = match from {
        Some(ref f) => Some(repo.revparse_single(f)?.id()),
        None => None,
    };

    let mut revwalk = repo.revwalk()?;
    revwalk.push(to_oid)?;
    if let Some(from_id) = from_oid {
        revwalk.hide(from_id)?;
    }
    revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;

    let mut commits = Vec::new();
    for oid in revwalk {
        let commit = repo.find_commit(oid?)?;
        let hash = commit.id().to_string();
        let msg = commit.summary().unwrap_or("").to_string();
        commits.push((hash, msg));
    }

    Ok(commits)
}

pub fn find_invalid_commits() -> anyhow::Result<Vec<(String, String)>> {
    let config = load_config();
    let binding = config.types.all();
    let allowed_types: Vec<&str> = binding.keys().map(|k| k.as_str()).collect();

    let repo = get_repo();
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut invalid_commits = Vec::new();

    let mut oids: Vec<_> = revwalk.flatten().collect();
    oids.reverse(); // Oldest commits first

    for oid in oids {
        let commit = repo.find_commit(oid)?;
        let hash = oid.to_string(); // full hash
        let message = commit.message().unwrap_or("").trim().to_string();

        if !validate_conventional_commit_message(&message, &allowed_types) {
            invalid_commits.push((hash, message));
        }
    }

    Ok(invalid_commits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_conventional_commit_messages() {
        let allowed_types = &["feat", "fix", "docs"];

        let messages = [
            "feat: add login page",
            "fix: correct spelling error",
            "docs(readme): update usage section",
            "feat!: drop deprecated endpoint",
        ];

        for msg in messages {
            assert!(
                validate_conventional_commit_message(msg, allowed_types),
                "Expected message to be valid: '{}'",
                msg
            );
        }
    }

    #[test]
    fn test_invalid_conventional_commit_messages() {
        let allowed_types = &["feat", "fix"];

        let messages = [
            "feature: typo",                      // wrong type
            "feat (ui): spacing issue",           // space before scope
            "fix add missing check",              // missing colon
            "chore: bump deps",                   // disallowed type
            "feat: ",                              // no summary
        ];

        for msg in messages {
            assert!(
                !validate_conventional_commit_message(msg, allowed_types),
                "Expected message to be invalid: '{}'",
                msg
            );
        }
    }

    #[test]
    fn test_parse_commit_message() {
        let allowed_types = &["feat", "fix", "chore"];

        let input = "feat(account): add logout button";
        let parsed = parse_commit_message(input, allowed_types);

        assert!(parsed.is_some());
        let (typ, scope, summary) = parsed.unwrap();

        assert_eq!(typ, "feat");
        assert_eq!(scope, Some("account".to_string()));
        assert_eq!(summary, "add logout button");
    }

    #[test]
    fn test_parse_commit_message_no_scope() {
        let allowed_types = &["feat"];

        let input = "feat: standalone commit";
        let parsed = parse_commit_message(input, allowed_types);

        assert!(parsed.is_some());
        let (typ, scope, summary) = parsed.unwrap();

        assert_eq!(typ, "feat");
        assert_eq!(scope, None);
        assert_eq!(summary, "standalone commit");
    }

    #[test]
    fn test_parse_commit_message_invalid() {
        let allowed_types = &["feat"];

        let input = "fix(readme): not allowed type";
        let parsed = parse_commit_message(input, allowed_types);
        assert!(parsed.is_none());
    }
}
