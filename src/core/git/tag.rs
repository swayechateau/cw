use chrono::{DateTime, Utc};
use git2::{ObjectType, Repository};

/// Lists all Git tags sorted by creation date.
pub fn get_all_tags() -> anyhow::Result<Vec<String>> {
    let repo = Repository::discover(".")?;
    let tag_names = repo.tag_names(None)?;
    Ok(tag_names.iter().flatten().map(|s| s.to_string()).collect())
}

/// Gets the most recent Git tag.
pub fn get_latest_tag() -> Option<String> {
    let repo = Repository::discover(".").ok()?;
    let tags = repo.tag_names(None).ok()?;
    let mut latest: Option<(String, git2::Time)> = None;

    for tag_name in tags.iter().flatten() {
        let tag_ref = format!("refs/tags/{}", tag_name);
        if let Ok(reference) = repo.find_reference(&tag_ref) {
            if let Ok(target) = reference.peel(ObjectType::Commit) {
                if let Some(commit) = target.as_commit() {
                    let time = commit.time();
                    match &latest {
                        Some((_, prev_time)) if time.seconds() > prev_time.seconds() => {
                            latest = Some((tag_name.to_string(), time));
                        }
                        None => {
                            latest = Some((tag_name.to_string(), time));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    latest.map(|(tag, _)| tag)
}

/// Gets the latest Git tag or "Unreleased" if none exists.
pub fn latest_tag_or_unreleased() -> String {
    get_latest_tag().unwrap_or_else(|| "Unreleased".into())
}

/// Returns ISO-8601 tag date in `YYYY-MM-DD` format.
pub fn get_tag_commit_date(tag: &str) -> Option<String> {
    let repo = Repository::discover(".").ok()?;
    let tag_ref = format!("refs/tags/{}", tag);
    let reference = repo.find_reference(&tag_ref).ok()?;
    let obj = reference.peel(ObjectType::Commit).ok()?;
    let commit = obj.as_commit()?;

    let time = commit.time();
    let timestamp = time.seconds();
    let dt = DateTime::<Utc>::from_timestamp(timestamp, 0)?;
    Some(dt.format("%Y-%m-%d").to_string())
}
