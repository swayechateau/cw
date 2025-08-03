use std::collections::{BTreeMap, HashMap};

use git2::{Oid, Repository};

use crate::{config::changelog::Changelog, git::repo::get_repo};

// Main function remains the same
pub fn generate_changelog() -> anyhow::Result<BTreeMap<String, Vec<(String, String)>>> {
    let repo = get_repo();
    let revwalk = setup_revwalk(&repo)?;
    let commit_to_tag = map_commits_to_tags(&repo)?;

    let mut grouped: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let mut current_tag = "Unreleased".to_string();

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let hash = oid.to_string()[..7].to_string();
        let message = commit.summary().unwrap_or("").to_string();

        if let Some(tag) = commit_to_tag.get(&oid) {
            current_tag = tag.clone();
        }

        grouped
            .entry(current_tag.clone())
            .or_default()
            .push((hash, message));
    }

    Ok(grouped)
}

// Separated function for revwalk setup
fn setup_revwalk(repo: &Repository) -> Result<git2::Revwalk, git2::Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
    Ok(revwalk)
}

// Separated function for tag -> commit OID mapping
fn map_commits_to_tags(repo: &Repository) -> Result<HashMap<Oid, String>, git2::Error> {
    let mut commit_to_tag = HashMap::new();
    let tag_names = repo.tag_names(None)?;
    for tag in tag_names.iter().flatten() {
        if let Ok(reference) = repo.find_reference(&format!("refs/tags/{}", tag)) {
            if let Ok(target) = reference.peel_to_commit() {
                commit_to_tag.insert(target.id(), tag.to_string());
            }
        }
    }
    Ok(commit_to_tag)
}

/// Render a single changelog version section in Keep a Changelog format.
///
/// Example:
/// ```markdown
/// ## [1.2.0] - 2025-07-22
///
/// ### Added
/// - Add login endpoint (abc123)
///
/// ### Fixed
/// - Correct typo in UI (def456)
/// ```
pub fn render_markdown(
    version: &str,
    grouped_commits: &BTreeMap<String, Vec<(String, Option<String>, String)>>,
    group_by_scope: bool,
    date: &str,
    config: &Changelog,
    github_url: Option<&str>,
) -> String {
    let section_map = config.sections.group_commits_by_section(grouped_commits);

    let mut output = format!("## [{}] - {}\n", version, date);

    for (section, entries) in section_map {
        output.push_str(&format!("\n### {}\n", section));
        if group_by_scope {
            output.push_str(&render_grouped_by_scope(&entries, github_url));
        } else {
            output.push_str(&render_flat(&entries));
        }
    }

    output.push('\n');
    output
}

// Renders grouped entries by scope
fn render_grouped_by_scope(
    entries: &[(Option<String>, String, String)],
    github_url: Option<&str>,
) -> String {
    let mut scoped: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

    for (scope, summary, hash) in entries {
        let scope_key = scope.clone().unwrap_or_else(|| "unspecified".to_string());
        scoped
            .entry(scope_key)
            .or_default()
            .push((summary.clone(), hash.clone()));
    }

    let mut result = String::new();

    if let Some(unspecified) = scoped.remove("unspecified") {
        for (msg, hash) in unspecified {
            result.push_str(&format!("- {} ({})\n", msg, &hash[..7]));
        }
    }

    for (scope, lines) in scoped {
        result.push_str(&format!("\n#### {}\n", scope));
        for (msg, hash) in lines {
            let mut line = format!("- {}", msg);
            if let Some(linked_line) = link_pull_request(&msg, &hash, github_url) {
                line = linked_line;
            }
            line.push('\n');
            result.push_str(&line);
        }
    }

    result
}

// Flat render (no scope grouping)
fn render_flat(entries: &[(Option<String>, String, String)]) -> String {
    let mut result = String::new();

    for (scope, msg, hash) in entries {
        let line = if let Some(scope) = scope {
            format!("- **{}**: {} ({})", scope, msg, &hash[..7])
        } else {
            format!("- {} ({})", msg, &hash[..7])
        };
        result.push_str(&line);
        result.push('\n');
    }

    result
}

// Extracts PR number and returns formatted line with PR + commit link
fn link_pull_request(msg: &str, hash: &str, github_url: Option<&str>) -> Option<String> {
    if let Some((before, pr_str)) = msg.rsplit_once("(#") {
        if let Ok(pr_num) = pr_str.trim_end_matches(')').parse::<u32>() {
            if let Some(url) = github_url {
                return Some(format!(
                    "- {} ([#{}]({}/pull/{})) ([{}]({}/commit/{}))",
                    before.trim_end(),
                    pr_num,
                    url,
                    pr_num,
                    &hash[..7],
                    url,
                    hash
                ));
            }
        }
    }

    github_url.map(|url| format!("- {} ([{}]({}/commit/{}))", msg, &hash[..7], url, hash))
}
