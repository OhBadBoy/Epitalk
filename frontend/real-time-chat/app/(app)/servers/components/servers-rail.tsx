"use client";

import { useMemo, useState } from "react";
import Link from "next/link";
import { MessageSquare } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useServerStore } from "@/store/server.store";
import { useChannelStore } from "@/store/channel.store";
import { useAuthStore } from "@/store/auth.store";
import { serversApi } from "@/lib/api";
import { getErrorMessage } from "@/lib/api/errors";
import { UserSettings } from "./user-settings";
import { CreateServerModal } from "@/components/forms/create-server-modal";
import { useLanguage } from "@/components/language-provider";

function initials(name: string) {
  const cleaned = name.trim();
  if (!cleaned) return "NS";
  const parts = cleaned.split(/\s+/).filter(Boolean);
  const first = parts[0]?.[0] ?? "N";
  const second = parts[1]?.[0] ?? parts[0]?.[1] ?? "S";
  return (first + second).toUpperCase();
}

function extractInviteCode(value: string): string | null {
  const v = value.trim();
  if (!v) return null;

  const match = v.match(/\/invite\/([^/?#\s]+)/i);
  if (match?.[1]) return match[1].trim();

  const codeOnly = v.match(/^([a-z0-9_-]{4,64})$/i);
  if (codeOnly?.[1]) return codeOnly[1].trim();

  return null;
}

type Status = { type: "success" | "error" | "info"; text: string } | null;

export function ServersRail({ onRefresh, openSettings, setOpenSettings }: { onRefresh: () => Promise<void>; openSettings: boolean; setOpenSettings: (v: boolean) => void }) {
  const servers = useServerStore((s) =>
    Array.isArray(s.servers) ? s.servers : [],
  );
  const activeServerId = useServerStore((s) => s.activeServerId);
  const setActiveServer = useServerStore((s) => s.setActiveServer);
  const resetChannels = useChannelStore((s) => s.reset);
  const currentUser = useAuthStore((s) => s.user);
  const { language } = useLanguage();
  const isEnglish = language === "en";

  const activeServer = useMemo(
    () => servers.find((s) => s.id === activeServerId) ?? null,
    [servers, activeServerId]
  );

  const isOwner = !!activeServer && !!currentUser && activeServer.owner_id === currentUser.id;
  const [openCreateServer, setOpenCreateServer] = useState(false);
  const [inviteInput, setInviteInput] = useState("");
  const [inviteLink, setInviteLink] = useState<string | null>(null);
  const [loadingJoin, setLoadingJoin] = useState(false);
  const [loadingInvite, setLoadingInvite] = useState(false);
  const [loadingDanger, setLoadingDanger] = useState(false);
  const [status, setStatus] = useState<Status>(null);

  const setOk = (text: string) => setStatus({ type: "success", text });
  const setErr = (text: string) => setStatus({ type: "error", text });
  const setInfo = (text: string) => setStatus({ type: "info", text });

  const onCreateServer = () => {
    setOpenCreateServer(true);
  };

  const onJoin = async () => {
    const code = extractInviteCode(inviteInput);
    if (!code) {
      setErr(
        isEnglish
          ? "Paste a /invite/<code> link or a valid code."
          : "Colle un lien /invite/<code> ou un code valide."
      );
      return;
    }

    setLoadingJoin(true);
    setStatus(null);

    try {
      await serversApi.joinByInvite(code);
      setInviteInput("");
      await onRefresh();
      setOk(isEnglish ? "Server joined." : "Serveur rejoint.");
    } catch (err) {
      setErr(getErrorMessage(err));
    } finally {
      setLoadingJoin(false);
    }
  };

  const onInvite = async () => {
    if (!activeServerId) {
      setErr(isEnglish ? "Select a server." : "Selectionne un serveur.");
      return;
    }
    if (!isOwner) {
      setErr(
        isEnglish
          ? "Only the owner can generate an invite."
          : "Seul le createur peut generer une invitation."
      );
      return;
    }

    setLoadingInvite(true);
    setStatus(null);
    setInviteLink(null);

    try {
      const invite = await serversApi.createInvite(activeServerId);
      const link = `${window.location.origin}/invite/${invite.code}`;
      setInviteLink(link);

      await navigator.clipboard.writeText(link).catch(() => {});
      setOk(
        isEnglish
          ? "Invite created (link copied)."
          : "Invitation generee (lien copie)."
      );
    } catch (err) {
      setErr(getErrorMessage(err));
    } finally {
      setLoadingInvite(false);
    }
  };

  const onCopyInvite = async () => {
    if (!inviteLink) return;
    await navigator.clipboard.writeText(inviteLink).catch(() => {});
    setInfo(isEnglish ? "Link copied." : "Lien copie.");
  };

  const onLeaveOrDelete = async () => {
    if (!activeServerId || !activeServer) {
      setErr(isEnglish ? "Select a server." : "Selectionne un serveur.");
      return;
    }

    setLoadingDanger(true);
    setStatus(null);

    try {
      if (isOwner) {
        const ok = confirm(
          isEnglish
            ? "You are the owner. Delete this server?"
            : "Tu es le createur. Supprimer le serveur ?"
        );
        if (!ok) return;
        await serversApi.delete(activeServerId);
        setInfo(isEnglish ? "Server deleted." : "Serveur supprime.");
      } else {
        const ok = confirm(
          isEnglish ? "Leave this server?" : "Quitter ce serveur ?"
        );
        if (!ok) return;
        await serversApi.leave(activeServerId);
        setInfo(isEnglish ? "Server left." : "Serveur quitte.");
      }

      await onRefresh();

      const after = useServerStore.getState().servers;
      const stillThere = after.some((s) => s.id === activeServerId);
      if (!stillThere) {
        const next = after[0]?.id ?? null;
        if (next) setActiveServer(next);
      }
    } finally {
      setLoadingDanger(false);
    }
  };

  const statusClasses =
    status?.type === "success"
      ? "border-emerald-200 text-emerald-700 bg-emerald-50"
      : status?.type === "error"
      ? "border-red-200 text-red-700 bg-red-50"
      : "border-[#023BFC]/20 text-[#023BFC] bg-[var(--accent)]";

  return (
    <aside className="w-[88px] my-4 ml-3 rounded-2xl flex flex-col items-center gap-4 py-6 bg-gradient-to-b from-[var(--surface)] to-[var(--card)] border border-[var(--border)] shadow-lg">

      {/* Logo — navigates to DM */}
      <Link
        href="/dm"
        className="w-12 h-12 rounded-xl bg-gradient-to-br from-[#023BFC] to-[#3D6AFF] flex items-center justify-center shadow-lg glow-blue-sm hover:scale-105 transition-transform duration-200"
        title="Messages privés"
      >
        <MessageSquare className="w-6 h-6 text-white" />
      </Link>

      <div className="w-10 h-px bg-gradient-to-r from-transparent via-[var(--border)] to-transparent" />

      {/* Add server */}
      <button
        onClick={onCreateServer}
        className="w-12 h-12 server-icon bg-[var(--card)] hover:bg-[var(--accent)] border-2 border-dashed border-[#023BFC]/30 hover:border-[#023BFC] text-[#023BFC] text-2xl flex items-center justify-center transition-all duration-300 hover:scale-105"
        title="Creer un serveur"
      >
        <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
        </svg>
      </button>

      {/* Servers list */}
      <div className="flex-1 w-full flex flex-col items-center gap-3 overflow-auto px-3 py-2">
        {servers.map((s) => {
          const active = s.id === activeServerId;
          return (
            <button
              key={s.id}
              onClick={() => {
                if (s.id !== activeServerId) {
                  resetChannels();
                }
                setActiveServer(s.id);
              }}
              className={[
                "w-12 h-12 transition-all duration-300 flex items-center justify-center font-semibold text-sm",
                active
                  ? "server-icon-active bg-gradient-to-br from-[#023BFC] to-[#3D6AFF] text-white rounded-2xl"
                  : "server-icon bg-[var(--card)] text-[var(--foreground)] border border-[var(--border)] hover:border-[#023BFC]/50 hover:text-[#023BFC]",
              ].join(" ")}
              title={s.name}
            >
              {initials(s.name)}
            </button>
          );
        })}
      </div>

      <div className="w-10 h-px bg-gradient-to-r from-transparent via-[var(--border)] to-transparent" />

      {/* User Settings (Profile) */}
      <div>
        <UserSettings />
      </div>

      {/* Modal settings */}
      {openSettings && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6">
          <div
            className="absolute inset-0 bg-black/40 backdrop-blur-sm"
            onClick={() => setOpenSettings(false)}
          />
          <div className="relative w-full max-w-lg max-h-[90vh] flex flex-col rounded-3xl glass border border-white/30 shadow-2xl overflow-hidden">
            {/* Header with gradient */}
            <div className="shrink-0 px-4 sm:px-6 py-4 sm:py-5 border-b border-[var(--border)]/50 flex items-center bg-gradient-to-r from-[#023BFC]/5 to-transparent">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-[#023BFC] to-[#0066FF] flex items-center justify-center shadow-lg">
                  <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                </div>
                <div className="min-w-0">
                  <div className="font-bold text-[var(--foreground)]">
                    {isEnglish ? "Settings" : "Parametres"}
                  </div>
                  <div className="text-xs text-[var(--muted-foreground)] truncate">
                    {activeServer?.name ??
                      (isEnglish ? "No server selected" : "Aucun serveur selectionne")}
                  </div>
                </div>
              </div>
              <button
                title={isEnglish ? "Close settings" : "Fermer les paramètres"}
                aria-label={isEnglish ? "Close settings" : "Fermer les paramètres"}
                className="ml-auto w-9 h-9 rounded-xl bg-[var(--surface)] hover:bg-[var(--border)] border border-[var(--border)] flex items-center justify-center transition-all duration-300 shrink-0"
                onClick={() => setOpenSettings(false)}
              >
                <svg className="w-4 h-4 text-[var(--muted-foreground)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <div className="flex-1 min-h-0 overflow-y-auto p-4 sm:p-6 space-y-4 sm:space-y-5">
              {/* Join */}
              <div className="rounded-2xl border border-[var(--border)]/50 p-4 sm:p-5 bg-[var(--card)] hover:bg-[var(--card)] transition-all duration-300 neu-shadow-sm">
                <div className="flex items-center gap-2 mb-3">
                  <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-[#10B981] to-[#059669] flex items-center justify-center">
                    <svg className="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z" />
                    </svg>
                  </div>
                  <div className="text-sm font-bold text-[var(--foreground)]">
                    {isEnglish ? "Join a server" : "Rejoindre un serveur"}
                  </div>
                </div>
                <div className="text-xs text-[var(--muted-foreground)] mb-4">
                  {isEnglish
                    ? "Paste an invite code or full invite link."
                    : "Colle un code ou un lien complet d'invitation."}
                </div>

                <div className="flex flex-col sm:flex-row gap-3">
                  <input
                    value={inviteInput}
                    onChange={(e) => setInviteInput(e.target.value)}
                    placeholder={
                      isEnglish ? "Invite link or code..." : "Lien d'invite ou code..."
                    }
                    className="h-11 flex-1 rounded-xl border border-[var(--border)] px-4 text-sm bg-[var(--card)] focus:border-[#023BFC] focus:ring-2 focus:ring-[#023BFC]/20 outline-none transition-all duration-300"
                  />
                  <Button onClick={onJoin} disabled={loadingJoin || !inviteInput.trim()} className="btn-premium h-11 px-5">
                    {loadingJoin ? "..." : isEnglish ? "Join" : "Rejoindre"}
                  </Button>
                </div>
              </div>

              {/* Invite */}
              <div className="rounded-2xl border border-[var(--border)]/50 p-4 sm:p-5 bg-[var(--card)] hover:bg-[var(--card)] transition-all duration-300 neu-shadow-sm">
                <div className="flex items-center gap-2 mb-3">
                  <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-[#023BFC] to-[#0066FF] flex items-center justify-center">
                    <svg className="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                    </svg>
                  </div>
                  <div className="text-sm font-bold text-[var(--foreground)]">
                    {isEnglish ? "Invite to my server" : "Inviter sur mon serveur"}
                  </div>
                </div>
                <div className="text-xs text-[var(--muted-foreground)] mb-4">
                  {isEnglish
                    ? "Generate an invite link to share."
                    : "Génère un lien d'invitation à partager."}
                </div>

                <div className="flex gap-3 flex-wrap">
                  <Button
                    variant="outline"
                    onClick={onInvite}
                    disabled={loadingInvite || !activeServerId}
                    title={
                      isOwner
                        ? isEnglish ? "Generate" : "Generer"
                        : isEnglish ? "Owner only" : "Reserve au createur"
                    }
                    className="h-11 px-5 rounded-xl border-[var(--border)] hover:border-[#023BFC] hover:bg-[#023BFC]/5 transition-all duration-300"
                  >
                    {loadingInvite
                      ? "..."
                      : isEnglish
                      ? "Generate invite"
                      : "Generer une invitation"}
                  </Button>

                  {inviteLink && (
                    <>
                      <div className="w-full sm:flex-1 sm:w-auto min-w-0 text-xs rounded-xl border border-[var(--border)] px-4 py-3 bg-[var(--surface)] font-mono overflow-hidden text-ellipsis text-[var(--foreground)] break-all">
                        {inviteLink}
                      </div>
                      <Button variant="outline" onClick={onCopyInvite} className="h-11 px-4 rounded-xl border-[#023BFC] text-[#023BFC] hover:bg-[#023BFC] hover:text-white transition-all duration-300">
                        {isEnglish ? "Copy" : "Copier"}
                      </Button>
                    </>
                  )}
                </div>
              </div>

              {/* Danger */}
              <div className="rounded-2xl border border-red-200/50 p-4 sm:p-5 bg-red-50/30 hover:bg-red-50/50 transition-all duration-300">
                <div className="flex items-center gap-2 mb-3">
                  <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-red-500 to-red-600 flex items-center justify-center">
                    <svg className="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                    </svg>
                  </div>
                  <div className="text-sm font-bold text-red-700">
                    {isOwner
                      ? isEnglish ? "Danger zone" : "Zone dangereuse"
                      : isEnglish ? "Leave server" : "Quitter le serveur"}
                  </div>
                </div>
                <div className="text-xs text-red-600/80 mb-4">
                  {isOwner
                    ? isEnglish
                      ? "Deletion is permanent and irreversible."
                      : "La suppression est definitive et irreversible."
                    : isEnglish
                    ? "You can leave this server at any time."
                    : "Tu peux quitter ce serveur a tout moment."}
                </div>

                <Button
                  variant={isOwner ? "destructive" : "outline"}
                  onClick={onLeaveOrDelete}
                  disabled={loadingDanger || !activeServerId}
                  className={isOwner ? "h-11 px-5 rounded-xl bg-gradient-to-r from-red-500 to-red-600 hover:from-red-600 hover:to-red-700 border-0 shadow-lg" : "h-11 px-5 rounded-xl border-red-300 text-red-600 hover:bg-red-50"}
                >
                  {loadingDanger
                    ? "..."
                    : isOwner
                    ? isEnglish ? "Delete permanently" : "Supprimer definitivement"
                    : isEnglish ? "Leave" : "Quitter"}
                </Button>
              </div>

              {status && (
                <div className={`text-xs border rounded-xl px-4 py-3 ${statusClasses}`}>
                  {status.text}
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Modal création serveur */}
      <CreateServerModal
        open={openCreateServer}
        onOpenChange={setOpenCreateServer}
        onSuccess={onRefresh}
      />
    </aside>
  );
}
