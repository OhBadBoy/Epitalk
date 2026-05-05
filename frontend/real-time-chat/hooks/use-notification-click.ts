import { useEffect } from "react";
import { useDmStore } from "@/store/dm.store";

/**
 * Hook to handle notification clicks and navigate to the corresponding DM
 */
export function useNotificationClick() {
  useEffect(() => {
    const handleNotificationClick = (event: Event) => {
      if (event instanceof CustomEvent && event.detail?.userId) {
        const { userId } = event.detail;
        // Navigate to the DM with this user
        useDmStore.getState().setActivePeer(userId);
      }
    };

    window.addEventListener("notification:dm-click", handleNotificationClick);

    return () => {
      window.removeEventListener("notification:dm-click", handleNotificationClick);
    };
  }, []);
}
