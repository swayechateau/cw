use std::collections::HashSet;

use dialoguer::{MultiSelect, Select, theme::ColorfulTheme};

use crate::git::{
    get_repo, get_status_files, show_unstaged_diff, stage_all_files, stage_file, unstage_file,
};

#[derive(clap::Args)]
pub struct AddArgs {
    /// Add all files (equivalent to `git add .`)
    #[arg(long, short)]
    pub all: bool,
    /// Exclude already staged files from the selection list
    #[arg(long, short)]
    pub exclude_staged: bool,
}

pub fn run(args: AddArgs) -> anyhow::Result<()> {
    let repo = get_repo();

    if args.all {
        let res = stage_all_files(&repo);
        if res.is_err() {
            println!("ℹ️ No modified, untracked, or staged files.");
        } else {
            println!("✅ Staged all files.");
        }

        return res;
    }

    let (mut all_files, staged_set) = get_status_files(&repo)?;

    if args.exclude_staged {
        all_files.retain(|f| !staged_set.contains(f));
    }

    if all_files.is_empty() {
        println!("ℹ️ No modified, untracked, or staged files.");
        return Ok(());
    }

    println!(
        "📁 {} file(s): {} staged, {} unstaged",
        all_files.len(),
        staged_set.len(),
        all_files.len() - staged_set.len()
    );

    let theme = ColorfulTheme::default();
    let prompt = if args.exclude_staged {
        "Select files to stage (currently unstaged):"
    } else {
        "Select files to stage or unstage (space to toggle, enter to confirm):"
    };

    let selections = MultiSelect::with_theme(&theme)
        .with_prompt(prompt)
        .items(&all_files)
        .defaults(
            &all_files
                .iter()
                .map(|f| staged_set.contains(f))
                .collect::<Vec<_>>(),
        )
        .interact()?;

    let selected_files: HashSet<&String> = selections.iter().map(|&i| &all_files[i]).collect();

    let view_diffs = dialoguer::Confirm::with_theme(&theme)
        .with_prompt("Would you like to preview diffs for selected files?")
        .default(false)
        .interact()?;

    if view_diffs && !selected_files.is_empty() {
        let mut files: Vec<&String> = selected_files.iter().copied().collect();
        files.sort();

        loop {
            let preview_options: Vec<String> = files
                .iter()
                .map(|f| format!("{} (diff)", f))
                .chain(std::iter::once("Done viewing diffs".into()))
                .collect();

            let idx = Select::with_theme(&theme)
                .with_prompt("Select a file to view diff")
                .items(&preview_options)
                .default(0)
                .interact()?;

            if idx >= files.len() {
                break; // Exit
            }

            let file = files[idx];
            println!("\n🔍 Diff for '{}':\n", file);
            show_unstaged_diff(file)?;
        }
    }

    let mut changed = false;

    for path in &all_files {
        let is_selected = selected_files.contains(path);
        let is_staged = staged_set.contains(path);

        if is_selected && !is_staged {
            stage_file(&repo, path)?;
            println!("✅ Staged {}", path);
            changed = true;
        } else if !is_selected && is_staged {
            unstage_file(&repo, path)?;
            println!("🚫 Unstaged {}", path);
            changed = true;
        }
    }

    if !changed {
        println!("ℹ️ No changes to staging.");
    }

    Ok(())
}
