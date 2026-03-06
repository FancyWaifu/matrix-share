import type { RoomSummary } from "../types";

interface RoomListProps {
  rooms: RoomSummary[];
  selectedRoomId: string | null;
  onSelectRoom: (roomId: string) => void;
}

const avatarColors = [
  "bg-cyan-500",
  "bg-purple-500",
  "bg-pink-500",
  "bg-amber-500",
  "bg-emerald-500",
  "bg-blue-500",
  "bg-rose-500",
  "bg-indigo-500",
];

function roomColor(name: string): string {
  let hash = 0;
  for (let i = 0; i < name.length; i++)
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  return avatarColors[Math.abs(hash) % avatarColors.length];
}

export function RoomList({
  rooms,
  selectedRoomId,
  onSelectRoom,
}: RoomListProps) {
  if (rooms.length === 0) {
    return (
      <div className="p-4 text-slate-500 text-sm text-center">
        No rooms joined yet.
        <br />
        <span className="text-slate-600 text-xs">
          Join a room in your Matrix client first.
        </span>
      </div>
    );
  }

  return (
    <div className="p-2 space-y-0.5">
      {rooms.map((room) => {
        const isSelected = selectedRoomId === room.room_id;
        const initial = room.name.charAt(0).toUpperCase();

        return (
          <button
            key={room.room_id}
            onClick={() => onSelectRoom(room.room_id)}
            className={`w-full text-left px-3 py-2.5 rounded-xl flex items-center gap-3 transition-all ${
              isSelected
                ? "bg-cyan-500/10 border border-cyan-500/20"
                : "border border-transparent hover:bg-white/[0.04]"
            }`}
          >
            {/* Room avatar */}
            <div
              className={`w-8 h-8 rounded-lg ${roomColor(room.name)} flex items-center justify-center text-white text-xs font-bold shrink-0`}
            >
              {initial}
            </div>

            <div className="min-w-0 flex-1">
              <div
                className={`text-sm font-medium truncate ${
                  isSelected ? "text-cyan-50" : "text-slate-200"
                }`}
              >
                {room.name}
              </div>
              <div className="text-[11px] text-slate-500">
                {room.member_count} member
                {room.member_count !== 1 ? "s" : ""}
              </div>
            </div>
          </button>
        );
      })}
    </div>
  );
}
