"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { authApi } from "@/lib/api";
import { useAuthStore } from "@/store/auth.store";
import { terminateSession } from "@/lib/auth/session";
import { NotificationPermissionBanner } from "@/components/notification-permission-banner";
import { NotificationManager } from "@/components/notification-manager";

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const token = useAuthStore((s) => s.token);
  const hasHydrated = useAuthStore((s) => s.hasHydrated);
  const [isChecking, setIsChecking] = useState(true);

  useEffect(() => {
    let cancelled = false;

    const restoreSession = async () => {
      if (!hasHydrated) {
        return;
      }

      if (!token) {
        if (!cancelled) {
          setIsChecking(false);
          router.replace("/login");
        }
        return;
      }

      try {
        await authApi.me();
        if (!cancelled) setIsChecking(false);
      } catch {
        if (!cancelled) {
          terminateSession();
          router.replace("/login");
        }
      }
    };

    void restoreSession();

    return () => {
      cancelled = true;
    };
  }, [hasHydrated, router, token]);

  if (isChecking) {
    return (
      <div className="h-screen w-screen flex items-center justify-center text-sm text-zinc-500">
        Restauration de la session...
      </div>
    );
  }

  return (
    <div className="h-screen w-screen flex">
      <NotificationPermissionBanner />
      <NotificationManager />
      <main className="flex-1">{children}</main>
    </div>
  );
}
