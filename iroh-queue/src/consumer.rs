use std::future::Future;
use std::sync::Arc;

use iroh::protocol::Router;
use iroh::{Endpoint, EndpointId};
use iroh_gossip::net::Gossip;
use iroh_gossip::TopicId;
use uuid::Uuid;

use crate::error::{QueueError, Result};
use crate::gossip::{GossipBridge, SignalReceiver, SignalSender};
use crate::protocol::{self, GossipSignal, QueueRequest, QueueResponse};

/// Handle to a claimed job, used for sending ack/nack after processing.
pub struct ClaimHandle {
    send: iroh::endpoint::SendStream,
    recv: iroh::endpoint::RecvStream,
    job_id: Uuid,
}

impl ClaimHandle {
    /// Acknowledge successful processing.
    pub async fn ack(mut self, result: Option<Vec<u8>>) -> Result<()> {
        protocol::write_message(
            &mut self.send,
            &QueueRequest::Ack {
                job_id: self.job_id,
                result,
            },
        )
        .await?;
        let _resp: QueueResponse = protocol::read_message(&mut self.recv).await?;
        self.send
            .finish()
            .map_err(|e| QueueError::Connection(e.to_string()))?;
        Ok(())
    }

    /// Report processing failure; the job will be re-enqueued.
    pub async fn nack(mut self, reason: String) -> Result<()> {
        protocol::write_message(
            &mut self.send,
            &QueueRequest::Nack {
                job_id: self.job_id,
                reason,
            },
        )
        .await?;
        let _resp: QueueResponse = protocol::read_message(&mut self.recv).await?;
        self.send
            .finish()
            .map_err(|e| QueueError::Connection(e.to_string()))?;
        Ok(())
    }

    pub fn job_id(&self) -> Uuid {
        self.job_id
    }
}

/// High-level consumer that listens for job signals and processes them.
pub struct Consumer {
    endpoint: Endpoint,
    signal_receiver: SignalReceiver,
    signal_sender: SignalSender,
}

impl Consumer {
    pub fn new(
        endpoint: Endpoint,
        signal_receiver: SignalReceiver,
        signal_sender: SignalSender,
    ) -> Self {
        Self {
            endpoint,
            signal_receiver,
            signal_sender,
        }
    }

    /// Attempt to claim a specific job from a producer via direct QUIC stream.
    pub async fn claim_job(
        endpoint: &Endpoint,
        producer_id: EndpointId,
        job_id: Uuid,
    ) -> Result<(Vec<u8>, u8, ClaimHandle)> {
        let conn = endpoint
            .connect(producer_id, protocol::ALPN)
            .await
            .map_err(|e| QueueError::Connection(e.to_string()))?;

        let (mut send, mut recv) = conn
            .open_bi()
            .await
            .map_err(|e| QueueError::Connection(e.to_string()))?;

        // Send claim request
        protocol::write_message(&mut send, &QueueRequest::Claim { job_id }).await?;

        // Read response
        let response: QueueResponse = protocol::read_message(&mut recv).await?;

        match response {
            QueueResponse::Granted {
                job_id,
                payload,
                priority,
            } => {
                let handle = ClaimHandle {
                    send,
                    recv,
                    job_id,
                };
                Ok((payload, priority, handle))
            }
            QueueResponse::AlreadyClaimed { job_id } => Err(QueueError::AlreadyClaimed(job_id)),
            QueueResponse::Error { message } => Err(QueueError::Connection(message)),
            _ => Err(QueueError::Connection("unexpected response".into())),
        }
    }

    /// Run the consumer loop, processing jobs with the provided handler function.
    ///
    /// The handler receives `(job_id, payload)` and returns an optional result payload.
    /// On handler success, the job is acked. On failure, it's nacked.
    pub async fn run<F, Fut>(&mut self, handler: F) -> Result<()>
    where
        F: Fn(Uuid, Vec<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<Option<Vec<u8>>, String>> + Send,
    {
        let handler = Arc::new(handler);

        while let Some(result) = self.signal_receiver.recv().await {
            let (signal, _from) = result?;

            match signal {
                GossipSignal::JobAvailable {
                    job_id,
                    producer,
                    priority: _,
                    timestamp_ms: _,
                } => {
                    tracing::info!(%job_id, %producer, "job available, attempting claim");

                    match Self::claim_job(&self.endpoint, producer, job_id).await {
                        Ok((payload, _priority, handle)) => {
                            tracing::info!(%job_id, "job claimed, processing");

                            // Broadcast that we claimed it (informational)
                            let _ = self
                                .signal_sender
                                .broadcast(GossipSignal::JobClaimed {
                                    job_id,
                                    consumer: self.endpoint.id(),
                                })
                                .await;

                            // Process the job
                            let handler = handler.clone();
                            match handler(job_id, payload).await {
                                Ok(result) => {
                                    if let Err(e) = handle.ack(result).await {
                                        tracing::error!(%job_id, "failed to ack: {e}");
                                    }
                                }
                                Err(reason) => {
                                    if let Err(e) = handle.nack(reason.clone()).await {
                                        tracing::error!(%job_id, "failed to nack: {e}");
                                    }
                                }
                            }
                        }
                        Err(QueueError::AlreadyClaimed(_)) => {
                            tracing::debug!(%job_id, "job already claimed by another consumer");
                        }
                        Err(e) => {
                            tracing::error!(%job_id, "failed to claim job: {e}");
                        }
                    }
                }
                GossipSignal::JobClaimed { job_id, consumer } => {
                    tracing::debug!(%job_id, %consumer, "job claimed by peer");
                }
                GossipSignal::ConsumerReady { .. } => {
                    // Could be used for load balancing in the future
                }
            }
        }

        Ok(())
    }
}

/// A fully wired consumer node: endpoint + gossip + router + consumer logic.
pub struct ConsumerNode {
    pub router: Router,
    pub consumer: Consumer,
    pub gossip: Gossip,
}

impl ConsumerNode {
    /// Spawn a consumer node that listens for job signals and processes them.
    pub async fn spawn(
        endpoint: Endpoint,
        gossip: Gossip,
        topic_id: TopicId,
        bootstrap_peers: Vec<EndpointId>,
    ) -> Result<Self> {
        // Consumer only needs gossip ALPN (no queue protocol handler)
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        let bridge = GossipBridge::new(gossip.clone(), topic_id);
        let (sender, receiver) = bridge.join(bootstrap_peers).await?;

        let consumer = Consumer::new(endpoint, receiver, sender);

        Ok(Self {
            router,
            consumer,
            gossip,
        })
    }
}
