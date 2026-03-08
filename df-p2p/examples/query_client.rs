//! Query client — sends a SQL query to any peer node and displays results.
//!
//! Discovers peers from the shared `peers/` directory (or connects to a
//! specific peer via `--peer`). The receiving peer plans the query, distributes
//! execution across its known peers, and returns the results.
//!
//! Examples:
//!   cargo run --example query_client -- "SELECT city, SUM(amount) as total FROM data GROUP BY city"
//!   cargo run --example query_client -- --peer peers/<id>.json "SELECT * FROM data"

use anyhow::{Context, Result};
use arrow::util::pretty::pretty_format_batches;
use clap::Parser;
use df_p2p::context::QUERY_ALPN;
use iroh::EndpointAddr;
use iroh_arrow::protocol::MAX_RESPONSE_SIZE;
use tracing::info;

#[derive(Parser)]
struct Args {
    /// SQL query to execute
    sql: String,

    /// Directory containing peer address files
    #[arg(long, default_value = "peers")]
    peers_dir: String,

    /// Specific peer address file to connect to (overrides auto-discovery)
    #[arg(long)]
    peer: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();

    // Find a peer to connect to
    let peer_addr: EndpointAddr = if let Some(peer_file) = &args.peer {
        let json =
            std::fs::read_to_string(peer_file).context(format!("failed to read {peer_file}"))?;
        serde_json::from_str(&json).context(format!("failed to parse {peer_file}"))?
    } else {
        find_any_peer(&args.peers_dir)?
    };

    let endpoint = iroh::Endpoint::builder().bind().await?;

    info!("connecting to peer");
    let conn = endpoint
        .connect(peer_addr, QUERY_ALPN)
        .await
        .context("failed to connect to peer")?;
    let (mut send, mut recv) = conn.open_bi().await?;

    // Send SQL query, receive Arrow IPC results
    iroh_arrow::stream::send_bytes(&mut send, args.sql.as_bytes()).await?;
    let batches = iroh_arrow::stream::recv_batches(&mut recv, MAX_RESPONSE_SIZE).await?;

    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    info!(num_batches = batches.len(), total_rows, "received results");

    let formatted = pretty_format_batches(&batches)?;
    info!("\n{formatted}");

    Ok(())
}

/// Pick any peer address file from the directory.
fn find_any_peer(dir: &str) -> Result<EndpointAddr> {
    for entry in std::fs::read_dir(dir).context(format!("failed to read {dir}/"))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let json = std::fs::read_to_string(&path)?;
            return serde_json::from_str(&json)
                .context(format!("failed to parse {}", path.display()));
        }
    }
    anyhow::bail!("no peer address files found in {dir}/")
}
