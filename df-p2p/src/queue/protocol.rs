use iroh::endpoint::{RecvStream, SendStream};
use iroh::EndpointId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::{QueueError, Result};

/// ALPN protocol identifier for the queue direct stream protocol.
pub const ALPN: &[u8] = b"iroh-queue/0";

/// Maximum message size (16 MiB).
const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

// --- Gossip signals (broadcast to all peers in the topic) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipSignal {
    /// A producer has a new job available for consumption.
    JobAvailable {
        job_id: Uuid,
        producer: EndpointId,
        priority: u8,
        timestamp_ms: u64,
    },
    /// A consumer has successfully claimed a job.
    JobClaimed {
        job_id: Uuid,
        consumer: EndpointId,
    },
}

// --- Direct stream messages (point-to-point, reliable) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueueRequest {
    /// Consumer requests to claim a specific job.
    Claim { job_id: Uuid },
    /// Consumer acknowledges successful processing.
    Ack {
        job_id: Uuid,
        result: Option<Vec<u8>>,
    },
    /// Consumer reports processing failure; job should be re-enqueued.
    Nack { job_id: Uuid, reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueueResponse {
    /// Job claim granted -- includes the payload.
    Granted {
        job_id: Uuid,
        payload: Vec<u8>,
        priority: u8,
    },
    /// Job was already claimed by another consumer.
    AlreadyClaimed { job_id: Uuid },
    /// Acknowledgement accepted.
    Acked { job_id: Uuid },
    /// An error occurred.
    Error { message: String },
}

// --- Length-prefixed framing helpers ---

/// Write a postcard-serialized message with a 4-byte big-endian length prefix.
pub async fn write_message<T: Serialize>(send: &mut SendStream, msg: &T) -> Result<()> {
    let encoded = postcard::to_allocvec(msg)?;
    let len = (encoded.len() as u32).to_be_bytes();
    send.write_all(&len)
        .await
        .map_err(|e| QueueError::Connection(e.to_string()))?;
    send.write_all(&encoded)
        .await
        .map_err(|e| QueueError::Connection(e.to_string()))?;
    Ok(())
}

/// Read a length-prefixed postcard message from a recv stream.
pub async fn read_message<T: for<'de> Deserialize<'de>>(recv: &mut RecvStream) -> Result<T> {
    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf)
        .await
        .map_err(|e| QueueError::Connection(e.to_string()))?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_MESSAGE_SIZE {
        return Err(QueueError::Connection(format!(
            "message too large: {len} bytes (max {MAX_MESSAGE_SIZE})"
        )));
    }
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf)
        .await
        .map_err(|e| QueueError::Connection(e.to_string()))?;
    Ok(postcard::from_bytes(&buf)?)
}
