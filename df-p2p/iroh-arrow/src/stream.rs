//! Helpers for sending and receiving Arrow record batches over iroh QUIC streams.

use anyhow::{Context, Result};
use arrow::array::RecordBatch;
use iroh::endpoint::{RecvStream, SendStream};

/// Send record batches over an iroh QUIC send stream as Arrow IPC.
///
/// After writing, the stream is finished (FIN sent). The caller should keep
/// the parent `Connection` alive until the peer has read all data (e.g. via
/// `conn.closed().await`).
pub async fn send_batches(send: &mut SendStream, batches: &[RecordBatch]) -> Result<()> {
    let ipc_bytes = crate::codec::encode_batches(batches)?;
    send.write_all(&ipc_bytes).await?;
    send.finish()?;
    Ok(())
}

/// Receive record batches from an iroh QUIC recv stream.
///
/// Reads until the peer finishes (FIN), then decodes the Arrow IPC payload.
pub async fn recv_batches(
    recv: &mut RecvStream,
    max_size: usize,
) -> Result<Vec<RecordBatch>> {
    let bytes = recv
        .read_to_end(max_size)
        .await
        .context("failed to read Arrow IPC from stream")?;
    crate::codec::decode_batches(&bytes)
}

/// Send raw bytes over an iroh QUIC send stream and finish.
pub async fn send_bytes(send: &mut SendStream, data: &[u8]) -> Result<()> {
    send.write_all(data).await?;
    send.finish()?;
    Ok(())
}

/// Receive raw bytes from an iroh QUIC recv stream.
pub async fn recv_bytes(recv: &mut RecvStream, max_size: usize) -> Result<Vec<u8>> {
    recv.read_to_end(max_size)
        .await
        .context("failed to read from stream")
}

// Stream functions are thin wrappers around codec + QUIC write/read.
// They are tested end-to-end via the protocol module tests (see protocol.rs).
