use serde::{Deserialize, Serialize};

// Custom Matrix event type strings
pub const EVENT_FILE_OFFER: &str = "com.fileshare.offer";
pub const EVENT_FILE_REQUEST: &str = "com.fileshare.request";
pub const EVENT_FILE_CHUNK: &str = "com.fileshare.chunk";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOfferContent {
    pub offer_id: String,
    pub filename: String,
    pub size: u64,
    pub mimetype: String,
    pub sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iroh_ticket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRequestContent {
    pub offer_id: String,
    pub target_user: String,
    pub target_device: String,
    pub requester_device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunkContent {
    pub offer_id: String,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub data: String, // base64-encoded
}
