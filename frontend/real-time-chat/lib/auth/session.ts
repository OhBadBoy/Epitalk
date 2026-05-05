import { useAuthStore } from "@/store/auth.store";
import { useWebSocketStore } from "@/store/websocket.store";

export function terminateSession(): void {
  useWebSocketStore.getState().disconnect();
  useAuthStore.getState().logout();
}
