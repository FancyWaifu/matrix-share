use crate::events::*;
use crate::iroh_transfer;
use crate::matrix_client;
use crate::state::*;
use crate::transfer;

use clap::{Parser, Subcommand};
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "matrix-fileshare")]
#[command(about = "P2P file sharing over Matrix")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Login to a Matrix homeserver
    Login {
        homeserver: String,
        username: String,
        password: String,
    },
    /// List joined rooms
    Rooms {
        homeserver: String,
        username: String,
        password: String,
    },
    /// List file offers in a room
    Offers {
        homeserver: String,
        username: String,
        password: String,
        room_id: String,
    },
    /// Share a file to a room
    Share {
        homeserver: String,
        username: String,
        password: String,
        room_id: String,
        file_path: String,
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Download a file from an offer
    Download {
        homeserver: String,
        username: String,
        password: String,
        room_id: String,
        offer_id: String,
        save_dir: String,
    },
    /// Serve a file via Iroh P2P (prints ticket, no Matrix needed)
    IrohServe {
        file_path: String,
    },
    /// Download a file via Iroh P2P ticket (no Matrix needed)
    IrohGet {
        ticket: String,
        save_path: String,
    },
}

fn fix_room_id(id: &str) -> String {
    // Strip shell-escape backslash before '!' if present
    let id = id.strip_prefix("\\!").map(|rest| format!("!{}", rest)).unwrap_or_else(|| id.to_string());
    if id.starts_with('!') {
        id
    } else {
        format!("!{}", id)
    }
}

pub async fn run_cli(cli: Cli) -> Result<(), String> {
    match cli.command {
        Commands::Login { homeserver, username, password } => {
            cmd_login(&homeserver, &username, &password).await
        }
        Commands::Rooms { homeserver, username, password } => {
            cmd_rooms(&homeserver, &username, &password).await
        }
        Commands::Offers { homeserver, username, password, room_id } => {
            cmd_offers(&homeserver, &username, &password, &fix_room_id(&room_id)).await
        }
        Commands::Share { homeserver, username, password, room_id, file_path, description } => {
            cmd_share(&homeserver, &username, &password, &fix_room_id(&room_id), &file_path, description).await
        }
        Commands::Download { homeserver, username, password, room_id, offer_id, save_dir } => {
            cmd_download(&homeserver, &username, &password, &fix_room_id(&room_id), &offer_id, &save_dir).await
        }
        Commands::IrohServe { file_path } => {
            cmd_iroh_serve(&file_path).await
        }
        Commands::IrohGet { ticket, save_path } => {
            cmd_iroh_get(&ticket, &save_path).await
        }
    }
}

async fn cmd_login(homeserver: &str, username: &str, password: &str) -> Result<(), String> {
    let state = Arc::new(AppState::new());
    let (user_id, device_id) = matrix_client::login_headless(homeserver, username, password, &state).await?;
    println!("Logged in as {} (device: {})", user_id, device_id);
    Ok(())
}

async fn cmd_rooms(homeserver: &str, username: &str, password: &str) -> Result<(), String> {
    let state = Arc::new(AppState::new());
    matrix_client::login_headless(homeserver, username, password, &state).await?;

    println!("Syncing...");
    do_initial_sync(&state).await?;

    let rooms = matrix_client::get_rooms(&state).await?;
    if rooms.is_empty() {
        println!("No rooms found.");
    } else {
        println!("{:<50} {:<10} {}", "ROOM ID", "MEMBERS", "NAME");
        println!("{}", "-".repeat(80));
        for room in &rooms {
            println!("{:<50} {:<10} {}", room.room_id, room.member_count, room.name);
        }
    }
    Ok(())
}

async fn cmd_offers(homeserver: &str, username: &str, password: &str, room_id: &str) -> Result<(), String> {
    let state = Arc::new(AppState::new());
    matrix_client::login_headless(homeserver, username, password, &state).await?;

    println!("Syncing...");
    do_initial_sync(&state).await?;

    let offers = matrix_client::get_file_offers(&state, room_id).await?;
    if offers.is_empty() {
        println!("No file offers in this room.");
    } else {
        println!("{:<38} {:<30} {:<12} {:<6} {}", "OFFER ID", "FILENAME", "SIZE", "IROH", "SENDER");
        println!("{}", "-".repeat(110));
        for offer in &offers {
            let has_iroh = if offer.iroh_ticket.is_some() { "yes" } else { "no" };
            println!("{:<38} {:<30} {:<12} {:<6} {}",
                offer.offer_id,
                offer.filename,
                format_bytes(offer.size),
                has_iroh,
                offer.sender_user_id,
            );
        }
    }
    Ok(())
}

async fn cmd_share(
    homeserver: &str,
    username: &str,
    password: &str,
    room_id: &str,
    file_path: &str,
    description: Option<String>,
) -> Result<(), String> {
    let state = Arc::new(AppState::new());
    matrix_client::login_headless(homeserver, username, password, &state).await?;

    // Initialize Iroh P2P (non-fatal)
    match iroh_transfer::init_iroh(&state).await {
        Ok(()) => println!("Iroh P2P initialized"),
        Err(e) => eprintln!("Iroh unavailable (Matrix-only): {}", e),
    }

    println!("Syncing...");
    do_initial_sync(&state).await?;

    let offer_id = matrix_client::offer_file(&state, room_id, file_path, description, None).await?;
    println!("File offered with ID: {}", offer_id);
    println!("Waiting for download requests... (Ctrl+C to stop)");

    let progress = matrix_client::cli_progress();
    matrix_client::start_sync_headless(state, progress).await;

    Ok(())
}

async fn cmd_download(
    homeserver: &str,
    username: &str,
    password: &str,
    room_id: &str,
    offer_id: &str,
    save_dir: &str,
) -> Result<(), String> {
    let state = Arc::new(AppState::new());
    matrix_client::login_headless(homeserver, username, password, &state).await?;

    // Initialize Iroh P2P (non-fatal)
    match iroh_transfer::init_iroh(&state).await {
        Ok(()) => println!("Iroh P2P initialized"),
        Err(e) => eprintln!("Iroh unavailable (Matrix-only): {}", e),
    }

    println!("Syncing...");
    do_initial_sync(&state).await?;

    let offers = matrix_client::get_file_offers(&state, room_id).await?;
    let offer = offers
        .iter()
        .find(|o| o.offer_id == offer_id)
        .ok_or(format!("Offer {} not found in room", offer_id))?;

    println!("Found: {} ({}) from {}", offer.filename, format_bytes(offer.size), offer.sender_user_id);

    let save_path = std::path::Path::new(save_dir).join(&offer.filename);
    let progress = matrix_client::cli_progress();

    // Try Iroh P2P first if a ticket is available
    if let Some(ref ticket) = offer.iroh_ticket {
        println!("Attempting Iroh P2P download...");

        let iroh_result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            iroh_transfer::download_via_iroh(
                &state,
                ticket,
                save_path.to_str().unwrap_or(save_dir),
                &progress,
                offer_id,
                offer.size,
            ),
        )
        .await;

        match iroh_result {
            Ok(Ok(())) => {
                println!("\nIroh P2P download complete: {}", save_path.display());
                iroh_transfer::shutdown_iroh(&state).await;
                return Ok(());
            }
            Ok(Err(e)) => {
                warn!("Iroh download failed: {}", e);
                println!("Iroh failed, falling back to Matrix transfer...");
            }
            Err(_) => {
                warn!("Iroh download timed out");
                println!("Iroh timed out, falling back to Matrix transfer...");
            }
        }
    }

    // Fallback: Matrix to-device chunk transfer
    println!("Using Matrix to-device transfer...");

    transfer::start_receiving(&state, offer, save_dir).await?;

    let our_device_id = {
        let client_guard = state.client.read().await;
        let client = client_guard.as_ref().ok_or("Not logged in")?;
        client.device_id().map(|d| d.to_string()).ok_or("No device ID")?
    };

    {
        let client_guard = state.client.read().await;
        let client = client_guard.as_ref().ok_or("Not logged in")?;

        let room_id_owned = matrix_sdk::ruma::OwnedRoomId::try_from(room_id)
            .map_err(|e| format!("Invalid room ID: {}", e))?;
        let room = client.get_room(&room_id_owned).ok_or("Room not found")?;

        let sender_device = offer.sender_device_id.clone()
            .unwrap_or_else(|| "*".to_string());

        let request_content = FileRequestContent {
            offer_id: offer_id.to_string(),
            target_user: offer.sender_user_id.clone(),
            target_device: sender_device,
            requester_device_id: our_device_id,
        };

        let raw_content = serde_json::to_value(&request_content)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        room.send_raw(EVENT_FILE_REQUEST, raw_content)
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        info!("Sent file request for offer {}", offer_id);
    }

    println!("Request sent. Waiting for file transfer...");

    let state_clone = state.clone();

    let offer_id_owned = offer_id.to_string();
    tokio::spawn(async move {
        matrix_client::start_sync_headless(state_clone, progress).await;
    });

    // Poll for completion
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let transfers = state.active_transfers.lock().await;
        if !transfers.contains_key(&offer_id_owned) {
            break;
        }
    }

    let save_path = std::path::Path::new(save_dir).join(&offer.filename);
    iroh_transfer::shutdown_iroh(&state).await;
    if save_path.exists() {
        println!("Download complete: {}", save_path.display());
        Ok(())
    } else {
        Err("Download failed — hash verification error (see logs)".to_string())
    }
}

async fn do_initial_sync(state: &AppState) -> Result<(), String> {
    let client_guard = state.client.read().await;
    let client = client_guard.as_ref().ok_or("Not logged in")?;

    match client
        .sync_once(matrix_sdk::config::SyncSettings::default())
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            let err_str = format!("{}", e);
            if err_str.contains("401") || err_str.contains("M_UNKNOWN_TOKEN") {
                if let Some(user_id) = client.user_id() {
                    let username = user_id.localpart();
                    let homeserver = client.homeserver().to_string();
                    let safe_username = username.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
                    let safe_host = homeserver.replace("://", "_").replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != '.', "_");
                    let data_dir = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join("matrix-fileshare");
                    let _ = std::fs::remove_file(data_dir.join(format!("session_{}_{}.json", safe_username, safe_host)));
                    let _ = std::fs::remove_dir_all(data_dir.join(format!("{}_{}", safe_username, safe_host)));
                }
                Err("Session expired. Saved session has been cleared — please run the command again to do a fresh login.".to_string())
            } else {
                Err(format!("Initial sync failed: {}", e))
            }
        }
    }
}

async fn cmd_iroh_serve(file_path: &str) -> Result<(), String> {
    let state = Arc::new(AppState::new());
    iroh_transfer::init_iroh(&state).await?;

    let ticket = iroh_transfer::add_blob_for_offer(&state, file_path).await?;
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    println!("File: {} ({})", file_path, format_bytes(metadata.len()));
    println!("Iroh ticket:\n{}", ticket);
    println!("\nWaiting for downloads... (Ctrl+C to stop)");

    // Keep alive until interrupted
    tokio::signal::ctrl_c().await.ok();
    iroh_transfer::shutdown_iroh(&state).await;
    Ok(())
}

async fn cmd_iroh_get(ticket_str: &str, save_path: &str) -> Result<(), String> {
    let state = Arc::new(AppState::new());
    iroh_transfer::init_iroh(&state).await?;

    println!("Downloading via Iroh P2P...");
    let progress = matrix_client::cli_progress();

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        iroh_transfer::download_via_iroh(
            &state,
            ticket_str,
            save_path,
            &progress,
            "test",
            0,
        ),
    )
    .await;

    match result {
        Ok(Ok(())) => {
            println!("Download complete: {}", save_path);
            // Verify file
            if let Ok(metadata) = std::fs::metadata(save_path) {
                println!("File size: {}", format_bytes(metadata.len()));
            }
        }
        Ok(Err(e)) => {
            return Err(format!("Iroh download failed: {}", e));
        }
        Err(_) => {
            return Err("Iroh download timed out after 30s".to_string());
        }
    }

    iroh_transfer::shutdown_iroh(&state).await;
    Ok(())
}

use crate::state::format_bytes;
