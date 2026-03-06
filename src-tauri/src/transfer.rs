use crate::events::*;
use crate::state::*;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use futures::stream::{self, TryStreamExt};
use matrix_sdk::ruma::{OwnedDeviceId, OwnedUserId, TransactionId, to_device::DeviceIdOrAllDevices};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs;
use tracing::{error, info};

const CHUNK_SIZE: usize = 48 * 1024; // 48KB — proven reliable for to-device delivery
const CONCURRENT_SENDS: usize = 4; // Send 4 chunks in parallel for ~4× throughput

pub async fn send_file_chunks(
    state: &AppState,
    progress: &ProgressFn,
    offer_id: &str,
    target_user: &str,
    target_device: &str,
) -> Result<(), String> {
    let pending = {
        let offers = state.pending_offers.lock().await;
        offers
            .get(offer_id)
            .ok_or(format!("No pending offer: {}", offer_id))?
            .clone()
    };

    // Clone client and release the lock so concurrent tasks can use it
    let client = {
        let guard = state.client.read().await;
        guard.as_ref().ok_or("Not logged in")?.clone()
    };

    let file_data = fs::read(&pending.file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let total_chunks = ((file_data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE) as u32;

    info!(
        "Sending {} chunks for offer {} to {}:{} ({} concurrent)",
        total_chunks, offer_id, target_user, target_device, CONCURRENT_SENDS
    );

    let target_user_id = OwnedUserId::try_from(target_user)
        .map_err(|e| format!("Invalid user ID: {}", e))?;

    let file_data = Arc::new(file_data);
    let bytes_sent = Arc::new(AtomicU64::new(0));
    let offer_id_owned = offer_id.to_string();
    let target_device_owned = target_device.to_string();

    stream::iter((0..total_chunks).map(Ok::<_, String>))
        .try_for_each_concurrent(CONCURRENT_SENDS, |i| {
            let client = client.clone();
            let file_data = file_data.clone();
            let target_user_id = target_user_id.clone();
            let target_device = target_device_owned.clone();
            let offer_id = offer_id_owned.clone();
            let bytes_sent = bytes_sent.clone();
            let progress = progress.clone();
            let total_size = pending.size;

            async move {
                let start = (i as usize) * CHUNK_SIZE;
                let end = std::cmp::min(start + CHUNK_SIZE, file_data.len());
                let chunk_data = &file_data[start..end];

                let chunk = FileChunkContent {
                    offer_id: offer_id.clone(),
                    chunk_index: i,
                    total_chunks,
                    data: BASE64.encode(chunk_data),
                };

                let chunk_json = serde_json::to_value(&chunk)
                    .map_err(|e| format!("Failed to serialize chunk: {}", e))?;

                let txn_id = TransactionId::new();
                let request = matrix_sdk::ruma::api::client::to_device::send_event_to_device::v3::Request::new_raw(
                    EVENT_FILE_CHUNK.into(),
                    txn_id,
                    [(
                        target_user_id.clone(),
                        [(
                            DeviceIdOrAllDevices::DeviceId(OwnedDeviceId::from(target_device.as_str())),
                            serde_json::from_value(chunk_json).map_err(|e| format!("Serialize error: {}", e))?,
                        )]
                        .into(),
                    )]
                    .into(),
                );

                client
                    .send(request)
                    .await
                    .map_err(|e| format!("Failed to send chunk {}/{}: {}", i + 1, total_chunks, e))?;

                // Track progress atomically across concurrent sends
                let chunk_bytes = (end - start) as u64;
                let new_total = bytes_sent.fetch_add(chunk_bytes, Ordering::Relaxed) + chunk_bytes;

                (progress)(
                    "transfer-progress",
                    serde_json::to_value(&TransferProgress {
                        offer_id: offer_id.clone(),
                        bytes_transferred: std::cmp::min(new_total, total_size),
                        total_bytes: total_size,
                        status: "sending".to_string(),
                    })
                    .unwrap(),
                );

                Ok(())
            }
        })
        .await?;

    info!("Finished sending all {} chunks for offer {}", total_chunks, offer_id_owned);

    (progress)(
        "transfer-progress",
        serde_json::to_value(&TransferProgress {
            offer_id: offer_id_owned.clone(),
            bytes_transferred: pending.size,
            total_bytes: pending.size,
            status: "sending".to_string(),
        })
        .unwrap(),
    );

    (progress)(
        "transfer-complete",
        serde_json::json!({
            "offer_id": offer_id_owned,
            "file_path": pending.file_path.to_string_lossy()
        }),
    );

    Ok(())
}

pub async fn handle_incoming_chunk(
    state: &AppState,
    progress: &ProgressFn,
    chunk: FileChunkContent,
) -> Result<(), String> {
    let mut transfers = state.active_transfers.lock().await;

    let transfer = transfers
        .get_mut(&chunk.offer_id)
        .ok_or(format!("No active transfer for: {}", chunk.offer_id))?;

    // Skip duplicate chunks
    if transfer.chunks_received.contains(&chunk.chunk_index) {
        info!("Skipping duplicate chunk {}/{} for offer {}",
            chunk.chunk_index + 1, chunk.total_chunks, chunk.offer_id);
        return Ok(());
    }

    // Decode base64 chunk data
    let chunk_data = BASE64
        .decode(&chunk.data)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    // Calculate offset and write chunk data
    let offset = (chunk.chunk_index as usize) * CHUNK_SIZE;
    let end = offset + chunk_data.len();
    if end > transfer.data.len() {
        transfer.data.resize(end, 0);
    }
    transfer.data[offset..end].copy_from_slice(&chunk_data);
    transfer.chunks_received.insert(chunk.chunk_index);

    // Use unique chunk count for stable progress reporting
    let bytes_received = std::cmp::min(
        (transfer.chunks_received.len() as u64) * CHUNK_SIZE as u64,
        transfer.total_size,
    );

    // Emit progress
    (progress)(
        "transfer-progress",
        serde_json::to_value(&TransferProgress {
            offer_id: chunk.offer_id.clone(),
            bytes_transferred: bytes_received,
            total_bytes: transfer.total_size,
            status: "receiving".to_string(),
        })
        .unwrap(),
    );

    // Check if transfer is complete (all unique chunks received)
    if transfer.chunks_received.len() as u32 >= transfer.total_chunks {
        info!(
            "All {} chunks received for offer {}, verifying hash",
            transfer.total_chunks, chunk.offer_id
        );

        // Truncate to exact file size
        transfer.data.truncate(transfer.total_size as usize);

        // Verify SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&transfer.data);
        let computed_hash = format!("{:x}", hasher.finalize());

        if computed_hash != transfer.sha256 {
            error!(
                "Hash mismatch for {}: expected {}, got {}. Received {}/{} unique chunks, data len {}",
                chunk.offer_id, transfer.sha256, computed_hash,
                transfer.chunks_received.len(), transfer.total_chunks,
                transfer.data.len()
            );
            (progress)(
                "transfer-failed",
                serde_json::json!({
                    "offer_id": chunk.offer_id,
                    "error": "Hash verification failed"
                }),
            );
            transfers.remove(&chunk.offer_id);
            return Err("Hash verification failed".to_string());
        }

        // Save file to disk
        let save_path = transfer.save_path.clone();
        let data = transfer.data.clone();
        let offer_id = chunk.offer_id.clone();
        transfers.remove(&offer_id);
        drop(transfers);

        fs::write(&save_path, &data)
            .await
            .map_err(|e| format!("Failed to save file: {}", e))?;

        info!("File saved to {:?}", save_path);

        (progress)(
            "transfer-complete",
            serde_json::json!({
                "offer_id": offer_id,
                "file_path": save_path.to_string_lossy()
            }),
        );
    }

    Ok(())
}

pub async fn start_receiving(
    state: &AppState,
    offer: &FileOfferData,
    save_dir: &str,
) -> Result<(), String> {
    let save_path = PathBuf::from(save_dir).join(&offer.filename);

    let total_chunks = ((offer.size as usize + CHUNK_SIZE - 1) / CHUNK_SIZE) as u32;

    let transfer = ActiveTransfer {
        offer_id: offer.offer_id.clone(),
        filename: offer.filename.clone(),
        total_size: offer.size,
        sha256: offer.sha256.clone(),
        chunks_received: HashSet::new(),
        total_chunks,
        data: Vec::with_capacity(offer.size as usize),
        save_path,
    };

    state
        .active_transfers
        .lock()
        .await
        .insert(offer.offer_id.clone(), transfer);

    info!(
        "Ready to receive {} ({} chunks)",
        offer.filename, total_chunks
    );

    Ok(())
}
