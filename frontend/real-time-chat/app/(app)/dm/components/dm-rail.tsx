"use client";

import Link from "next/link";
import { MessageSquare } from "lucide-react";
import { UserSettings } from "../../servers/components/user-settings";

export function DmRail() {
  return (
    <aside className="w-[88px] my-4 ml-3 rounded-2xl flex flex-col items-center gap-4 py-6 bg-gradient-to-b from-[var(--surface)] to-[var(--card)] border border-[var(--border)] shadow-lg">
      {/* Logo B — active state (we're on DM page) */}
      <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-[#023BFC] to-[#3D6AFF] flex items-center justify-center shadow-lg ring-2 ring-[#023BFC]/40 ring-offset-2">
        <MessageSquare className="w-6 h-6 text-white" />
      </div>

      <div className="w-10 h-px bg-gradient-to-r from-transparent via-[var(--border)] to-transparent" />

      {/* Back to servers */}
      <Link
        href="/servers"
        className="w-12 h-12 server-icon bg-[var(--card)] hover:bg-[var(--accent)] border border-[var(--border)] hover:border-[#023BFC]/50 text-[var(--muted-foreground)] hover:text-[#023BFC] flex items-center justify-center transition-all duration-300"
        title="Retour aux serveurs"
      >
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-4 0h4" />
        </svg>
      </Link>

      <div className="flex-1" />

      {/* User Settings */}
      <div className="mt-auto">
        <UserSettings />
      </div>
    </aside>
  );
}
