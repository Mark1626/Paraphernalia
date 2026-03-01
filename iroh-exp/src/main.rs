mod progress;
mod receive;
mod send;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sendme", about = "Multi-provider file transfer with iroh-blobs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a file or directory
    Send {
        /// Path to the file or directory to send
        path: PathBuf,
    },
    /// Receive from one or more providers
    Receive {
        /// Primary provider ticket
        ticket: String,
        /// Additional provider tickets for parallel download
        #[arg(short, long = "provider")]
        providers: Vec<String>,
        /// Download strategy: "split" (parallel, default) or "sequential" (fallback)
        #[arg(long, default_value = "split")]
        strategy: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Send { path } => send::run(path).await,
        Commands::Receive {
            ticket,
            providers,
            strategy,
        } => receive::run(ticket, providers, strategy).await,
    }
}
