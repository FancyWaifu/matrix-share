// ============================================================
// useRooms — React hook for room list and file offers (Tauri)
// ============================================================

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { RoomSummary, FileOfferData, FileOffer, MemberInfo } from "../types";

export type { RoomSummary };

export function useRooms(isConnected: boolean) {
  const [rooms, setRooms] = useState<RoomSummary[]>([]);

  useEffect(() => {
    if (!isConnected) {
      setRooms([]);
      return;
    }

    // Fetch rooms initially
    invoke<RoomSummary[]>("get_rooms")
      .then(setRooms)
      .catch(console.error);

    // Listen for updates from Rust sync loop
    const unlisten = listen<RoomSummary[]>("rooms-updated", (event) => {
      setRooms(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [isConnected]);

  return rooms;
}

export function useFileOffers(isConnected: boolean, roomId: string | null) {
  const [offers, setOffers] = useState<FileOffer[]>([]);

  const fetchOffers = useCallback(async () => {
    if (!isConnected || !roomId) {
      setOffers([]);
      return;
    }

    try {
      const data = await invoke<FileOfferData[]>("get_file_offers", { roomId });
      setOffers(
        data.map((d) => ({
          offerId: d.offer_id,
          roomId: d.room_id,
          senderUserId: d.sender_user_id,
          senderDeviceId: d.sender_device_id || undefined,
          filename: d.filename,
          size: d.size,
          mimetype: d.mimetype,
          sha256: d.sha256,
          description: d.description || undefined,
          senderOnline: true,
          irohTicket: d.iroh_ticket || undefined,
          targetUser: d.target_user || undefined,
        }))
      );
    } catch (err) {
      console.error("Failed to fetch offers:", err);
    }
  }, [isConnected, roomId]);

  useEffect(() => {
    fetchOffers();

    if (!isConnected) return;

    // Refresh when a new file-offer event comes in
    const unlisten = listen<FileOfferData>("file-offer", () => {
      fetchOffers();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [fetchOffers, isConnected]);

  return offers;
}

export function useRoomMembers(isConnected: boolean, roomId: string | null) {
  const [members, setMembers] = useState<MemberInfo[]>([]);

  useEffect(() => {
    if (!isConnected || !roomId) {
      setMembers([]);
      return;
    }

    invoke<MemberInfo[]>("get_room_members", { roomId })
      .then(setMembers)
      .catch(console.error);
  }, [isConnected, roomId]);

  return members;
}
