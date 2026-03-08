//! P2P peer node — starts an iroh endpoint, registers data, and waits for queries.
//!
//! Each peer:
//! 1. Binds an iroh endpoint and saves its address to `peers/` directory
//! 2. Serves physical plan execution (worker role) on the iroh-arrow ALPN
//! 3. Serves SQL query handling on the df-p2p-query ALPN — when a query arrives,
//!    it auto-discovers other peers from `peers/`, plans the SQL, applies the
//!    distributed optimizer rule, and executes across discovered peers
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
use iroh::protocol::Router;
use iroh::{EndpointAddr, EndpointId};
use iroh_arrow::protocol::{ALPN, IrohArrow, RequestHandler};
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

/// Handles SQL queries from query_client: discovers peers, plans, distributes, executes.
#[derive(Clone)]
struct SqlQueryHandler {
    ctx: SessionContext,
    peers_dir: PathBuf,
    own_id: EndpointId,
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

        // Discover peers at query time (auto-detect new nodes)
        let peers = discover_peers(&self.peers_dir, &self.own_id)?;
        info!(peer_count = peers.len(), "discovered peers");

        if peers.is_empty() {
            anyhow::bail!("no peer workers discovered in {}", self.peers_dir.display());
        }

        // Update DistributedContext with currently known peers
        let dist_ctx = self
            .ctx
            .state()
            .config()
            .get_extension::<DistributedContext>()
            .expect("DistributedContext must be registered");
        dist_ctx.set_workers(peers);

        // Plan and execute distributed query
        let logical = self.ctx.state().create_logical_plan(sql).await?;
        let physical = self.ctx.state().create_physical_plan(&logical).await?;

        let disp_physical_plan = DisplayableExecutionPlan::new(physical.as_ref());
        info!("Received plan {}", disp_physical_plan.indent(false));

        let distributed = apply_network_boundaries(physical)?;

        let disp_physical_plan = DisplayableExecutionPlan::new(distributed.as_ref());
        info!("Distributed plan {}", disp_physical_plan.indent(false));

        let batches = collect(distributed, self.ctx.task_ctx()).await;

        // Close pooled connections regardless of success/failure
        dist_ctx.close_connections();

        Ok(batches?)
    }
}

/// Scan the peers directory for address files, skipping our own.
fn discover_peers(dir: &PathBuf, own_id: &EndpointId) -> Result<Vec<EndpointAddr>> {
    let own_id_str = own_id.to_string();
    let mut addrs = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        // Skip our own address file
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
        // Skip hidden directories and _delta_log
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

    // Create session context with DistributedContext (workers discovered dynamically)
    let dist_ctx = DistributedContext::new(endpoint.clone());
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

    // Build router with both protocol handlers on the same endpoint
    let plan_handler = df_p2p::worker::plan_protocol_handler(ctx.clone());
    let query_handler = IrohArrow::new(SqlQueryHandler {
        ctx,
        peers_dir,
        own_id,
    });

    let router = Router::builder(endpoint)
        .accept(ALPN, plan_handler)
        .accept(QUERY_ALPN, query_handler)
        .spawn();

    info!(%own_id, "peer listening, waiting for queries (Ctrl+C to stop)");

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
