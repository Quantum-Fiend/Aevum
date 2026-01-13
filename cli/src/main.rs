use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing_subscriber;

mod commands;

use commands::{attach, record, replay, inspect, rewind};

#[derive(Parser)]
#[command(name = "aevum")]
#[command(about = "Aevum - Time-Travel Debugging Platform", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Attach to a running process
    Attach {
        /// Process ID to attach to
        #[arg(short, long)]
        pid: u32,

        /// Output trace file
        #[arg(short, long)]
        output: String,
    },

    /// Record execution of a command
    Record {
        /// Command to execute
        command: String,

        /// Arguments for the command
        args: Vec<String>,

        /// Output trace file
        #[arg(short, long, default_value = "trace.aevum")]
        output: String,

        /// Enable cluster recording mode
        #[arg(long)]
        cluster: bool,
    },

    /// Replay a trace file
    Replay {
        /// Trace file to replay
        trace_file: String,

        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
    },

    /// Rewind to a specific point in the trace
    Rewind {
        /// Trace file
        trace_file: String,

        /// Step number to rewind to
        #[arg(short, long)]
        step: u64,
    },

    /// Inspect a trace file
    Inspect {
        /// Trace file to inspect
        trace_file: String,

        /// Show causality analysis
        #[arg(long)]
        causality: bool,

        /// Filter by event type
        #[arg(short, long)]
        event_type: Option<String>,
    },

    /// Compare two traces
    Diff {
        /// First trace file
        trace1: String,

        /// Second trace file
        trace2: String,
    },

    /// List all traces in a directory
    List {
        /// Directory to search
        #[arg(default_value = ".")]
        directory: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    match cli.command {
        Commands::Attach { pid, output } => {
            attach::execute(pid, &output).await?;
        }
        Commands::Record { command, args, output, cluster } => {
            record::execute(&command, &args, &output, cluster).await?;
        }
        Commands::Replay { trace_file, interactive } => {
            replay::execute(&trace_file, interactive).await?;
        }
        Commands::Rewind { trace_file, step } => {
            rewind::execute(&trace_file, step).await?;
        }
        Commands::Inspect { trace_file, causality, event_type } => {
            inspect::execute(&trace_file, causality, event_type.as_deref()).await?;
        }
        Commands::Diff { trace1, trace2 } => {
            println!("Comparing {} and {}", trace1, trace2);
            println!("Diff functionality coming soon!");
        }
        Commands::List { directory } => {
            println!("Listing traces in {}", directory);
            println!("List functionality coming soon!");
        }
    }

    Ok(())
}
