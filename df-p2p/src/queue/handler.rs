use std::fmt;

use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use iroh::EndpointId;

use super::error::QueueError;
use super::protocol::{self, QueueRequest, QueueResponse};
use super::store::JobStore;

/// Protocol handler for the queue direct stream protocol.
///
/// Runs on the producer side, accepting incoming QUIC connections from consumers
/// and handling claim/ack/nack requests.
#[derive(Clone)]
pub struct QueueProtocol {
    store: JobStore,
}

impl fmt::Debug for QueueProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueueProtocol").finish()
    }
}

impl QueueProtocol {
    pub fn new(store: JobStore) -> Self {
        Self { store }
    }
}

impl ProtocolHandler for QueueProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let remote_id = connection.remote_id();
        tracing::debug!(%remote_id, "accepted queue connection");

        loop {
            let (send, recv) = match connection.accept_bi().await {
                Ok(streams) => streams,
                Err(_) => break,
            };

            let store = self.store.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_stream(store, remote_id, send, recv).await {
                    tracing::warn!(%remote_id, "stream error: {e}");
                }
            });
        }

        Ok(())
    }
}

async fn handle_stream(
    store: JobStore,
    remote_id: EndpointId,
    mut send: iroh::endpoint::SendStream,
    mut recv: iroh::endpoint::RecvStream,
) -> super::Result<()> {
    let request: QueueRequest = protocol::read_message(&mut recv).await?;

    match request {
        QueueRequest::Claim { job_id } => {
            tracing::info!(%remote_id, %job_id, "claim request");

            match store.try_claim(job_id, remote_id).await {
                Ok(job) => {
                    protocol::write_message(
                        &mut send,
                        &QueueResponse::Granted {
                            job_id,
                            payload: job.payload,
                            priority: job.priority,
                        },
                    )
                    .await?;

                    // Wait for ack/nack on the same stream (with timeout)
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(300),
                        protocol::read_message::<QueueRequest>(&mut recv),
                    )
                    .await
                    {
                        Ok(Ok(QueueRequest::Ack { job_id, result })) => {
                            store.ack(job_id, result).await?;
                            protocol::write_message(
                                &mut send,
                                &QueueResponse::Acked { job_id },
                            )
                            .await?;
                            tracing::info!(%job_id, "job completed");
                        }
                        Ok(Ok(QueueRequest::Nack { job_id, reason })) => {
                            store.nack(job_id, reason.clone()).await?;
                            protocol::write_message(
                                &mut send,
                                &QueueResponse::Error {
                                    message: format!("job nacked: {reason}"),
                                },
                            )
                            .await?;
                        }
                        Ok(Ok(other)) => {
                            tracing::warn!("unexpected request after claim: {other:?}");
                        }
                        Ok(Err(e)) => {
                            tracing::warn!(%job_id, "error reading ack: {e}");
                        }
                        Err(_) => {
                            tracing::warn!(%job_id, "ack timeout, job will be reaped");
                        }
                    }
                }
                Err(QueueError::AlreadyClaimed(_)) => {
                    protocol::write_message(
                        &mut send,
                        &QueueResponse::AlreadyClaimed { job_id },
                    )
                    .await?;
                }
                Err(e) => {
                    protocol::write_message(
                        &mut send,
                        &QueueResponse::Error {
                            message: e.to_string(),
                        },
                    )
                    .await?;
                }
            }
        }
        QueueRequest::Ack { job_id, result } => {
            store.ack(job_id, result).await?;
            protocol::write_message(&mut send, &QueueResponse::Acked { job_id }).await?;
        }
        QueueRequest::Nack { job_id, reason } => {
            store.nack(job_id, reason.clone()).await?;
            protocol::write_message(
                &mut send,
                &QueueResponse::Error {
                    message: format!("job nacked: {reason}"),
                },
            )
            .await?;
        }
    }

    send.finish().map_err(|e| QueueError::Connection(e.to_string()))?;
    Ok(())
}
