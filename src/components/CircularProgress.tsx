import type { TransferStatus } from "../types";

interface CircularProgressProps {
  size?: number;
  strokeWidth?: number;
  percent: number;
  status: TransferStatus;
}

export function CircularProgress({
  size = 40,
  strokeWidth = 3,
  percent,
  status,
}: CircularProgressProps) {
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference - (circumference * Math.min(percent, 100)) / 100;

  const strokeColor =
    status === "failed"
      ? "#f87171"
      : status === "complete"
        ? "#4ade80"
        : "#22d3ee";

  return (
    <div className="relative" style={{ width: size, height: size }}>
      <svg width={size} height={size} className="progress-ring">
        <circle
          className="progress-ring-bg"
          cx={size / 2}
          cy={size / 2}
          r={radius}
          strokeWidth={strokeWidth}
        />
        <circle
          className="progress-ring-fg"
          cx={size / 2}
          cy={size / 2}
          r={radius}
          strokeWidth={strokeWidth}
          stroke={strokeColor}
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          strokeLinecap="round"
        />
      </svg>
      <div className="absolute inset-0 flex items-center justify-center">
        {status === "complete" ? (
          <svg
            className="w-4 h-4 text-emerald-400"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <polyline points="20,6 9,17 4,12" />
          </svg>
        ) : status === "failed" ? (
          <svg
            className="w-4 h-4 text-red-400"
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
        ) : status === "pending" ? (
          <svg
            className="w-3.5 h-3.5 text-slate-500 spin-slow"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <path d="M21 12a9 9 0 11-6.219-8.56" />
          </svg>
        ) : (
          <span className="text-[10px] font-semibold text-slate-200">
            {percent}%
          </span>
        )}
      </div>
    </div>
  );
}
