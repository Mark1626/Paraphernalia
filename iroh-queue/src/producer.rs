use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use iroh::protocol::Router;
use iroh::{Endpoint, EndpointId};
use iroh_gossip::net::Gossip;
use iroh_gossip::TopicId;
use uuid::Uuid;

use crate::error::Result;
use crate::gossip::{GossipBridge, SignalSender};
use crate::handler::QueueProtocol;
use crate::protocol::{self, GossipSignal};
use crate::queue::JobStore;

/// High-level producer API for enqueuing jobs and serving them to consumers.
pub struct Producer {
    store: JobStore,
    signal_sender: SignalSender,
    endpoint_id: EndpointId,
    claim_timeout: Duration,
}

impl Producer {
    pub fn new(
        store: JobStore,
        signal_sender: SignalSender,
        endpoint_id: EndpointId,
        claim_timeout: Duration,
    ) -> Self {
        Self {
            store,
            signal_sender,
            endpoint_id,
            claim_timeout,
        }
    }

    /// Enqueue a new job and broadcast its availability via gossip.
    pub async fn enqueue(&self, payload: Vec<u8>, priority: u8) -> Result<Uuid> {
        let job = self.store.enqueue(payload, priority, self.endpoint_id).await;
        let job_id = job.id;

        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        self.signal_sender
            .broadcast(GossipSignal::JobAvailable {
                job_id,
                producer: self.endpoint_id,
                priority,
                timestamp_ms,
            })
            .await?;

        tracing::info!(%job_id, "job enqueued and announced");
        Ok(job_id)
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

                if let Err(e) = self
                    .signal_sender
                    .broadcast(GossipSignal::JobAvailable {
                        job_id,
                        producer: self.endpoint_id,
                        priority: 0,
                        timestamp_ms,
                    })
                    .await
                {
                    tracing::error!(%job_id, "failed to re-broadcast reaped job: {e}");
                }
            }
        }
    }

    pub fn store(&self) -> &JobStore {
        &self.store
    }
}

/// A fully wired producer node: endpoint + gossip + router + producer logic.
pub struct ProducerNode {
    pub router: Router,
    pub producer: Arc<Producer>,
    pub gossip: Gossip,
}

impl ProducerNode {
    /// Spawn a producer node that listens for consumer connections.
    pub async fn spawn(
        endpoint: Endpoint,
        gossip: Gossip,
        topic_id: TopicId,
        bootstrap_peers: Vec<EndpointId>,
        claim_timeout: Duration,
    ) -> Result<Self> {
        let store = JobStore::new();
        let handler = QueueProtocol::new(store.clone());

        let router = Router::builder(endpoint.clone())
            .accept(protocol::ALPN, handler)
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        let bridge = GossipBridge::new(gossip.clone(), topic_id);
        let (sender, _receiver) = bridge.join(bootstrap_peers).await?;

        let producer = Arc::new(Producer::new(
            store,
            sender,
            endpoint.id(),
            claim_timeout,
        ));

        // Start the stale job reaper in the background
        let reaper = producer.clone();
        tokio::spawn(async move { reaper.run_reaper().await });

        Ok(Self {
            router,
            producer,
            gossip,
        })
    }
}
