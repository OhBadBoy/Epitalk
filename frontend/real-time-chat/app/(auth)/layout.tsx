"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { authApi } from "@/lib/api";
import { useAuthStore } from "@/store/auth.store";
import { useWebSocketStore } from "@/store/websocket.store";

export default function AuthLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const router = useRouter();
  const token = useAuthStore((s) => s.token);
  const hasHydrated = useAuthStore((s) => s.hasHydrated);
  const disconnect = useWebSocketStore((s) => s.disconnect);
  const isConnected = useWebSocketStore((s) => s.isConnected);
  const [isChecking, setIsChecking] = useState(true);

  useEffect(() => {
    let cancelled = false;

    const checkSession = async () => {
      if (!hasHydrated) {
        return;
      }

      if (!token) {
        if (isConnected) {
          disconnect();
        }
        if (!cancelled) setIsChecking(false);
        return;
      }

      try {
        await authApi.me();
        if (!cancelled) {
          router.replace("/servers");
        }
      } catch {
        if (!cancelled) setIsChecking(false);
      }
    };

    void checkSession();

    return () => {
      cancelled = true;
    };
  }, [disconnect, hasHydrated, isConnected, router, token]);

  useEffect(() => {
    // Ensure WS is closed when entering auth pages
    if (isConnected) {
      disconnect();
    }
  }, [disconnect, isConnected]);

  if (isChecking) {
    return (
      <div className="min-h-screen flex items-center justify-center text-sm text-zinc-500">
        Verification de session...
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center">
      {children}
    </div>
  );
}
