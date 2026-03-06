// ============================================================
// useTransfer — React hook for file transfer (Tauri)
// ============================================================

import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import type {
  Transfer,
  FileOffer,
  TransferProgressEvent,
  TransferCompleteEvent,
  TransferFailedEvent,
} from "../types";

export function useTransfer(isConnected: boolean) {
  const [transfers, setTransfers] = useState<Map<string, Transfer>>(new Map());

  const updateTransfer = useCallback(
    (offerId: string, updates: Partial<Transfer>) => {
      setTransfers((prev) => {
        const next = new Map(prev);
        const existing = next.get(offerId);
        if (existing) {
          next.set(offerId, { ...existing, ...updates });
        }
        return next;
      });
    },
    []
  );

  // Listen for transfer events from Rust backend
  useEffect(() => {
    if (!isConnected) return;

    const unlistenProgress = listen<TransferProgressEvent>(
      "transfer-progress",
      (event) => {
        const { offer_id, bytes_transferred, total_bytes, status } = event.payload;
        updateTransfer(offer_id, {
          bytesTransferred: bytes_transferred,
          size: total_bytes,
          status: status === "sending" ? "sending" : "receiving",
        });
      }
    );

    const unlistenComplete = listen<TransferCompleteEvent>(
      "transfer-complete",
      (event) => {
        setTransfers((prev) => {
          const next = new Map(prev);
          const existing = next.get(event.payload.offer_id);
          if (existing) {
            next.set(event.payload.offer_id, {
              ...existing,
              status: "complete",
              bytesTransferred: existing.size,
              completedAt: Date.now(),
            });
          }
          return next;
        });
      }
    );

    const unlistenFailed = listen<TransferFailedEvent>(
      "transfer-failed",
      (event) => {
        updateTransfer(event.payload.offer_id, {
          status: "failed",
          error: event.payload.error,
          completedAt: Date.now(),
        });
      }
    );

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
      unlistenFailed.then((fn) => fn());
    };
  }, [isConnected, updateTransfer]);

  // Offer file using a native file path
  const offerFileNative = useCallback(
    async (roomId: string, filePath: string, description?: string) => {
      try {
        const offerId = await invoke<string>("offer_file", {
          roomId,
          filePath,
          description: description || null,
        });

        const filename = filePath.split("/").pop() || filePath.split("\\").pop() || "file";

        const transfer: Transfer = {
          offerId,
          direction: "send",
          status: "pending",
          filename,
          size: 0,
          mimetype: "application/octet-stream",
          bytesTransferred: 0,
          roomId,
        };
        setTransfers((prev) => new Map(prev).set(offerId, transfer));
      } catch (err) {
        console.error("Failed to offer file:", err);
      }
    },
    []
  );

  // Request a file (receiver side)
  const requestFile = useCallback(
    async (offer: FileOffer) => {
      // Ask user where to save
      const savePath = await save({
        defaultPath: offer.filename,
        title: "Save file as...",
      });

      if (!savePath) return; // User cancelled

      const saveDir = savePath.substring(0, savePath.lastIndexOf("/")) || savePath.substring(0, savePath.lastIndexOf("\\"));

      const transfer: Transfer = {
        offerId: offer.offerId,
        direction: "receive",
        status: "receiving",
        filename: offer.filename,
        size: offer.size,
        mimetype: offer.mimetype,
        bytesTransferred: 0,
        roomId: offer.roomId,
      };
      setTransfers((prev) => new Map(prev).set(offer.offerId, transfer));

      try {
        await invoke("request_file", {
          roomId: offer.roomId,
          offerId: offer.offerId,
          saveDir,
          senderUserId: offer.senderUserId,
          senderDeviceId: offer.senderDeviceId || "",
          irohTicket: offer.irohTicket || null,
        });
      } catch (err: any) {
        updateTransfer(offer.offerId, {
          status: "failed",
          error: typeof err === "string" ? err : err.message || "Request failed",
          completedAt: Date.now(),
        });
      }
    },
    [updateTransfer]
  );

  return {
    transfers,
    offerFileNative,
    requestFile,
    getTransfer: (offerId: string) => transfers.get(offerId),
  };
}
