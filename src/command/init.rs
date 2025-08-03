use std::fs;
use std::path::PathBuf;

use crate::{
    config::{CWConfig, global_user_config_path},
    git::get_repo_root,
};

#[derive(clap::Args)]
pub struct InitArgs {
    /// Generate a global config file
    #[arg(long, short, default_value = "false")]
    pub global: bool,
    /// Overwrite existing config file if it exists
    #[arg(long, short, default_value = "false")]
    pub force: bool,
}

pub fn run(args: InitArgs) -> anyhow::Result<()> {
    let config_path: PathBuf = if args.global {
        global_user_config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine global config path"))?
    } else {
        let root = get_repo_root().ok_or_else(|| anyhow::anyhow!("Not inside a Git repository"))?;
        root.join(".cwizard.toml")
    };

    if config_path.exists() && !args.force {
        println!(
            "✅ Config file already exists at {} (use --force to overwrite)",
            config_path.display()
        );
        return Ok(());
    }

    let config = CWConfig::new_config();

    let toml_content = toml::to_string_pretty(&config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize config to TOML: {}", e))?;

    println!("\n📄 Preview:\n{}", toml_content);

    // Ensure parent directory exists (especially important for global config)
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&config_path, toml_content)?;

    println!("✅ Configuration written to {}", config_path.display());

    Ok(())
}
