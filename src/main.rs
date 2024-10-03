use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Rusty Kube")]
#[command(about = "A Rust-based CLI tool for Kubernetes linting and optimization", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint Kubernetes manifests for issues
    Lint {
        /// Path to the manifest file or directory
        #[arg(short, long)]
        path: String,
    },

    /// Optimize Kubernetes manifests for performance
    Optimize {
        /// Path to the manifest file or directory
        #[arg(short, long)]
        path: String,
    },
}


fn main() {
    let cli = Cli::parse();  // This will parse the CLI arguments.
    let y = 2i64;
    println!("{:p}", &y);


    match &cli.command {
        Commands::Lint { path } => {
            println!("Linting Kubernetes manifests at: {}", path);
        }
        Commands::Optimize { path } => {
            println!("Optimizing Kubernetes manifests at: {}", path);
        }
    }
}
