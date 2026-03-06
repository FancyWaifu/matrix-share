import type { Transfer } from "../types";

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

interface TransferProgressProps {
  transfer: Transfer;
}

export function TransferProgress({ transfer }: TransferProgressProps) {
  const percent =
    transfer.size > 0
      ? Math.min(100, Math.round((transfer.bytesTransferred / transfer.size) * 100))
      : 0;

  const statusLabel = {
    pending: "Waiting...",
    sending: `Sending ${formatFileSize(transfer.bytesTransferred)} / ${formatFileSize(transfer.size)}`,
    receiving: `Receiving ${formatFileSize(transfer.bytesTransferred)} / ${formatFileSize(transfer.size)}`,
    complete: "Complete",
    failed: "Failed",
  }[transfer.status];

  return (
    <div>
      {/* Progress bar */}
      <div className="w-full h-1.5 bg-gray-800 rounded-full overflow-hidden">
        <div
          className={`h-full transition-all duration-200 rounded-full ${
            transfer.status === "failed" ? "bg-red-500" : "bg-blue-500"
          }`}
          style={{ width: `${percent}%` }}
        />
      </div>

      <div className="flex justify-between items-center mt-1">
        <span className="text-gray-500 text-xs">{statusLabel}</span>
        {(transfer.status === "sending" || transfer.status === "receiving") && (
          <span className="text-gray-500 text-xs">{percent}%</span>
        )}
      </div>
    </div>
  );
}
