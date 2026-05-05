"use client";

import { useEffect, useState } from "react";
import { useRequestNotificationPermission } from "@/store/notifications.store";
import { Bell, X } from "lucide-react";

export function NotificationPermissionBanner() {
  const [showBanner, setShowBanner] = useState(false);
  const [isRequesting, setIsRequesting] = useState(false);
  const { requestPermission } = useRequestNotificationPermission();

  useEffect(() => {
    // Check if notifications are supported and permission not yet requested
    if (typeof window !== "undefined" && "Notification" in window) {
      if (Notification.permission === "default") {
        // Wait a bit before showing the banner
        const timer = setTimeout(() => {
          setShowBanner(true);
        }, 2000);
        return () => clearTimeout(timer);
      }
    }
  }, []);

  const handleRequestPermission = async () => {
    setIsRequesting(true);
    try {
      await requestPermission();
      setShowBanner(false);
    } finally {
      setIsRequesting(false);
    }
  };

  if (!showBanner) return null;

  return (
    <div className="fixed top-4 left-4 right-4 bg-blue-50 dark:bg-blue-950 border border-blue-200 dark:border-blue-800 rounded-lg p-4 flex items-center gap-4 z-50 max-w-md">
      <Bell className="h-5 w-5 text-blue-600 dark:text-blue-400 flex-shrink-0" />
      <div className="flex-1 min-w-0">
        <p className="font-medium text-sm text-blue-900 dark:text-blue-100">
          Activer les notifications
        </p>
        <p className="text-xs text-blue-700 dark:text-blue-300 mt-1">
          Recevez des alertes quand vous recevez des messages
        </p>
      </div>
      <div className="flex items-center gap-2 flex-shrink-0">
        <button
          onClick={handleRequestPermission}
          disabled={isRequesting}
          className="px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white text-xs rounded font-medium transition-colors disabled:opacity-50"
        >
          {isRequesting ? "..." : "Oui"}
        </button>
        <button
          onClick={() => setShowBanner(false)}
          className="p-1 hover:bg-blue-100 dark:hover:bg-blue-900 rounded transition-colors"
        >
          <X className="h-4 w-4 text-blue-600 dark:text-blue-400" />
        </button>
      </div>
    </div>
  );
}
