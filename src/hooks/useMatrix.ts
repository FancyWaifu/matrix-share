// ============================================================
// useMatrix — React hook for Matrix client lifecycle (Tauri)
// ============================================================

import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ConnectionStatus } from "../types";

export interface UseMatrixReturn {
  status: ConnectionStatus;
  statusMessage: string | null;
  userId: string | null;
  deviceId: string | null;
  error: string | null;
  connect: (homeserverUrl: string, username: string, password: string) => Promise<void>;
  disconnect: () => Promise<void>;
}

export function useMatrix(): UseMatrixReturn {
  const [status, setStatus] = useState<ConnectionStatus>("disconnected");
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [userId, setUserId] = useState<string | null>(null);
  const [deviceId, setDeviceId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Listen for login status updates from Rust backend
  useEffect(() => {
    const unlisten = listen<string>("login-status", (event) => {
      setStatusMessage(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const connect = useCallback(
    async (homeserverUrl: string, username: string, password: string) => {
      setStatus("connecting");
      setStatusMessage("Connecting...");
      setError(null);

      try {
        const result = await invoke<{ user_id: string; device_id: string }>("login", {
          homeserver: homeserverUrl,
          username,
          password,
        });

        setUserId(result.user_id);
        setDeviceId(result.device_id);
        setStatusMessage(null);
        setStatus("syncing");
      } catch (err: any) {
        setError(typeof err === "string" ? err : err.message || "Login failed");
        setStatusMessage(null);
        setStatus("error");
      }
    },
    []
  );

  const disconnect = useCallback(async () => {
    try {
      await invoke("logout");
    } catch {
      // ignore logout errors
    }
    setUserId(null);
    setDeviceId(null);
    setStatusMessage(null);
    setStatus("disconnected");
    setError(null);
  }, []);

  return {
    status,
    statusMessage,
    userId,
    deviceId,
    error,
    connect,
    disconnect,
  };
}
