use std::future::Future;
use std::sync::Arc;

use iroh::{Endpoint, EndpointId};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::error::{QueueError, Result};
use super::gossip::{SignalReceiver, SignalSender};
use super::protocol::{self, GossipSignal, QueueRequest, QueueResponse};
use super::store::JobStore;

/// Handle to a claimed job, used for sending ack/nack after processing.
pub struct ClaimHandle {
    send: iroh::endpoint::SendStream,
    recv: iroh::endpoint::RecvStream,
    job_id: Uuid,
}

impl ClaimHandle {
    /// Acknowledge successful processing with an optional result payload.
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
///
/// Listens on both the gossip channel (for signals from remote peers) and a local
/// channel (for signals from the co-located producer, since gossip doesn't deliver
/// messages to the sender).
///
/// When the producer is the local peer, jobs are claimed and acked directly via the
/// in-memory `JobStore` (iroh doesn't support self-connections). For remote producers,
/// the standard QUIC claim/ack flow is used.
pub struct Consumer {
    endpoint: Endpoint,
    signal_receiver: SignalReceiver,
    signal_sender: SignalSender,
    local_receiver: mpsc::UnboundedReceiver<GossipSignal>,
    /// Local job store for direct claim/ack when the producer is ourself.
    local_store: JobStore,
}

impl Consumer {
    pub fn new(
        endpoint: Endpoint,
        signal_receiver: SignalReceiver,
        signal_sender: SignalSender,
        local_receiver: mpsc::UnboundedReceiver<GossipSignal>,
        local_store: JobStore,
    ) -> Self {
        Self {
            endpoint,
            signal_receiver,
            signal_sender,
            local_receiver,
            local_store,
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

        protocol::write_message(&mut send, &QueueRequest::Claim { job_id }).await?;

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

        loop {
            // Listen on both gossip (remote peers) and local channel (co-located producer)
            let signal = tokio::select! {
                Some(result) = self.signal_receiver.recv() => result?.0,
                Some(signal) = self.local_receiver.recv() => signal,
                else => break,
            };

            match signal {
                GossipSignal::JobAvailable {
                    job_id,
                    producer,
                    priority: _,
                    timestamp_ms: _,
                } => {
                    let is_local = producer == self.endpoint.id();
                    tracing::info!(%job_id, %producer, is_local, "job available, attempting claim");

                    if is_local {
                        // Local job: claim and ack directly via the in-memory store
                        // (iroh doesn't support self-connections)
                        match self.local_store.try_claim(job_id, self.endpoint.id()).await {
                            Ok(job) => {
                                tracing::info!(%job_id, "local job claimed, processing");
                                let handler = handler.clone();
                                match handler(job_id, job.payload).await {
                                    Ok(result) => {
                                        if let Err(e) = self.local_store.ack(job_id, result).await {
                                            tracing::error!(%job_id, "failed to ack local job: {e}");
                                        }
                                    }
                                    Err(reason) => {
                                        if let Err(e) = self.local_store.nack(job_id, reason).await {
                                            tracing::error!(%job_id, "failed to nack local job: {e}");
                                        }
                                    }
                                }
                            }
                            Err(QueueError::AlreadyClaimed(_)) => {
                                tracing::debug!(%job_id, "local job already claimed");
                            }
                            Err(e) => {
                                tracing::error!(%job_id, "failed to claim local job: {e}");
                            }
                        }
                    } else {
                        // Remote job: claim via QUIC connection to the producer
                        match Self::claim_job(&self.endpoint, producer, job_id).await {
                            Ok((payload, _priority, handle)) => {
                                tracing::info!(%job_id, "remote job claimed, processing");

                                let _ = self
                                    .signal_sender
                                    .broadcast(GossipSignal::JobClaimed {
                                        job_id,
                                        consumer: self.endpoint.id(),
                                    })
                                    .await;

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
                }
                GossipSignal::JobClaimed { job_id, consumer } => {
                    tracing::debug!(%job_id, %consumer, "job claimed by peer");
                }
            }
        }

        Ok(())
    }
}
