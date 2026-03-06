// ============================================================
// Matrix File Share — Type Definitions (Tauri version)
// ============================================================

// --- Transfer State ---

export type TransferDirection = "send" | "receive";
export type TransferStatus =
  | "pending"
  | "sending"
  | "receiving"
  | "complete"
  | "failed";

export interface Transfer {
  offerId: string;
  direction: TransferDirection;
  status: TransferStatus;
  filename: string;
  size: number;
  mimetype: string;
  bytesTransferred: number;
  roomId: string;
  error?: string;
  completedAt?: number;
}

// --- File Offer (as displayed in UI) ---

export interface FileOffer {
  offerId: string;
  roomId: string;
  senderUserId: string;
  senderDeviceId?: string;
  filename: string;
  size: number;
  mimetype: string;
  sha256: string;
  description?: string;
  senderOnline: boolean;
  irohTicket?: string;
}

// --- Room Summary ---

export interface RoomSummary {
  room_id: string;
  name: string;
  member_count: number;
}

// --- Matrix Connection State ---

export type ConnectionStatus = "disconnected" | "connecting" | "syncing" | "error";

// --- Backend types (matching Rust structs) ---

export interface FileOfferData {
  offer_id: string;
  filename: string;
  size: number;
  mimetype: string;
  sha256: string;
  description?: string;
  sender_user_id: string;
  sender_device_id?: string;
  room_id: string;
  iroh_ticket?: string;
}

export interface TransferProgressEvent {
  offer_id: string;
  bytes_transferred: number;
  total_bytes: number;
  status: string;
}

export interface TransferCompleteEvent {
  offer_id: string;
  file_path: string;
}

export interface TransferFailedEvent {
  offer_id: string;
  error: string;
}
