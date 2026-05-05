"use client";

import { Button } from "@/components/ui/button";
import { useServerStore } from "@/store/server.store";
import { useAuthStore } from "@/store/auth.store";
import { serversApi } from "@/lib/api";
import { getErrorMessage } from "@/lib/api/errors";
import { useMemo, useState } from "react";
import { UserSettings } from "./user-settings";
import { useLanguage } from "@/components/language-provider";
import { CreateServerModal } from "@/components/forms/create-server-modal";
import { ConfirmActionDialog } from "@/components/ui/confirm-action-dialog";

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

export function ServersBar({ onRefresh }: { onRefresh: () => Promise<void> }) {
  const servers = useServerStore((s) => s.servers);
  const activeServerId = useServerStore((s) => s.activeServerId);
  const setActiveServer = useServerStore((s) => s.setActiveServer);
  const currentUser = useAuthStore((s) => s.user);

  const activeServer = useMemo(
    () => servers.find((s) => s.id === activeServerId) ?? null,
    [servers, activeServerId]
  );

  const isOwner = !!activeServer && !!currentUser && activeServer.owner_id === currentUser.id;

  const [openCreateServer, setOpenCreateServer] = useState(false);
  const { language } = useLanguage();
  const isEnglish = language === "en";
  const [inviteInput, setInviteInput] = useState("");
  const [joinLoading, setJoinLoading] = useState(false);

  const [inviteLink, setInviteLink] = useState<string | null>(null);
  const [inviteLoading, setInviteLoading] = useState(false);
  const [dangerLoading, setDangerLoading] = useState(false);
  const [openDangerConfirm, setOpenDangerConfirm] = useState(false);

  const [status, setStatus] = useState<Status>(null);

  const setOk = (text: string) => setStatus({ type: "success", text });
  const setErr = (text: string) => setStatus({ type: "error", text });
  const setInfo = (text: string) => setStatus({ type: "info", text });

  const onCreateServer = () => {
    setOpenCreateServer(true);
  };

  const onInvite = async () => {
    if (!activeServerId) return;

    if (!isOwner) {
      setErr(
        isEnglish
          ? "Only the owner can generate an invite."
          : "Seul le créateur peut générer une invitation.",
      );
      return;
    }

    setInviteLoading(true);
    setStatus(null);
    setInviteLink(null);

    try {
      const invite = await serversApi.createInvite(activeServerId);
      const link = `${window.location.origin}/invite/${invite.code}`;

      setInviteLink(link);
      await navigator.clipboard.writeText(link).catch(() => {});
      setOk(
        isEnglish
          ? "Invite generated (link copied)."
          : "Invitation générée (lien copié).",
      );
    } catch (err) {
      setErr(getErrorMessage(err));
    } finally {
      setInviteLoading(false);
    }
  };

  const onCopyInvite = async () => {
    if (!inviteLink) return;
    await navigator.clipboard.writeText(inviteLink).catch(() => {});
    setInfo(isEnglish ? "Link copied." : "Lien copié.");
  };

  const onJoin = async () => {
    const code = extractInviteCode(inviteInput);
    if (!code) {
      setErr(
        isEnglish
          ? "Paste a /invite/<code> link or a valid code."
          : "Colle un lien /invite/<code> ou un code valide.",
      );
      return;
    }

    setJoinLoading(true);
    setStatus(null);

    try {
      await serversApi.joinByInvite(code);
      setInviteInput("");
      await onRefresh();
      setOk(isEnglish ? "Server joined." : "Serveur rejoint.");
    } catch (err) {
      setErr(getErrorMessage(err));
    } finally {
      setJoinLoading(false);
    }
  };

  const onLeaveOrDelete = async () => {
    if (!activeServerId || !activeServer) return;

    setOpenDangerConfirm(true);
  };

  const confirmLeaveOrDelete = async () => {
    if (!activeServerId || !activeServer) return;

    setStatus(null);
    setDangerLoading(true);
    try {
      if (isOwner) {
        await serversApi.delete(activeServerId);
        setInfo(isEnglish ? "Server deleted." : "Serveur supprimé.");
      } else {
        await serversApi.leave(activeServerId);
        setInfo(isEnglish ? "Server left." : "Serveur quitté.");
      }

      await onRefresh();

      // recaler un serveur actif si celui-ci a disparu
      const after = useServerStore.getState().servers;
      const stillThere = after.some((s) => s.id === activeServerId);
      if (!stillThere) {
        const next = after[0]?.id ?? null;
        if (next) setActiveServer(next);
      }

      setOpenDangerConfirm(false);
    } finally {
      setDangerLoading(false);
    }
  };

  const joinDisabled = joinLoading || inviteInput.trim().length === 0;

  const statusClasses =
    status?.type === "success"
      ? "border-green-200 text-green-700 bg-green-50"
      : status?.type === "error"
      ? "border-red-200 text-red-700 bg-red-50"
      : "border-[var(--border)] text-[var(--foreground)] bg-[var(--surface)]";

  return (
    <div className="border-b px-4 py-2">
      <div className="flex items-center gap-2">
        {servers.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            {isEnglish ? "No server" : "Aucun serveur"}
          </p>
        ) : (
          servers.map((server) => (
            <Button
              key={server.id}
              variant={server.id === activeServerId ? "default" : "outline"}
              onClick={() => setActiveServer(server.id)}
            >
              {server.name}
            </Button>
          ))
        )}

        <div className="ml-auto flex items-center gap-2">
          <input
            value={inviteInput}
            onChange={(e) => setInviteInput(e.target.value)}
            placeholder={isEnglish ? "Invite link or code..." : "Lien d’invite ou code..."}
            className="h-9 w-56 rounded-md border px-3 text-sm bg-background"
          />

          <Button variant="outline" onClick={onJoin} disabled={joinDisabled}>
            {joinLoading ? "..." : isEnglish ? "Join" : "Rejoindre"}
          </Button>

          <Button
            variant="outline"
            onClick={onInvite}
            disabled={!activeServerId || inviteLoading}
            title={
              isOwner
                ? isEnglish
                  ? "Generate an invite"
                  : "Générer une invitation"
                : isEnglish
                  ? "Owner only"
                  : "Réservé au créateur"
            }
          >
            {inviteLoading ? "..." : isEnglish ? "Invite" : "Inviter"}
          </Button>

          <Button
            variant={isOwner ? "destructive" : "outline"}
            onClick={onLeaveOrDelete}
            disabled={!activeServerId}
            title={
              isOwner
                ? isEnglish
                  ? "Delete server"
                  : "Supprimer le serveur"
                : isEnglish
                  ? "Leave server"
                  : "Quitter le serveur"
            }
          >
            {isOwner ? (isEnglish ? "Delete" : "Supprimer") : isEnglish ? "Leave" : "Quitter"}
          </Button>

          <Button onClick={onCreateServer}>
            + {isEnglish ? "New server" : "Nouveau serveur"}
          </Button>

          {/* User Settings */}
          <div className="border-l pl-2 ml-2">
            <UserSettings />
          </div>
        </div>
      </div>

      {/* ✅ Zone de retour utilisateur (status + lien invite) */}
      {(status || inviteLink) && (
        <div className="mt-2 flex items-center gap-2">
          {status && (
            <div className={`text-xs border rounded-md px-3 py-2 ${statusClasses}`}>
              {status.text}
            </div>
          )}

          {inviteLink && (
            <div className="flex items-center gap-2 ml-auto">
              <div className="text-xs border rounded-md px-3 py-2 bg-background">
                <span className="text-muted-foreground">
                  {isEnglish ? "Link:" : "Lien :"}
                </span>{" "}
                <span className="font-mono">{inviteLink}</span>
              </div>
              <Button size="sm" variant="outline" onClick={onCopyInvite}>
                {isEnglish ? "Copy" : "Copier"}
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  setInviteLink(null);
                  setInfo(isEnglish ? "Link hidden." : "Lien masqué.");
                }}
              >
                {isEnglish ? "Close" : "Fermer"}
              </Button>
            </div>
          )}
        </div>
      )}

      {/* Modal création serveur */}
      <CreateServerModal
        open={openCreateServer}
        onOpenChange={setOpenCreateServer}
        onSuccess={onRefresh}
      />

      <ConfirmActionDialog
        open={openDangerConfirm}
        onOpenChange={setOpenDangerConfirm}
        title={
          isOwner
            ? isEnglish
              ? "Delete this server?"
              : "Supprimer ce serveur ?"
            : isEnglish
              ? "Leave this server?"
              : "Quitter ce serveur ?"
        }
        description={
          isOwner
            ? isEnglish
              ? "The server and its content will be permanently deleted."
              : "Le serveur et son contenu seront supprimés définitivement."
            : isEnglish
              ? "You will be removed from this server."
              : "Vous serez retiré de ce serveur."
        }
        confirmLabel={isOwner ? (isEnglish ? "Delete" : "Supprimer") : isEnglish ? "Leave" : "Quitter"}
        confirmVariant={isOwner ? "destructive" : "default"}
        loading={dangerLoading}
        onConfirm={confirmLeaveOrDelete}
      />
    </div>
  );
}
