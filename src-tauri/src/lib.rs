pub mod cli;
mod events;
mod iroh_transfer;
pub mod matrix_client;
mod state;
mod transfer;

use state::{AppState, FileOfferData, RoomSummary, SharedState};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tracing::{info, warn};

#[tauri::command]
async fn login(
    homeserver: String,
    username: String,
    password: String,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<serde_json::Value, String> {
    let (user_id, device_id) = matrix_client::login(&homeserver, &username, &password, &state, &app).await?;

    // Initialize Iroh P2P (non-fatal if it fails)
    match iroh_transfer::init_iroh(&state).await {
        Ok(()) => info!("Iroh P2P initialized"),
        Err(e) => warn!("Iroh P2P unavailable (Matrix-only mode): {}", e),
    }

    // Start sync loop in background
    matrix_client::start_sync(app, state.inner().clone()).await;

    Ok(serde_json::json!({
        "user_id": user_id,
        "device_id": device_id
    }))
}

#[tauri::command]
async fn get_rooms(state: State<'_, SharedState>) -> Result<Vec<RoomSummary>, String> {
    matrix_client::get_rooms(&state).await
}

#[tauri::command]
async fn get_file_offers(
    room_id: String,
    state: State<'_, SharedState>,
) -> Result<Vec<FileOfferData>, String> {
    matrix_client::get_file_offers(&state, &room_id).await
}

#[tauri::command]
async fn offer_file(
    room_id: String,
    file_path: String,
    description: Option<String>,
    state: State<'_, SharedState>,
) -> Result<String, String> {
    matrix_client::offer_file(&state, &room_id, &file_path, description).await
}

#[tauri::command]
async fn request_file(
    room_id: String,
    offer_id: String,
    save_dir: String,
    sender_user_id: String,
    sender_device_id: String,
    iroh_ticket: Option<String>,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    // Look up the offer details
    let offers = matrix_client::get_file_offers(&state, &room_id).await?;
    let offer = offers
        .iter()
        .find(|o| o.offer_id == offer_id)
        .ok_or("Offer not found")?
        .clone();

    let save_path = std::path::Path::new(&save_dir).join(&offer.filename);
    let progress = matrix_client::tauri_progress(&app);

    // Use ticket from frontend if provided, otherwise from offer lookup
    let ticket_to_use = iroh_ticket.or(offer.iroh_ticket.clone());

    // Try Iroh P2P first if a ticket is available
    if let Some(ref ticket) = ticket_to_use {
        info!("Attempting Iroh P2P download for offer {}", offer_id);

        let iroh_result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            iroh_transfer::download_via_iroh(
                &state,
                ticket,
                save_path.to_str().unwrap_or(&save_dir),
                &progress,
                &offer_id,
                offer.size,
            ),
        )
        .await;

        match iroh_result {
            Ok(Ok(())) => {
                info!("Iroh P2P download succeeded for offer {}", offer_id);
                (progress)(
                    "transfer-complete",
                    serde_json::json!({
                        "offer_id": offer_id,
                        "file_path": save_path.to_string_lossy()
                    }),
                );
                return Ok(());
            }
            Ok(Err(e)) => {
                warn!("Iroh download failed, falling back to Matrix: {}", e);
            }
            Err(_) => {
                warn!("Iroh download timed out (10s), falling back to Matrix");
            }
        }
    }

    // Fallback: Matrix to-device chunk transfer
    info!("Using Matrix to-device transfer for offer {}", offer_id);

    // Start receiving (prepare transfer state)
    transfer::start_receiving(&state, &offer, &save_dir).await?;

    // Send file request event to room so the sender knows to start sending
    let client_guard = state.client.read().await;
    let client = client_guard.as_ref().ok_or("Not logged in")?;

    let room_id_owned = matrix_sdk::ruma::OwnedRoomId::try_from(room_id.as_str())
        .map_err(|e| format!("Invalid room ID: {}", e))?;
    let room = client
        .get_room(&room_id_owned)
        .ok_or("Room not found")?;

    let our_device_id = client.device_id()
        .map(|d| d.to_string())
        .ok_or("No device ID available")?;

    let request_content = events::FileRequestContent {
        offer_id: offer_id.clone(),
        target_user: sender_user_id,
        target_device: sender_device_id,
        requester_device_id: our_device_id,
    };

    let raw_content = serde_json::to_value(&request_content)
        .map_err(|e| format!("Failed to serialize: {}", e))?;

    room.send_raw(events::EVENT_FILE_REQUEST, raw_content)
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    info!("Sent file request for offer {}", offer_id);

    Ok(())
}

#[tauri::command]
async fn cancel_transfer(offer_id: String, state: State<'_, SharedState>) -> Result<(), String> {
    state.active_transfers.lock().await.remove(&offer_id);
    state.pending_offers.lock().await.remove(&offer_id);
    Ok(())
}

#[tauri::command]
async fn logout(state: State<'_, SharedState>) -> Result<(), String> {
    iroh_transfer::shutdown_iroh(&state).await;
    matrix_client::logout(&state).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let shared_state: SharedState = Arc::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(shared_state)
        .invoke_handler(tauri::generate_handler![
            login,
            get_rooms,
            get_file_offers,
            offer_file,
            request_file,
            cancel_transfer,
            logout,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
