use crate::state::*;

use iroh::Endpoint;
use iroh::protocol::Router;
use iroh_blobs::store::mem::MemStore;
use iroh_blobs::ticket::BlobTicket;
use iroh_blobs::BlobsProtocol;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};

/// Initialize the Iroh P2P endpoint, blob store, and protocol router.
/// Non-fatal — if this fails, we fall back to Matrix-only transfers.
pub async fn init_iroh(state: &AppState) -> Result<(), String> {
    let endpoint = Endpoint::bind()
        .await
        .map_err(|e| format!("Iroh endpoint bind failed: {}", e))?;

    let store = MemStore::new();
    let blobs = BlobsProtocol::new(&store, None);

    let router = Router::builder(endpoint.clone())
        .accept(iroh_blobs::ALPN, blobs)
        .spawn();

    let addr = endpoint.addr();
    info!("Iroh endpoint ready: id={}", addr.id);

    *state.iroh_endpoint.lock().await = Some(endpoint);
    *state.iroh_store.lock().await = Some(store);
    *state.iroh_router.lock().await = Some(router);

    Ok(())
}

/// Shutdown Iroh cleanly.
pub async fn shutdown_iroh(state: &AppState) {
    if let Some(router) = state.iroh_router.lock().await.take() {
        if let Err(e) = router.shutdown().await {
            warn!("Iroh router shutdown error: {}", e);
        }
    }
    if let Some(endpoint) = state.iroh_endpoint.lock().await.take() {
        endpoint.close().await;
    }
    *state.iroh_store.lock().await = None;
    info!("Iroh shutdown complete");
}

/// Add a file to the Iroh blob store and return a BlobTicket string.
/// The ticket encodes our endpoint address + blob hash so the receiver
/// can download directly via P2P.
pub async fn add_blob_for_offer(state: &AppState, file_path: &str) -> Result<String, String> {
    let store = state.iroh_store.lock().await
        .as_ref().ok_or("Iroh not initialized")?.clone();
    let endpoint = state.iroh_endpoint.lock().await
        .as_ref().ok_or("Iroh not initialized")?.clone();

    let data = fs::read(file_path)
        .await
        .map_err(|e| format!("Failed to read file for Iroh: {}", e))?;

    let tag = store
        .add_slice(&data)
        .await
        .map_err(|e| format!("Failed to add blob to Iroh store: {}", e))?;

    let addr = endpoint.addr();
    let ticket = BlobTicket::new(addr, tag.hash, tag.format);

    info!(
        "Iroh blob added: hash={}, ticket={}...{}",
        tag.hash,
        &ticket.to_string()[..20],
        &ticket.to_string()[ticket.to_string().len().saturating_sub(8)..]
    );

    Ok(ticket.to_string())
}

/// Download a file via Iroh P2P using a BlobTicket string.
/// Returns Ok(()) on success, Err on failure (caller should fall back to Matrix).
pub async fn download_via_iroh(
    state: &AppState,
    ticket_str: &str,
    save_path: &str,
    progress: &ProgressFn,
    offer_id: &str,
    total_size: u64,
) -> Result<(), String> {
    let ticket: BlobTicket = ticket_str
        .parse()
        .map_err(|e| format!("Failed to parse Iroh ticket: {}", e))?;

    let store = state.iroh_store.lock().await
        .as_ref().ok_or("Iroh not initialized")?.clone();
    let endpoint = state.iroh_endpoint.lock().await
        .as_ref().ok_or("Iroh not initialized")?.clone();

    info!(
        "Downloading via Iroh: hash={}, peer={}",
        ticket.hash(),
        ticket.addr().id
    );

    // Report that we're starting an Iroh download
    (progress)(
        "transfer-progress",
        serde_json::json!({
            "offer_id": offer_id,
            "bytes_transferred": 0,
            "total_bytes": total_size,
            "status": "receiving"
        }),
    );

    // Download the blob from the remote peer
    let downloader = store.downloader(&endpoint);
    downloader
        .download(ticket.hash(), vec![ticket.addr().id])
        .await
        .map_err(|e| format!("Iroh download failed: {}", e))?;

    // Export the downloaded blob to the save path
    let abs_path = Path::new(save_path).to_path_buf();
    store
        .blobs()
        .export(ticket.hash(), abs_path)
        .await
        .map_err(|e| format!("Iroh export failed: {}", e))?;

    info!("Iroh download complete, saved to {}", save_path);

    Ok(())
}
