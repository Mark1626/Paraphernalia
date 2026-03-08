use anyhow::{Context, Result};
use arrow::array::RecordBatch;
use datafusion::physical_plan::display::DisplayableExecutionPlan;
use datafusion::prelude::SessionContext;
use datafusion_proto::bytes::physical_plan_from_bytes;
use futures::StreamExt;
use iroh::Endpoint;
use iroh::protocol::Router;
use iroh_arrow::protocol::{ALPN, IrohArrow, RequestHandler};
use tracing::info;

/// Receives serialized DataFusion physical plans with a partition number,
/// executes only the specified partition, and returns record batches.
///
/// Wire format: `[partition as u32 LE][plan bytes]`
#[derive(Clone)]
struct PhysicalPlanHandler {
    ctx: SessionContext,
}

impl std::fmt::Debug for PhysicalPlanHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicalPlanHandler")
            .finish_non_exhaustive()
    }
}

impl RequestHandler for PhysicalPlanHandler {
    async fn handle(&self, request: &[u8]) -> anyhow::Result<Vec<RecordBatch>> {
        anyhow::ensure!(
            request.len() >= 4,
            "request too short: missing partition header"
        );

        let partition = u32::from_le_bytes(request[..4].try_into().unwrap()) as usize;
        let plan_bytes = &request[4..];

        info!(
            request_len = request.len(),
            partition, "received plan from peer"
        );

        let task_ctx = self.ctx.task_ctx();
        let plan = physical_plan_from_bytes(plan_bytes, &task_ctx)
            .context("failed to deserialize physical plan")?;

        let displayable_plan = DisplayableExecutionPlan::new(plan.as_ref());
        info!(
            partition,
            "executing physical plan for partition {}:\n{}",
            partition,
            displayable_plan.indent(false)
        );

        let mut stream = plan
            .execute(partition, task_ctx)
            .context("failed to execute partition")?;

        let mut batches = Vec::new();
        while let Some(batch) = stream.next().await {
            batches.push(batch.context("error reading batch from partition stream")?);
        }

        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        info!(
            partition,
            num_batches = batches.len(),
            total_rows,
            "sending results back to peer"
        );

        Ok(batches)
    }
}

/// Create the IrohArrow protocol handler for physical plan execution.
/// Use this when building a custom Router with multiple protocol handlers.
pub fn plan_protocol_handler(ctx: SessionContext) -> impl iroh::protocol::ProtocolHandler {
    IrohArrow::new(PhysicalPlanHandler { ctx }).with_max_request_size(1024 * 1024)
}

/// A worker node that accepts DataFusion physical plans over iroh and streams back results.
///
/// Uses iroh's `Router` with `IrohArrow<PhysicalPlanHandler>` — the accept loop is spawned
/// automatically when the worker is created.
pub struct Worker {
    router: Router,
}

impl Worker {
    /// Create a new worker. The accept loop starts immediately in the background.
    pub async fn new(ctx: SessionContext) -> Result<Self> {
        let endpoint = Endpoint::builder()
            .bind()
            .await
            .context("failed to bind iroh endpoint")?;

        let handler =
            IrohArrow::new(PhysicalPlanHandler { ctx }).with_max_request_size(1024 * 1024);
        let router = Router::builder(endpoint).accept(ALPN, handler).spawn();

        info!(endpoint_id = %router.endpoint().id(), "worker accepting connections");

        Ok(Self { router })
    }

    /// Create a worker using an existing endpoint. Use this when the endpoint
    /// is shared with other roles (e.g. a peer node that is both worker and scheduler).
    pub async fn with_endpoint(ctx: SessionContext, endpoint: Endpoint) -> Result<Self> {
        let handler =
            IrohArrow::new(PhysicalPlanHandler { ctx }).with_max_request_size(1024 * 1024);
        let router = Router::builder(endpoint).accept(ALPN, handler).spawn();

        info!(endpoint_id = %router.endpoint().id(), "worker accepting connections");

        Ok(Self { router })
    }

    /// Access the underlying iroh endpoint (for address info, node ID, etc.).
    pub fn endpoint(&self) -> &Endpoint {
        self.router.endpoint()
    }

    /// Shut down the worker and close all connections.
    pub async fn shutdown(self) -> Result<()> {
        self.router
            .shutdown()
            .await
            .map_err(|e| anyhow::anyhow!("shutdown error: {e}"))?;
        Ok(())
    }
}
