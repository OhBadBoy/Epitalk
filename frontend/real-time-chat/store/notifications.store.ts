import { create } from "zustand";
import { toast } from "sonner";
import { useAuthStore } from "@/store/auth.store";
import React from "react";
import { ServerMessageToast } from "@/components/notifications/server-message-toast";
import { DMMessageToast } from "@/components/notifications/dm-message-toast";

export type NotificationType = "dm" | "mention" | "channel_update" | "server_message" | "info" | "error";

export interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message: string;
  timestamp: Date;
  userId?: string;
  data?: Record<string, any>;
  serverId?: string;
  serverName?: string;
  channelId?: string;
  channelName?: string;
}

type NotificationState = {
  isEnabled: boolean;
  notifications: Notification[];
  
  // Actions
  setEnabled: (enabled: boolean) => void;
  showNotification: (config: {
    type: NotificationType;
    title: string;
    message: string;
    userId?: string;
    data?: Record<string, any>;
    isHistorical?: boolean;
  }) => void;
  clearNotifications: () => void;
  clearOldNotifications: (olderThanMs: number) => void;
};

export const useNotificationStore = create<NotificationState>((set, get) => ({
  isEnabled: true,
  notifications: [],

  setEnabled: (enabled) => set({ isEnabled: enabled }),

  showNotification: (config) => {
    const { type, title, message, userId, data, isHistorical } = config;
    const currentUser = useAuthStore.getState().user;
    
    console.log("📢 [Notif] ===== showNotification START =====");
    console.log("📢 [Notif] Type:", type);
    console.log("📢 [Notif] Title:", title);
    console.log("📢 [Notif] Message:", message);
    console.log("📢 [Notif] userId param:", userId, "| currentUser.id:", currentUser?.id);
    console.log("📢 [Notif] isHistorical:", isHistorical);
    console.log("📢 [Notif] isEnabled:", get().isEnabled);
    
    // For server messages, userId is optional and should not be used to filter
    // Only for DMs should we check if it's the current user
    if (type === "dm") {
      const isCurrentUser = userId && currentUser && userId === currentUser.id;
      
      // Don't show notification for own DMs
      if (isCurrentUser) {
        console.log("📢 [Notif] ❌ SKIPPED - own DM");
        return;
      }
    }
    
    // Don't show notifications for historical messages (loaded from archive)
    if (isHistorical) {
      console.log("📢 [Notif] ❌ SKIPPED - historical message");
      return;
    }
    
    // Check if notifications are enabled
    if (!get().isEnabled) {
      console.log("📢 [Notif] ❌ SKIPPED - notifications disabled");
      return;
    }

    console.log("📢 [Notif] ✅ Passed all filters - proceeding to display");

    // Check browser notification permission
    const shouldShowSystemNotification =
      typeof window !== "undefined" &&
      Notification.permission === "granted";

    console.log("📢 [Notif] System notification permission:", Notification.permission);

    // Show toast notification in-app
    if (type === "server_message") {
      console.log("📢 [Notif] 🟢 Showing SERVER MESSAGE toast (custom component)");
      toast.custom((t: any) =>
        React.createElement(ServerMessageToast, {
          serverName: data?.serverName || "Serveur",
          channelName: data?.channelName || "channel",
          username: title.split(" in ")[0] || "Utilisateur", // Extract username from title
          message: message,
          onDismiss: () => toast.dismiss(t.id),
          onView: () => {
            console.log("📢 [Notif] Server message toast clicked!");
            window.dispatchEvent(
              new CustomEvent("notification:server-message-click", {
                detail: { ...data },
              })
            );
            toast.dismiss(t.id);
          },
        })
      );
    } else if (type === "dm") {
      console.log("📢 [Notif] 🟢 Showing DM toast (custom component)");
      toast.custom((t: any) =>
        React.createElement(DMMessageToast, {
          username: title.replace("Message de ", ""), // Extract username from title
          message: message,
          onDismiss: () => toast.dismiss(t.id),
          onView: () => {
            console.log("📢 [Notif] DM toast action clicked!");
            window.dispatchEvent(
              new CustomEvent("notification:dm-click", {
                detail: { userId, ...data },
              })
            );
            toast.dismiss(t.id);
          },
        })
      );
    } else if (type === "mention") {
      const toastConfig = {
        title,
        description: message,
      };
      console.log("📢 [Notif] 🟢 Showing MENTION toast (sonner success)");
      toast.success(title, toastConfig);
    } else if (type === "error") {
      console.log("📢 [Notif] 🔴 Showing ERROR toast (sonner)");
      toast.error(title);
    } else {
      console.log("📢 [Notif] 🔵 Showing INFO toast (sonner)");
      toast.info(title);
    }

    // Show system notification if available
    if (shouldShowSystemNotification && (type === "dm" || type === "server_message")) {
      try {
        const tag = type === "dm" ? `dm-${userId}` : `server-msg-${data?.channelId}`;
        console.log("📢 [Notif] 📲 Showing system notification with tag:", tag);
        new Notification(title, {
          body: message,
          icon: "/favicon.ico",
          tag,
          badge: "/favicon.ico",
        });
      } catch (error) {
        console.error("📢 [Notif] ❌ Failed to show system notification:", error);
      }
    }

    // Store notification in state
    const notification: Notification = {
      id: `${Date.now()}-${Math.random()}`,
      type,
      title,
      message,
      timestamp: new Date(),
      userId,
      data,
      serverId: data?.serverId,
      serverName: data?.serverName,
      channelId: data?.channelId,
      channelName: data?.channelName,
    };

    console.log("📢 [Notif] ===== END (SUCCESS) =====");

    set((state) => ({
      notifications: [notification, ...state.notifications].slice(0, 100), // Keep last 100
    }));
  },

  clearNotifications: () => set({ notifications: [] }),

  clearOldNotifications: (olderThanMs) => {
    const cutoff = Date.now() - olderThanMs;
    set((state) => ({
      notifications: state.notifications.filter(
        (n) => n.timestamp.getTime() > cutoff
      ),
    }));
  },
}));

/**
 * Hook to request browser notification permission
 */
export function useRequestNotificationPermission() {
  const requestPermission = async () => {
    if (typeof window === "undefined") return false;
    if (!("Notification" in window)) return false;

    if (Notification.permission === "granted") {
      return true;
    }

    if (Notification.permission !== "denied") {
      try {
        const permission = await Notification.requestPermission();
        return permission === "granted";
      } catch (error) {
        console.error("Failed to request notification permission:", error);
        return false;
      }
    }

    return false;
  };

  return { requestPermission };
}
