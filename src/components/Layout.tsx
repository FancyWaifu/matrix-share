import { type ReactNode, useState } from "react";

interface LayoutProps {
  sidebar: ReactNode;
  main: ReactNode;
  userId: string | null;
  onLogout: () => void;
  isDragging: boolean;
  selectedRoomName?: string;
}

export function Layout({
  sidebar,
  main,
  userId,
  onLogout,
  isDragging,
  selectedRoomName,
}: LayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="h-screen flex flex-col text-white" style={{ background: "#0a0f1e" }}>
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-2.5 bg-white/[0.03] backdrop-blur-xl border-b border-white/[0.06]">
        <div className="flex items-center gap-3">
          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            className="lg:hidden text-slate-400 hover:text-white transition-colors"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
          <div className="flex items-center gap-2">
            <div className="w-6 h-6 rounded-lg bg-cyan-500/15 flex items-center justify-center">
              <svg className="w-3.5 h-3.5 text-cyan-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
                <polyline points="17,8 12,3 7,8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
            </div>
            <h1 className="text-sm font-semibold text-white">FileShare</h1>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <span className="text-xs text-slate-500 hidden sm:inline">
            {userId}
          </span>
          <button
            onClick={onLogout}
            className="text-xs text-slate-500 hover:text-slate-300 transition-colors px-2 py-1 rounded-lg hover:bg-white/[0.05]"
          >
            Sign out
          </button>
        </div>
      </header>

      <div className="flex flex-1 overflow-hidden relative">
        {/* Sidebar */}
        <aside
          className={`
            ${sidebarOpen ? "translate-x-0" : "-translate-x-full"}
            lg:translate-x-0 fixed lg:static inset-y-0 left-0 z-20
            w-64 bg-white/[0.02] backdrop-blur-xl border-r border-white/[0.06]
            transition-transform duration-200 ease-in-out
            overflow-y-auto pt-12 lg:pt-0
          `}
        >
          <div className="p-3 border-b border-white/[0.06]">
            <h2 className="text-[11px] font-semibold text-slate-500 uppercase tracking-wider">
              Rooms
            </h2>
          </div>
          {sidebar}
        </aside>

        {/* Mobile overlay */}
        {sidebarOpen && (
          <div
            className="fixed inset-0 bg-black/50 backdrop-blur-sm z-10 lg:hidden"
            onClick={() => setSidebarOpen(false)}
          />
        )}

        {/* Main content */}
        <main className="flex-1 overflow-y-auto">
          {main}
        </main>

        {/* Drag-and-drop overlay */}
        {isDragging && (
          <div className="absolute inset-0 z-30 flex items-center justify-center bg-black/60 backdrop-blur-sm fade-in">
            <div className="border-2 border-dashed rounded-2xl p-16 text-center drag-active max-w-md">
              <svg
                className="w-16 h-16 mx-auto text-cyan-400 mb-4"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
                <polyline points="17,8 12,3 7,8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
              <p className="text-lg font-medium text-white">Drop to share</p>
              {selectedRoomName && (
                <p className="text-sm text-slate-400 mt-1">
                  in {selectedRoomName}
                </p>
              )}
              {!selectedRoomName && (
                <p className="text-sm text-slate-500 mt-1">
                  Select a room first
                </p>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
