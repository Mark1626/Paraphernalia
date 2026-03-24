use std::sync::Arc;

use anyhow::Result;
use arrow::array::RecordBatch;
use iroh::Endpoint;
use tracing::info;

use crate::queue::Producer;

/// ALPN protocol identifier for SQL query submission to a peer.
pub const QUERY_ALPN: &[u8] = b"mark/df-p2p-query/0";

/// Session config extension that provides queue-based distributed execution.
///
/// Instead of static worker assignment, work is submitted to a distributed
/// queue via gossip and picked up by available workers dynamically.
/// The producer enqueues serialized plans and awaits results delivered
/// through oneshot channels when consumers ack completed jobs.
pub struct DistributedContext {
    pub endpoint: Endpoint,
    producer: Arc<Producer>,
}

impl std::fmt::Debug for DistributedContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DistributedContext").finish_non_exhaustive()
    }
}

impl DistributedContext {
    pub fn new(endpoint: Endpoint, producer: Arc<Producer>) -> Self {
        Self { endpoint, producer }
    }

    /// Submit a plan partition to the work queue and await the result.
    /// An available worker will pick it up, execute it, and return the batches.
    ///
    /// Wire format of the job payload: `[partition as u32 LE][plan bytes]`
    pub async fn submit_to_queue(
        &self,
        partition: usize,
        data: &[u8],
    ) -> Result<Vec<RecordBatch>> {
        info!(partition, data_len = data.len(), "submitting plan to queue");

        // Prepend partition number (u32 LE) to plan bytes
        let partition_bytes = (partition as u32).to_le_bytes();
        let mut payload = Vec::with_capacity(4 + data.len());
        payload.extend_from_slice(&partition_bytes);
        payload.extend_from_slice(data);

        let result_bytes = self
            .producer
            .enqueue_and_wait(payload, 0)
            .await
            .map_err(|e| anyhow::anyhow!("queue error: {e}"))?;

        let batches = iroh_arrow::codec::decode_batches(&result_bytes)?;

        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        info!(
            partition,
            num_batches = batches.len(),
            total_rows,
            "received results from queue"
        );

        Ok(batches)
    }
}
