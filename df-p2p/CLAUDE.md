# CLAUDE.md

## Project overview

df-p2p is a PoC for distributed DataFusion query execution using iroh for P2P networking. It replaces Arrow Flight/gRPC (used by the reference project `datafusion-distributed`) with iroh's QUIC-based transport.

The `iroh-arrow` crate provides a reusable protocol handler for sending/receiving Arrow data over iroh QUIC connections. The `df-p2p` crate builds on it with DataFusion-specific scheduling and execution.

Every node is a peer — it can both serve query execution requests and initiate distributed queries to other peers.

Reference project: `/Users/nimalanm/Documents/opensource/db/datafusion-distributed`

## Build & run

```bash
cargo check                          # type-check

# P2P multi-node demo with dynamic queue scheduling (two+ terminals):
# Terminal 1:
cargo run --example peer_node -- --table-path testdata/sample.csv
# Terminal 2:
cargo run --example peer_node -- --table-path testdata/sample.csv
# Terminal 3 (query any peer):
cargo run --example query_client -- "SELECT city, SUM(amount) as total FROM data GROUP BY city"
```

```bash
cargo test -p iroh-arrow                 # iroh-arrow unit tests (codec, protocol)
cargo test                               # all workspace tests
```

The `peer_node` / `query_client` examples are the primary end-to-end verification. Works with a single peer (self-consumption via local mpsc channel) or multiple peers.

## Key files

### iroh-arrow crate (transport layer)

- `iroh-arrow/src/protocol.rs` - `IrohArrow<H>` protocol handler implementing iroh's `ProtocolHandler` trait; `RequestHandler` trait for pluggable request handling; ALPN (`b"mark/iroh-arrow/0"`), MAX_REQUEST_SIZE, MAX_RESPONSE_SIZE
- `iroh-arrow/src/codec.rs` - `encode_batches()` / `decode_batches()` using Arrow IPC StreamWriter/StreamReader
- `iroh-arrow/src/stream.rs` - `send_batches()` / `recv_batches()` / `send_bytes()` / `recv_bytes()` helpers over iroh QUIC streams

### df-p2p crate (DataFusion integration)

- `src/context.rs` - `DistributedContext` session config extension holding iroh `Endpoint` + `Producer`; `submit_to_queue()` enqueues plans to the distributed work queue and awaits results
- `src/worker.rs` - `Worker` wraps a queue `Consumer` loop; spawns a tokio task that listens for gossip signals, claims jobs, deserializes physical plans, executes them, and acks with Arrow IPC encoded results
- `src/optimizer_rule.rs` - `apply_network_boundaries(plan)` walks the physical plan tree via `transform_down` and replaces the first `CoalescePartitionsExec` with `DistributedCoalesceExec`
- `src/operators/distributed_coalesce.rs` - `DistributedCoalesceExec` leaf ExecutionPlan node; serialized child plan in `Stage`; on `execute(partition)` calls `DistributedContext::submit_to_queue()` to enqueue work for dynamic scheduling
- `src/stage.rs` - `Stage { stage_id, encoded_plan }` holds a serialized physical plan (protobuf bytes)

### queue module (distributed work queue, inlined from iroh-queue PoC)

- `src/queue/protocol.rs` - `GossipSignal` (JobAvailable, JobClaimed), `QueueRequest` (Claim, Ack, Nack), `QueueResponse` (Granted, AlreadyClaimed, Acked, Error); length-prefixed postcard framing; ALPN (`b"iroh-queue/0"`)
- `src/queue/store.rs` - `JobStore` thread-safe in-memory job store with FIFO ordering; `enqueue()` returns oneshot receiver for result delivery; `ack()` delivers result bytes through the channel; `reap_stale()` re-enqueues timed-out jobs
- `src/queue/gossip.rs` - `GossipBridge` joins an iroh-gossip topic; `SignalSender`/`SignalReceiver` typed wrappers for broadcasting/receiving `GossipSignal` messages
- `src/queue/handler.rs` - `QueueProtocol` implements iroh `ProtocolHandler`; accepts consumer connections, handles claim/ack/nack request-response cycles on bi-streams
- `src/queue/producer.rs` - `Producer` enqueues jobs and broadcasts availability via gossip; `enqueue_and_wait()` blocks until a consumer processes the job and returns the result; runs stale job reaper
- `src/queue/consumer.rs` - `Consumer` listens for gossip signals, races to claim jobs via direct QUIC to producer, processes with a pluggable handler fn, acks/nacks; `ClaimHandle` for lifecycle management
- `src/queue/mod.rs` - Re-exports + `work_queue_topic()` well-known TopicId

### Examples

- `examples/peer_node.rs` - Unified P2P node: triple ALPN (queue protocol + gossip + SQL query handling), auto-discovers peers from `peers/` directory for gossip bootstrapping, supports CSV (`--table-path`) and hive-partitioned parquet (`--dataset-dir`)
- `examples/query_client.rs` - Connects to any peer and sends a SQL query, displays results

## Architecture

```
Peer A (query initiator / producer)     Peer B (consumer / worker)
   │                                     │
   │  SQL → logical plan → physical plan │
   │  optimizer: CoalescePartitionsExec  │
   │     → DistributedCoalesceExec       │
   │  serialize(child_plan) into Stage   │
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
   │  (Any peer can be both producer     │
   │   and consumer simultaneously)      │
```

**Dynamic scheduling via distributed queue**: The optimizer rule replaces `CoalescePartitionsExec` with `DistributedCoalesceExec`, which serializes the child subtree into a `Stage`. On `execute(partition)`, instead of sending directly to a specific worker (static assignment), the plan is enqueued as a job in the distributed queue. Available worker peers (consumers) hear the gossip signal, race to claim the job, execute the plan, and ack with Arrow IPC encoded results. Results are delivered back to the producer via oneshot channels.

**Queue protocol**: Uses iroh-gossip for broadcast signaling (job availability, claim notifications) and direct QUIC bi-streams for reliable claim/ack/nack exchanges. The `QueueProtocol` handler runs on each peer to accept consumer connections. Each peer is both a producer (can submit work) and a consumer (can pull and execute work).

`DistributedContext` (registered as a session config extension) holds a `Producer` and submits work via `submit_to_queue()`.

`IrohArrow<H>` handles the SQL query submission lifecycle: accept bi-stream → read SQL → plan/distribute/execute → send Arrow IPC response.

## iroh 0.96 API conventions

These are the correct API names (iroh renamed many types from earlier versions):

| What | iroh 0.96 API |
|---|---|
| Node identity | `endpoint.id()` -> `EndpointId` |
| Node address | `endpoint.addr()` -> `EndpointAddr` (sync) |
| Wait for relay | `endpoint.online().await` |
| Protocol routing | `Router::builder(endpoint).accept(alpn, handler).spawn()` |
| Accept connections | `endpoint.accept().await` -> `Option<Incoming>` |
| Establish connection | `incoming.accept()?.await?` -> `Connection` |
| Connect to peer | `endpoint.connect(addr, alpn).await?` -> `Connection` |
| Bi-stream | `conn.open_bi().await?` / `conn.accept_bi().await?` -> `(SendStream, RecvStream)` |
| Send data | `send.write_all(&bytes).await?`, `send.finish()?` (sync!) |
| Receive data | `recv.read_to_end(limit).await?` |
| Wait for close | `conn.closed().await` -> `ConnectionError` |
| Types location | `iroh::{Endpoint, EndpointAddr, EndpointId}`, `iroh::endpoint::{Incoming, Connection, SendStream, RecvStream}` |

**Important**: `SendStream::finish()` is synchronous and does NOT wait for data to be flushed. Dropping `Connection` immediately after `finish()` will send CONNECTION_CLOSE and abort in-flight data. Always `conn.closed().await` on the responder side.

## Wire protocols

### Queue protocol (`iroh-queue/0` ALPN)
Over iroh QUIC bi-directional stream, length-prefixed postcard messages:
1. **Claim flow**: Consumer → `Claim{job_id}` → Producer responds `Granted{payload}` → Consumer processes → `Ack{result: Arrow IPC bytes}` → Producer responds `Acked`
2. **Job payload**: `[partition as u32 LE][physical plan protobuf bytes]`
3. **Gossip signals**: `JobAvailable{job_id, producer, priority}`, `JobClaimed{job_id, consumer}`

### SQL query protocol (`mark/df-p2p-query/0` ALPN)
Over iroh QUIC bi-directional stream (via `IrohArrow`):
1. **Request**: SQL string bytes, then `send.finish()`
2. **Response**: Arrow IPC stream bytes (schema + batches), then `send.finish()`

## Dependencies

- iroh = "0.96", iroh-gossip = "0.96", datafusion = "52.2.0", datafusion-proto = "52.2.0", arrow/arrow-ipc = "57.0.0"
- postcard = "1" (queue wire format), uuid = "1" (job IDs), serde = "1", thiserror = "2"
- Rust edition 2024

## Testing

The `iroh-arrow` crate has unit tests covering:
- `codec.rs` — 7 tests: encode/decode roundtrips, empty batches, explicit schema, null preservation, invalid input
- `protocol.rs` — 6 tests: end-to-end echo, empty response, multi-batch, empty request, handler errors, config

Stream functions (`stream.rs`) are thin wrappers tested indirectly via the protocol tests. Direct stream tests require raw iroh QUIC bi-streams which can't be extracted from the Router's `ProtocolHandler::accept` context.

## Known iroh warnings

`WARN iroh_quinn_proto::connection: sent PATH_ABANDON after path was already discarded` — harmless timing race in iroh's QUIC multipath implementation (tracked in iroh#3930). Suppress with:
```
RUST_LOG=info,iroh_quinn_proto::connection=error
```

## Code style

- Use `anyhow::Result` for error handling
- Use `tracing` for logging (info for operations, error for failures)
- Use `clap` for CLI argument parsing in examples
- Keep library code in `src/`, examples in `examples/`
- The `iroh-arrow` crate is transport-only (no DataFusion dependency); DataFusion logic lives in `df-p2p`
