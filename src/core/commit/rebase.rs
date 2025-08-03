use std::{
    collections::HashMap,
    env, fs,
    process::{Command, Stdio},
};

use crate::{
    config::load_config,
    core::commit::fix::{
        CommitFixEntry, CommitFixFile, generate_fix_file, load_fix_file, remove_fix_file,
        save_fix_file,
    },
    git::{find_invalid_commits, validate_conventional_commit_message},
};
use anyhow::Result;
use chrono::Utc;
use dialoguer::{Confirm, Input};

pub fn rebase(interactive: bool, regen: bool, cleanup: bool) -> Result<()> {
    println!("🔍 Scanning commit history...");
    let invalid_commits = find_invalid_commits()?;

    if invalid_commits.is_empty() {
        if regen {
            println!("🚫 No invalid commits found. No need to regenerate `.cw-fix.json`.");
            return Ok(());
        }
        if cleanup {
            remove_fix_file().ok();
            println!("🧹 Cleaned up `.cw-fix.json`.");
            return Ok(());
        }
        println!("✅ All commit messages are valid.");
        return Ok(());
    }

    println!("❌ Found {} invalid commit(s):", invalid_commits.len());
    for (hash, msg) in &invalid_commits {
        println!(" - [{}] {}", &hash[..7], msg);
    }

    // --regen mode: just regenerate the fix file and return
    if regen {
        let fix_file = generate_fix_file(invalid_commits)?;
        println!(
            "📝 Regenerated `.cw-fix.json` with {} entries.",
            fix_file.fixes.len()
        );
        return Ok(());
    }

    println!("🚀 Starting commit wizard fix...");
    rewrite_commits(&invalid_commits, interactive)?;

    if cleanup {
        remove_fix_file().ok();
        println!("🧹 Cleaned up `.cw-fix.json`.");
    }

    Ok(())
}

/// Rewrites only the specified commits using Git interactive rebase.
fn rewrite_commits(commits: &[(String, String)], interactive: bool) -> Result<()> {
    if commits.is_empty() {
        println!("⚠️ No commits to rewrite.");
        return Ok(());
    }

    // TODO: implement a better interactive mode
    let mut should_auto_fix = true;
    if interactive {
        // Prompt to fix
        let should_fix = Confirm::new()
            .with_prompt("✨ Would you like to fix the commit messages now?")
            .default(true)
            .interact()?;

        if !should_fix {
            println!("🚫 Fix aborted.");
            return Ok(());
        }
        // prompt to auto-fix
        should_auto_fix = Confirm::new()
            .with_prompt("✨ Would you like to auto-fix the commit messages now?")
            .default(false)
            .interact()?;
    }
    // Load or initialize fix map
    let mut existing: HashMap<String, CommitFixEntry> = match load_fix_file() {
        Ok(fix_file) => {
            if !fix_file.validate_integrity() {
                anyhow::bail!(
                    "❌ Detected an invalid or corrupt `.cw-fix.json`. Please regenerate it with --regen."
                );
            }
            fix_file
                .fixes
                .into_iter()
                .map(|e| (e.hash.clone(), e))
                .collect()
        }
        Err(_) => HashMap::new(),
    };

    let now = Utc::now().to_rfc3339();
    let config = load_config();
    let binding = config.types.all();
    let allowed_types: Vec<&str> = binding.keys().map(|k| k.as_str()).collect();
    for (hash, original_message) in commits {
        if existing.contains_key(hash)
            && old_msg_and_new_commit_are_diff(
                original_message,
                &existing.get(hash).unwrap().new_message,
                &allowed_types,
            )
        {
            continue;
        }

        println!("🔧 Fixing commit [{}]", &hash[..7]);
        let new_message = if should_auto_fix {
            format!("chore(misc): {}", original_message)
        } else {
            Input::new()
                .with_prompt("✍ Enter new Conventional Commit message")
                .default(original_message.clone())
                .interact_text()?
        };

        existing.insert(
            hash.clone(),
            CommitFixEntry {
                hash: hash.clone(),
                original_message: original_message.clone(),
                new_message,
                timestamp: Some(now.clone()),
            },
        );
    }

    // Save updated fixes
    let fix_file = CommitFixFile {
        fixes: existing.into_values().collect::<Vec<CommitFixEntry>>(),
    };
    save_fix_file(&fix_file)?;

    println!("📦 Preparing to rewrite {} commit(s)...", commits.len());

    let json_path = std::env::temp_dir().join("rebase_fixes_internal.json");
    fs::write(&json_path, serde_json::to_string(&fix_file).unwrap())?;

    // Get path to current binary
    let self_path = env::current_exe().unwrap();

    // Launch git rebase with our binary as editors
    let status = Command::new("git")
        .arg("rebase")
        .arg("-i")
        .arg("--root")
        .env("REBASE_FIXES_FILE", &json_path)
        .env(
            "GIT_SEQUENCE_EDITOR",
            format!("{} fix --editor-seq", self_path.display()),
        )
        .env(
            "GIT_EDITOR",
            format!("{} fix --editor-msg", self_path.display()),
        )
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        println!("Rebase completed.");
    } else {
        eprintln!("Rebase failed.");
    }

    let rewritten = commits_rewritten(
        &fix_file
            .fixes
            .iter()
            .map(|f| f.hash.clone())
            .collect::<Vec<_>>(),
    )?;
    if rewritten {
        println!("✅ Rebase finished. Messages updated.");
    } else {
        println!("ℹ️ Rebase completed but no commits were updated.");
    }

    Ok(())
}

fn commits_rewritten(old_hashes: &[String]) -> Result<bool> {
    // Use `git rev-list` to find current hashes of HEAD
    let output = Command::new("git")
        .arg("rev-list")
        .arg("--reverse")
        .arg("--all") // or limit to the expected range
        .output()?;

    let new_hashes = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();

    for old in old_hashes {
        if !new_hashes.contains(old) {
            return Ok(true); // A commit was rewritten
        }
    }

    Ok(false)
}

fn old_msg_and_new_commit_are_diff(old: &String, new: &String, types: &[&str]) -> bool {
    // Check if the old message is the same as the new message
    old != new && validate_conventional_commit_message(&new, types)
}
