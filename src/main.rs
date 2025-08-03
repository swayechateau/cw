use std::process::exit;

use clap::{Parser, Subcommand};
use cw::{
    command::{add, bump, changelog, check, commit, fix, init, push},
    git::check::git_not_installed,
};

/// A lightweight conventional commits assistant.
#[derive(Parser)]
#[command(name = "cw", version, author, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Guide the user through writing a Conventional Commit message
    Commit(commit::CommitArgs),
    /// Check commits in a range for Conventional Commit compliance
    Check(check::CheckArgs),
    /// Add files to staging area interactively
    Add(add::AddArgs),
    /// Push commits to a remote repository
    Push(push::PushArgs),
    /// Bump version based on Conventional Commits
    Bump(bump::BumpArgs),
    /// Initialize a .cwizard.toml config file in your repo
    Init(init::InitArgs),
    /// Apply fixes to a range of commits
    Fix(fix::FixArgs),
    /// Generate a changelog based on Conventional Commits
    Changelog(changelog::ChangelogArgs),
}

fn main() -> anyhow::Result<()> {
    // Precheck: ensure Git is installed
    if git_not_installed() {
        eprintln!(
            "Error: Git is not installed or not available in PATH.\nPlease install Git before using `cw`."
        );
        exit(1);
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Commit(args) => commit::run(args)?,
        Commands::Check(args) => check::run(args)?,
        Commands::Add(args) => add::run(args)?,
        Commands::Push(args) => push::run(args)?,
        Commands::Bump(args) => bump::run(args)?,
        Commands::Init(args) => init::run(args)?,
        Commands::Fix(args) => fix::run(args)?,
        Commands::Changelog(args) => changelog::run(args)?,
    }

    Ok(())
}
