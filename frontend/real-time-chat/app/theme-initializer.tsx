"use client";

import { useEffect } from "react";
import { useAuthStore } from "@/store/auth.store";
import { useAppearanceStore } from "@/store/appearance.store";

const FONT_SIZE_MAP = { sm: "14px", base: "16px", lg: "18px", xl: "20px" } as const;

export function ThemeInitializer() {
  const user = useAuthStore((s) => s.user);
  const fontSize = useAppearanceStore((s) => s.fontSize);

  // Theme comes from user profile when logged in, default to "light" otherwise
  const theme = (user?.theme as "light" | "ash" | "dim" | "dark") || "light";

  useEffect(() => {
    const root = document.documentElement;
    root.classList.remove("light", "ash", "dim", "dark");
    if (theme !== "light") {
      root.classList.add(theme);
    }
  }, [theme]);

  useEffect(() => {
    document.documentElement.style.setProperty("--chat-font-size", FONT_SIZE_MAP[fontSize]);
  }, [fontSize]);

  return null;
}
