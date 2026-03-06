use matrix_sdk::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Progress callback: (event_name, json_payload)
pub type ProgressFn = Arc<dyn Fn(&str, serde_json::Value) + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSummary {
    pub room_id: String,
    pub name: String,
    pub member_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOfferData {
    pub offer_id: String,
    pub filename: String,
    pub size: u64,
    pub mimetype: String,
    pub sha256: String,
    pub description: Option<String>,
    pub sender_user_id: String,
    pub sender_device_id: Option<String>,
    pub room_id: String,
    pub iroh_ticket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    pub offer_id: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub status: String,
}

#[derive(Debug)]
pub struct ActiveTransfer {
    pub offer_id: String,
    pub filename: String,
    pub total_size: u64,
    pub sha256: String,
    pub chunks_received: HashSet<u32>,
    pub total_chunks: u32,
    pub data: Vec<u8>,
    pub save_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PendingOffer {
    pub offer_id: String,
    pub file_path: PathBuf,
    pub room_id: String,
    pub sha256: String,
    pub size: u64,
}

pub struct AppState {
    pub client: RwLock<Option<Client>>,
    pub active_transfers: Mutex<HashMap<String, ActiveTransfer>>,
    pub pending_offers: Mutex<HashMap<String, PendingOffer>>,
    pub sync_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    pub iroh_router: Mutex<Option<iroh::protocol::Router>>,
    pub iroh_store: Mutex<Option<iroh_blobs::store::mem::MemStore>>,
    pub iroh_endpoint: Mutex<Option<iroh::Endpoint>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            client: RwLock::new(None),
            active_transfers: Mutex::new(HashMap::new()),
            pending_offers: Mutex::new(HashMap::new()),
            sync_handle: Mutex::new(None),
            iroh_router: Mutex::new(None),
            iroh_store: Mutex::new(None),
            iroh_endpoint: Mutex::new(None),
        }
    }
}

pub type SharedState = Arc<AppState>;

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
