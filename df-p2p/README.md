# df-p2p

Proof of Concept for distributed [Apache DataFusion](https://datafusion.apache.org/) query execution using [iroh](https://iroh.computer/) for P2P networking.

Inspired by [datafusion-distributed](https://github.com/probably-nothing-labs/datafusion-distributed), which uses Arrow Flight/gRPC. This project replaces that transport layer with iroh's QUIC-based P2P connections, enabling:

- **NAT traversal** via iroh's relay servers
- **P2P discovery** using cryptographic endpoint IDs (no static URLs needed)
- **QUIC transport** for fast, multiplexed, encrypted streams
- **True P2P** — every node is both a worker and a query initiator

## Architecture

```
Peer A (query initiator)               Peer B (worker)
   │                                     │
   │  SQL → logical → physical plan      │
   │  optimizer rule: replace            │
   │    CoalescePartitionsExec with       │
   │    DistributedCoalesceExec          │
   │                                     │
   │  ──── serialized plan (protobuf) ─> │
   │                                     │  deserialize → execute
   │  <──── Arrow IPC batches ────────   │
   │                                     │
   │  (roles are symmetric — Peer B      │
   │   could query Peer A too)           │
```

**How it works:**

1. A peer receives a SQL query and plans it locally (SQL → logical plan → physical plan)
2. The `apply_network_boundaries` optimizer rule finds `CoalescePartitionsExec` and replaces it with `DistributedCoalesceExec`, serializing the child subtree into a `Stage`
3. During execution, `DistributedCoalesceExec` uses `DistributedContext` (a session config extension) to send the serialized plan to remote peers over iroh QUIC
4. Remote peers deserialize the physical plan, execute it via DataFusion, and stream back Arrow IPC batches

### Project structure

```
iroh-arrow/            Transport layer (no DataFusion dependency)
  src/protocol.rs      IrohArrow<H> protocol handler, RequestHandler trait
  src/codec.rs         Arrow IPC encode/decode for record batches
  src/stream.rs        send/recv helpers over iroh QUIC streams
src/
  lib.rs               Public API
  context.rs           DistributedContext — session extension with execute_on_worker()
  worker.rs            Worker node: accepts plans, executes, returns batches
  optimizer_rule.rs    apply_network_boundaries() — replaces CoalescePartitionsExec
  stage.rs             Stage { stage_id, encoded_plan }
  operators/
    distributed_coalesce.rs  DistributedCoalesceExec ExecutionPlan node
examples/
  in_process.rs        Self-contained demo (worker + scheduler in one process)
  peer_node.rs         Unified P2P node with auto-discovery (CSV + parquet)
  query_client.rs      Send SQL queries to any peer node
testdata/
  sample.csv           Sample dataset for demos
```

## Quick start

### Prerequisites

- Rust 2024 edition (1.85+)
- Internet access (iroh uses relay servers for peer discovery)

### In-process demo

Runs a worker and scheduler in the same process. No separate processes needed.

```bash
cargo run --example in_process
```

### P2P demo (three terminals)

Peers auto-discover each other via a shared `peers/` directory.

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

The query client discovers a peer from `peers/`, sends SQL over iroh QUIC. The receiving peer discovers other peers, plans the query, applies the distributed optimizer rule, and distributes execution. Results stream back as Arrow IPC batches.

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
| `datafusion` | 52.2.0 | SQL query engine |
| `datafusion-proto` | 52.2.0 | Physical plan protobuf serialization |
| `arrow` / `arrow-ipc` | 57.0.0 | Columnar data format and IPC serialization |
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
- [x] Multi-worker partitioned execution (round-robin across N peers)
- [x] Parquet table support (hive-partitioned via `--dataset-dir`)
- [x] Dynamic peer auto-discovery (shared `peers/` directory)
- [x] Separate query client (`query_client`)
- [x] Unit tests for iroh-arrow crate (codec, protocol)
- [ ] Streaming results (currently buffers entire result set)
- [ ] Multi-stage query plans (nested distribution)

## TODO

- [] When the connection pool's lock is poisoned we panic
