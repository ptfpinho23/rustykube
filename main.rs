mod commands;
mod utils;
mod lint_rules;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Rusty Kube")]
#[command(about = "A Rust-based CLI tool for Kubernetes linting and optimization")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Lint {
        #[arg(short, long)]
        path: String,

        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Lint { path, json } => commands::lint::run_lint(path, *json),
    }
}
