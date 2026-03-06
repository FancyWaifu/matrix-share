import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMatrix } from "./hooks/useMatrix";
import { useRooms, useFileOffers } from "./hooks/useRooms";
import { useTransfer } from "./hooks/useTransfer";
import { Login } from "./components/Login";
import { Layout } from "./components/Layout";
import { RoomList } from "./components/RoomList";
import { Room } from "./components/Room";

export default function App() {
  const { status, statusMessage, userId, error, connect, disconnect } = useMatrix();
  const isConnected = status === "syncing";
  const rooms = useRooms(isConnected);
  const [selectedRoomId, setSelectedRoomId] = useState<string | null>(null);
  const offers = useFileOffers(isConnected, selectedRoomId);
  const { transfers, offerFileNative, requestFile } = useTransfer(isConnected);
  const [isDragging, setIsDragging] = useState(false);

  // Drag-and-drop file handling via Tauri native events
  useEffect(() => {
    if (!isConnected) return;

    // Prevent browser default file opening
    const prevent = (e: Event) => e.preventDefault();
    document.addEventListener("dragover", prevent);
    document.addEventListener("drop", prevent);

    const unlisten = getCurrentWindow().onDragDropEvent((event) => {
      const { type } = event.payload;
      if (type === "over" || type === "enter" || (type as string) === "hover") {
        setIsDragging(true);
      } else if (type === "drop") {
        setIsDragging(false);
        if (selectedRoomId && "paths" in event.payload) {
          const paths = event.payload.paths as string[];
          for (const path of paths) {
            offerFileNative(selectedRoomId, path);
          }
        }
      } else {
        setIsDragging(false);
      }
    });

    return () => {
      document.removeEventListener("dragover", prevent);
      document.removeEventListener("drop", prevent);
      unlisten.then((fn) => fn());
    };
  }, [isConnected, selectedRoomId, offerFileNative]);

  // Not logged in
  if (!isConnected && status !== "connecting") {
    return (
      <Login
        onLogin={connect}
        status={status}
        statusMessage={statusMessage}
        error={error}
      />
    );
  }

  // Loading
  if (status === "connecting") {
    return (
      <div
        className="min-h-screen flex flex-col items-center justify-center gap-3"
        style={{ background: "#0a0f1e" }}
      >
        <svg
          className="w-8 h-8 text-cyan-400 spin-slow"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        >
          <path d="M21 12a9 9 0 11-6.219-8.56" />
        </svg>
        <div className="text-slate-400 text-sm">
          {statusMessage || "Connecting..."}
        </div>
      </div>
    );
  }

  const selectedRoom = rooms.find((r) => r.room_id === selectedRoomId);

  return (
    <Layout
      userId={userId}
      onLogout={disconnect}
      isDragging={isDragging}
      selectedRoomName={selectedRoom?.name}
      sidebar={
        <RoomList
          rooms={rooms}
          selectedRoomId={selectedRoomId}
          onSelectRoom={setSelectedRoomId}
        />
      }
      main={
        selectedRoomId && selectedRoom ? (
          <Room
            roomName={selectedRoom.name}
            offers={offers}
            transfers={transfers}
            currentUserId={userId || ""}
            roomId={selectedRoomId}
            onOfferFile={offerFileNative}
            onRequestFile={requestFile}
          />
        ) : (
          <div className="flex flex-col items-center justify-center h-full gap-3">
            <div className="w-16 h-16 rounded-2xl bg-white/[0.04] border border-white/[0.06] flex items-center justify-center">
              <svg
                className="w-7 h-7 text-slate-600"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
                <polyline points="17,8 12,3 7,8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
            </div>
            <p className="text-slate-500 text-sm">Select a room to start sharing</p>
          </div>
        )
      }
    />
  );
}
