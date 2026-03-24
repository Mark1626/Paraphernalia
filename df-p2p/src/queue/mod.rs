pub mod consumer;
pub mod error;
pub mod gossip;
pub mod handler;
pub mod producer;
pub mod protocol;
pub mod store;

pub use consumer::{ClaimHandle, Consumer};
pub use error::{QueueError, Result};
pub use gossip::{GossipBridge, SignalReceiver, SignalSender};
pub use handler::QueueProtocol;
pub use producer::Producer;
pub use protocol::ALPN as QUEUE_ALPN;
pub use store::{Job, JobState, JobStore};

use iroh_gossip::TopicId;

/// Well-known gossip topic for the df-p2p work queue.
/// All peers join this topic to coordinate work distribution.
pub fn work_queue_topic() -> TopicId {
    let mut bytes = [0u8; 32];
    bytes[..19].copy_from_slice(b"df-p2p/work-queue/0");
    bytes.into()
}
