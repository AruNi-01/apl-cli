mod cli;
mod client;
mod commands;
mod config;
mod models;
mod output;

use clap::Parser;

fn main() {
    if let Err(e) = commands::execute(cli::Cli::parse()) {
        eprintln!("\x1b[31merror:\x1b[0m {:#}", e);
        std::process::exit(1);
    }
}
