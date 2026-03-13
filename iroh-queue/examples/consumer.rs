//! Consumer node — claims and processes jobs from a producer.
//!
//! Run with:
//!   cargo run --example consumer -- <PRODUCER_NODE_ID>
//!
//! For verbose iroh internals: RUST_LOG=debug cargo run --example consumer -- <PRODUCER_NODE_ID>

use std::time::Duration;

use iroh::{Endpoint, EndpointId};
use iroh_gossip::net::Gossip;
use iroh_gossip::TopicId;

use iroh_queue::ConsumerNode;

/// Must match the producer's topic ID.
const TOPIC: [u8; 32] = [1u8; 32];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn,consumer=info,iroh_queue=info")),
        )
        .init();

    let producer_id: EndpointId = std::env::args()
        .nth(1)
        .expect("usage: consumer <PRODUCER_NODE_ID>")
        .parse()
        .expect("invalid node ID");

    let topic_id = TopicId::from(TOPIC);

    let endpoint = Endpoint::builder().bind().await?;
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let node = ConsumerNode::spawn(
        endpoint.clone(),
        gossip,
        topic_id,
        vec![producer_id],
    )
    .await?;

    let consumer_id = endpoint.id();
    tracing::info!(%consumer_id, "endpoint ID");

    tracing::info!("waiting for endpoint to come online...");
    node.router.endpoint().online().await;

    tracing::info!(%producer_id, "consumer online, connected to producer");
    tracing::info!("waiting for jobs...");

    let router = node.router;
    let mut consumer = node.consumer;

    // Run the consumer loop in a spawned task so we can handle Ctrl+C
    let consumer_handle = tokio::spawn(async move {
        consumer
            .run(|job_id, payload| async move {
                let task = String::from_utf8_lossy(&payload).to_string();
                tracing::info!(%job_id, task, "processing job");

                // Simulate work
                tokio::time::sleep(Duration::from_millis(500)).await;

                tracing::info!(%job_id, "completed job");
                Ok(Some(format!("done: {task}").into_bytes()))
            })
            .await
    });

    tokio::signal::ctrl_c().await?;
    tracing::info!("shutting down...");
    consumer_handle.abort();
    router.shutdown().await?;
    tracing::info!("done");
    Ok(())
}
