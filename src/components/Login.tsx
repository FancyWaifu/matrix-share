import { useState } from "react";

interface LoginProps {
  onLogin: (homeserverUrl: string, username: string, password: string) => Promise<void>;
  status: string;
  statusMessage?: string | null;
  error: string | null;
}

export function Login({ onLogin, status, statusMessage, error }: LoginProps) {
  const [homeserver, setHomeserver] = useState("https://matrix.whiskeyden.xyz");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onLogin(homeserver, username, password);
  };

  const isLoading = status === "connecting";

  return (
    <div className="min-h-screen flex items-center justify-center p-4" style={{ background: "#0a0f1e" }}>
      <div className="w-full max-w-sm">
        {/* Logo / Title */}
        <div className="text-center mb-8">
          <div className="w-14 h-14 mx-auto mb-4 rounded-2xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center">
            <svg className="w-7 h-7 text-cyan-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
              <polyline points="17,8 12,3 7,8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
          </div>
          <h1 className="text-xl font-semibold text-white">Matrix FileShare</h1>
          <p className="text-slate-500 mt-1 text-sm">
            P2P file sharing over Matrix
          </p>
        </div>

        {/* Glass card */}
        <form
          onSubmit={handleSubmit}
          className="bg-white/[0.04] backdrop-blur-xl border border-white/[0.08] rounded-2xl p-6 space-y-4"
        >
          <div>
            <label className="block text-xs font-medium text-slate-400 mb-1.5">
              Homeserver
            </label>
            <input
              type="url"
              value={homeserver}
              onChange={(e) => setHomeserver(e.target.value)}
              className="w-full px-3 py-2.5 bg-white/[0.05] border border-white/[0.1] rounded-xl text-white text-sm placeholder-slate-600 focus:outline-none focus:border-cyan-500/50 focus:ring-1 focus:ring-cyan-500/20 transition-all"
              placeholder="https://matrix.example.com"
              required
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-400 mb-1.5">
              Username
            </label>
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              className="w-full px-3 py-2.5 bg-white/[0.05] border border-white/[0.1] rounded-xl text-white text-sm placeholder-slate-600 focus:outline-none focus:border-cyan-500/50 focus:ring-1 focus:ring-cyan-500/20 transition-all"
              placeholder="username"
              required
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-400 mb-1.5">
              Password
            </label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full px-3 py-2.5 bg-white/[0.05] border border-white/[0.1] rounded-xl text-white text-sm placeholder-slate-600 focus:outline-none focus:border-cyan-500/50 focus:ring-1 focus:ring-cyan-500/20 transition-all"
              required
            />
          </div>

          {error && (
            <div className="text-red-400 text-xs bg-red-500/10 border border-red-500/20 rounded-xl px-3 py-2.5">
              {error}
            </div>
          )}

          <button
            type="submit"
            disabled={isLoading}
            className="w-full py-2.5 bg-cyan-500 hover:bg-cyan-400 disabled:bg-white/[0.06] disabled:text-slate-500 text-white rounded-xl text-sm font-medium transition-all cursor-pointer disabled:cursor-not-allowed"
          >
            {isLoading ? statusMessage || "Connecting..." : "Sign In"}
          </button>
        </form>
      </div>
    </div>
  );
}
