"use client";

import { create } from "zustand";
import { persist } from "zustand/middleware";

export type Theme = "light" | "ash" | "dim" | "dark";
export type FontSize = "sm" | "base" | "lg" | "xl";

interface AppearanceState {
  theme: Theme;
  fontSize: FontSize;
  setTheme: (t: Theme) => void;
  setFontSize: (s: FontSize) => void;
}

export const useAppearanceStore = create<AppearanceState>()(
  persist(
    (set) => ({
      theme: "light",
      fontSize: "base",
      setTheme: (theme) => set({ theme }),
      setFontSize: (fontSize) => set({ fontSize }),
    }),
    { name: "epitalk-appearance" },
  ),
);
