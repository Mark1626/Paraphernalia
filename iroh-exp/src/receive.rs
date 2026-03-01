use std::str::FromStr;

use anyhow::{Context, Result};
use iroh::address_lookup::MemoryLookup;
use iroh::Endpoint;
use iroh_blobs::api::downloader::{DownloadOptions, DownloadProgressItem, Shuffled, SplitStrategy};
use iroh_blobs::format::collection::Collection;
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::ticket::BlobTicket;
use iroh_blobs::HashAndFormat;
use n0_future::StreamExt;

use crate::progress;

pub async fn run(ticket_str: String, provider_strs: Vec<String>, strategy: String) -> Result<()> {
    // Parse primary ticket
    let primary = BlobTicket::from_str(&ticket_str).context("invalid primary ticket")?;
    let hash = primary.hash();
    let format = primary.format();

    // Parse additional provider tickets and validate hash match
    let mut all_tickets = vec![primary.clone()];
    for (i, p) in provider_strs.iter().enumerate() {
        let t = BlobTicket::from_str(p)
            .with_context(|| format!("invalid provider ticket #{}", i + 2))?;
        if t.hash() != hash {
            anyhow::bail!(
                "provider ticket #{} has different hash: expected {}, got {}",
                i + 2,
                hash,
                t.hash()
            );
        }
        all_tickets.push(t);
    }

    let split_strategy = match strategy.as_str() {
        "split" => SplitStrategy::Split,
        "sequential" => SplitStrategy::None,
        other => anyhow::bail!("unknown strategy: {other} (use 'split' or 'sequential')"),
    };

    println!(
        "Downloading from {} provider(s) using {strategy} strategy",
        all_tickets.len()
    );

    // Create temp directory for blob store
    let cwd = std::env::current_dir()?;
    let recv_dir = cwd.join(format!(".sendme-recv-{}", hash.to_hex()));
    std::fs::create_dir_all(&recv_dir)?;

    let spinner = progress::make_spinner("Setting up endpoint...");

    // Create MemoryLookup with all provider addresses so the Downloader can resolve them
    let memory_lookup = MemoryLookup::new();
    for ticket in &all_tickets {
        memory_lookup.add_endpoint_info(ticket.addr().clone());
    }

    // Create endpoint with address lookup (no server ALPNs needed for receiving)
    let endpoint = Endpoint::builder()
        .alpns(vec![])
        .address_lookup(memory_lookup)
        .bind()
        .await
        .context("failed to bind endpoint")?;

    // Create blob store
    let store = FsStore::load(&recv_dir)
        .await
        .context("failed to create blob store")?;

    spinner.set_message("Downloading...");

    // Collect provider IDs
    let provider_ids: Vec<_> = all_tickets.iter().map(|t| t.addr().id).collect();
    for (i, id) in provider_ids.iter().enumerate() {
        println!("  Provider {}: {id}", i + 1);
    }

    // Create downloader and start download
    let downloader = store.downloader(&endpoint);
    let request = HashAndFormat::new(hash, format);
    let download = downloader.download_with_opts(DownloadOptions::new(
        request,
        Shuffled::new(provider_ids),
        split_strategy,
    ));

    // Stream progress
    let pb = progress::make_download_bar(None);
    let mut stream = download.stream().await?;

    while let Some(item) = stream.next().await {
        match item {
            DownloadProgressItem::Progress(bytes) => {
                pb.set_position(bytes);
            }
            DownloadProgressItem::TryProvider { id, .. } => {
                tracing::info!("trying provider {id}");
            }
            DownloadProgressItem::ProviderFailed { id, .. } => {
                tracing::warn!("provider {id} failed");
            }
            DownloadProgressItem::PartComplete { .. } => {
                tracing::debug!("part complete");
            }
            DownloadProgressItem::DownloadError => {
                pb.finish_and_clear();
                anyhow::bail!("download failed");
            }
            DownloadProgressItem::Error(e) => {
                pb.finish_and_clear();
                anyhow::bail!("download error: {e}");
            }
        }
    }

    pb.finish_and_clear();
    spinner.finish_and_clear();
    println!("Download complete!");

    // Export files from collection
    let output_dir = cwd.clone();
    let collection = Collection::load(hash, store.as_ref()).await?;

    for (name, blob_hash) in collection.iter() {
        let target = output_dir.join(name);

        // Ensure parent directory exists
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let export_bar = progress::make_export_bar(name);

        store
            .blobs()
            .export(*blob_hash, target.clone())
            .await
            .with_context(|| format!("failed to export {name}"))?;

        export_bar.finish_with_message(format!("{name} -> {}", target.display()));
    }

    println!(
        "Exported {} file(s) to {}",
        collection.iter().count(),
        output_dir.display()
    );

    // Cleanup temp directory
    if let Err(e) = std::fs::remove_dir_all(&recv_dir) {
        tracing::warn!("failed to remove temp dir {}: {e}", recv_dir.display());
    }

    Ok(())
}
