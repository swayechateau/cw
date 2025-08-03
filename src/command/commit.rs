use crate::{config::load_config, git::commit::perform_commit};
use anyhow::bail;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};

#[derive(clap::Args)]
pub struct CommitArgs {
    #[arg(long, short = 't')]
    r#type: Option<String>,

    #[arg(long, short = 's')]
    scope: Option<String>,

    #[arg(long, short = 'm')]
    message: Option<String>,

    #[arg(long, short = 'b', default_value = "false")]
    breaking: bool,

    #[arg(long, short = 'd')]
    breaking_message: Option<String>,

    #[arg(long)]
    body: Option<String>,

    #[arg(long, short = 'f')]
    footer: Vec<String>,
}

pub fn run(prmpt: CommitArgs) -> anyhow::Result<()> {
    let theme = ColorfulTheme::default();
    let config = load_config();
    let binding = config.types.all();
    let commit_types: Vec<(String, String)> = binding
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let non_interactive = prmpt.r#type.is_some() && prmpt.message.is_some();
    if non_interactive
        && prmpt.breaking
        && prmpt
            .breaking_message
            .as_ref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
    {
        bail!(
            "--breaking (-b) flag was provided but no breaking message was given via --breaking-message (-d)."
        );
    }

    // --- Commit type ---
    let commit_type = match prmpt.r#type {
        Some(ref t) if binding.contains_key(t) => t.clone(),
        Some(ref t) => bail!(
            "Invalid commit type '{}'. Allowed types: {:?}",
            t,
            binding.keys()
        ),
        None => {
            let items: Vec<_> = commit_types
                .iter()
                .map(|(k, d)| format!("{:<10} {}", k, d))
                .collect();

            let idx = Select::with_theme(&theme)
                .with_prompt("What type of change are you committing?")
                .items(&items)
                .default(0)
                .interact()?;

            commit_types[idx].0.clone()
        }
    };

    // --- Summary ---
    let summary = match prmpt.message {
        Some(ref m) if m.trim().is_empty() => anyhow::bail!("Commit message cannot be empty."),
        Some(m) => m,
        None => Input::<String>::with_theme(&theme)
            .with_prompt("Write a short summary (imperative tone, e.g. 'add feature')")
            .validate_with(|input: &String| {
                if input.trim().is_empty() {
                    Err("Summary cannot be empty")
                } else {
                    Ok(())
                }
            })
            .interact_text()?,
    };

    // Declare optional variables
    let (mut scope, mut breaking, mut breaking_message, mut body, mut footers) = (
        String::new(),
        prmpt.breaking,
        None,
        prmpt.body.clone(),
        prmpt.footer.clone(),
    );

    if !non_interactive {
        // --- Scope ---
        scope = match prmpt.scope {
            Some(s) => {
                let allowed = &config.scopes.valid_scopes(&commit_type);

                if allowed.is_empty() || allowed.contains(&s) {
                    s
                } else {
                    anyhow::bail!(
                        "Invalid scope '{}'. Allowed for type '{}': {:?}",
                        s,
                        commit_type,
                        allowed
                    );
                }
            }
            None => prompt_scope(&theme, &commit_type, &config)?,
        };

        // --- Breaking message ---
        (breaking, breaking_message) =
            prompt_breaking_change(&theme, prmpt.breaking, &prmpt.breaking_message)?;

        // --- Body ---
        if body.is_none() {
            let use_body = Confirm::with_theme(&theme)
                .with_prompt("Would you like to add a longer description?")
                .default(false)
                .interact()?;

            if use_body {
                body = Some(
                    Input::<String>::with_theme(&theme)
                        .with_prompt("Provide a longer description (press Enter when done)")
                        .allow_empty(true)
                        .interact_text()?,
                );
            }
        }

        // --- Footer ---
        footers = prompt_footers(&theme, &prmpt.footer)?;
    }
    // --- Build commit message ---
    let mut commit_msg = format!(
        "{}{}{}: {}",
        commit_type,
        if breaking { "!" } else { "" },
        if scope.is_empty() {
            "".to_string()
        } else {
            format!("({})", scope)
        },
        summary.trim()
    );

    if let Some(ref b) = body {
        commit_msg.push_str(&format!("\n\n{}", b.trim()));
    }

    if let Some(ref desc) = breaking_message {
        commit_msg.push_str(&format!("\n\nBREAKING CHANGE: {}", desc.trim()));
    }

    for footer in &footers {
        if !footer.trim().is_empty() {
            commit_msg.push_str(&format!("\n{}", footer));
        }
    }

    println!(
        "\n🚧 Commit message preview:\n-------------------------\n{}\n",
        commit_msg
    );

    if non_interactive
        || Confirm::with_theme(&theme)
            .with_prompt("Do you want to commit this?")
            .default(true)
            .interact()?
    {
        if let Err(e) = perform_commit(&commit_msg) {
            println!("❌ Failed to commit: {e}");
        } else {
            println!("✅ Commit created.");
        }
    } else {
        println!("❌ Commit cancelled.");
    }

    Ok(())
}

fn prompt_scope(
    theme: &ColorfulTheme,
    commit_type: &str,
    config: &crate::config::CWConfig,
) -> anyhow::Result<String> {
    let want_scope = Confirm::with_theme(theme)
        .with_prompt("Do you want to add a scope?")
        .default(false)
        .interact()?;

    if !want_scope {
        return Ok(String::new());
    }

    let use_custom = Confirm::with_theme(theme)
        .with_prompt("Use a custom scope?")
        .default(false)
        .interact()?;

    if use_custom {
        let input = Input::<String>::with_theme(theme)
            .with_prompt("Enter your custom scope (leave empty for none)")
            .allow_empty(true)
            .interact_text()?;
        return Ok(input);
    }

    let valid_scopes = config.scopes.valid_scopes(commit_type);

    if valid_scopes.is_empty() {
        println!("ℹ️ No predefined scopes available for '{}'.", commit_type);
        return Ok(String::new());
    }

    let idx = Select::with_theme(theme)
        .with_prompt("Select a scope")
        .items(&valid_scopes)
        .default(0)
        .interact_opt()?;

    Ok(idx.map(|i| valid_scopes[i].clone()).unwrap_or_default())
}

fn prompt_breaking_change(
    theme: &ColorfulTheme,
    cli_flag: bool,
    cli_message: &Option<String>,
) -> anyhow::Result<(bool, Option<String>)> {
    let breaking = cli_flag
        || Confirm::with_theme(theme)
            .with_prompt("Does this commit include breaking changes?")
            .default(false)
            .interact()?;

    if !breaking {
        return Ok((false, None));
    }

    let message = if let Some(bm) = cli_message {
        if bm.trim().is_empty() {
            anyhow::bail!("Breaking message cannot be empty if --breaking is set.");
        }
        Some(bm.clone())
    } else {
        Some(
            Input::<String>::with_theme(theme)
                .with_prompt("Describe the breaking change")
                .validate_with(|input: &String| {
                    if input.trim().is_empty() {
                        Err("Breaking change description cannot be empty")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?,
        )
    };

    Ok((true, message))
}

fn prompt_footers(theme: &ColorfulTheme, existing: &[String]) -> anyhow::Result<Vec<String>> {
    if !existing.is_empty() {
        return Ok(existing.to_vec());
    }

    let use_footer = Confirm::with_theme(theme)
        .with_prompt("Would you like to add a footer? (e.g. Closes #123, Co-authored-by)")
        .default(false)
        .interact()?;

    let mut lines = Vec::new();
    if use_footer {
        loop {
            let footer = Input::<String>::with_theme(theme)
                .with_prompt("Enter footer line (or leave empty to finish)")
                .allow_empty(true)
                .interact_text()?;

            let trimmed = footer.trim();
            if trimmed.is_empty() {
                break;
            }

            lines.push(trimmed.to_string());
        }
    }

    Ok(lines)
}
