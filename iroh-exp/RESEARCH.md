# Iroh Multi-Provider Download Research

## Goal

Build a version of [sendme](https://github.com/n0-computer/sendme) that can receive from multiple senders simultaneously.

## Architecture Overview

Iroh (core: `iroh 0.32+`, blobs: `iroh-blobs 0.97.0`) is composed of two main layers:

- **`iroh`** (core networking): Provides the `Endpoint` type for QUIC-based P2P connections, identified by public keys. Supports hole-punching, relay fallback, and concurrent QUIC streams.
- **`iroh-blobs`**: Content-addressed blob transfer protocol built on `iroh`. Blobs are identified by BLAKE3 hashes and transferred with incremental verification (each 16 KiB chunk is verified during streaming).

## Multi-Provider Download Support

**iroh-blobs has first-class, built-in support for downloading from multiple senders simultaneously.**

### The `Downloader` Component

```rust
let downloader = Downloader::new(&store, &endpoint);

let progress = downloader.download(
    request,    // WHAT to download (impl SupportedRequest)
    providers,  // WHERE to download from (impl ContentDiscovery)
);
```

### `ContentDiscovery` Trait

Returns a stream of provider `EndpointId` values (node public keys):

```rust
trait ContentDiscovery {
    fn find_providers(&self, hash: HashAndFormat) -> Boxed<EndpointId>;
}
```

Built-in implementations:
- **`Shuffled`** — wraps a `Vec<EndpointId>`, returns providers in randomized order
- **Any `IntoIterator<Item = EndpointId>`** — pass a plain `Vec`, slice, or iterator

Custom implementations can integrate with DHT, tracker, or other discovery systems.

### `SplitStrategy` Enum

```rust
pub enum SplitStrategy {
    None,   // Sequential: try providers one at a time with fallback
    Split,  // Parallel: split blob into chunks, download from different providers concurrently
}
```

#### Mode 1: Sequential Fallback (`SplitStrategy::None`)
- Tries provider 1, downloads as much as possible
- On failure, moves to provider 2 but **only requests remaining data**
- Resilience/redundancy mechanism, not parallel downloading

#### Mode 2: True Parallel Multi-Provider (`SplitStrategy::Split`)
- `split_request` divides a blob into multiple chunk-level `GetRequest` objects
- Requests executed **concurrently** via `buffered_unordered(32)` — up to **32 parallel download tasks**
- Each task still does sequential fallback if its assigned provider fails
- Progress from all tasks aggregated into a unified `DownloadProgress` stream

### Detailed Flow (Parallel Mode)

1. Caller creates download request with multiple providers
2. Downloader fetches blob metadata to determine total size
3. `split_request()` divides blob into per-chunk `GetRequest` objects
4. Up to 32 concurrent tasks spawned via `buffered_unordered(32)`
5. Each task:
   - Iterates through `ContentDiscovery` stream for provider candidates
   - Uses `ConnectionPool` to get/establish QUIC connection
   - Checks locally available data (`local.is_complete()`)
   - Requests only **missing** chunks (`local.missing()`)
   - Falls back to next provider on failure
6. Progress aggregated through `tokio::sync::mpsc` channel

## Capability Matrix

| Capability | Supported | Mechanism |
|---|---|---|
| Multiple simultaneous connections | Yes | `Endpoint` + `ConnectionPool` |
| Sequential provider fallback | Yes | `SplitStrategy::None` |
| True parallel multi-provider download | Yes | `SplitStrategy::Split` (up to 32 concurrent) |
| Partial progress preservation | Yes | Tracks local bytes, requests only missing ranges |
| Pluggable provider discovery | Yes | `ContentDiscovery` trait |
| Collections from multiple providers | Yes | `GetManyRequest` split per-hash |
| Built-in global discovery (DHT) | Experimental | Available in `iroh-experiments`, not in main crate |

## Content Discovery (Experimental)

The iroh team has been experimenting with:
- **BitTorrent mainline DHT** as bootstrap layer for tracker discovery
- **Tracker nodes** storing signed announcements from providers
- **Probing**: trackers verify announcers have data by requesting random 1 KiB chunks
- **Rate limiting** by IP address and node ID

Available in `iroh-experiments`, not yet in `iroh-blobs` proper. The `ContentDiscovery` trait is the integration point.

## Caveats

- Existing examples (`transfer.rs`, `transfer-collection.rs`) only demonstrate **single-sender** transfers
- No official multi-provider examples yet
- Content discovery is pluggable — you must supply your own `ContentDiscovery` or use a simple provider list
- Multi-provider is handled at the `Downloader` orchestration layer, not the protocol layer

## Conclusion

**Fully feasible.** iroh-blobs provides all the building blocks needed. The main work is:
1. Setting up multiple sender nodes that serve the same blob
2. Using `Downloader` with `SplitStrategy::Split` and a list of providers on the receiver side
3. Optionally implementing a custom `ContentDiscovery` for dynamic provider lookup

## References

- [iroh GitHub](https://github.com/n0-computer/iroh)
- [iroh-blobs GitHub](https://github.com/n0-computer/iroh-blobs)
- [iroh-blobs API docs](https://docs.rs/iroh-blobs/latest/iroh_blobs/)
- [iroh-blobs downloader module](https://docs.rs/iroh-blobs/latest/iroh_blobs/api/downloader/index.html)
- [iroh-blobs 0.90 blog post](https://www.iroh.computer/blog/iroh-blobs-0-90-new-features)
- [iroh-blobs 0.95 blog post](https://www.iroh.computer/blog/iroh-blobs-0-95-new-features)
- [Content Discovery blog post](https://www.iroh.computer/blog/iroh-content-discovery)
- [Blobs protocol docs](https://docs.iroh.computer/protocols/blobs)
