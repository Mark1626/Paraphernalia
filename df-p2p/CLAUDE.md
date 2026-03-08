# CLAUDE.md

## Project overview

df-p2p is a PoC for distributed DataFusion query execution using iroh for P2P networking. It replaces Arrow Flight/gRPC (used by the reference project `datafusion-distributed`) with iroh's QUIC-based transport.

The `iroh-arrow` crate provides a reusable protocol handler for sending/receiving Arrow data over iroh QUIC connections. The `df-p2p` crate builds on it with DataFusion-specific scheduling and execution.

Every node is a peer — it can both serve query execution requests and initiate distributed queries to other peers.

Reference project: `/Users/nimalanm/Documents/opensource/db/datafusion-distributed`

## Build & run

```bash
cargo check                          # type-check
cargo run --example in_process       # self-contained single-process demo
RUST_LOG=debug cargo run --example in_process  # verbose logging

# P2P multi-node demo (two terminals):
# Terminal 1:
cargo run --example peer_node -- --table-path testdata/sample.csv --addr-file peer1.json
# Terminal 2:
cargo run --example peer_node -- --table-path testdata/sample.csv --addr-file peer2.json \
  --peer peer1.json --query "SELECT city, SUM(amount) FROM data GROUP BY city"
```

```bash
cargo test -p iroh-arrow                 # iroh-arrow unit tests (codec, protocol)
cargo test                               # all workspace tests
```

The `in_process` and `peer_node` / `query_client` examples are the primary end-to-end verification.

## Key files

### iroh-arrow crate (transport layer)

- `iroh-arrow/src/protocol.rs` - `IrohArrow<H>` protocol handler implementing iroh's `ProtocolHandler` trait; `RequestHandler` trait for pluggable request handling; ALPN (`b"mark/iroh-arrow/0"`), MAX_REQUEST_SIZE, MAX_RESPONSE_SIZE
- `iroh-arrow/src/codec.rs` - `encode_batches()` / `decode_batches()` using Arrow IPC StreamWriter/StreamReader
- `iroh-arrow/src/stream.rs` - `send_batches()` / `recv_batches()` / `send_bytes()` / `recv_bytes()` helpers over iroh QUIC streams

### df-p2p crate (DataFusion integration)

- `src/context.rs` - `DistributedContext` session config extension holding iroh `Endpoint` + worker addresses; `execute_on_worker()` handles connect → send plan → receive batches
- `src/worker.rs` - `Worker` struct using iroh `Router` with `IrohArrow<PhysicalPlanHandler>`; `new()` creates own endpoint, `with_endpoint()` shares an existing one for P2P; `plan_protocol_handler()` factory for custom Router setups; deserializes physical plans, executes via DataFusion, returns record batches
- `src/optimizer_rule.rs` - `apply_network_boundaries(plan)` walks the physical plan tree via `transform_down` and replaces the first `CoalescePartitionsExec` with `DistributedCoalesceExec`
- `src/operators/distributed_coalesce.rs` - `DistributedCoalesceExec` leaf ExecutionPlan node; serialized child plan in `Stage`; on `execute(partition)` calls `DistributedContext::execute_on_worker()` to send work to remote peers
- `src/stage.rs` - `Stage { stage_id, encoded_plan }` holds a serialized physical plan (protobuf bytes)
- `examples/in_process.rs` - Both worker and scheduler in one process, sharing a `SessionContext`
- `examples/peer_node.rs` - Unified P2P node: dual ALPN (plan execution + SQL query handling), auto-discovers peers from `peers/` directory, supports CSV (`--table-path`) and hive-partitioned parquet (`--dataset-dir`)
- `examples/query_client.rs` - Connects to any peer and sends a SQL query, displays results

## Architecture

```
Peer A (query initiator)               Peer B (worker)
   │                                     │
   │  SQL → logical plan → physical plan │
   │  optimizer: CoalescePartitionsExec  │
   │     → DistributedCoalesceExec       │
   │  serialize(child_plan)              │
   │  ──── protobuf bytes ────────────>  │
   │                                     │  deserialize → execute
   │  <──── Arrow IPC batches ────────   │
   │                                     │
   │  (Peer A is also a worker —         │
   │   Peer B could query A too)         │
```

The optimizer rule replaces `CoalescePartitionsExec` with `DistributedCoalesceExec`, which serializes the child subtree into a `Stage` and sends it to remote peers. Each peer runs a `Worker` that deserializes and executes the plan locally.

`DistributedContext` (registered as a session config extension) provides the iroh `Endpoint` and worker addresses, and encapsulates the connect → send → receive logic via `execute_on_worker()`.

`IrohArrow<H>` handles the connection lifecycle: accept bi-stream → read request → delegate to `RequestHandler` → send Arrow IPC response → `conn.closed().await`.

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

## Wire protocol

Over iroh QUIC bi-directional stream:
1. **Request** (initiator -> worker): serialized DataFusion physical plan (protobuf via `datafusion-proto`), then `send.finish()`
2. **Response** (worker -> initiator): Arrow IPC stream bytes (schema + batches), then `send.finish()`

## Dependencies

- iroh = "0.96", datafusion = "52.2.0", datafusion-proto = "52.2.0", arrow/arrow-ipc = "57.0.0", tracing = "0.1.44"
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
