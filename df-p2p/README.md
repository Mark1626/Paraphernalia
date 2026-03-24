# df-p2p

Proof of Concept for distributed [Apache DataFusion](https://datafusion.apache.org/) query execution using [iroh](https://iroh.computer/) for P2P networking.

Inspired by [datafusion-distributed](https://github.com/probably-nothing-labs/datafusion-distributed), which uses Arrow Flight/gRPC. This project replaces that transport layer with iroh's QUIC-based P2P connections, enabling:

- **NAT traversal** via iroh's relay servers
- **P2P discovery** using cryptographic endpoint IDs (no static URLs needed)
- **QUIC transport** for fast, multiplexed, encrypted streams
- **True P2P** — every node is both a worker and a query initiator

## Architecture

```
Peer A (query initiator / producer)     Peer B (consumer / worker)
   │                                     │
   │  SQL → logical → physical plan      │
   │  optimizer rule: replace            │
   │    CoalescePartitionsExec with       │
   │    DistributedCoalesceExec          │
   │                                     │
   │  Producer.enqueue_and_wait(payload) │
   │  ──── gossip: JobAvailable ──────>  │
   │                                     │  Consumer hears signal
   │  <──── QUIC: Claim(job_id) ───────  │
   │  ──── QUIC: Granted(payload) ────>  │
   │                                     │  deserialize → execute plan
   │  <──── QUIC: Ack(Arrow IPC) ──────  │
   │  oneshot channel delivers result    │
   │                                     │
   │  (roles are symmetric — Peer B      │
   │   could query Peer A too)           │
```

**How it works:**

1. A peer receives a SQL query and plans it locally (SQL → logical plan → physical plan)
2. The `apply_network_boundaries` optimizer rule finds `CoalescePartitionsExec` and replaces it with `DistributedCoalesceExec`, serializing the child subtree into a `Stage`
3. During execution, `DistributedCoalesceExec` enqueues each partition as a job in the distributed work queue via `DistributedContext::submit_to_queue()`
4. Job availability is broadcast via iroh-gossip to all peers in the topic
5. Available worker peers (consumers) race to claim jobs via direct QUIC connections, execute the physical plan, and ack with Arrow IPC encoded results
6. Results are delivered back to the producer via oneshot channels

### Project structure

```
iroh-arrow/            Transport layer (no DataFusion dependency)
  src/protocol.rs      IrohArrow<H> protocol handler, RequestHandler trait
  src/codec.rs         Arrow IPC encode/decode for record batches
  src/stream.rs        send/recv helpers over iroh QUIC streams
src/
  lib.rs               Public API
  context.rs           DistributedContext — session extension with submit_to_queue()
  worker.rs            Worker: queue consumer that executes plans from jobs
  optimizer_rule.rs    apply_network_boundaries() — replaces CoalescePartitionsExec
  stage.rs             Stage { stage_id, encoded_plan }
  operators/
    distributed_coalesce.rs  DistributedCoalesceExec ExecutionPlan node
  queue/               Distributed work queue (inlined from iroh-queue PoC)
    mod.rs             Re-exports, well-known gossip TopicId
    protocol.rs        GossipSignal, QueueRequest/Response, postcard framing
    store.rs           JobStore with oneshot channels for result delivery
    gossip.rs          GossipBridge, SignalSender/Receiver
    handler.rs         QueueProtocol — iroh ProtocolHandler for claim/ack/nack
    producer.rs        Producer — enqueue_and_wait() blocks until result
    consumer.rs        Consumer — listens for gossip, claims jobs, runs handler
    error.rs           QueueError enum
examples/
  peer_node.rs         Unified P2P node with gossip + queue scheduling
  query_client.rs      Send SQL queries to any peer node
testdata/
  sample.csv           Sample dataset for demos
```

## Quick start

### Prerequisites

- Rust 2024 edition (1.85+)
- Internet access (iroh uses relay servers for peer discovery)

### P2P demo (three terminals)

Peers auto-discover each other via a shared `peers/` directory and coordinate work via iroh-gossip. Works with a single peer (self-consumption) or multiple peers.

**Terminal 1 — start peer 1:**

```bash
RUST_LOG=info,iroh_quinn_proto::connection=error cargo run --example peer_node -- --table-path testdata/sample.csv
```

**Terminal 2 — start peer 2:**

```bash
cargo run --example peer_node -- --table-path testdata/sample.csv
```

**Terminal 3 — send a query to any peer:**

```bash
cargo run --example query_client -- "SELECT city, SUM(amount) as total FROM data GROUP BY city ORDER BY total DESC"
```

The query client discovers a peer from `peers/`, sends SQL over iroh QUIC. The receiving peer plans the query, applies the distributed optimizer rule, and enqueues work to the distributed queue. Available peer consumers claim and execute the work dynamically. Results flow back via the queue protocol as Arrow IPC batches.

Expected output:
```
+---------------+-------+
| city          | total |
+---------------+-------+
| San Francisco | 800   |
| Chicago       | 525   |
| New York      | 425   |
+---------------+-------+
```

### Parquet datasets

Register hive-partitioned parquet tables from a directory (each subdirectory becomes a table):

```bash
cargo run --example peer_node -- --dataset-dir /path/to/parquet/tables
```

### Enable debug logging

```bash
RUST_LOG=debug cargo run --example peer_node -- --table-path testdata/sample.csv
```

## Key dependencies

| Crate | Version | Role |
|---|---|---|
| `iroh` | 0.96 | P2P networking (QUIC, relay, discovery) |
| `iroh-gossip` | 0.96 | Broadcast signaling for work queue coordination |
| `datafusion` | 52.2.0 | SQL query engine |
| `datafusion-proto` | 52.2.0 | Physical plan protobuf serialization |
| `arrow` / `arrow-ipc` | 57.0.0 | Columnar data format and IPC serialization |
| `postcard` | 1 | Queue wire format (length-prefixed serialization) |
| `tokio` | 1 | Async runtime |

### Run tests

```bash
cargo test -p iroh-arrow    # transport layer tests (codec + protocol)
cargo test                   # all workspace tests
```

## Roadmap

- [x] Physical plan serialization via `datafusion-proto`
- [x] Distributed physical optimizer rule (`apply_network_boundaries`)
- [x] `DistributedCoalesceExec` over iroh streams
- [x] Unified P2P peer node example
- [x] Multi-worker partitioned execution
- [x] Dynamic queue-based scheduling via iroh-gossip (workers pull work)
- [x] Parquet table support (hive-partitioned via `--dataset-dir`)
- [x] Dynamic peer auto-discovery (shared `peers/` directory + gossip)
- [x] Separate query client (`query_client`)
- [x] Unit tests for iroh-arrow crate (codec, protocol)
- [x] Stale job reaper (re-enqueues timed-out jobs)
- [ ] Streaming results (currently buffers entire result set)
- [ ] Multi-stage query plans (nested distribution)
- [x] Self-consumption (peer consuming its own jobs via local mpsc channel)
