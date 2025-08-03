use std::{collections::BTreeMap, fs};

use chrono::Utc;
use serde::Serialize;

use crate::{
    config::load_config,
    core::commit::changelog::{generate_changelog, render_markdown},
    git::{
        check::get_github_web_url,
        commit::{collect_commits_in_range, parse_commit_message},
        repo::get_repo_root,
        tag::get_tag_commit_date,
    },
};

#[derive(clap::Args)]
pub struct ChangelogArgs {
    /// Generate changelog for all tags
    #[arg(long, short)]
    all: bool,
    /// Print to stdout without writing to file
    #[arg(long)]
    pub dry_run: bool,
    /// Start commit (e.g. tag or hash)
    #[arg(long, short)]
    from: Option<String>,
    /// End commit (e.g. HEAD, tag or hash)
    #[arg(long, short, default_value = "HEAD")]
    to: String,
    /// Output file (default: CHANGELOG.md)
    #[arg(long, short)]
    output: Option<String>,
    /// Append to existing changelog (if output is a markdown file)
    #[arg(long, short = 'p')]
    append: bool,
    /// Export changelog as machine-readable JSON
    #[arg(long, short)]
    json: bool,
    /// Group entries by scope under each section
    #[arg(long, short)]
    group_by_scope: bool,
    /// Manually specify the upcoming tag (instead of defaulting to Unreleased)
    #[arg(long, short)]
    next_tag: Option<String>,
}

#[derive(Serialize)]
struct ChangelogEntry {
    hash: String,
    message: String,
}

#[derive(Serialize)]
struct Changelog {
    kind: String,
    description: String,
    entries: Vec<ChangelogEntry>,
}

pub fn run(args: ChangelogArgs) -> anyhow::Result<()> {
    let ChangelogArgs {
        dry_run,
        all,
        from,
        to,
        output,
        append,
        json,
        group_by_scope,
        next_tag,
    } = args;

    let config = load_config();
    let binding = config.types.all();
    let allowed_types: Vec<&str> = binding.keys().map(|k| k.as_str()).collect();

    // Get commits by tag OR by range depending on `--all`
    let commits_by_tag = if all {
        generate_changelog()?
    } else {
        let commits = collect_commits_in_range(from.clone(), to.clone())?;
        let tag = next_tag.clone().unwrap_or_else(|| "Unreleased".to_string());
        BTreeMap::from([(tag, commits)])
    };

    let mut markdown_output = String::new();
    let mut json_output_data = Vec::new();

    for (original_tag, commits) in commits_by_tag {
        // Handle `--next-tag` override
        let tag = if original_tag == "Unreleased" {
            next_tag.clone().unwrap_or(original_tag)
        } else {
            original_tag
        };

        // Group and parse commits
        let mut groups: BTreeMap<String, Vec<(String, Option<String>, String)>> = BTreeMap::new();
        for (hash, msg) in commits {
            if let Some((kind, scope, summary)) = parse_commit_message(&msg, &allowed_types) {
                groups.entry(kind).or_default().push((hash, scope, summary));
            }
        }

        if groups.is_empty() {
            continue;
        }

        // Format date
        let date = if tag == "Unreleased" {
            Utc::now().format("%Y-%m-%d").to_string()
        } else {
            get_tag_commit_date(&tag).unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string())
        };

        let github_url = get_github_web_url();

        // Markdown output
        markdown_output.push_str(&render_markdown(
            &tag,
            &groups,
            group_by_scope,
            &date,
            &config.changelog,
            github_url.as_deref(),
        ));
        markdown_output.push('\n');

        // JSON output
        if json {
            let changelog: Vec<Changelog> = groups
                .iter()
                .map(|(kind, commits)| {
                    let description = config
                        .types
                        .all()
                        .get(kind)
                        .cloned()
                        .unwrap_or_else(|| kind.clone());

                    let entries = commits
                        .iter()
                        .map(|(hash, scope, summary)| {
                            let message = if let Some(scope) = scope {
                                format!("{}: {}", scope, summary)
                            } else {
                                summary.clone()
                            };
                            ChangelogEntry {
                                hash: hash.clone(),
                                message,
                            }
                        })
                        .collect();

                    Changelog {
                        kind: kind.clone(),
                        description,
                        entries,
                    }
                })
                .collect();

            json_output_data.push((tag.clone(), changelog));
        }
    }

    if json {
        let final_json = serde_json::to_string_pretty(&json_output_data)?;
        if let Some(ref path) = output {
            fs::write(path, final_json)?;
            println!("✅ Changelog written as JSON to `{}`.", path);
        } else {
            println!("{}", final_json);
        }
    } else {
        if markdown_output.trim().is_empty() {
            println!("ℹ️ No changelog entries to display.");
            return Ok(());
        }

        let path = output.or_else(|| {
            get_repo_root().map(|root| root.join("CHANGELOG.md").to_string_lossy().to_string())
        });

        if let Some(ref path) = path {
            let final_md = if append && std::path::Path::new(path).exists() {
                let mut existing = fs::read_to_string(path)?;
                if !existing.starts_with("# Changelog") {
                    existing = format!("# Changelog\n\n{}", existing);
                }
                format!("{}\n{}", existing.trim_end(), markdown_output)
            } else {
                format!("# Changelog\n\n{}", markdown_output)
            };

            if dry_run {
                println!("# Changelog\n\n{}", markdown_output.trim());
                return Ok(());
            }

            fs::write(path, final_md)?;
            println!("✅ Changelog written to `{}`.", path);
        } else {
            println!("# Changelog\n\n{}", markdown_output);
        }
    }

    Ok(())
}
