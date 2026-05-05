import { create } from "zustand";
import type { Member } from "@/lib/api/schemas/servers.schema";

type MemberStore = {
  members: Member[];
  loading: boolean;

  setMembers: (members: Member[]) => void;
  setLoading: (loading: boolean) => void;
  clearMembers: () => void;
};

export const useMemberStore = create<MemberStore>((set) => ({
  members: [],
  loading: false,

  setMembers: (members) => set({ members, loading: false }),
  setLoading: (loading) => set({ loading }),
  clearMembers: () => set({ members: [], loading: false }),
}));
