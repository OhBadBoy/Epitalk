import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { User, AuthResponse } from "@/lib/api/schemas/auth.schema";

type AuthState = {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  hasHydrated: boolean;

  // Actions
  setAuth: (response: AuthResponse) => void;
  logout: () => void;
  setUser: (user: User) => void;
  setHasHydrated: (value: boolean) => void;
};

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      token: null,
      isAuthenticated: false,
      hasHydrated: false,

      setAuth: (response: AuthResponse) => {
        // Stocker le token dans localStorage pour fetchClient
        if (typeof window !== "undefined") {
          localStorage.setItem("token", response.token);
        }
        set({
          user: response.user,
          token: response.token,
          isAuthenticated: true,
        });
      },

      logout: () => {
        if (typeof window !== "undefined") {
          localStorage.removeItem("token");
          // Clear persisted store
          localStorage.removeItem("auth-storage");
        }
        set({
          user: null,
          token: null,
          isAuthenticated: false,
        });
      },

      setUser: (user: User) => set({ user }),
      setHasHydrated: (value: boolean) => set({ hasHydrated: value }),
    }),
    {
      name: "auth-storage",
      partialize: (state) => ({
        user: state.user,
        token: state.token,
        isAuthenticated: state.isAuthenticated,
      }),
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);
