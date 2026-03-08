use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use anyhow::Result;
use arrow::array::RecordBatch;
use iroh::endpoint::Connection;
use iroh::{Endpoint, EndpointAddr};
use iroh_arrow::protocol::{ALPN, MAX_RESPONSE_SIZE};
use tracing::info;

/// ALPN protocol identifier for SQL query submission to a peer.
pub const QUERY_ALPN: &[u8] = b"mark/df-p2p-query/0";

/// Session config extension that provides iroh networking context
/// for distributed execution operators.
///
/// Worker addresses can be updated dynamically via `set_workers()`,
/// enabling auto-discovery of new peers at query time.
///
/// Connections to workers are pooled: the first `execute_on_worker` call
/// to a given worker establishes a QUIC connection; subsequent calls reuse it
/// by opening new bi-streams on the same connection. Call `close_connections()`
/// after the query completes to tear them down.
pub struct DistributedContext {
    pub endpoint: Endpoint,
    worker_addrs: RwLock<Vec<EndpointAddr>>,
    connections: Mutex<HashMap<usize, Connection>>,
}

impl std::fmt::Debug for DistributedContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.worker_addrs.read().map(|a| a.len()).unwrap_or(0);
        f.debug_struct("DistributedContext")
            .field("worker_count", &count)
            .finish_non_exhaustive()
    }
}

impl DistributedContext {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            worker_addrs: RwLock::new(Vec::new()),
            connections: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_workers(endpoint: Endpoint, addrs: Vec<EndpointAddr>) -> Self {
        Self {
            endpoint,
            worker_addrs: RwLock::new(addrs),
            connections: Mutex::new(HashMap::new()),
        }
    }

    /// Replace the list of known worker addresses.
    /// Also clears the connection pool since the worker set changed.
    pub fn set_workers(&self, addrs: Vec<EndpointAddr>) {
        self.close_connections();
        *self.worker_addrs.write().unwrap() = addrs;
    }

    /// Number of currently known workers.
    pub fn worker_count(&self) -> usize {
        self.worker_addrs.read().unwrap().len()
    }

    /// Get a cached connection or establish a new one to the given worker.
    async fn get_or_connect(&self, worker_index: usize) -> Result<Connection> {
        // Check cache first
        {
            let conns = self.connections.lock().unwrap();
            if let Some(conn) = conns.get(&worker_index) {
                return Ok(conn.clone());
            }
        }

        // Resolve worker address
        let worker_addr = {
            let addrs = self.worker_addrs.read().unwrap();
            addrs
                .get(worker_index)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "worker index {worker_index} out of range (have {} workers)",
                        addrs.len(),
                    )
                })?
                .clone()
        };

        info!(worker_index, "opening connection to worker");
        let conn = self.endpoint.connect(worker_addr, ALPN).await?;

        // Cache for reuse (benign race if another partition connected concurrently)
        {
            let mut conns = self.connections.lock().unwrap();
            conns.entry(worker_index).or_insert_with(|| conn.clone());
        }

        Ok(conn)
    }

    /// Send `data` (a serialized physical plan) to the worker at the given index,
    /// requesting execution of only the specified `partition`. Returns the resulting
    /// Arrow record batches. Reuses a pooled connection when available.
    ///
    /// Wire format: `[partition as u32 LE][plan bytes]`
    pub async fn execute_on_worker(
        &self,
        worker_index: usize,
        partition: usize,
        data: &[u8],
    ) -> Result<Vec<RecordBatch>> {
        info!(
            worker_index,
            partition,
            data_len = data.len(),
            "sending plan to worker"
        );

        let conn = self.get_or_connect(worker_index).await?;
        let (mut send, mut recv) = conn.open_bi().await?;

        // Prepend partition number (u32 LE) to plan bytes
        let partition_bytes = (partition as u32).to_le_bytes();
        let mut payload = Vec::with_capacity(4 + data.len());
        payload.extend_from_slice(&partition_bytes);
        payload.extend_from_slice(data);

        iroh_arrow::stream::send_bytes(&mut send, &payload).await?;
        let batches = iroh_arrow::stream::recv_batches(&mut recv, MAX_RESPONSE_SIZE).await?;

        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        info!(
            worker_index,
            num_batches = batches.len(),
            total_rows,
            "received results from worker"
        );

        Ok(batches)
    }

    /// Close all pooled connections. Call after a query completes.
    pub fn close_connections(&self) {
        let mut conns = self.connections.lock().unwrap();
        let count = conns.len();
        for (_, conn) in conns.drain() {
            conn.close(0u32.into(), b"query complete");
        }
        if count > 0 {
            info!(count, "closed pooled worker connections");
        }
    }
}
