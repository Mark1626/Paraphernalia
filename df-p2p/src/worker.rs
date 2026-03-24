use anyhow::Result;
use datafusion::physical_plan::display::DisplayableExecutionPlan;
use datafusion::prelude::SessionContext;
use datafusion_proto::bytes::physical_plan_from_bytes;
use futures::StreamExt;
use tracing::info;
use uuid::Uuid;

use crate::queue::Consumer;

/// A worker that pulls execution plans from the distributed queue and processes them.
///
/// Wraps a `Consumer` loop that listens for gossip signals, claims available jobs,
/// deserializes and executes DataFusion physical plans, and acks with Arrow IPC results.
pub struct Worker {
    handle: tokio::task::JoinHandle<crate::queue::Result<()>>,
}

impl Worker {
    /// Spawn a consumer loop that pulls plans from the queue and executes them.
    pub fn spawn(ctx: SessionContext, mut consumer: Consumer) -> Self {
        let handle = tokio::spawn(async move {
            consumer
                .run(move |job_id, payload| {
                    let ctx = ctx.clone();
                    async move { execute_plan(&ctx, job_id, payload).await }
                })
                .await
        });

        Self { handle }
    }

    /// Wait for the worker to finish (runs until gossip disconnects or error).
    pub async fn join(self) -> Result<()> {
        self.handle
            .await
            .map_err(|e| anyhow::anyhow!("worker task panicked: {e}"))?
            .map_err(|e| anyhow::anyhow!("worker error: {e}"))
    }
}

/// Execute a serialized DataFusion physical plan from a queue job payload.
///
/// Payload wire format: `[partition as u32 LE][plan protobuf bytes]`
async fn execute_plan(
    ctx: &SessionContext,
    job_id: Uuid,
    payload: Vec<u8>,
) -> std::result::Result<Option<Vec<u8>>, String> {
    if payload.len() < 4 {
        return Err("payload too short: missing partition header".to_string());
    }

    let partition = u32::from_le_bytes(payload[..4].try_into().unwrap()) as usize;
    let plan_bytes = &payload[4..];

    info!(
        %job_id,
        partition,
        plan_len = plan_bytes.len(),
        "executing plan from queue"
    );

    let task_ctx = ctx.task_ctx();
    let plan = physical_plan_from_bytes(plan_bytes, &task_ctx)
        .map_err(|e| format!("failed to deserialize physical plan: {e}"))?;

    let displayable = DisplayableExecutionPlan::new(plan.as_ref());
    info!(
        %job_id,
        partition,
        "plan:\n{}",
        displayable.indent(false)
    );

    let mut stream = plan
        .execute(partition, task_ctx)
        .map_err(|e| format!("failed to execute partition {partition}: {e}"))?;

    let mut batches = Vec::new();
    while let Some(batch) = stream.next().await {
        batches.push(batch.map_err(|e| format!("error reading batch: {e}"))?);
    }

    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    info!(
        %job_id,
        partition,
        num_batches = batches.len(),
        total_rows,
        "encoding results"
    );

    let encoded = iroh_arrow::codec::encode_batches(&batches)
        .map_err(|e| format!("failed to encode batches: {e}"))?;

    Ok(Some(encoded))
}
