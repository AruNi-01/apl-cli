mod cli;
mod client;
mod commands;
mod config;
mod models;
mod output;
mod upgrade;

use clap::Parser;

fn main() {
    let parsed = cli::Cli::parse();
    let is_upgrade = matches!(parsed.command, cli::Commands::Upgrade);

    if let Err(e) = commands::execute(parsed) {
        eprintln!("\x1b[31merror:\x1b[0m {:#}", e);
        std::process::exit(1);
    }

    if !is_upgrade {
        upgrade::check_version_hint();
    }
}
