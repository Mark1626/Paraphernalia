# sendme-multi

A CLI file transfer tool built on [iroh-blobs](https://github.com/n0-computer/iroh-blobs) that can download from **multiple senders in parallel**.

iroh-blobs splits a blob into chunk ranges and fetches them from up to 32 providers concurrently via its built-in `Downloader` with `SplitStrategy::Split`.

## Build

```
cargo build --release
```

## Usage

### Single sender, single receiver

**Terminal A** (sender):

```
cargo run -- send ./myfile.txt
```

This prints a ticket string. The sender keeps running until you press Ctrl+C.

**Terminal B** (receiver):

```
cargo run -- receive <ticket>
```

The file is downloaded and exported to the current directory.

### Multiple senders, single receiver

All senders must serve **byte-identical content** (same files produce the same BLAKE3 hashes). Start each sender independently on different machines or terminals:

**Terminal A:**

```
cargo run -- send ./data/
```

Output:
```
Ticket: blob4abc...
```

**Terminal B:**

```
cargo run -- send ./data/
```

Output:
```
Ticket: blob4xyz...
```

**Terminal C:**

```
cargo run -- send ./data/
```

Output:
```
Ticket: blob4def...
```

Each sender gets a unique ticket (different node identity), but the blob hash inside each ticket is the same because the file content is identical.

**Terminal D** (receiver) -- pass the first ticket as the main argument, then add more with `-p`:

```
cargo run -- receive <ticket-A> -p <ticket-B> -p <ticket-C>
```

The receiver validates that all tickets reference the same hash, then downloads chunks from all three senders in parallel.

### Download strategies

The `--strategy` flag controls how multiple providers are used:

| Strategy | Flag | Behavior |
|---|---|---|
| **Split** (default) | `--strategy split` | Splits the blob into chunk ranges and downloads them from different providers concurrently (up to 32 parallel tasks). Best for large files. |
| **Sequential** | `--strategy sequential` | Tries providers one at a time. If one fails, picks up where it left off with the next provider. Useful as a resilient fallback. |

```
cargo run -- receive <ticket-A> -p <ticket-B> --strategy sequential
```

### Sending directories

`send` accepts a directory path and recursively imports all files into a collection:

```
cargo run -- send ./my-project/
```

The receiver exports the full directory structure to the current working directory.

## How it works

1. **Send**: imports files into a local `FsStore`, creates a `Collection` (name-to-hash mapping), serves it over the iroh-blobs protocol via a QUIC endpoint with relay fallback, and prints a `BlobTicket`.

2. **Receive**: parses all tickets, registers provider addresses in a `MemoryLookup` so the `Downloader` can resolve node IDs to network addresses, then downloads using `SplitStrategy::Split` for parallel multi-provider transfer. After download, the `Collection` is loaded and each blob is exported to the filesystem.

The `Downloader` manages all connection pooling and chunk assignment internally. The receiver never calls `endpoint.connect()` directly -- it provides endpoint addresses and the downloader handles the rest.

## Environment variables

- `RUST_LOG` -- controls log verbosity (e.g., `RUST_LOG=info` or `RUST_LOG=iroh_blobs=debug`)

## Resume support

FsStore persists download state to disk. If a download is interrupted, re-running the same receive command will resume from where it left off (the temp directory `.sendme-recv-<hash>` must still exist).
