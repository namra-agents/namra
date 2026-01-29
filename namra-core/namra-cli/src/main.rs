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

    /// View and manage run history
    Runs {
        #[command(subcommand)]
        command: RunsCommand,
    },

    /// Display version information
    Version,
}

#[derive(Subcommand)]
enum RunsCommand {
    /// List recent runs
    List {
        /// Filter by agent name
        #[arg(long)]
        agent: Option<String>,

        /// Maximum number of runs to show
        #[arg(long, default_value = "20")]
        limit: u32,

        /// Show runs since duration (e.g., 1h, 24h, 7d)
        #[arg(long)]
        since: Option<String>,

        /// Show only successful runs
        #[arg(long)]
        success: bool,

        /// Show only failed runs
        #[arg(long)]
        failed: bool,
    },

    /// Show details of a specific run
    Show {
        /// Run ID (or prefix)
        id: String,

        /// Show verbose output including full tool inputs/outputs
        #[arg(short, long)]
        verbose: bool,
    },

    /// Export runs to file
    Export {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Export format: json, csv, or excel
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Filter by agent name
        #[arg(long)]
        agent: Option<String>,

        /// Include tool call details
        #[arg(long)]
        include_tools: bool,

        /// Include reasoning/thought steps
        #[arg(long)]
        include_thoughts: bool,
    },

    // NOTE: Delete command is implemented but not exposed to users yet
    // /// Delete runs
    // Delete {
    //     /// Run ID to delete
    //     id: Option<String>,
    //
    //     /// Delete runs older than duration (e.g., 30d, 7d)
    //     #[arg(long)]
    //     older_than: Option<String>,
    //
    //     /// Confirm deletion (required)
    //     #[arg(long)]
    //     confirm: bool,
    // },
    /// Show run statistics
    Stats {
        /// Filter by agent name
        #[arg(long)]
        agent: Option<String>,

        /// Time range for stats (e.g., 7d, 30d)
        #[arg(long, default_value = "7d")]
        range: String,
    },
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

        Commands::Runs { command } => match command {
            RunsCommand::List {
                agent,
                limit,
                since,
                success,
                failed,
            } => {
                commands::runs::list(agent.as_deref(), limit, since.as_deref(), success, failed)?;
            }

            RunsCommand::Show { id, verbose } => {
                commands::runs::show(&id, verbose)?;
            }

            RunsCommand::Export {
                output,
                format,
                agent,
                include_tools,
                include_thoughts,
            } => {
                commands::runs::export(
                    &output,
                    &format,
                    agent.as_deref(),
                    include_tools,
                    include_thoughts,
                )?;
            }

            // RunsCommand::Delete {
            //     id,
            //     older_than,
            //     confirm,
            // } => {
            //     commands::runs::delete(id.as_deref(), older_than.as_deref(), confirm)?;
            // }
            RunsCommand::Stats { agent, range } => {
                commands::runs::stats(agent.as_deref(), &range)?;
            }
        },

        Commands::Version => {
            println!("namra {}", env!("CARGO_PKG_VERSION"));
            println!("Rust runtime version: {}", rustc_version());
        }
    }

    Ok(())
}

fn rustc_version() -> &'static str {
    "1.75+"
}
