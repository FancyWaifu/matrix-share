# Matrix Share

P2P file sharing over [Matrix](https://matrix.org), built with [Tauri v2](https://tauri.app). Uses [Iroh](https://iroh.computer) for direct peer-to-peer transfers with automatic Matrix to-device fallback.

## How It Works

1. **Sender** offers a file to a Matrix room (E2EE encrypted event)
2. **Receiver** sees the offer and requests download
3. Transfer happens via **Iroh P2P** (direct, fast) or falls back to **Matrix to-device messages** (chunked, works through NAT)

All signaling goes through Matrix with end-to-end encryption. File data travels peer-to-peer when possible.

## Features

- Desktop GUI with drag-and-drop file sharing
- Full CLI for headless/scripted use
- Iroh P2P with QUIC and NAT traversal
- Automatic fallback to Matrix chunked transfer
- E2EE — file offers are encrypted room events
- SHA-256 verification on all transfers
- Standalone Iroh mode (no Matrix needed)
- Session persistence across restarts

## Screenshots

*GUI: Room view with file offers and transfer progress*

## Prerequisites

- [Node.js](https://nodejs.org) (v18+)
- [Rust](https://rustup.rs) (2021 edition)
- Tauri v2 system dependencies ([see docs](https://v2.tauri.app/start/prerequisites/))
- Access to a Matrix homeserver

## Build & Run

```bash
# Install frontend dependencies
npm install

# Development (GUI with hot reload)
npm run tauri dev

# Production build
npm run tauri build

# CLI only (no GUI needed)
cd src-tauri
cargo build
./target/debug/matrix-fileshare --help
```

## GUI Usage

### 1. Launch

```bash
npm run tauri dev        # development
# or
npm run tauri build      # then run the built app from src-tauri/target/release/
```

### 2. Sign In

Enter your Matrix homeserver URL (e.g. `https://matrix.org`), username, and password. The app will log in, initialize Iroh P2P, and start syncing.

### 3. Select a Room

Your joined rooms appear in the left sidebar. Click one to open it. The main panel shows all file offers shared in that room.

### 4. Share a File

Two ways:
- **Click "Share File"** at the bottom of the room view to open a file picker
- **Drag and drop** a file anywhere into the window

The file is offered to the room as an encrypted event. Your app stays available to serve the file to anyone who requests it.

### 5. Download a File

Click the **download button** on any file offer from another user. You'll be prompted to choose where to save it. The transfer starts automatically — Iroh P2P first, Matrix fallback if needed.

### 6. Transfer Progress

Active transfers show a circular progress indicator with bytes transferred. Completed transfers turn green, failed ones turn red with an error message.

## CLI Usage

```bash
# Login (creates persistent session)
matrix-fileshare login <homeserver> <username> <password>

# List your rooms
matrix-fileshare rooms <homeserver> <username> <password>

# List file offers in a room
matrix-fileshare offers <homeserver> <username> <password> <room_id>

# Share a file (stays running to serve requests)
matrix-fileshare share <homeserver> <username> <password> <room_id> <file_path> [-d "description"]

# Download a file by offer ID
matrix-fileshare download <homeserver> <username> <password> <room_id> <offer_id> <save_dir>

# Direct P2P (no Matrix needed)
matrix-fileshare iroh-serve <file_path>    # prints ticket
matrix-fileshare iroh-get <ticket> <save_path>
```

## Architecture

```
┌─────────────────────────────────┐
│         React Frontend          │
│  (Tailwind, React Router, Vite) │
└──────────────┬──────────────────┘
               │ Tauri invoke / events
┌──────────────┴──────────────────┐
│          Rust Backend           │
│                                 │
│  matrix-sdk ──── E2EE signaling │
│  iroh ────────── P2P transfer   │
│  transfer.rs ─── Matrix chunks  │
└─────────────────────────────────┘
```

**Transfer flow:**

1. Sender creates an Iroh blob ticket and posts a `com.fileshare.offer` event to the room
2. Receiver attempts Iroh P2P download (10s timeout)
3. If Iroh fails, falls back to `com.fileshare.request` + `com.fileshare.chunk` via Matrix to-device messages (48KB chunks, 4 concurrent)
4. SHA-256 hash verified on completion

**Key dependencies:**

| Crate | Purpose |
|-------|---------|
| `matrix-sdk 0.10` | Matrix client with E2EE + SQLite store |
| `iroh 0.96` / `iroh-blobs 0.98` | P2P file transfer over QUIC |
| `tauri 2` | Desktop app shell |
| `clap 4` | CLI argument parsing |
