use crate::cli::OutputFormat;
use crate::models::*;
use colored::Colorize;
use std::collections::BTreeMap;

pub fn namespaces(list: &[NamespaceInfo], fmt: &OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            let data: Vec<serde_json::Value> = list
                .iter()
                .map(|ns| {
                    serde_json::json!({
                        "name": ns.namespace_name,
                        "format": ns.format,
                        "isPublic": ns.is_public,
                        "itemCount": ns.items.len(),
                        "comment": ns.comment,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
        OutputFormat::Text => {
            println!(
                "{:<35} {:<12} {:<8} {}",
                "Namespace".bold(),
                "Format".bold(),
                "Public".bold(),
                "Items".bold()
            );
            println!("{}", "─".repeat(70));
            for ns in list {
                println!(
                    "{:<35} {:<12} {:<8} {}",
                    ns.namespace_name,
                    ns.format,
                    if ns.is_public { "Yes" } else { "No" },
                    ns.items.len()
                );
            }
        }
    }
}

pub fn items(list: &[ConfigItem], fmt: &OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            let map: BTreeMap<&str, &str> = list
                .iter()
                .map(|i| (i.key.as_str(), i.value.as_str()))
                .collect();
            println!("{}", serde_json::to_string_pretty(&map).unwrap());
        }
        OutputFormat::Text => {
            if list.is_empty() {
                println!("(no items)");
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

pub fn single_item(item: &ConfigItem, fmt: &OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            let map = serde_json::json!({ &item.key: &item.value });
            println!("{}", serde_json::to_string_pretty(&map).unwrap());
        }
        OutputFormat::Text => {
            println!("{}: {}", item.key.bold(), item.value);
            if let Some(ref c) = item.comment {
                if !c.is_empty() {
                    println!("  {}", format!("# {c}").dimmed());
                }
            }
        }
    }
}

pub fn env_clusters(list: &[EnvCluster], fmt: &OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(list).unwrap());
        }
        OutputFormat::Text => {
            for ec in list {
                println!("{}:  {}", ec.env.bold(), ec.clusters.join(", "));
            }
        }
    }
}
