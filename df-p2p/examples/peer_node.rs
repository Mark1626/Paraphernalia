//! P2P peer node with dynamic queue-based scheduling.
//!
//! Each peer:
//! 1. Binds an iroh endpoint and saves its address to `peers/` directory
//! 2. Joins a gossip topic for work queue coordination
//! 3. Runs as both a producer (can submit work to the queue) and a consumer
//!    (pulls work from the queue and executes it)
//! 4. Serves SQL query handling on the df-p2p-query ALPN -- when a query arrives,
//!    the optimizer inserts DistributedCoalesceExec which submits work to the queue,
//!    and available peer consumers pick it up dynamically
//!
//! Use `query_client` to send SQL queries to any peer.
//!
//! Examples:
//!   # Terminal 1:
//!   cargo run --example peer_node -- --table-path testdata/sample.csv
//!
//!   # Terminal 2:
//!   cargo run --example peer_node -- --table-path testdata/sample.csv
//!
//!   # Terminal 3 (query any peer):
//!   cargo run --example query_client -- "SELECT city, SUM(amount) as total FROM data GROUP BY city"

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use arrow::array::RecordBatch;
use clap::Parser;
use datafusion::datasource::file_format::parquet::ParquetFormat;
use datafusion::datasource::listing::{
    ListingOptions, ListingTable, ListingTableConfig, ListingTableUrl,
};
use datafusion::physical_plan::collect;
use datafusion::physical_plan::display::DisplayableExecutionPlan;
use datafusion::prelude::*;
use df_p2p::context::{DistributedContext, QUERY_ALPN};
use df_p2p::optimizer_rule::apply_network_boundaries;
use df_p2p::queue::{self, Consumer, GossipBridge, JobStore, Producer, QueueProtocol};
use df_p2p::worker::Worker;
use iroh::protocol::Router;
use iroh::{EndpointAddr, EndpointId};
use iroh_arrow::protocol::{IrohArrow, RequestHandler};
use iroh_gossip::net::Gossip;
use tracing::info;

#[derive(Parser)]
struct Args {
    /// Path to a CSV file to register as a single table
    #[arg(long)]
    table_path: Option<String>,

    /// Name for the CSV table (used with --table-path)
    #[arg(long, default_value = "data")]
    table_name: String,

    /// Directory containing parquet tables (each subdirectory becomes a table)
    #[arg(long)]
    dataset_dir: Option<String>,

    /// Directory for peer address auto-discovery
    #[arg(long, default_value = "peers")]
    peers_dir: String,
}

/// Handles SQL queries from query_client: plans, distributes via queue, executes.
#[derive(Clone)]
struct SqlQueryHandler {
    ctx: SessionContext,
}

impl std::fmt::Debug for SqlQueryHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqlQueryHandler").finish_non_exhaustive()
    }
}

impl RequestHandler for SqlQueryHandler {
    async fn handle(&self, request: &[u8]) -> anyhow::Result<Vec<RecordBatch>> {
        let sql = std::str::from_utf8(request)?;
        info!(sql, "received SQL query from client");

        // Plan and execute -- DistributedCoalesceExec submits work to the queue
        let logical = self.ctx.state().create_logical_plan(sql).await?;
        let physical = self.ctx.state().create_physical_plan(&logical).await?;

        let disp = DisplayableExecutionPlan::new(physical.as_ref());
        info!("Physical plan:\n{}", disp.indent(false));

        let distributed = apply_network_boundaries(physical)?;

        let disp = DisplayableExecutionPlan::new(distributed.as_ref());
        info!("Distributed plan:\n{}", disp.indent(false));

        let batches = collect(distributed, self.ctx.task_ctx()).await?;
        Ok(batches)
    }
}

/// Read peer address files from the peers directory, skipping our own.
fn discover_peers(dir: &Path, own_id: &EndpointId) -> Result<Vec<EndpointAddr>> {
    let own_id_str = own_id.to_string();
    let mut addrs = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(addrs),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if stem == own_id_str {
                continue;
            }
        }
        let json = std::fs::read_to_string(&path)?;
        let addr: EndpointAddr = serde_json::from_str(&json)?;
        addrs.push(addr);
    }

    Ok(addrs)
}

/// Register all subdirectories in `dir` as hive-partitioned parquet tables.
async fn register_parquet_tables(ctx: &SessionContext, dir: &str) -> Result<()> {
    let base = Path::new(dir);
    let mut count = 0;

    for entry in std::fs::read_dir(base).context(format!("failed to read {dir}"))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) if !n.starts_with('_') && !n.starts_with('.') => n.to_string(),
            _ => continue,
        };

        let table_url = ListingTableUrl::parse(path.to_str().unwrap())?;
        let format = Arc::new(ParquetFormat::default());
        let options = ListingOptions::new(format).with_collect_stat(false);

        let config = ListingTableConfig::new(table_url)
            .with_listing_options(options)
            .infer_schema(&ctx.state())
            .await
            .context(format!("failed to infer schema for {name}"))?;

        let table = ListingTable::try_new(config)?;
        ctx.register_table(&name, Arc::new(table))?;
        count += 1;
    }

    info!(count, dir, "registered parquet tables");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    let peers_dir = PathBuf::from(&args.peers_dir);
    std::fs::create_dir_all(&peers_dir)?;

    // Bind endpoint
    let endpoint = iroh::Endpoint::builder().bind().await?;
    endpoint.online().await;
    let own_id = endpoint.id();

    // Save our address so other peers (and query_client) can discover us
    let addr = endpoint.addr();
    let addr_path = peers_dir.join(format!("{own_id}.json"));
    std::fs::write(&addr_path, serde_json::to_string_pretty(&addr)?)?;
    info!(path = %addr_path.display(), "address saved");

    // Discover peers for gossip bootstrapping
    let peer_addrs = discover_peers(&peers_dir, &own_id)?;
    let peer_ids: Vec<EndpointId> = peer_addrs.iter().map(|a| a.id).collect();
    info!(peer_count = peer_ids.len(), "discovered peers for gossip bootstrap");

    // Set up gossip
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let topic = queue::work_queue_topic();
    let bridge = GossipBridge::new(gossip.clone(), topic);
    let (sender, receiver) = bridge.join(peer_ids).await?;

    // Set up job store and queue protocol handler
    let store = JobStore::new();
    let queue_handler = QueueProtocol::new(store.clone());

    // Local channel so the co-located consumer hears about its own producer's jobs
    // (gossip doesn't deliver messages back to the sender)
    let (local_tx, local_rx) = tokio::sync::mpsc::unbounded_channel();

    // Create producer
    let consumer_store = store.clone();
    let producer = Arc::new(Producer::new(
        store,
        sender.clone(),
        local_tx,
        own_id,
        Duration::from_secs(120),
    ));

    // Start stale job reaper
    let reaper = producer.clone();
    tokio::spawn(async move { reaper.run_reaper().await });

    // Create DistributedContext and SessionContext
    let dist_ctx = DistributedContext::new(endpoint.clone(), producer);
    let config = SessionConfig::new()
        .set_bool("datafusion.optimizer.enable_dynamic_filter_pushdown", false)
        .with_extension(Arc::new(dist_ctx));
    let ctx = SessionContext::new_with_config(config);

    // Register tables
    if let Some(dataset_dir) = &args.dataset_dir {
        register_parquet_tables(&ctx, dataset_dir).await?;
    } else if let Some(table_path) = &args.table_path {
        ctx.register_csv(&args.table_name, table_path, CsvReadOptions::default())
            .await?;
        info!(
            table = args.table_name,
            path = table_path,
            "registered csv table"
        );
    } else {
        anyhow::bail!("provide either --table-path or --dataset-dir");
    }

    // Create consumer and spawn worker loop
    // Pass local_store so the consumer can claim/ack local jobs directly
    // (iroh doesn't support self-connections)
    let consumer = Consumer::new(endpoint.clone(), receiver, sender, local_rx, consumer_store);
    let _worker = Worker::spawn(ctx.clone(), consumer);

    // Build router with queue + gossip + query protocols
    let query_handler = IrohArrow::new(SqlQueryHandler { ctx });

    let router = Router::builder(endpoint)
        .accept(queue::QUEUE_ALPN, queue_handler)
        .accept(iroh_gossip::ALPN, gossip.clone())
        .accept(QUERY_ALPN, query_handler)
        .spawn();

    info!(%own_id, "peer listening with dynamic queue scheduling (Ctrl+C to stop)");

    tokio::signal::ctrl_c().await?;
    info!("shutting down");

    // Clean up address file
    let _ = std::fs::remove_file(&addr_path);

    router
        .shutdown()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(())
}
