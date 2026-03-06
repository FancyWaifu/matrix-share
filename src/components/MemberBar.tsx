import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { MemberInfo } from "../types";

const avatarColors = [
  "bg-purple-500",
  "bg-pink-500",
  "bg-amber-500",
  "bg-emerald-500",
  "bg-blue-500",
  "bg-rose-500",
  "bg-indigo-500",
  "bg-cyan-500",
];

function memberColor(userId: string): string {
  let hash = 0;
  for (let i = 0; i < userId.length; i++)
    hash = userId.charCodeAt(i) + ((hash << 5) - hash);
  return avatarColors[Math.abs(hash) % avatarColors.length];
}

function displayName(member: MemberInfo): string {
  if (member.display_name) return member.display_name;
  // Extract localpart from @user:server
  return member.user_id.split(":")[0].slice(1);
}

function initial(member: MemberInfo): string {
  return displayName(member).charAt(0).toUpperCase();
}

interface MemberBarProps {
  members: MemberInfo[];
  currentUserId: string;
  onSendToUser: (userId: string, filePath: string) => void;
}

export function MemberBar({
  members,
  currentUserId,
  onSendToUser,
}: MemberBarProps) {
  const [expanded, setExpanded] = useState(false);
  const [sending, setSending] = useState<string | null>(null);

  const otherMembers = members.filter((m) => m.user_id !== currentUserId);

  if (otherMembers.length === 0) return null;

  const handleMemberClick = async (userId: string) => {
    const selected = await open({
      multiple: false,
      title: `Send file to ${userId.split(":")[0].slice(1)}`,
    });

    if (!selected) return;

    setSending(userId);
    try {
      onSendToUser(userId, selected);
    } finally {
      setSending(null);
    }
  };

  return (
    <div className="px-4 pt-3 pb-1">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-2 text-xs text-slate-400 hover:text-slate-300 transition-colors w-full"
      >
        <svg
          className={`w-3 h-3 transition-transform ${expanded ? "rotate-90" : ""}`}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <polyline points="9,18 15,12 9,6" />
        </svg>
        <span>
          {otherMembers.length} member{otherMembers.length !== 1 ? "s" : ""}
        </span>
        <span className="text-slate-600">— click to send file directly</span>
      </button>

      {expanded && (
        <div className="mt-2 flex flex-wrap gap-1.5">
          {otherMembers.map((member) => (
            <button
              key={member.user_id}
              onClick={() => handleMemberClick(member.user_id)}
              disabled={sending === member.user_id}
              className="flex items-center gap-1.5 px-2.5 py-1.5 bg-white/[0.04] border border-white/[0.08] hover:border-cyan-500/30 hover:bg-cyan-500/5 rounded-lg transition-all disabled:opacity-50"
              title={`Send file to ${member.user_id}`}
            >
              <div
                className={`w-5 h-5 rounded-md ${memberColor(member.user_id)} flex items-center justify-center text-white text-[10px] font-bold`}
              >
                {initial(member)}
              </div>
              <span className="text-xs text-slate-300 max-w-[120px] truncate">
                {displayName(member)}
              </span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
