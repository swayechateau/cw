use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

use crate::constants::COMMIT_FIX_FILE_NAME;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitFixEntry {
    pub hash: String,
    pub original_message: String,
    pub new_message: String,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitFixFile {
    pub fixes: Vec<CommitFixEntry>,
}

impl CommitFixFile {
    /// Find the fixed message for a given commit hash
    pub fn get_message_for(&self, hash: &str) -> Option<String> {
        self.fixes
            .iter()
            .find(|entry| entry.hash == hash)
            .map(|entry| entry.new_message.clone())
    }

    /// Return the original message for a given hash
    pub fn get_original_message(&self, hash: &str) -> Option<String> {
        self.fixes
            .iter()
            .find(|entry| entry.hash == hash)
            .map(|entry| entry.original_message.clone())
    }

    /// Checks if all entries are structurally sound
    pub fn validate_integrity(&self) -> bool {
        self.fixes.iter().all(|f| {
            !f.hash.is_empty() && !f.new_message.is_empty() && !f.original_message.is_empty()
        })
    }

    /// Sort entries to preserve chronological order for rebasing
    pub fn sort_by_original_order(&mut self, original_order: &[String]) {
        self.fixes.sort_by_key(|entry| {
            original_order
                .iter()
                .position(|h| h == &entry.hash)
                .unwrap_or(usize::MAX)
        });
    }

    /// Check whether a fix already exists for a given commit
    pub fn contains(&self, hash: &str) -> bool {
        self.fixes.iter().any(|entry| entry.hash == hash)
    }

    /// Commit map to a vector of tuples
    pub fn commit_map(&self) -> HashMap<String, String> {
        self.fixes
            .iter()
            .map(|entry| (entry.hash.clone(), entry.new_message.clone()))
            .collect()
    }
}

/// Returns the full path to `.cw-fix.json`
pub fn fix_file_path() -> PathBuf {
    PathBuf::from(COMMIT_FIX_FILE_NAME)
}

/// Saves the fix file to disk
pub fn save_fix_file(fix_file: &CommitFixFile) -> Result<()> {
    let json = serde_json::to_string_pretty(fix_file)?;
    fs::write(fix_file_path(), json)?;
    Ok(())
}

/// Loads the fix file from disk
pub fn load_fix_file() -> Result<CommitFixFile> {
    let content = fs::read_to_string(fix_file_path())?;
    let fix_file: CommitFixFile = serde_json::from_str(&content)?;
    Ok(fix_file)
}

/// Deletes `.cw-fix.json` if it exists
pub fn remove_fix_file() -> Result<()> {
    let path = fix_file_path();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn generate_fix_file(commits: Vec<(String, String)>) -> Result<CommitFixFile> {
    let now = Utc::now().to_rfc3339();
    let fix_file = CommitFixFile {
        fixes: commits
            .iter()
            .map(|(hash, msg)| CommitFixEntry {
                hash: hash.clone(),
                original_message: msg.clone(),
                new_message: msg.clone(),
                timestamp: Some(now.clone()),
            })
            .collect(),
    };
    save_fix_file(&fix_file)?;
    Ok(fix_file)
}
