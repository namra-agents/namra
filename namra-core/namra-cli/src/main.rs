//! Namra CLI - Command-line interface for the Namra agent framework

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;

#[derive(Parser)]
#[command(name = "namra")]
#[command(about = "Namra - Enterprise Agent Framework", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Namra project
    Init {
        /// Project name
        name: String,

        /// Namespace for multi-tenancy
        #[arg(long)]
        namespace: Option<String>,
    },

    /// Validate agent configuration files
    Validate {
        /// Path to configuration file(s)
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,
    },

    /// Run an agent
    Run {
        /// Path to agent configuration file
        #[arg(value_name = "FILE")]
        config: PathBuf,

        /// Input prompt for the agent
        #[arg(short, long)]
        input: String,

        /// Enable streaming output
        #[arg(short, long)]
        stream: bool,
    },

    /// Display version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, namespace } => {
            commands::init::execute(&name, namespace.as_deref())?;
        }

        Commands::Validate { files } => {
            commands::validate::execute(&files)?;
        }

        Commands::Run {
            config,
            input,
            stream,
        } => {
            commands::run::execute(&config, &input, stream).await?;
        }

        Commands::Version => {
            println!("nexus {}", env!("CARGO_PKG_VERSION"));
            println!("Rust runtime version: {}", rustc_version());
        }
    }

    Ok(())
}

fn rustc_version() -> &'static str {
    "1.75+"
}
