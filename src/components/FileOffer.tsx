import type { FileOffer as FileOfferType, Transfer } from "../types";
import { FileIcon } from "./FileIcon";
import { CircularProgress } from "./CircularProgress";

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

interface FileOfferProps {
  offer: FileOfferType;
  transfer?: Transfer;
  isOwnOffer: boolean;
  onRequest: (offer: FileOfferType) => void;
}

export function FileOfferCard({
  offer,
  transfer,
  isOwnOffer,
  onRequest,
}: FileOfferProps) {
  const hasActiveTransfer =
    transfer &&
    transfer.status !== "pending" &&
    transfer.status !== "complete" &&
    transfer.status !== "failed";

  const isComplete = transfer?.status === "complete";
  const isFailed = transfer?.status === "failed";
  const isPending = transfer?.status === "pending";

  const percent =
    transfer && transfer.size > 0
      ? Math.min(100, Math.round((transfer.bytesTransferred / transfer.size) * 100))
      : 0;

  const showProgress = hasActiveTransfer || isComplete || isFailed || isPending;

  return (
    <div
      className={`bg-white/[0.04] backdrop-blur-lg border rounded-xl p-4 transition-all fade-in ${
        hasActiveTransfer
          ? "border-cyan-500/20 glow-active"
          : isComplete
            ? "border-emerald-500/20"
            : isFailed
              ? "border-red-500/20"
              : "border-white/[0.06] hover:border-white/[0.1]"
      }`}
    >
      <div className="flex items-start gap-3">
        {/* Icon area — shows progress ring during transfer, file icon otherwise */}
        <div className="flex-shrink-0">
          {showProgress && transfer ? (
            <CircularProgress
              percent={percent}
              status={transfer.status}
              size={40}
              strokeWidth={3}
            />
          ) : (
            <div className="w-10 h-10 rounded-xl bg-white/[0.06] flex items-center justify-center">
              <FileIcon mimetype={offer.mimetype} className="w-5 h-5" />
            </div>
          )}
        </div>

        <div className="flex-1 min-w-0">
          {/* Filename */}
          <div className="text-white text-sm font-medium truncate">
            {offer.filename}
          </div>

          {/* Meta */}
          <div className="text-slate-500 text-xs mt-0.5 flex items-center gap-2">
            <span>{formatFileSize(offer.size)}</span>
            <span className="text-slate-600">from</span>
            <span className="text-slate-400">
              {offer.senderUserId.split(":")[0].slice(1)}
            </span>
          </div>

          {offer.description && (
            <div className="text-slate-400 text-xs mt-1.5 leading-relaxed">
              {offer.description}
            </div>
          )}

          {/* Active transfer status */}
          {hasActiveTransfer && transfer && (
            <div className="text-cyan-400/80 text-xs mt-2">
              {transfer.status === "sending" ? "Sending" : "Receiving"}{" "}
              {formatFileSize(transfer.bytesTransferred)} /{" "}
              {formatFileSize(transfer.size)}
            </div>
          )}

          {isComplete && (
            <div className="text-emerald-400 text-xs mt-2 flex items-center gap-1">
              <svg className="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                <polyline points="20,6 9,17 4,12" />
              </svg>
              Transfer complete
            </div>
          )}

          {isFailed && (
            <div className="text-red-400 text-xs mt-2">
              Failed: {transfer?.error || "Unknown error"}
            </div>
          )}
        </div>

        {/* Action button */}
        {!isOwnOffer && !hasActiveTransfer && !isComplete && (
          <button
            onClick={() => onRequest(offer)}
            className="flex-shrink-0 px-3.5 py-1.5 bg-cyan-500 hover:bg-cyan-400 text-white text-xs rounded-lg font-medium transition-all"
          >
            Download
          </button>
        )}

        {isOwnOffer && !hasActiveTransfer && !isComplete && (
          <div className="flex-shrink-0 text-xs text-slate-500 py-1.5 px-2 bg-white/[0.04] rounded-lg">
            Shared
          </div>
        )}
      </div>
    </div>
  );
}
