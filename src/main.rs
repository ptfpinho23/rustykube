use clap::{Parser, Subcommand};
use serde::Deserialize;
use serde_yaml::Value;
use std::fs;

#[derive(Parser)]
#[command(name = "Rusty Kube")]
#[command(about = "A Rust-based CLI tool for Kubernetes linting and optimization", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Lint {
        #[arg(short, long)]
        path: String,
    },

    Optimize {
        #[arg(short, long)]
        path: String,
    },
}

#[derive(Debug, Deserialize)]
struct Metadata {
    name: Option<String>,
    labels: Option<std::collections::HashMap<String, String>>,
}

fn lint_manifest(path: &str) {

    let contents = fs::read_to_string(path).expect("Failed to read file");

    let docs: Vec<Value> = match serde_yaml::Deserializer::from_str(&contents).collect::<Vec<_>>() {
        v if v.is_empty() => {
            println!("Error: The provided YAML file is empty.");
            return;
        }
        v => v.into_iter().map(|doc| serde_yaml::Value::deserialize(doc).unwrap()).collect(),
    };

    for (i, doc) in docs.iter().enumerate() {
        println!("Processing document #{}", i + 1);
        println!("Processing - #{:?}", doc);
        
        if let Some(metadata) = doc.get("metadata") {
            let name = metadata.get("name").and_then(Value::as_str).unwrap_or("<no name>");
            println!("Checking resource: {}", name);

            if metadata.get("labels").is_none() {
                println!("Warning: Resource '{}' is missing labels!", name);
            } else {
                println!("Resource '{}' has labels.", name);
            }
        } else {
            println!("Warning: Missing metadata in document #{}", i + 1);
        }
    }
}


fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Lint { path } => {
            println!("Linting Kubernetes manifests at: {}", path);
            lint_manifest(path);
        }
        Commands::Optimize { path } => {
            println!("Optimizing Kubernetes manifests at: {}", path);
        }
    }
}
