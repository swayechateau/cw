use crate::{
    config::load_config,
    git::{collect_commits_in_range, get_latest_tag, get_repo, next_semantic_version},
};
use anyhow::bail;
use std::process::Command;

#[derive(clap::Args)]
pub struct BumpArgs {
    /// Run in CI mode, only output the new version
    #[arg(long)]
    ci: bool,
    /// Create a new git tag
    #[arg(long)]
    tag: bool,
    /// Push the created git tag to the remote
    #[arg(long)]
    push: bool,
    /// The current version to bump from
    #[arg(long)]
    from: Option<String>,
    /// The version to bump to
    #[arg(long, default_value = "HEAD")]
    to: String,
}

pub fn run(args: BumpArgs) -> anyhow::Result<()> {
    let BumpArgs {
        ci,
        tag,
        push,
        mut from,
        to: to_arg,
    } = args;

    let config = load_config();
    let repo = get_repo();

    // Step 1: Determine `to` ref
    let to = if to_arg != "HEAD" {
        to_arg
    } else if let Some(ref default_branch) = config.default.branch {
        default_branch.clone()
    } else {
        "main".to_string()
    };

    // Step 2: Determine `from` ref
    if from.is_none() {
        from = get_latest_tag().or(Some("v0.0.0".to_string()));
    }

    // Step 3: Check if there are commits to consider
    let changes = collect_commits_in_range(from.clone(), to.clone())?;
    if changes.is_empty() {
        println!(
            "📭 No changes since {}. No new tag needed.",
            from.clone().unwrap_or_default()
        );
        return Ok(());
    }

    // Step 4: Determine semantic version bump
    let new_version = match next_semantic_version(from.clone(), to.clone())? {
        Some(version) => version,
        None => {
            if !ci {
                println!("📦 No version bump needed (no relevant commits).");
            }
            return Ok(());
        }
    };

    println!("🔢 Next version: {}", new_version);

    // Step 5: Optionally create a Git tag
    if tag {
        let tag_names = repo.tag_names(None)?;
        if tag_names.iter().flatten().any(|t| t == new_version) {
            bail!("❌ Tag `{}` already exists.", new_version);
        }

        let obj = repo.revparse_single("HEAD")?;
        let sig = repo.signature()?;
        repo.tag(
            &new_version,
            &obj,
            &sig,
            &format!("Release {}", new_version),
            false,
        )?;
        println!("🏷️  Created Git tag `{}`", new_version);

        if push {
            let status = Command::new("git")
                .args(["push", "origin", &new_version])
                .status()?;

            if status.success() {
                println!("🚀 Pushed tag `{}` to origin", new_version);
            } else {
                bail!("❌ Failed to push tag `{}`.", new_version);
            }
        }
    } else if push {
        println!("⚠️  Ignored `--push` because `--tag` was not set.");
    }

    if ci {
        println!("{}", new_version);
    }

    Ok(())
}
