"use client";

import { useState, useEffect } from "react";
import { ServersLoader } from "./components/servers-loader";
import { ServersRail } from "./components/servers-rail";
import { ChatPanel } from "./components/chat-panel";
import { MembersPanel } from "./components/members-panel";
import { ChannelsSidebar } from "./components/channels-sidebar";
import { Menu, Users, X } from "lucide-react";

type MobileView = "sidebar" | "chat" | "members";

export default function ServersPage() {
  const [mobileView, setMobileView] = useState<MobileView>("chat");
  const [isDesktop, setIsDesktop] = useState(false);
  const [openServerSettings, setOpenServerSettings] = useState(false);

  useEffect(() => {
    const mq = window.matchMedia("(min-width: 768px)");
    setIsDesktop(mq.matches);
    const handler = (e: MediaQueryListEvent) => setIsDesktop(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);

  return (
    <ServersLoader>
      {({ refresh }) => (
        <div className="h-full w-full flex overflow-hidden relative">
          {!isDesktop && (
            <div className="fixed top-0 left-0 right-0 z-30 h-12 bg-[var(--card)] border-b flex items-center justify-between px-3">
              <button
                onClick={() => setMobileView(mobileView === "sidebar" ? "chat" : "sidebar")}
                className="p-2 rounded-lg hover:bg-[var(--surface)]"
              >
                {mobileView === "sidebar" ? <X className="h-5 w-5" /> : <Menu className="h-5 w-5" />}
              </button>
              <span className="text-sm font-semibold">EpiTalk</span>
              <button
                onClick={() => setMobileView(mobileView === "members" ? "chat" : "members")}
                className="p-2 rounded-lg hover:bg-[var(--surface)]"
              >
                {mobileView === "members" ? <X className="h-5 w-5" /> : <Users className="h-5 w-5" />}
              </button>
            </div>
          )}

          {isDesktop && (
            <div className="flex h-full">
              <ServersRail
                onRefresh={refresh}
                openSettings={openServerSettings}
                setOpenSettings={setOpenServerSettings}
              />
            </div>
          )}

          {(isDesktop || mobileView === "sidebar") && (
            <div
              className={
                isDesktop
                  ? "relative h-full w-60 lg:w-72 shrink-0 bg-[var(--card)] border-r border-[var(--border)]/20 overflow-hidden"
                  : "fixed inset-0 z-20 pt-12 bg-[var(--card)] overflow-hidden"
              }
            >
              {!isDesktop && (
                <div className="border-b p-2">
                  <ServersRail
                    onRefresh={refresh}
                    openSettings={openServerSettings}
                    setOpenSettings={setOpenServerSettings}
                  />
                </div>
              )}
              <ChannelsSidebar onOpenSettings={() => setOpenServerSettings(true)} />
            </div>
          )}

          {(isDesktop || mobileView === "chat") && (
            <div
              className={`flex-1 min-w-0 h-full bg-[var(--card)] overflow-hidden${
                !isDesktop ? " pt-12" : ""
              }`}
            >
              <ChatPanel />
            </div>
          )}

          {(isDesktop || mobileView === "members") && (
            <div
              className={
                isDesktop
                  ? "relative h-full w-60 lg:w-72 shrink-0 bg-[var(--card)] border-l border-[var(--border)]/20 overflow-hidden"
                  : "fixed inset-0 z-20 pt-12 bg-[var(--card)] overflow-hidden"
              }
            >
              <MembersPanel onRefresh={refresh} />
            </div>
          )}
        </div>
      )}
    </ServersLoader>
  );
}
