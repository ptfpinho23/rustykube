mod commands;
mod utils;
mod lint_rules;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rustykube")]
#[command(about = "🚀 Blazing fast CLI tool for Kubernetes resource management through linting, optimization & validation")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint Kubernetes manifests for best practices and issues
    Lint {
        /// Path to the Kubernetes manifest file or directory
        #[arg(short, long)]
        path: String,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,

        /// Exit with non-zero code on any issues found
        #[arg(long)]
        strict: bool,

        /// Specify which rules to run (comma-separated)
        #[arg(long)]
        rules: Option<String>,
    },
    
    /// Validate Kubernetes manifests syntax and schema
    Validate {
        /// Path to the Kubernetes manifest file or directory
        #[arg(short, long)]
        path: String,

        /// Kubernetes API version to validate against
        #[arg(long, default_value = "1.28")]
        api_version: String,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },
    
    /// Optimize Kubernetes manifests for better performance and resource usage
    Optimize {
        /// Path to the Kubernetes manifest file or directory
        #[arg(short, long)]
        path: String,

        /// Output optimized manifests to specified directory
        #[arg(short, long)]
        output: Option<String>,

        /// Apply optimizations in-place (overwrites original files)
        #[arg(long)]
        in_place: bool,

        /// Enable aggressive optimizations (may change behavior)
        #[arg(long)]
        aggressive: bool,
    },
    
    /// Automatically fix common issues in Kubernetes manifests
    Fix {
        /// Path to the Kubernetes manifest file or directory
        #[arg(short, long)]
        path: String,

        /// Apply fixes in-place (overwrites original files)
        #[arg(long)]
        in_place: bool,

        /// Dry run - show what would be fixed without making changes
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Analyze and provide detailed insights about Kubernetes manifests
    Analyze {
        /// Path to the Kubernetes manifest file or directory
        #[arg(short, long)]
        path: String,

        /// Generate detailed report with recommendations
        #[arg(long)]
        detailed: bool,

        /// Output analysis in JSON format
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        println!("🔧 Running RustyKube in verbose mode...");
    }

    match &cli.command {
        Commands::Lint { path, json, strict, rules } => {
            commands::lint::run_lint(path, *json, *strict, rules.as_deref(), cli.verbose)
        },
        Commands::Validate { path, api_version, json } => {
            commands::validate::run_validate(path, api_version, *json, cli.verbose)
        },
        Commands::Optimize { path, output, in_place, aggressive } => {
            commands::optimize::run_optimize(path, output.as_deref(), *in_place, *aggressive, cli.verbose)
        },
        Commands::Fix { path, in_place, dry_run } => {
            commands::fix::run_fix(path, *in_place, *dry_run, cli.verbose)
        },
        Commands::Analyze { path, detailed, json } => {
            commands::analyze::run_analyze(path, *detailed, *json, cli.verbose)
        },
    }
}
