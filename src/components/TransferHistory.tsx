import { useState } from "react";
import type { Transfer } from "../types";

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

interface TransferHistoryProps {
  transfers: Map<string, Transfer>;
  roomId: string;
}

export function TransferHistory({ transfers, roomId }: TransferHistoryProps) {
  const [expanded, setExpanded] = useState(false);

  const history = Array.from(transfers.values())
    .filter(
      (t) =>
        t.roomId === roomId &&
        (t.status === "complete" || t.status === "failed")
    )
    .sort((a, b) => (b.completedAt || 0) - (a.completedAt || 0));

  if (history.length === 0) return null;

  return (
    <div className="fade-in">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-2 text-xs text-slate-500 hover:text-slate-300 transition-colors w-full px-1 py-1.5"
      >
        <svg
          className={`w-3 h-3 transition-transform ${expanded ? "rotate-90" : ""}`}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        >
          <polyline points="9,18 15,12 9,6" />
        </svg>
        <span className="uppercase tracking-wide font-medium">
          Recent Activity
        </span>
        <span className="text-slate-600">({history.length})</span>
      </button>

      {expanded && (
        <div className="space-y-1 mt-1 fade-in">
          {history.slice(0, 10).map((t) => (
            <div
              key={t.offerId}
              className="flex items-center gap-2 text-xs px-1 py-1 rounded-lg hover:bg-white/[0.03] transition-colors"
            >
              {t.status === "complete" ? (
                <svg
                  className="w-3.5 h-3.5 text-emerald-400 shrink-0"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <polyline points="20,6 9,17 4,12" />
                </svg>
              ) : (
                <svg
                  className="w-3.5 h-3.5 text-red-400 shrink-0"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              )}
              <span className="text-slate-300 truncate flex-1">
                {t.filename}
              </span>
              <span className="text-slate-500 shrink-0">
                {t.direction === "send" ? "sent" : "received"}
              </span>
              <span className="text-slate-600 shrink-0">
                {formatFileSize(t.size)}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
