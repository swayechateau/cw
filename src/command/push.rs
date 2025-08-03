use crate::{
    config::load_config,
    git::{
        collect_commits_in_range, get_current_branch, git_push,
        validate_conventional_commit_message,
    },
};
use anyhow::Result;
use clap::Args;
use std::process::exit;

#[derive(Args)]
pub struct PushArgs {
    /// Start commit (e.g. tag or hash). Defaults to last tag.
    #[arg(long)]
    pub from: Option<String>,

    /// End commit (default: HEAD)
    #[arg(long, default_value = "HEAD")]
    pub to: String,

    /// Push destination (default: origin)
    #[arg(long, default_value = "origin")]
    pub remote: String,

    /// Branch to push (default: current branch)
    #[arg(long)]
    pub branch: Option<String>,
}

pub fn run(args: PushArgs) -> Result<()> {
    let config = load_config();
    let binding = config.types.all();
    let allowed_types: Vec<&str> = binding.keys().map(|k| k.as_str()).collect();

    let commits = collect_commits_in_range(args.from.clone(), args.to.clone())?;
    let mut invalid = vec![];

    for (hash, msg) in &commits {
        if !validate_conventional_commit_message(msg, &allowed_types) {
            invalid.push((hash.clone(), msg.clone()));
        }
    }

    if !invalid.is_empty() {
        eprintln!("❌ Cannot push. Found {} invalid commit(s):", invalid.len());
        for (hash, msg) in invalid {
            eprintln!(" - [{}] {}", hash, msg);
        }
        exit(1);
    }

    // Determine current branch if not explicitly set
    let branch = match args.branch {
        Some(b) => b,
        None => get_current_branch()?,
    };

    // Push using git CLI (git2 doesn't do push over network)
    git_push(&args.remote, &branch)?;

    Ok(())
}
