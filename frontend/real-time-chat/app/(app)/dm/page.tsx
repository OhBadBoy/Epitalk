"use client";

import { useState, useEffect } from "react";
import { DmRail } from "./components/dm-rail";
import { DmSidebar } from "./components/dm-sidebar";
import { DmChatPanel } from "./components/dm-chat-panel";
import { useDmStore } from "@/store/dm.store";

type MobileView = "sidebar" | "chat";

export default function DmPage() {
  const activePeerId = useDmStore((s) => s.activePeerId);
  const [mobileView, setMobileView] = useState<MobileView>("sidebar");
  const [isDesktop, setIsDesktop] = useState(false);

  useEffect(() => {
    const mq = window.matchMedia("(min-width: 768px)");
    setIsDesktop(mq.matches);
    const handler = (e: MediaQueryListEvent) => setIsDesktop(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);

  useEffect(() => {
    if (!isDesktop) {
      setMobileView(activePeerId ? "chat" : "sidebar");
    }
  }, [activePeerId, isDesktop]);

  return (
    <div className="h-full w-full flex overflow-hidden relative">
      {isDesktop && (
        <div className="flex h-full">
          <DmRail />
        </div>
      )}

      {(isDesktop || mobileView === "sidebar") && (
        <div
          className={
            isDesktop
              ? "shrink-0 h-full overflow-hidden w-60 lg:w-72"
              : "flex flex-col w-full h-full overflow-hidden"
          }
        >
          <DmSidebar />
        </div>
      )}

      {(isDesktop || mobileView === "chat") && (
        <div className="flex-1 min-w-0 h-full overflow-hidden">
          <DmChatPanel />
        </div>
      )}
    </div>
  );
}
