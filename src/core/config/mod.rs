//! Commit Wizard configuration loader.
pub mod breaking;
pub mod changelog;
pub mod default;
pub mod scopes;
pub mod types;
pub mod versioning;

use crate::{
    config::{
        breaking::Breaking, changelog::Changelog, default::CWDefault, scopes::CommitScopes,
        types::CommitTypes, versioning::CWVersioning,
    },
    git::get_repo_root,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

trait Patchable {
    fn patch(&mut self);
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CWConfig {
    pub default: CWDefault,
    /// The commit types to use
    #[serde(default)]
    pub types: CommitTypes,
    /// The commit scopes to use
    #[serde(default)]
    pub scopes: CommitScopes,
    /// The breaking change configuration
    #[serde(default)]
    pub breaking: Breaking,
    /// The version bump configuration
    #[serde(default)]
    pub versioning: CWVersioning,
    /// The changelog configuration
    #[serde(default)]
    pub changelog: Changelog,
}

impl Default for CWConfig {
    fn default() -> Self {
        Self::new_config()
    }
}

impl Patchable for CWConfig {
    fn patch(&mut self) {
        self.types.patch();
        self.breaking.patch();
        self.versioning.patch();
        self.changelog.patch();
    }
}

impl CWConfig {
    pub fn new_config() -> Self {
        CWConfig {
            default: CWDefault::default(),
            types: CommitTypes::default(),
            scopes: CommitScopes::default(),
            breaking: Breaking::default(),
            versioning: CWVersioning::default(),
            changelog: Changelog::default(),
        }
    }
}

/// Attempt to load `.cwizard.toml` from the root of the current Git project.
/// If the file is not found, the user-level global config is checked.
/// If no valid config is found, the default config is used.
pub fn load_config() -> CWConfig {
    if let Some(config_content) = get_repo_root()
        .and_then(|root| fs::read_to_string(root.join(".cwizard.toml")).ok())
        .or_else(|| global_user_config_path().and_then(|p| fs::read_to_string(p).ok()))
    {
        return toml::from_str::<CWConfig>(&config_content)
            .map(patch_with_defaults)
            .unwrap_or_else(|_| {
                eprintln!("⚠️ Failed to parse config. Falling back to defaults.");
                CWConfig::default()
            });
    }

    eprintln!("⚠️ No configuration found. Falling back to defaults.");
    CWConfig::default()
}

fn patch_with_defaults(mut config: CWConfig) -> CWConfig {
    config.patch();
    config
}

pub fn global_user_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut p| {
        p.push("cwizard");
        p.push("config.toml");
        p
    })
}
