use std::process;

use crate::{
    config::load_config,
    git::{collect_commits_in_range, validate_conventional_commit_message},
};

#[derive(clap::Args)]
pub struct CheckArgs {
    /// Start commit (e.g. tag or hash)
    #[arg(long)]
    from: Option<String>,

    /// End commit (e.g. HEAD, tag or hash)
    #[arg(long, default_value = "HEAD")]
    to: String,
}

pub fn run(args: CheckArgs) -> anyhow::Result<()> {
    let CheckArgs { from, to } = args;
    let config = load_config();
    let binding = config.types.all();
    let allowed_types: Vec<&str> = binding.keys().map(|k| k.as_str()).collect();

    let commits = collect_commits_in_range(from.clone(), to.clone())?;

    let mut invalid_count = 0;

    println!(
        "🔍 Checking {} commits from {}...\n",
        commits.len(),
        from.unwrap_or_else(|| "initial".to_string())
    );

    for (hash, msg) in commits {
        if validate_conventional_commit_message(&msg, &allowed_types) {
            println!("✅ [{}] {}", hash, msg);
        } else {
            println!("❌ [{}] {}", hash, msg);
            invalid_count += 1;
        }
    }

    if invalid_count > 0 {
        println!("\n❗ Found {} invalid commit(s).", invalid_count);
        process::exit(1);
    }

    println!("\n🎉 All commits are valid Conventional Commits!");
    Ok(())
}
