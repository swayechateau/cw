use anyhow::Result;
use git2::{DiffFormat, DiffOptions, Repository, StatusOptions};
use std::{collections::HashSet, path::Path};

/// Stages all files in the Git index.
pub fn stage_all_files(repo: &Repository) -> Result<()> {
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok(())
}

/// Stages a file in the Git index.
pub fn stage_file(repo: &Repository, path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        anyhow::bail!("{} does not exist on disk.", path);
    }

    let mut index = repo.index()?;
    index.add_path(Path::new(path))?;
    index.write()?;
    Ok(())
}

/// Unstages all files in the Git index.
pub fn unstage_all_files(repo: &Repository) -> Result<()> {
    let mut index = repo.index()?;
    index.clear()?;
    index.write()?;
    Ok(())
}

/// Unstages a file from the Git index.
pub fn unstage_file(repo: &Repository, path: &str) -> Result<()> {
    let head = repo.head()?.peel_to_tree()?;
    let mut index = repo.index()?;
    index.read_tree(&head)?;
    index.remove_path(Path::new(path))?;
    index.write()?;
    Ok(())
}

/// Get the status of all files in the repository.
pub fn get_status_files(repo: &Repository) -> Result<(Vec<String>, HashSet<String>)> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .include_unmodified(false)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true)
        .renames_from_rewrites(true)
        .recurse_untracked_dirs(true)
        .show(git2::StatusShow::IndexAndWorkdir);

    let statuses = repo.statuses(Some(&mut opts))?;
    let mut all = Vec::new();
    let mut staged = HashSet::new();

    for entry in statuses.iter() {
        if let Some(path) = entry.path() {
            all.push(path.to_string());
            if entry.status().is_index_new()
                || entry.status().is_index_modified()
                || entry.status().is_index_deleted()
                || entry.status().is_index_renamed()
                || entry.status().is_index_typechange()
            {
                staged.insert(path.to_string());
            }
        }
    }

    all.sort();
    all.dedup();

    Ok((all, staged))
}

/// Shows the diff for a specific file path.
pub fn show_unstaged_diff(path: &str) -> Result<()> {
    let repo = Repository::discover(".")?;
    let mut opts = DiffOptions::new();
    opts.pathspec(path);

    let diff = repo.diff_index_to_workdir(None, Some(&mut opts))?;

    if diff.deltas().len() == 0 {
        println!("\x1b[2m(No unstaged changes for '{}')\x1b[0m", path); // dim text
        return Ok(());
    }

    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let line_str = std::str::from_utf8(line.content()).unwrap_or("");
        match line.origin() {
            '+' => print!("\x1b[32m+{}\x1b[0m", &line_str), // green
            '-' => print!("\x1b[31m-{}\x1b[0m", &line_str), // red
            _ => print!(" {}", &line_str),                  // default (no color)
        }
        true
    })?;

    Ok(())
}
