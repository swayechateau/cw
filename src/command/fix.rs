use crate::core::commit::fix::CommitFixFile;
use crate::core::commit::rebase;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

#[derive(clap::Args)]
pub struct FixArgs {
    /// Skip prompts and use default behaviour
    #[arg(long, short)]
    interactive: bool,
    /// Regenerate the fix file and exit
    #[arg(long, short)]
    regen: bool,
    /// Remove the fix file after fixing
    #[arg(long, short)]
    cleanup: bool,
    /// Run the message editor
    #[arg(long)]
    editor_msg: Option<String>,
    /// Run the sequence editor
    #[arg(long)]
    editor_seq: Option<String>,
}

pub fn run(args: FixArgs) -> anyhow::Result<()> {
    let FixArgs {
        interactive,
        regen,
        cleanup,
        editor_msg,
        editor_seq,
    } = args;
    if editor_msg.is_some() {
        run_message_editor(editor_msg.as_deref().unwrap())?;
        return Ok(());
    }
    if editor_seq.is_some() {
        run_sequence_editor(editor_seq.as_deref().unwrap())?;
        return Ok(());
    }

    rebase::rebase(interactive, regen, cleanup)
}

fn run_sequence_editor(todo_path: &str) -> std::io::Result<()> {
    let json_path = env::var("REBASE_FIXES_FILE").expect("Missing REBASE_FIXES_FILE env");
    let data = fs::read_to_string(json_path)?;
    let fix_list: CommitFixFile = serde_json::from_str(&data).expect("Invalid JSON");

    let mut short_hashes = HashMap::new();
    for fix in &fix_list.fixes {
        short_hashes.insert(fix.hash[..7].to_string(), true);
    }

    let file = File::open(todo_path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line in reader.lines() {
        let l = line?;
        if l.starts_with("pick ") {
            let parts: Vec<&str> = l.splitn(3, ' ').collect();
            if let Some(hash) = parts.get(1) {
                let short = &hash[..7];
                if short_hashes.contains_key(short) {
                    lines.push(format!("reword {}", hash));
                    continue;
                }
            }
        }
        lines.push(l);
    }

    fs::write(todo_path, lines.join("\n") + "\n")?;
    Ok(())
}

fn run_message_editor(msg_path: &str) -> std::io::Result<()> {
    let json_path = env::var("REBASE_FIXES_FILE").expect("Missing REBASE_FIXES_FILE env");
    let data = fs::read_to_string(&json_path)?;
    let fix_list: CommitFixFile = serde_json::from_str(&data).expect("Invalid JSON");

    let content = fs::read_to_string(msg_path)?;
    let first_line = content.lines().next().unwrap_or("").trim();

    let fix = fix_list
        .fixes
        .iter()
        .find(|f| f.original_message.trim() == first_line);

    // println!("Trying to edit commit: {}", );
    println!("Looking for matching fix...");

    if let Some(fix) = fix {
        println!("✅ Updating commit message to: {}", fix.new_message);
        fs::write(msg_path, &fix.new_message)?;
    } else {
        eprintln!("❌ Could not find fix for commit message: '{}'", first_line);
    }

    Ok(())
}
