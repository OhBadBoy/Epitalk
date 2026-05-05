import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Channel } from "@/lib/api/schemas/channels.schema";

type ChannelState = {
  channels: Channel[];
  activeChannelId: string | null;
  setChannels: (channels: Channel[]) => void;
  setActiveChannel: (id: string | null) => void;
  reset: () => void;
};

export const useChannelStore = create<ChannelState>()(
  persist(
    (set) => ({
      channels: [],
      activeChannelId: null,
      setChannels: (channels) => set({ channels }),
      setActiveChannel: (id) => set({ activeChannelId: id }),
      reset: () => set({ channels: [], activeChannelId: null }),
    }),
    {
      name: "channel-store",
      partialize: (state) => ({ activeChannelId: state.activeChannelId }),
    },
  ),
);
