use std::io::IsTerminal;

use anyhow::{bail, Result};
use colored::Colorize;

use crate::cli::{Cli, Commands, OutputFormat};
use crate::client::ApolloClient;
use crate::config::{AplConfig, Resolved};
use crate::models::*;
use crate::output;

pub fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init {
            portal_url,
            token,
            env,
            app_id,
            cluster,
            operator,
        } => cmd_init(portal_url, token, env, app_id, cluster, operator),

        Commands::Show => cmd_show(),

        Commands::Envs => {
            let r = resolve(&cli, None)?;
            let c = ApolloClient::new(&r);
            let data = c.env_clusters()?;
            output::env_clusters(&data, &cli.format);
            Ok(())
        }

        Commands::Ns => {
            let r = resolve(&cli, None)?;
            let c = ApolloClient::new(&r);
            let data = c.list_namespaces()?;
            output::namespaces(&data, &cli.format);
            Ok(())
        }

        Commands::Get {
            ref namespace,
            ref key,
            ref keys,
        } => cmd_get(&cli, namespace, key.as_deref(), keys.as_deref()),

        Commands::Set {
            ref namespace,
            ref key,
            ref value,
            ref comment,
            ref operator,
            yes,
        } => cmd_set(
            &cli,
            namespace,
            key,
            value,
            comment.clone(),
            operator.as_deref(),
            yes,
        ),

        Commands::Delete {
            ref namespace,
            ref key,
            ref operator,
            yes,
        } => cmd_delete(&cli, namespace, key, operator.as_deref(), yes),

        Commands::Publish {
            ref namespace,
            ref title,
            ref comment,
            ref operator,
            yes,
        } => cmd_publish(
            &cli,
            namespace,
            title.clone(),
            comment.clone(),
            operator.as_deref(),
            yes,
        ),
    }
}

// ── init ───────────────────────────────────────────────────────

fn cmd_init(
    portal_url: String,
    token: String,
    env: String,
    app_id: String,
    cluster: String,
    operator: String,
) -> Result<()> {
    let cfg = AplConfig {
        portal_url: Some(portal_url),
        token: Some(token),
        default_env: Some(env),
        default_app_id: Some(app_id),
        default_cluster: Some(cluster),
        default_operator: Some(operator),
    };
    cfg.save()?;
    let path = AplConfig::path();
    println!("{} {}", "Created".green().bold(), path.display());
    Ok(())
}

// ── show ───────────────────────────────────────────────────────

fn cmd_show() -> Result<()> {
    if !AplConfig::exists() {
        println!(
            "{} .apollo-cli.toml not found in current directory.",
            "Warning:".yellow().bold()
        );
        println!("Run `apl init` to create one.");
        return Ok(());
    }
    let cfg = AplConfig::load()?;
    println!("{}", "Current configuration:".bold());
    println!("  portal_url : {}", cfg.portal_url.as_deref().unwrap_or("(not set)"));
    println!("  token      : {}", mask_token(cfg.token.as_deref()));
    println!("  env        : {}", cfg.default_env.as_deref().unwrap_or("UAT"));
    println!("  app_id     : {}", cfg.default_app_id.as_deref().unwrap_or("(not set)"));
    println!("  cluster    : {}", cfg.default_cluster.as_deref().unwrap_or("default"));
    println!("  operator   : {}", cfg.default_operator.as_deref().unwrap_or("apollo"));
    Ok(())
}

fn mask_token(t: Option<&str>) -> String {
    match t {
        None => "(not set)".into(),
        Some(s) if s.len() <= 8 => "****".into(),
        Some(s) => format!("{}****{}", &s[..4], &s[s.len() - 4..]),
    }
}

// ── get ────────────────────────────────────────────────────────

fn cmd_get(cli: &Cli, ns: &str, key: Option<&str>, keys: Option<&str>) -> Result<()> {
    let r = resolve(cli, None)?;
    let c = ApolloClient::new(&r);

    match (key, keys) {
        (Some(k), _) => {
            let item = c.get_item(ns, k)?;
            output::single_item(&item, &cli.format);
        }
        (None, Some(ks)) => {
            let want: Vec<&str> = ks.split(',').map(str::trim).collect();
            let info = c.get_namespace(ns)?;
            let filtered: Vec<&ConfigItem> =
                info.items.iter().filter(|i| want.contains(&i.key.as_str())).collect();
            let refs: Vec<&ConfigItem> = filtered.into_iter().collect();
            print_item_refs(&refs, &cli.format);
        }
        (None, None) => {
            let info = c.get_namespace(ns)?;
            output::items(&info.items, &cli.format);
        }
    }
    Ok(())
}

fn print_item_refs(list: &[&ConfigItem], fmt: &OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            let map: std::collections::BTreeMap<&str, &str> =
                list.iter().map(|i| (i.key.as_str(), i.value.as_str())).collect();
            println!("{}", serde_json::to_string_pretty(&map).unwrap());
        }
        OutputFormat::Text => {
            if list.is_empty() {
                println!("(no matching items)");
                return;
            }
            let max_key = list.iter().map(|i| i.key.len()).max().unwrap_or(10).max(10);
            println!("{:<width$}  {}", "Key".bold(), "Value".bold(), width = max_key);
            println!("{}", "─".repeat(max_key + 50));
            for item in list {
                println!("{:<width$}  {}", item.key, item.value, width = max_key);
            }
        }
    }
}

// ── set ────────────────────────────────────────────────────────

fn cmd_set(
    cli: &Cli,
    ns: &str,
    key: &str,
    value: &str,
    comment: Option<String>,
    operator: Option<&str>,
    yes: bool,
) -> Result<()> {
    let r = resolve(cli, operator)?;
    guard_pro_write(&r)?;
    let c = ApolloClient::new(&r);

    let existing = c.try_get_item(ns, key)?;
    let is_new = existing.is_none();
    let old_value = existing.as_ref().map(|e| e.value.as_str());

    if !is_new && old_value == Some(value) {
        println!("Value is already \"{}\". Nothing to do.", value);
        return Ok(());
    }

    println!();
    if is_new {
        println!("  {} new configuration", "CREATE".green().bold());
    } else {
        println!("  {} configuration", "UPDATE".yellow().bold());
    }
    println!("  Env       : {}", r.env);
    println!("  App       : {}", r.app_id);
    println!("  Namespace : {}", ns);
    println!("  Key       : {}", key);
    if let Some(old) = old_value {
        println!("  Old Value : {}", old.dimmed());
    }
    println!("  New Value : {}", value.green());
    if let Some(ref c) = comment {
        println!("  Comment   : {}", c);
    }
    println!("  Operator  : {}", r.operator);
    println!();

    if !confirm("Proceed?", yes)? {
        println!("Cancelled.");
        return Ok(());
    }

    let req = UpdateItemRequest {
        key: key.into(),
        value: value.into(),
        comment,
        data_change_last_modified_by: r.operator.clone(),
        data_change_created_by: Some(r.operator.clone()),
    };
    c.update_item(ns, key, &req, true)?;
    println!("{}", "Done.".green().bold());
    Ok(())
}

// ── delete ─────────────────────────────────────────────────────

fn cmd_delete(
    cli: &Cli,
    ns: &str,
    key: &str,
    operator: Option<&str>,
    yes: bool,
) -> Result<()> {
    let r = resolve(cli, operator)?;
    guard_pro_write(&r)?;
    let c = ApolloClient::new(&r);

    let existing = c.try_get_item(ns, key)?;
    if existing.is_none() {
        println!("Key \"{}\" not found in namespace \"{}\". Nothing to delete.", key, ns);
        return Ok(());
    }
    let item = existing.unwrap();

    println!();
    println!("  {} configuration", "DELETE".red().bold());
    println!("  Env       : {}", r.env);
    println!("  App       : {}", r.app_id);
    println!("  Namespace : {}", ns);
    println!("  Key       : {}", key);
    println!("  Value     : {}", item.value.dimmed());
    println!("  Operator  : {}", r.operator);
    println!();

    if !confirm("This cannot be undone. Proceed?", yes)? {
        println!("Cancelled.");
        return Ok(());
    }

    c.delete_item(ns, key, &r.operator)?;
    println!("{}", "Deleted.".green().bold());
    Ok(())
}

// ── publish ────────────────────────────────────────────────────

fn cmd_publish(
    cli: &Cli,
    ns: &str,
    title: Option<String>,
    comment: Option<String>,
    operator: Option<&str>,
    yes: bool,
) -> Result<()> {
    let r = resolve(cli, operator)?;
    guard_pro_write(&r)?;
    let c = ApolloClient::new(&r);

    let release_title = title.unwrap_or_else(|| {
        chrono_free_title()
    });

    println!();
    println!("  {} namespace", "PUBLISH".cyan().bold());
    println!("  Env       : {}", r.env);
    println!("  App       : {}", r.app_id);
    println!("  Namespace : {}", ns);
    println!("  Title     : {}", release_title);
    if let Some(ref c) = comment {
        println!("  Comment   : {}", c);
    }
    println!("  Operator  : {}", r.operator);
    println!();

    if !confirm("Proceed?", yes)? {
        println!("Cancelled.");
        return Ok(());
    }

    let req = PublishRequest {
        release_title,
        release_comment: comment,
        released_by: r.operator.clone(),
    };
    let info = c.publish(ns, &req)?;
    println!("{} Release: {}", "Published.".green().bold(), info.name);
    Ok(())
}

fn chrono_free_title() -> String {
    format!("apl-cli-release")
}

// ── helpers ────────────────────────────────────────────────────

fn resolve(cli: &Cli, operator: Option<&str>) -> Result<Resolved> {
    Resolved::from_cli(
        cli.portal_url.as_deref(),
        cli.token.as_deref(),
        cli.env.as_deref(),
        cli.app_id.as_deref(),
        cli.cluster.as_deref(),
        operator,
    )
}

fn guard_pro_write(r: &Resolved) -> Result<()> {
    if r.is_pro() {
        bail!(
            "Write operations are blocked for PRO environment. \
             Use Apollo Portal directly for production changes."
        );
    }
    Ok(())
}

fn confirm(prompt: &str, yes: bool) -> Result<bool> {
    if yes {
        return Ok(true);
    }
    if !std::io::stdin().is_terminal() {
        bail!(
            "Confirmation required but stdin is not a terminal. \
             Pass --yes to skip, or run in an interactive terminal."
        );
    }
    Ok(dialoguer::Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()?)
}
