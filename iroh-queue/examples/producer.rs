//! Producer node — enqueues jobs and serves them to consumers.
//!
//! Run with:
//!   cargo run --example producer
//!
//! For verbose iroh internals: RUST_LOG=debug cargo run --example producer
//! Copy the printed node ID and pass it to the consumer.

use std::time::Duration;

use iroh::Endpoint;
use iroh_gossip::net::Gossip;
use iroh_gossip::TopicId;
use tokio::io::{AsyncBufReadExt, BufReader};

use iroh_queue::ProducerNode;

/// Fixed topic ID for this demo. In production, derive from a queue name.
const TOPIC: [u8; 32] = [1u8; 32];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn,producer=info,iroh_queue=info")),
        )
        .init();

    let topic_id = TopicId::from(TOPIC);

    let endpoint = Endpoint::builder().bind().await?;
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let node = ProducerNode::spawn(
        endpoint.clone(),
        gossip,
        topic_id,
        vec![],
        Duration::from_secs(30),
    )
    .await?;

    let node_id = endpoint.id();
    tracing::info!(%node_id, "endpoint ID");

    tracing::info!("waiting for endpoint to come online...");
    node.router.endpoint().online().await;

    tracing::info!("producer online");
    tracing::info!("start a consumer with: RUST_LOG=info cargo run --example consumer -- {node_id}");
    tracing::info!("type a message and press Enter to enqueue a job (Ctrl+C to quit)");

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let mut job_num: u64 = 0;

    loop {
        tokio::select! {
            line = lines.next_line() => {
                match line? {
                    Some(line) if !line.trim().is_empty() => {
                        let payload = line.trim().as_bytes().to_vec();
                        let priority = 1;
                        let job_id = node.producer.enqueue(payload, priority).await?;
                        job_num += 1;
                        tracing::info!(%job_id, job_num, "enqueued job");
                    }
                    Some(_) => {} // empty line, ignore
                    None => break, // EOF
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("shutting down...");
                break;
            }
        }
    }

    node.router.shutdown().await?;
    tracing::info!("done");
    Ok(())
}
