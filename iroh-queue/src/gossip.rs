use bytes::Bytes;
use iroh::EndpointId;
use iroh_gossip::api::{Event, GossipReceiver, GossipSender};
use iroh_gossip::net::Gossip;
use iroh_gossip::TopicId;
use n0_future::StreamExt;

use crate::error::{QueueError, Result};
use crate::protocol::GossipSignal;

/// Bridge between iroh-gossip and typed `GossipSignal` messages.
pub struct GossipBridge {
    gossip: Gossip,
    topic_id: TopicId,
}

impl GossipBridge {
    pub fn new(gossip: Gossip, topic_id: TopicId) -> Self {
        Self { gossip, topic_id }
    }

    /// Join the gossip topic with the given bootstrap peers.
    /// Returns typed sender and receiver handles.
    ///
    /// When `peers` is empty (e.g. the first producer), this uses `subscribe`
    /// which returns immediately. When peers are provided, it uses
    /// `subscribe_and_join` which waits until at least one peer is connected.
    pub async fn join(
        &self,
        peers: Vec<EndpointId>,
    ) -> Result<(SignalSender, SignalReceiver)> {
        let topic = if peers.is_empty() {
            self.gossip
                .subscribe(self.topic_id, vec![])
                .await
                .map_err(|e| QueueError::Gossip(e.to_string()))?
        } else {
            self.gossip
                .subscribe_and_join(self.topic_id, peers)
                .await
                .map_err(|e| QueueError::Gossip(e.to_string()))?
        };

        let (sender, receiver) = topic.split();
        Ok((SignalSender { inner: sender }, SignalReceiver { inner: receiver }))
    }

    pub fn topic_id(&self) -> TopicId {
        self.topic_id
    }
}

/// Typed wrapper around the gossip sender for broadcasting `GossipSignal` messages.
#[derive(Clone)]
pub struct SignalSender {
    inner: GossipSender,
}

impl SignalSender {
    /// Broadcast a signal to all peers in the topic.
    pub async fn broadcast(&self, signal: GossipSignal) -> Result<()> {
        let bytes = postcard::to_allocvec(&signal)?;
        self.inner
            .broadcast(Bytes::from(bytes))
            .await
            .map_err(|e| QueueError::Gossip(e.to_string()))?;
        Ok(())
    }
}

/// Typed wrapper around the gossip receiver for consuming `GossipSignal` messages.
pub struct SignalReceiver {
    inner: GossipReceiver,
}

impl SignalReceiver {
    /// Receive the next signal from the gossip topic.
    /// Skips non-message events (Joined, NeighborUp/Down, etc.).
    pub async fn recv(&mut self) -> Option<Result<(GossipSignal, EndpointId)>> {
        loop {
            match self.inner.next().await? {
                Ok(Event::Received(msg)) => {
                    match postcard::from_bytes::<GossipSignal>(&msg.content) {
                        Ok(signal) => return Some(Ok((signal, msg.delivered_from))),
                        Err(e) => return Some(Err(e.into())),
                    }
                }
                Ok(_) => continue, // Skip Joined, NeighborUp, NeighborDown events
                Err(e) => return Some(Err(QueueError::Gossip(e.to_string()))),
            }
        }
    }
}
