use crate::events::*;
use crate::iroh_transfer;
use crate::state::*;
use crate::transfer;

use matrix_sdk::{
    authentication::matrix::{MatrixSession, MatrixSessionTokens},
    config::SyncSettings,
    ruma::OwnedRoomId,
    Client, SessionMeta,
};
use matrix_sdk::ruma::{OwnedDeviceId, OwnedUserId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::fs;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct SavedSession {
    homeserver_url: String,
    user_id: String,
    device_id: String,
    access_token: String,
}

fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("matrix-fileshare")
}

fn session_file_path(username: &str, homeserver_url: &str) -> PathBuf {
    let safe_username = username.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
    let safe_host = homeserver_url.replace("://", "_").replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != '.', "_");
    data_dir().join(format!("session_{}_{}.json", safe_username, safe_host))
}

fn db_path_for(username: &str, homeserver_url: &str) -> PathBuf {
    let safe_username = username.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
    let safe_host = homeserver_url.replace("://", "_").replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != '.', "_");
    data_dir().join(format!("{}_{}", safe_username, safe_host))
}

fn save_session(session: &SavedSession, username: &str, homeserver_url: &str) -> Result<(), String> {
    let path = session_file_path(username, homeserver_url);
    let json = serde_json::to_string_pretty(session).map_err(|e| format!("Serialize error: {}", e))?;
    std::fs::create_dir_all(path.parent().unwrap()).ok();
    std::fs::write(&path, json).map_err(|e| format!("Failed to save session: {}", e))?;
    info!("Session saved to {:?}", path);
    Ok(())
}

fn load_session(username: &str, homeserver_url: &str) -> Option<SavedSession> {
    let path = session_file_path(username, homeserver_url);
    let json = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&json).ok()
}

fn clear_session(username: &str, homeserver_url: &str) {
    let _ = std::fs::remove_file(session_file_path(username, homeserver_url));
    let _ = std::fs::remove_dir_all(db_path_for(username, homeserver_url));
}

pub async fn login(
    homeserver: &str,
    username: &str,
    password: &str,
    state: &AppState,
    app: &AppHandle,
) -> Result<(String, String), String> {
    let emit_status = |msg: &str| {
        let _ = app.emit("login-status", msg);
    };

    let homeserver_url = if homeserver.starts_with("http") {
        homeserver.to_string()
    } else {
        format!("https://{}", homeserver)
    };

    if let Some(saved) = load_session(username, &homeserver_url) {
        emit_status("Restoring session...");
        info!("Found saved session for {}, attempting restore", saved.user_id);

        let db_path = db_path_for(username, &homeserver_url);
        std::fs::create_dir_all(&db_path).ok();

        if let Ok(client) = Client::builder()
            .homeserver_url(&saved.homeserver_url)
            .sqlite_store(&db_path, None)
            .build()
            .await
        {
            let session = MatrixSession {
                meta: SessionMeta {
                    user_id: OwnedUserId::try_from(saved.user_id.as_str()).unwrap(),
                    device_id: OwnedDeviceId::from(saved.device_id.as_str()),
                },
                tokens: MatrixSessionTokens {
                    access_token: saved.access_token,
                    refresh_token: None,
                },
            };

            match client.restore_session(session).await {
                Ok(_) => {
                    info!("Session restored successfully for {}", saved.user_id);
                    emit_status("Session restored, syncing...");
                    *state.client.write().await = Some(client);
                    return Ok((saved.user_id, saved.device_id));
                }
                Err(e) => {
                    warn!("Session restore failed ({}), falling back to login", e);
                    clear_session(username, &homeserver_url);
                }
            }
        } else {
            warn!("Failed to build client for session restore, clearing");
            clear_session(username, &homeserver_url);
        }
    }

    emit_status("Resolving homeserver...");

    let db_path = db_path_for(username, &homeserver_url);
    let _ = std::fs::remove_dir_all(&db_path);
    std::fs::create_dir_all(&db_path).map_err(|e| format!("Failed to create data dir: {}", e))?;

    emit_status("Initializing...");

    let client = Client::builder()
        .homeserver_url(&homeserver_url)
        .sqlite_store(&db_path, None)
        .build()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;

    emit_status("Authenticating...");

    let login_result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        client
            .matrix_auth()
            .login_username(username, password)
            .initial_device_display_name("Matrix File Share")
            .send(),
    )
    .await;

    let login_response = match login_result {
        Ok(Ok(resp)) => resp,
        Ok(Err(e)) => {
            let err_str = format!("{}", e);
            if err_str.contains("429") || err_str.to_lowercase().contains("rate") || err_str.to_lowercase().contains("limit") {
                return Err("Rate limited by server. Please wait a minute and try again.".to_string());
            }
            return Err(format!("Login failed: {}", e));
        }
        Err(_) => {
            return Err("Login timed out. The server may be rate limiting you — wait a minute and try again.".to_string());
        }
    };

    let user_id = login_response.user_id.to_string();
    let device_id = login_response.device_id.to_string();

    info!("Logged in as {} on device {}", user_id, device_id);

    emit_status("Saving session...");
    let saved = SavedSession {
        homeserver_url: homeserver_url.clone(),
        user_id: user_id.clone(),
        device_id: device_id.clone(),
        access_token: login_response.access_token,
    };
    if let Err(e) = save_session(&saved, username, &homeserver_url) {
        warn!("Failed to save session: {}", e);
    }

    emit_status("Starting sync...");
    *state.client.write().await = Some(client);

    Ok((user_id, device_id))
}

pub async fn login_headless(
    homeserver: &str,
    username: &str,
    password: &str,
    state: &AppState,
) -> Result<(String, String), String> {
    let homeserver_url = if homeserver.starts_with("http") {
        homeserver.to_string()
    } else {
        format!("https://{}", homeserver)
    };

    if let Some(saved) = load_session(username, &homeserver_url) {
        println!("Restoring session...");
        info!("Found saved session for {}, attempting restore", saved.user_id);

        let db_path = db_path_for(username, &homeserver_url);
        std::fs::create_dir_all(&db_path).ok();

        if let Ok(client) = Client::builder()
            .homeserver_url(&saved.homeserver_url)
            .sqlite_store(&db_path, None)
            .build()
            .await
        {
            let session = MatrixSession {
                meta: SessionMeta {
                    user_id: OwnedUserId::try_from(saved.user_id.as_str()).unwrap(),
                    device_id: OwnedDeviceId::from(saved.device_id.as_str()),
                },
                tokens: MatrixSessionTokens {
                    access_token: saved.access_token,
                    refresh_token: None,
                },
            };

            match client.restore_session(session).await {
                Ok(_) => {
                    match client.whoami().await {
                        Ok(_) => {
                            info!("Session restored and verified for {}", saved.user_id);
                            println!("Session restored.");
                            *state.client.write().await = Some(client);
                            return Ok((saved.user_id, saved.device_id));
                        }
                        Err(e) => {
                            warn!("Session token expired ({}), falling back to login", e);
                            clear_session(username, &homeserver_url);
                        }
                    }
                }
                Err(e) => {
                    warn!("Session restore failed ({}), falling back to login", e);
                    clear_session(username, &homeserver_url);
                }
            }
        } else {
            warn!("Failed to build client for session restore, clearing");
            clear_session(username, &homeserver_url);
        }
    }

    println!("Logging in to {}...", homeserver_url);

    let db_path = db_path_for(username, &homeserver_url);
    let _ = std::fs::remove_dir_all(&db_path);
    std::fs::create_dir_all(&db_path).map_err(|e| format!("Failed to create data dir: {}", e))?;

    let client = Client::builder()
        .homeserver_url(&homeserver_url)
        .sqlite_store(&db_path, None)
        .build()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;

    let login_result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        client
            .matrix_auth()
            .login_username(username, password)
            .initial_device_display_name("Matrix File Share CLI")
            .send(),
    )
    .await;

    let login_response = match login_result {
        Ok(Ok(resp)) => resp,
        Ok(Err(e)) => {
            let err_str = format!("{}", e);
            if err_str.contains("429") || err_str.to_lowercase().contains("rate") || err_str.to_lowercase().contains("limit") {
                return Err("Rate limited by server. Please wait a minute and try again.".to_string());
            }
            return Err(format!("Login failed: {}", e));
        }
        Err(_) => {
            return Err("Login timed out. The server may be rate limiting you — wait a minute and try again.".to_string());
        }
    };

    let user_id = login_response.user_id.to_string();
    let device_id = login_response.device_id.to_string();

    info!("Logged in as {} on device {}", user_id, device_id);

    let saved = SavedSession {
        homeserver_url: homeserver_url.clone(),
        user_id: user_id.clone(),
        device_id: device_id.clone(),
        access_token: login_response.access_token,
    };
    if let Err(e) = save_session(&saved, username, &homeserver_url) {
        warn!("Failed to save session: {}", e);
    }

    *state.client.write().await = Some(client);

    Ok((user_id, device_id))
}

pub async fn get_rooms(state: &AppState) -> Result<Vec<RoomSummary>, String> {
    let client_guard = state.client.read().await;
    let client = client_guard.as_ref().ok_or("Not logged in")?;

    let rooms: Vec<RoomSummary> = client
        .joined_rooms()
        .into_iter()
        .filter(|room| !room.is_space())
        .map(|room| {
            let name = room
                .cached_display_name()
                .map(|n| n.to_string())
                .unwrap_or_else(|| room.room_id().to_string());
            let count = room.joined_members_count();
            let member_count = if count > 0 { count } else { room.active_members_count() };
            RoomSummary {
                room_id: room.room_id().to_string(),
                name,
                member_count,
            }
        })
        .collect();

    Ok(rooms)
}

pub async fn get_file_offers(
    state: &AppState,
    room_id: &str,
) -> Result<Vec<FileOfferData>, String> {
    let client_guard = state.client.read().await;
    let client = client_guard.as_ref().ok_or("Not logged in")?;

    let room_id = OwnedRoomId::try_from(room_id)
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let room = client.get_room(&room_id).ok_or("Room not found")?;

    let mut offers = Vec::new();
    let mut from_token: Option<String> = None;

    // Paginate through room history (up to 10 pages) to find all offers
    for _ in 0..10 {
        let options = matrix_sdk::room::MessagesOptions::backward()
            .from(from_token.as_deref());
        let messages = room
            .messages(options)
            .await
            .map_err(|e| format!("Failed to get messages: {}", e))?;

        if messages.chunk.is_empty() {
            break;
        }

        for event in &messages.chunk {
            let raw = event.raw();
            if let Ok(value) = raw.deserialize_as::<serde_json::Value>() {
                if value.get("type").and_then(|t| t.as_str()) == Some(EVENT_FILE_OFFER) {
                    if let Some(content) = value.get("content") {
                        if let Ok(offer_content) =
                            serde_json::from_value::<FileOfferContent>(content.clone())
                        {
                            let sender = value
                                .get("sender")
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string();

                            offers.push(FileOfferData {
                                offer_id: offer_content.offer_id,
                                filename: offer_content.filename,
                                size: offer_content.size,
                                mimetype: offer_content.mimetype,
                                sha256: offer_content.sha256,
                                description: offer_content.description,
                                sender_user_id: sender,
                                sender_device_id: offer_content.sender_device_id,
                                room_id: room_id.to_string(),
                                iroh_ticket: offer_content.iroh_ticket,
                            });
                        }
                    }
                }
            }
        }

        match messages.end {
            Some(token) => from_token = Some(token),
            None => break,
        }
    }

    Ok(offers)
}

pub async fn offer_file(
    state: &AppState,
    room_id: &str,
    file_path: &str,
    description: Option<String>,
) -> Result<String, String> {
    let client_guard = state.client.read().await;
    let client = client_guard.as_ref().ok_or("Not logged in")?;

    let room_id = OwnedRoomId::try_from(room_id)
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let room = client.get_room(&room_id).ok_or("Room not found")?;

    let path = PathBuf::from(file_path);
    let file_data = fs::read(&path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let size = file_data.len() as u64;

    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let sha256 = format!("{:x}", hasher.finalize());

    let mimetype = mime_guess_from_path(&path);
    let offer_id = Uuid::new_v4().to_string();

    let device_id = client.device_id().map(|d| d.to_string());

    // Generate Iroh ticket if available (non-fatal)
    let iroh_ticket = match iroh_transfer::add_blob_for_offer(state, file_path).await {
        Ok(ticket) => {
            info!("Iroh ticket generated for offer {}", offer_id);
            Some(ticket)
        }
        Err(e) => {
            warn!("Iroh ticket generation failed (Matrix-only): {}", e);
            None
        }
    };

    let content = FileOfferContent {
        offer_id: offer_id.clone(),
        filename: filename.clone(),
        size,
        mimetype,
        sha256: sha256.clone(),
        description,
        sender_device_id: device_id,
        iroh_ticket: iroh_ticket.clone(),
    };

    let raw_content = serde_json::to_value(&content)
        .map_err(|e| format!("Failed to serialize: {}", e))?;

    room.send_raw(EVENT_FILE_OFFER, raw_content)
        .await
        .map_err(|e| format!("Failed to send offer: {}", e))?;

    info!("Sent file offer: {} ({})", filename, offer_id);

    state.pending_offers.lock().await.insert(
        offer_id.clone(),
        PendingOffer {
            offer_id: offer_id.clone(),
            file_path: path,
            room_id: room_id.to_string(),
            sha256,
            size,
        },
    );

    Ok(offer_id)
}

fn mime_guess_from_path(path: &PathBuf) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "png" => "image/png".to_string(),
        "gif" => "image/gif".to_string(),
        "webp" => "image/webp".to_string(),
        "svg" => "image/svg+xml".to_string(),
        "pdf" => "application/pdf".to_string(),
        "zip" => "application/zip".to_string(),
        "tar" => "application/x-tar".to_string(),
        "gz" => "application/gzip".to_string(),
        "mp4" => "video/mp4".to_string(),
        "webm" => "video/webm".to_string(),
        "mp3" => "audio/mpeg".to_string(),
        "wav" => "audio/wav".to_string(),
        "txt" => "text/plain".to_string(),
        "html" | "htm" => "text/html".to_string(),
        "json" => "application/json".to_string(),
        "xml" => "application/xml".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

/// Create a ProgressFn that emits Tauri events
pub fn tauri_progress(app: &AppHandle) -> ProgressFn {
    let app = app.clone();
    Arc::new(move |event: &str, data: serde_json::Value| {
        let _ = app.emit(event, data);
    })
}

/// Create a ProgressFn that prints to stdout (for CLI)
pub fn cli_progress() -> ProgressFn {
    Arc::new(|event: &str, data: serde_json::Value| {
        match event {
            "transfer-progress" => {
                if let (Some(bytes), Some(total), Some(status)) = (
                    data.get("bytes_transferred").and_then(|v| v.as_u64()),
                    data.get("total_bytes").and_then(|v| v.as_u64()),
                    data.get("status").and_then(|v| v.as_str()),
                ) {
                    let pct = if total > 0 { (bytes * 100) / total } else { 0 };
                    eprint!("\r  {} {}% ({}/{})", status, pct, format_bytes(bytes), format_bytes(total));
                }
            }
            "transfer-complete" => {
                if let Some(path) = data.get("file_path").and_then(|v| v.as_str()) {
                    eprintln!("\n  Transfer complete: {}", path);
                } else {
                    eprintln!("\n  Transfer complete.");
                }
            }
            "transfer-failed" => {
                if let Some(err) = data.get("error").and_then(|v| v.as_str()) {
                    eprintln!("\n  Transfer failed: {}", err);
                }
            }
            _ => {}
        }
    })
}

use crate::state::format_bytes;

pub async fn start_sync(app_handle: AppHandle, state: Arc<AppState>) {
    let client = {
        let guard = state.client.read().await;
        match guard.as_ref() {
            Some(c) => c.clone(),
            None => {
                error!("Cannot start sync: not logged in");
                return;
            }
        }
    };

    let our_user_id = client.user_id().map(|u| u.to_string()).unwrap_or_default();
    let state_for_handle = state.clone();

    let handle = tokio::spawn(async move {
        info!("Starting Matrix sync loop");
        let settings = SyncSettings::default();
        let progress = tauri_progress(&app_handle);

        client
            .sync_with_callback(settings, |response| {
                let app = app_handle.clone();
                let st = state.clone();
                let progress = progress.clone();
                let our_user = our_user_id.clone();
                async move {
                    // Process to-device events for file chunks
                    for raw_event in &response.to_device {
                        if let Ok(Some(event_type)) = raw_event.get_field::<String>("type") {
                            if event_type == EVENT_FILE_CHUNK {
                                if let Ok(Some(content)) = raw_event.get_field::<FileChunkContent>("content") {
                                    let idx = content.chunk_index;
                                    let total = content.total_chunks;
                                    if let Err(e) = transfer::handle_incoming_chunk(&st, &progress, content).await {
                                        error!("Failed to handle chunk {}/{}: {}", idx + 1, total, e);
                                    }
                                }
                            }
                        }
                    }

                    // Process timeline events for file requests and offers
                    for (room_id, room_update) in &response.rooms.join {
                        for timeline_event in &room_update.timeline.events {
                            let raw = timeline_event.raw();
                            if let Ok(Some(event_type)) = raw.get_field::<String>("type") {
                                if event_type == EVENT_FILE_REQUEST {
                                    if let Ok(Some(content)) = raw.get_field::<FileRequestContent>("content") {
                                        if content.target_user == our_user {
                                            let sender = raw.get_field::<String>("sender")
                                                .ok().flatten().unwrap_or_default();
                                            info!("File request from {} for offer {} (send to device: {})",
                                                sender, content.offer_id, content.requester_device_id);

                                            let st_clone = st.clone();
                                            let progress_clone = progress.clone();
                                            let offer_id = content.offer_id.clone();
                                            let requester_device = content.requester_device_id.clone();

                                            tokio::spawn(async move {
                                                if let Err(e) = transfer::send_file_chunks(
                                                    &st_clone,
                                                    &progress_clone,
                                                    &offer_id,
                                                    &sender,
                                                    &requester_device,
                                                ).await {
                                                    error!("Failed to send file chunks: {}", e);
                                                }
                                            });
                                        }
                                    }
                                } else if event_type == EVENT_FILE_OFFER {
                                    if let Ok(Some(offer_content)) = raw.get_field::<FileOfferContent>("content") {
                                        let sender = raw.get_field::<String>("sender")
                                            .ok().flatten().unwrap_or_default();
                                        let offer_data = FileOfferData {
                                            offer_id: offer_content.offer_id,
                                            filename: offer_content.filename,
                                            size: offer_content.size,
                                            mimetype: offer_content.mimetype,
                                            sha256: offer_content.sha256,
                                            description: offer_content.description,
                                            sender_user_id: sender,
                                            sender_device_id: offer_content.sender_device_id,
                                            room_id: room_id.to_string(),
                                            iroh_ticket: offer_content.iroh_ticket,
                                        };
                                        let _ = app.emit("file-offer", &offer_data);
                                    }
                                }
                            }
                        }
                    }

                    // Emit room updates
                    if let Ok(rooms) = get_rooms(&st).await {
                        let _ = app.emit("rooms-updated", &rooms);
                    }
                    matrix_sdk::LoopCtrl::Continue
                }
            })
            .await
            .ok();
    });

    *state_for_handle.sync_handle.lock().await = Some(handle);
}

/// Start sync loop for CLI mode (blocks until cancelled)
pub async fn start_sync_headless(state: Arc<AppState>, progress: ProgressFn) {
    let client = {
        let guard = state.client.read().await;
        match guard.as_ref() {
            Some(c) => c.clone(),
            None => {
                error!("Cannot start sync: not logged in");
                return;
            }
        }
    };

    let our_user_id = client.user_id().map(|u| u.to_string()).unwrap_or_default();

    info!("Starting Matrix sync loop (headless)");
    let settings = SyncSettings::default();

    client
        .sync_with_callback(settings, |response| {
            let st = state.clone();
            let progress = progress.clone();
            let our_user = our_user_id.clone();
            async move {
                // Process to-device events for file chunks
                for raw_event in &response.to_device {
                    if let Ok(Some(event_type)) = raw_event.get_field::<String>("type") {
                        if event_type == EVENT_FILE_CHUNK {
                            if let Ok(Some(content)) = raw_event.get_field::<FileChunkContent>("content") {
                                let idx = content.chunk_index;
                                let total = content.total_chunks;
                                if let Err(e) = transfer::handle_incoming_chunk(&st, &progress, content).await {
                                    error!("Failed to handle chunk {}/{}: {}", idx + 1, total, e);
                                }
                            }
                        }
                    }
                }

                // Process timeline events for file requests
                for (_room_id, room_update) in &response.rooms.join {
                    for timeline_event in &room_update.timeline.events {
                        let raw = timeline_event.raw();
                        if let Ok(Some(event_type)) = raw.get_field::<String>("type") {
                            if event_type == EVENT_FILE_REQUEST {
                                if let Ok(Some(content)) = raw.get_field::<FileRequestContent>("content") {
                                    if content.target_user == our_user {
                                        let sender = raw.get_field::<String>("sender")
                                            .ok().flatten().unwrap_or_default();
                                        info!("File request from {} for offer {}", sender, content.offer_id);

                                        let st_clone = st.clone();
                                        let progress_clone = progress.clone();
                                        let offer_id = content.offer_id.clone();
                                        let requester_device = content.requester_device_id.clone();

                                        tokio::spawn(async move {
                                            if let Err(e) = transfer::send_file_chunks(
                                                &st_clone,
                                                &progress_clone,
                                                &offer_id,
                                                &sender,
                                                &requester_device,
                                            ).await {
                                                error!("Failed to send file chunks: {}", e);
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                matrix_sdk::LoopCtrl::Continue
            }
        })
        .await
        .ok();
}

pub async fn logout(state: &AppState) -> Result<(), String> {
    if let Some(handle) = state.sync_handle.lock().await.take() {
        handle.abort();
    }

    let client_guard = state.client.read().await;
    if let Some(client) = client_guard.as_ref() {
        if let Some(user_id) = client.user_id() {
            let username = user_id.localpart();
            let homeserver = client.homeserver().to_string();
            clear_session(username, &homeserver);
        }

        client
            .matrix_auth()
            .logout()
            .await
            .map_err(|e| format!("Logout failed: {}", e))?;
    }
    drop(client_guard);

    *state.client.write().await = None;
    state.active_transfers.lock().await.clear();
    state.pending_offers.lock().await.clear();

    Ok(())
}
