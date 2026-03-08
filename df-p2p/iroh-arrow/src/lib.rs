//! Transport layer for sending Arrow data over iroh QUIC connections.
//!
//! Provides Arrow IPC encoding/decoding and helpers for streaming record batches
//! over iroh's bidirectional QUIC streams.

pub mod codec;
pub mod protocol;
pub mod stream;
