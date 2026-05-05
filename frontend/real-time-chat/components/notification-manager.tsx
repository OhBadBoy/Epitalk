"use client";

import { useEffect, useState } from "react";
import { useNotificationStore } from "@/store/notifications.store";
import { Trash2, Bell } from "lucide-react";

export function NotificationManager() {
  const notifications = useNotificationStore((s) => s.notifications);
  const clearNotifications = useNotificationStore((s) => s.clearNotifications);
  const [showCounter, setShowCounter] = useState(false);

  useEffect(() => {
    // Show counter when there are more than 3 notifications
    setShowCounter(notifications.length > 3);
  }, [notifications.length]);

  if (!showCounter || notifications.length === 0) {
    return null;
  }

  return (
    <div className="fixed bottom-4 right-4 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg p-3 flex items-center gap-2 shadow-lg z-40">
      <div className="flex items-center gap-2">
        <Bell className="h-4 w-4 text-zinc-600 dark:text-zinc-400" />
        <span className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
          {notifications.length} notification{notifications.length > 1 ? "s" : ""}
        </span>
      </div>
      <button
        onClick={() => clearNotifications()}
        className="ml-2 p-1.5 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded transition-colors"
        title="Effacer toutes les notifications"
      >
        <Trash2 className="h-4 w-4 text-red-500 hover:text-red-600" />
      </button>
    </div>
  );
}
