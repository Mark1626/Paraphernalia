use std::time::{Duration, SystemTime, UNIX_EPOCH};

use iroh::EndpointId;
use tokio::sync::mpsc;

use super::error::{QueueError, Result};
use super::gossip::SignalSender;
use super::protocol::GossipSignal;
use super::store::JobStore;

/// High-level producer API for enqueuing jobs and serving them to consumers.
pub struct Producer {
    store: JobStore,
    signal_sender: SignalSender,
    /// Local channel so the co-located consumer hears about jobs too
    /// (gossip does not deliver messages back to the sender).
    local_sender: mpsc::UnboundedSender<GossipSignal>,
    endpoint_id: EndpointId,
    claim_timeout: Duration,
}

impl Producer {
    pub fn new(
        store: JobStore,
        signal_sender: SignalSender,
        local_sender: mpsc::UnboundedSender<GossipSignal>,
        endpoint_id: EndpointId,
        claim_timeout: Duration,
    ) -> Self {
        Self {
            store,
            signal_sender,
            local_sender,
            endpoint_id,
            claim_timeout,
        }
    }

    /// Enqueue a job and wait for a consumer to process it and return the result.
    ///
    /// Broadcasts availability via both gossip (for remote peers) and the local
    /// channel (for the co-located consumer, since gossip won't deliver to self).
    pub async fn enqueue_and_wait(&self, payload: Vec<u8>, priority: u8) -> Result<Vec<u8>> {
        let (job, rx) = self
            .store
            .enqueue(payload, priority, self.endpoint_id)
            .await;
        let job_id = job.id;

        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let signal = GossipSignal::JobAvailable {
            job_id,
            producer: self.endpoint_id,
            priority,
            timestamp_ms,
        };

        // Notify remote peers via gossip
        self.signal_sender.broadcast(signal.clone()).await?;

        // Notify local consumer (gossip doesn't deliver to self)
        let _ = self.local_sender.send(signal);

        tracing::info!(%job_id, "job enqueued and announced, waiting for result");
        rx.await.map_err(|_| QueueError::Shutdown)
    }

    /// Run the stale job reaper loop. Re-broadcasts timed-out jobs.
    pub async fn run_reaper(&self) {
        let mut interval = tokio::time::interval(self.claim_timeout / 2);
        loop {
            interval.tick().await;
            let reaped = self.store.reap_stale(self.claim_timeout).await;
            for job_id in reaped {
                let timestamp_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                let signal = GossipSignal::JobAvailable {
                    job_id,
                    producer: self.endpoint_id,
                    priority: 0,
                    timestamp_ms,
                };

                if let Err(e) = self.signal_sender.broadcast(signal.clone()).await {
                    tracing::error!(%job_id, "failed to re-broadcast reaped job: {e}");
                }
                let _ = self.local_sender.send(signal);
            }
        }
    }

    pub fn store(&self) -> &JobStore {
        &self.store
    }
}
