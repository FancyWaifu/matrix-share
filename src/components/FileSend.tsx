import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

interface FileSendProps {
  onSendFile: (filePath: string, description?: string) => Promise<void>;
  disabled?: boolean;
}

export function FileSend({ onSendFile, disabled }: FileSendProps) {
  const [sending, setSending] = useState(false);

  const handleClick = async () => {
    const selected = await open({
      multiple: false,
      title: "Select a file to share",
    });

    if (!selected) return;

    setSending(true);
    try {
      await onSendFile(selected);
    } catch (err) {
      console.error("FileSend error:", err);
    } finally {
      setSending(false);
    }
  };

  return (
    <div className="border border-dashed border-white/[0.1] hover:border-cyan-500/30 rounded-xl p-5 text-center transition-all group">
      <button
        onClick={handleClick}
        disabled={disabled || sending}
        className="px-4 py-2 bg-cyan-500 hover:bg-cyan-400 disabled:bg-white/[0.06] disabled:text-slate-500 text-white rounded-lg text-sm font-medium transition-all cursor-pointer disabled:cursor-not-allowed"
      >
        {sending ? (
          <span className="flex items-center gap-2">
            <svg className="w-4 h-4 spin-slow" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M21 12a9 9 0 11-6.219-8.56" />
            </svg>
            Sharing...
          </span>
        ) : (
          <span className="flex items-center gap-2">
            <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            Share File
          </span>
        )}
      </button>
      <p className="text-slate-600 text-xs mt-2 group-hover:text-slate-500 transition-colors">
        Click to browse or drag files into this window
      </p>
    </div>
  );
}
