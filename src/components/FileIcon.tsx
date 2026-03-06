import type { ReactNode } from "react";

interface FileIconProps {
  mimetype: string;
  className?: string;
}

function getIconData(mimetype: string): { color: string; icon: ReactNode } {
  if (mimetype.startsWith("image/")) {
    return {
      color: "text-pink-400",
      icon: (
        <>
          <rect x="3" y="3" width="18" height="18" rx="2" />
          <circle cx="8.5" cy="8.5" r="1.5" />
          <path d="M21 15l-5-5L5 21" />
        </>
      ),
    };
  }
  if (mimetype.startsWith("video/")) {
    return {
      color: "text-purple-400",
      icon: (
        <>
          <rect x="2" y="4" width="20" height="16" rx="2" />
          <polygon points="10,8 16,12 10,16" fill="currentColor" stroke="none" />
        </>
      ),
    };
  }
  if (mimetype.startsWith("audio/")) {
    return {
      color: "text-amber-400",
      icon: (
        <>
          <path d="M9 18V5l12-2v13" />
          <circle cx="6" cy="18" r="3" />
          <circle cx="18" cy="16" r="3" />
        </>
      ),
    };
  }
  if (mimetype.includes("pdf")) {
    return {
      color: "text-red-400",
      icon: (
        <>
          <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
          <polyline points="14,2 14,8 20,8" />
          <line x1="8" y1="13" x2="16" y2="13" />
          <line x1="8" y1="17" x2="13" y2="17" />
        </>
      ),
    };
  }
  if (
    mimetype.includes("zip") ||
    mimetype.includes("tar") ||
    mimetype.includes("gz") ||
    mimetype.includes("rar") ||
    mimetype.includes("7z")
  ) {
    return {
      color: "text-yellow-400",
      icon: (
        <>
          <path d="M21 8v13H3V8" />
          <rect x="1" y="3" width="22" height="5" rx="1" />
          <line x1="10" y1="12" x2="14" y2="12" />
        </>
      ),
    };
  }
  // Generic file
  return {
    color: "text-slate-400",
    icon: (
      <>
        <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
        <polyline points="14,2 14,8 20,8" />
      </>
    ),
  };
}

export function FileIcon({ mimetype, className }: FileIconProps) {
  const { color, icon } = getIconData(mimetype);

  return (
    <svg
      className={`${color} ${className || "w-5 h-5"}`}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {icon}
    </svg>
  );
}
