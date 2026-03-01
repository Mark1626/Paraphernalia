use std::path::PathBuf;

use anyhow::{Context, Result};
use iroh::Endpoint;
use iroh_blobs::api::blobs::AddPathOptions;
use iroh_blobs::format::collection::Collection;
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::ticket::BlobTicket;
use iroh_blobs::{BlobFormat, BlobsProtocol, ALPN};
use n0_future::StreamExt;
use walkdir::WalkDir;

use crate::progress;

pub async fn run(path: PathBuf) -> Result<()> {
    let path = std::fs::canonicalize(&path)
        .with_context(|| format!("path not found: {}", path.display()))?;

    // Create temp directory for blob store
    let suffix: [u8; 8] = rand::random();
    let hex = data_encoding::HEXLOWER.encode(&suffix);
    let cwd = std::env::current_dir()?;
    let blobs_dir = cwd.join(format!(".sendme-send-{hex}"));
    std::fs::create_dir_all(&blobs_dir)?;

    let spinner = progress::make_spinner("Setting up endpoint...");

    // Create endpoint and blob store
    let endpoint = Endpoint::builder()
        .alpns(vec![ALPN.to_vec()])
        .bind()
        .await
        .context("failed to bind endpoint")?;

    let store = FsStore::load(&blobs_dir)
        .await
        .context("failed to create blob store")?;

    // Import files
    spinner.set_message("Importing files...");

    let mut entries: Vec<(String, iroh_blobs::Hash)> = Vec::new();

    if path.is_file() {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let hash = import_file(&store, &path, &name).await?;
        entries.push((name, hash));
    } else if path.is_dir() {
        for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let full_path = entry.path().to_path_buf();
                let relative = full_path
                    .strip_prefix(&path)
                    .unwrap_or(&full_path)
                    .to_string_lossy()
                    .to_string();
                let hash = import_file(&store, &full_path, &relative).await?;
                entries.push((relative, hash));
            }
        }
    } else {
        anyhow::bail!("path is neither a file nor a directory: {}", path.display());
    }

    if entries.is_empty() {
        anyhow::bail!("no files found at {}", path.display());
    }

    spinner.set_message(format!("Imported {} file(s), creating collection...", entries.len()));

    // Create collection from imported files
    let collection = Collection::from_iter(entries.clone());
    let temp_tag = collection.store(&store).await?;
    let hash = temp_tag.hash();
    // Make the collection permanent
    store.tags().create(temp_tag.hash_and_format()).await?;

    // Set up protocol handler and router
    let blobs = BlobsProtocol::new(&store, None);
    let router = iroh::protocol::Router::builder(endpoint)
        .accept(ALPN, blobs)
        .spawn();

    spinner.set_message("Waiting for endpoint to come online...");

    // Wait for relay connection
    tokio::time::timeout(
        std::time::Duration::from_secs(30),
        router.endpoint().online(),
    )
    .await
    .context("timed out waiting for endpoint to come online")?;

    spinner.finish_and_clear();

    // Print ticket
    let addr = router.endpoint().addr();
    let ticket = BlobTicket::new(addr, hash, BlobFormat::HashSeq);

    println!("Serving {} file(s)", entries.len());
    for (name, hash) in &entries {
        println!("  {name} ({hash})");
    }
    println!();
    println!("Ticket: {ticket}");
    println!();
    println!("To receive, run:");
    println!("  cargo run -- receive {ticket}");
    println!();
    println!("Press Ctrl+C to stop serving.");

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    println!("\nShutting down...");
    router.shutdown().await?;

    // Cleanup temp directory
    if let Err(e) = std::fs::remove_dir_all(&blobs_dir) {
        tracing::warn!("failed to remove temp dir {}: {e}", blobs_dir.display());
    }

    Ok(())
}

async fn import_file(store: &FsStore, path: &PathBuf, name: &str) -> Result<iroh_blobs::Hash> {
    tracing::info!("importing {name}");

    let progress = store.blobs().add_path_with_opts(AddPathOptions {
        path: path.clone(),
        mode: iroh_blobs::api::blobs::ImportMode::TryReference,
        format: BlobFormat::Raw,
    });

    let mut stream = std::pin::pin!(progress.stream().await);
    let mut hash = None;

    while let Some(item) = stream.next().await {
        use iroh_blobs::api::blobs::AddProgressItem;
        match item {
            AddProgressItem::Done(tag) => {
                hash = Some(tag.hash());
            }
            AddProgressItem::Error(e) => {
                anyhow::bail!("error importing {name}: {e}");
            }
            _ => {}
        }
    }

    hash.context(format!("import of {name} did not produce a hash"))
}
