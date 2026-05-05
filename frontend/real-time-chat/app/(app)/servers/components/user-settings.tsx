"use client";

import Link from "next/link";
import { useAuthStore } from "@/store/auth.store";

const API_BASE =
  typeof window !== "undefined"
    ? process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001"
    : "http://localhost:3001";

export function UserSettings() {
  const user = useAuthStore((s) => s.user);

  const avatarUrl = user?.avatar_url ? `${API_BASE}${user.avatar_url}` : null;

  return (
    <Link
      href="/settings"
      className={`relative w-12 h-12 rounded-2xl border-2 transition-all duration-300 flex items-center justify-center overflow-hidden ${
        !user
          ? "bg-[var(--surface)] border-[var(--border)] text-[var(--muted-foreground)] pointer-events-none"
          : "border-[#023BFC]/30 shadow-lg hover:shadow-xl hover:scale-105"
      }`}
      title={user ? "Paramètres utilisateur" : "Chargement..."}
    >
      {user ? (
        avatarUrl ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img
            src={avatarUrl}
            alt={user.username}
            className="w-full h-full object-cover"
          />
        ) : (
          <span className="w-full h-full bg-gradient-to-br from-[#023BFC] to-[#3D6AFF] flex items-center justify-center text-white text-sm font-bold uppercase">
            {user.username.slice(0, 2)}
          </span>
        )
      ) : (
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
          />
        </svg>
      )}
    </Link>
  );
}
