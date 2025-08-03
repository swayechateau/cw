use crate::{
    config::{load_config, versioning::CWVersioning},
    git::{commit::parse_commit_message, tag::get_latest_tag},
};
use anyhow::Result;
use git2::{Oid, Repository, Revwalk};

/// Calculates the next semantic version based on commit history.
pub fn next_semantic_version(from: Option<String>, to: String) -> Result<Option<String>> {
    let repo = Repository::discover(".")?;
    let config = load_config();

    let binding = config.types.all();
    let allowed_types: Vec<&str> = binding.keys().map(|k| k.as_str()).collect();
    let versioning = &config.versioning;
    let breaking_keywords = &config.breaking.keywords;

    let (from_oid, to_oid) = resolve_commit_range(&repo, &from, &to)?;
    let mut revwalk = prepare_revwalk(&repo, from_oid, to_oid)?;

    let bump_type = determine_bump_type(
        &repo,
        &mut revwalk,
        &allowed_types,
        &versioning,
        &breaking_keywords,
    )?;

    match bump_type {
        Some(bt) => Ok(Some(bump_version(
            &bt,
            &get_latest_tag().unwrap_or_else(|| "0.0.0".to_string()),
        ))),
        None => Ok(None),
    }
}

fn resolve_commit_range(
    repo: &Repository,
    from: &Option<String>,
    to: &str,
) -> Result<(Option<Oid>, Oid)> {
    let to_oid = repo.revparse_single(to)?.id();
    let from_oid = match from {
        Some(from_ref) => Some(repo.revparse_single(from_ref)?.id()),
        None => None,
    };
    Ok((from_oid, to_oid))
}

fn prepare_revwalk(repo: &Repository, from_oid: Option<Oid>, to_oid: Oid) -> Result<Revwalk> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push(to_oid)?;
    if let Some(oid) = from_oid {
        revwalk.hide(oid)?;
    }
    Ok(revwalk)
}

fn determine_bump_type(
    repo: &Repository,
    revwalk: &mut Revwalk,
    allowed_types: &[&str],
    versioning: &CWVersioning,
    breaking_keywords: &[String],
) -> anyhow::Result<Option<String>> {
    for oid in revwalk {
        let commit = repo.find_commit(oid?)?;
        let message = commit.message().unwrap_or_default();

        if is_breaking_change(message, breaking_keywords) {
            return Ok(Some("major".into()));
        }

        if let Some((commit_type, _, _)) = parse_commit_message(message, allowed_types) {
            if versioning.minor.contains(&commit_type) {
                return Ok(Some("minor".into()));
            } else if versioning.patch.contains(&commit_type) {
                return Ok(Some("patch".into()));
            }
        }
    }
    Ok(None)
}

fn is_breaking_change(message: &str, keywords: &[String]) -> bool {
    keywords.iter().any(|kw| message.contains(kw)) || message.contains("!:")
}

fn bump_version(bump_type: &str, current_version: &str) -> String {
    let mut parts: Vec<u32> = current_version
        .trim_start_matches('v')
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    parts.resize(3, 0);

    match bump_type {
        "major" => {
            parts[0] += 1;
            parts[1] = 0;
            parts[2] = 0;
        }
        "minor" => {
            parts[1] += 1;
            parts[2] = 0;
        }
        "patch" => {
            parts[2] += 1;
        }
        _ => {}
    }

    format!("v{}.{}.{}", parts[0], parts[1], parts[2])
}
