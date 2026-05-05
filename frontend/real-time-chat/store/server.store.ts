import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Server } from "@/lib/api/schemas/servers.schema";

type ServerStore = {
  servers: Server[];
  activeServerId: string | null;

  setServers: (servers: Server[]) => void;
  setActiveServer: (serverId: string) => void;
};

export const useServerStore = create<ServerStore>()(
  persist(
    (set) => ({
      servers: [],
      activeServerId: null,

      setServers: (servers) => set({ servers }),
      setActiveServer: (serverId) => set({ activeServerId: serverId }),
    }),
    {
      name: "server-store",
      partialize: (state) => ({ activeServerId: state.activeServerId }),
    },
  ),
);
